use std::error::Error;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::Parser;
use rustypot::servo::dynamixel::xl330::Xl330Controller;
use serde::Serialize;
use serialport::{DataBits, FlowControl, Parity, SerialPortType, StopBits};

const IDS: [u8; 9] = [10, 11, 12, 13, 14, 15, 16, 17, 18];
const PROFILE: [Profile; 9] = [
    Profile::new(10, "body_rotation", 0, 0, 4095),
    Profile::new(11, "stewart_1", 1024, 1502, 2958),
    Profile::new(12, "stewart_2", -1024, 1138, 2844),
    Profile::new(13, "stewart_3", 1024, 1502, 2958),
    Profile::new(14, "stewart_4", -1024, 1138, 2594),
    Profile::new(15, "stewart_5", 1024, 1252, 2958),
    Profile::new(16, "stewart_6", -1024, 1138, 2594),
    Profile::new(17, "right_antenna", 0, 0, 4095),
    Profile::new(18, "left_antenna", 0, 0, 4095),
];

#[derive(Clone, Copy)]
struct Profile {
    id: u8,
    name: &'static str,
    offset: i32,
    min: i32,
    max: i32,
}

impl Profile {
    const fn new(id: u8, name: &'static str, offset: i32, min: i32, max: i32) -> Self {
        Self {
            id,
            name,
            offset,
            min,
            max,
        }
    }
}

#[derive(Parser)]
#[command(about = "Read-only Reachy Mini Lite Dynamixel N0 probe")]
struct Args {
    /// Explicit serial device path; otherwise CH343 VID:PID 1a86:55d3 is required.
    #[arg(long)]
    device: Option<PathBuf>,
    #[arg(long, default_value_t = 10)]
    timeout_ms: u64,
    #[arg(long, default_value_t = 2)]
    retries: u16,
}

#[derive(Serialize)]
struct Report {
    gate: &'static str,
    read_only: bool,
    device: String,
    usb_identity: String,
    baud: u32,
    framing: &'static str,
    timeout_ms: u64,
    exclusive_second_open_failed: bool,
    process_inspection: Vec<String>,
    detected_ids: Vec<u8>,
    motors: Vec<MotorReport>,
    passed: bool,
}

#[derive(Serialize)]
struct MotorReport {
    id: u8,
    name: &'static str,
    latency_us: u128,
    attempts: u32,
    model: u16,
    firmware: u8,
    baud_register: u8,
    homing_offset: i32,
    min_position: i32,
    max_position: i32,
    operating_mode: u8,
    shutdown: u8,
    torque_enabled: bool,
    present_position: i32,
    present_current: i16,
    voltage_raw: u16,
    hardware_error: u8,
    configuration_matches: bool,
    raw_reads: Vec<RawRead>,
}

#[derive(Serialize)]
struct RawRead {
    address: u8,
    length: u8,
    response_hex: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let (device, identity) = resolve_device(args.device)?;
    let process_inspection = inspect_processes();
    if !process_inspection.is_empty() {
        return Err(format!(
            "official Reachy daemon must be stopped before opening the bus: {}",
            process_inspection.join(", ")
        )
        .into());
    }
    let timeout = Duration::from_millis(args.timeout_ms);
    let builder = || {
        serialport::new(device.to_string_lossy(), 1_000_000)
            .timeout(timeout)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
    };
    let mut port = builder().open_native()?;
    port.set_exclusive(true)?;
    let exclusive_second_open_failed = builder().open_native().is_err();
    if !exclusive_second_open_failed {
        return Err("serial exclusivity check failed".into());
    }

    let mut controller = Xl330Controller::new()
        .with_protocol_v2()
        .with_serial_port(Box::new(port));
    let mut detected_ids = Vec::new();
    let mut motors = Vec::new();
    for profile in PROFILE {
        let started = Instant::now();
        let mut attempts = 0;
        let present = loop {
            attempts += 1;
            match controller.ping(profile.id) {
                Ok(value) => break value,
                Err(error) if should_retry(attempts, args.retries) => eprintln!(
                    "retry {attempts} for ID {} after ping error: {error}",
                    profile.id
                ),
                Err(error) => return Err(error),
            }
        };
        if !present {
            continue;
        }
        detected_ids.push(profile.id);
        motors.push(read_motor(
            &mut controller,
            profile,
            started.elapsed(),
            attempts,
        )?);
    }

    let passed = detected_ids == IDS
        && motors
            .iter()
            .all(|motor| !motor.torque_enabled && motor.configuration_matches)
        && process_inspection.is_empty();
    let report = Report {
        gate: "N0",
        read_only: true,
        device: device.display().to_string(),
        usb_identity: identity,
        baud: 1_000_000,
        framing: "8N1",
        timeout_ms: args.timeout_ms,
        exclusive_second_open_failed,
        process_inspection,
        detected_ids,
        motors,
        passed,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    if !passed {
        return Err("N0 audit did not pass".into());
    }
    Ok(())
}

fn read_motor(
    controller: &mut Xl330Controller,
    profile: Profile,
    latency: Duration,
    attempts: u32,
) -> Result<MotorReport, Box<dyn Error>> {
    let registers = [
        (0, 7),
        (8, 1),
        (11, 1),
        (20, 4),
        (48, 8),
        (63, 8),
        (126, 8),
        (144, 2),
    ];
    let mut raw_reads = Vec::new();
    for (address, length) in registers {
        let bytes = controller.read_raw_data(profile.id, address, length)?;
        raw_reads.push(RawRead {
            address,
            length,
            response_hex: bytes.iter().map(|byte| format!("{byte:02x}")).collect(),
        });
    }
    let model = controller.read_model_number(profile.id)?[0];
    let firmware = controller.read_firmware_version(profile.id)?[0];
    let baud_register = controller.read_baud_rate(profile.id)?[0];
    let homing_offset = controller.read_homing_offset(profile.id)?[0];
    let min_position = controller.read_raw_min_position_limit(profile.id)?[0];
    let max_position = controller.read_raw_max_position_limit(profile.id)?[0];
    let operating_mode = controller.read_operating_mode(profile.id)?[0];
    let shutdown = controller.read_shutdown(profile.id)?[0];
    let torque_enabled = controller.read_torque_enable(profile.id)?[0];
    let present_position = controller.read_raw_present_position(profile.id)?[0];
    let present_current = controller.read_present_current(profile.id)?[0];
    let voltage_raw = controller.read_present_input_voltage(profile.id)?[0];
    let hardware_error = controller.read_hardware_error_status(profile.id)?[0];
    let configuration_matches = baud_register == 3
        && homing_offset == profile.offset
        && min_position == profile.min
        && max_position == profile.max
        && operating_mode == 3
        && shutdown == 52;
    Ok(MotorReport {
        id: profile.id,
        name: profile.name,
        latency_us: latency.as_micros(),
        attempts,
        model,
        firmware,
        baud_register,
        homing_offset,
        min_position,
        max_position,
        operating_mode,
        shutdown,
        torque_enabled,
        present_position,
        present_current,
        voltage_raw,
        hardware_error,
        configuration_matches,
        raw_reads,
    })
}

fn resolve_device(override_path: Option<PathBuf>) -> Result<(PathBuf, String), Box<dyn Error>> {
    if let Some(path) = override_path {
        return Ok((path, "explicit-device-override".into()));
    }
    let matches: Vec<_> = serialport::available_ports()?
        .into_iter()
        .filter_map(|port| match port.port_type {
            SerialPortType::UsbPort(usb) if usb.vid == 0x1a86 && usb.pid == 0x55d3 => Some((
                PathBuf::from(port.port_name),
                format!("vid=1a86 pid=55d3 serial={:?}", usb.serial_number),
            )),
            _ => None,
        })
        .collect();
    match matches.as_slice() {
        [match_] => Ok(match_.clone()),
        [] => Err("no CH343 device with VID 1a86 PID 55d3 found; use --device to override".into()),
        _ => Err("multiple CH343 devices found; select one with --device".into()),
    }
}

fn inspect_processes() -> Vec<String> {
    let Ok(processes) = std::fs::read_dir("/proc") else {
        return vec!["process inspection unavailable".into()];
    };
    processes
        .flatten()
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .bytes()
                .all(|byte| byte.is_ascii_digit())
        })
        .filter_map(|entry| std::fs::read(entry.path().join("cmdline")).ok())
        .map(|bytes| String::from_utf8_lossy(&bytes).replace('\0', " "))
        .filter(|command| is_official_daemon_command(command))
        .collect()
}

fn is_official_daemon_command(command: &str) -> bool {
    command.contains("reachy-mini-daemon") || command.contains("reachy_mini.daemon")
}

fn should_retry(attempts: u32, retries: u16) -> bool {
    attempts <= u32::from(retries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_profile_has_the_expected_ids_and_configuration() {
        assert_eq!(PROFILE.map(|profile| profile.id), IDS);
        assert!(PROFILE.iter().all(|profile| profile.min <= profile.max));
    }

    #[test]
    fn official_daemon_commands_are_rejected_before_bus_access() {
        assert!(is_official_daemon_command(
            "/usr/bin/reachy-mini-daemon --robot-version lite"
        ));
        assert!(is_official_daemon_command(
            "python -m reachy_mini.daemon.app"
        ));
        assert!(!is_official_daemon_command(
            "cargo run --bin soma-reachy-probe"
        ));
    }

    #[test]
    fn maximum_retry_count_does_not_wrap_the_attempt_counter() {
        assert!(should_retry(255, u16::MAX));
        assert!(should_retry(256, u16::MAX));
        assert!(!should_retry(u32::from(u16::MAX) + 1, u16::MAX));
    }
}
