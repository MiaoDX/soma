use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::net::UnixDatagram;
use std::path::Path;

use fs2::FileExt;
use prost::Message;
use soma_core::{
    AppliedCommand, CommandResult, ControlTick, PlantHealth as CorePlantHealth,
    ReachyActuatorTarget, RejectionReason as CoreRejectionReason,
};
use soma_protocol::v1::{self, rt_request};

pub const RT_SOCKET: &str = "/tmp/soma-robot-rt.sock";
pub const RUNTIME_SOCKET: &str = "/tmp/soma-robot-runtime.sock";
pub const COMMAND_KEY: &str = "soma/reachy/command";
pub const STATE_KEY: &str = "soma/reachy/state";
pub const MAX_MESSAGE_SIZE: usize = 4096;

pub struct BestEffortDatagram {
    socket: UnixDatagram,
    destination: String,
}

impl BestEffortDatagram {
    pub fn new(destination: impl Into<String>) -> io::Result<Self> {
        let socket = UnixDatagram::unbound()?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            destination: destination.into(),
        })
    }

    pub fn try_send(&self, payload: &[u8]) -> bool {
        self.socket.send_to(payload, &self.destination).is_ok()
    }
}

pub struct OwnedDatagram {
    pub socket: UnixDatagram,
    _lock: File,
}

pub fn bind_owned_datagram(path: &str) -> io::Result<OwnedDatagram> {
    let lock_path = format!("{path}.lock");
    let mut lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)?;
    lock.try_lock_exclusive().map_err(|_| {
        io::Error::new(io::ErrorKind::AddrInUse, format!("{path} is already owned"))
    })?;
    lock.set_len(0)?;
    writeln!(lock, "{}", std::process::id())?;
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    Ok(OwnedDatagram {
        socket: UnixDatagram::bind(Path::new(path))?,
        _lock: lock,
    })
}

pub fn monotonic_ns() -> u64 {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `time` points to writable storage for one timespec.
    let result = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) };
    assert_eq!(result, 0, "CLOCK_MONOTONIC is required");
    time.tv_sec as u64 * 1_000_000_000 + time.tv_nsec as u64
}

pub fn stamp_request_received(request: &mut v1::RtRequest, received_ns: u64) {
    if let Some(rt_request::Request::Target(target)) = request.request.as_mut() {
        target.issued_at_ns = received_ns;
    }
}

pub fn decode_target(request: v1::RtRequest) -> Option<ReachyActuatorTarget> {
    let rt_request::Request::Target(target) = request.request? else {
        return None;
    };
    let positions_rad = target.positions_rad.try_into().ok()?;
    Some(ReachyActuatorTarget {
        positions_rad,
        sequence: target.sequence,
        timeline: target.timeline,
        issued_at_ns: target.issued_at_ns,
        ttl_ns: target.ttl_ns,
    })
}

pub fn update_state(state: &mut v1::ActuatorState, tick: ControlTick, state_age_ns: u64) {
    let (applied_source, applied_sequence) = match tick.applied.command {
        AppliedCommand::Target { sequence } => (v1::AppliedSource::Target as i32, sequence),
        AppliedCommand::MeasuredPositionHold { sequence } => {
            (v1::AppliedSource::MeasuredPositionHold as i32, sequence)
        }
    };
    let (command_disposition, rejection_reason) = match tick.command_result {
        CommandResult::NoCommand => (
            v1::CommandDisposition::NoCommand,
            v1::RejectionReason::Unspecified,
        ),
        CommandResult::Accepted { .. } => (
            v1::CommandDisposition::Accepted,
            v1::RejectionReason::Unspecified,
        ),
        CommandResult::Rejected { reason, .. } => (
            v1::CommandDisposition::Rejected,
            match reason {
                CoreRejectionReason::Timeline => v1::RejectionReason::Timeline,
                CoreRejectionReason::Sequence => v1::RejectionReason::Sequence,
                CoreRejectionReason::Expired => v1::RejectionReason::Expired,
            },
        ),
    };
    state.positions_rad.clear();
    state
        .positions_rad
        .extend_from_slice(&tick.measured.positions_rad);
    state.sequence = tick.measured.sequence;
    state.timeline = tick.measured.timeline;
    state.timestamp_ns = tick.measured.timestamp_ns;
    state.state_age_ns = state_age_ns;
    state.applied_source = applied_source;
    state.applied_sequence = applied_sequence;
    state.expiry_transition = tick.applied.expiry_transition;
    state.command_disposition = command_disposition as i32;
    state.rejection_reason = rejection_reason as i32;
    state.health = match tick.measured.health {
        CorePlantHealth::Healthy => v1::PlantHealth::Healthy,
        CorePlantHealth::StaleState => v1::PlantHealth::StaleState,
        CorePlantHealth::CommunicationFault => v1::PlantHealth::CommunicationFault,
        CorePlantHealth::ConfigurationMismatch => v1::PlantHealth::ConfigurationMismatch,
    } as i32;
    state.capture_monotonic_ns = monotonic_ns();
}

pub fn encode_state_into(
    state: &mut v1::ActuatorState,
    tick: ControlTick,
    state_age_ns: u64,
    payload: &mut Vec<u8>,
) -> Result<(), prost::EncodeError> {
    update_state(state, tick, state_age_ns);
    payload.clear();
    state.encode(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_receive_time_is_preserved_through_rt_decode() {
        let mut request = v1::RtRequest {
            request: Some(rt_request::Request::Target(v1::ActuatorTarget {
                positions_rad: vec![0.0; 9],
                sequence: 2,
                timeline: 1,
                issued_at_ns: 123,
                ttl_ns: 50,
            })),
        };
        stamp_request_received(&mut request, 900);
        let decoded = decode_target(request).unwrap();
        assert_eq!(decoded.issued_at_ns, 900);
        assert_eq!(decoded.positions_rad.len(), 9);
        assert!(decoded.is_expired(950));
    }

    #[test]
    fn periodic_state_encoding_reuses_preallocated_storage() {
        use soma_core::{AppliedControl, Lifecycle, PlantHealth, ReachyActuatorState};

        let tick = ControlTick {
            measured: ReachyActuatorState {
                positions_rad: [0.1; 9],
                sequence: 1,
                timeline: 2,
                timestamp_ns: 3,
                lifecycle: Lifecycle::Enabled,
                health: PlantHealth::Healthy,
            },
            command_result: CommandResult::NoCommand,
            applied: AppliedControl {
                positions_rad: [0.1; 9],
                command: AppliedCommand::MeasuredPositionHold { sequence: 1 },
                expiry_transition: false,
            },
        };
        let mut state = v1::ActuatorState {
            positions_rad: Vec::with_capacity(9),
            ..Default::default()
        };
        let mut payload = Vec::with_capacity(MAX_MESSAGE_SIZE);

        encode_state_into(&mut state, tick, 0, &mut payload).unwrap();
        let positions_ptr = state.positions_rad.as_ptr();
        let payload_ptr = payload.as_ptr();
        encode_state_into(&mut state, tick, 0, &mut payload).unwrap();

        assert_eq!(state.positions_rad.as_ptr(), positions_ptr);
        assert_eq!(payload.as_ptr(), payload_ptr);
        assert_eq!(state.positions_rad.capacity(), 9);
        assert_eq!(payload.capacity(), MAX_MESSAGE_SIZE);
    }

    #[test]
    fn socket_owner_lock_rejects_a_second_process_open() {
        let path = format!("/tmp/soma-runtime-lock-test-{}.sock", std::process::id());
        let first = bind_owned_datagram(&path).unwrap();
        let second_error = match bind_owned_datagram(&path) {
            Ok(_) => panic!("a second socket owner was admitted"),
            Err(error) => error,
        };
        assert_eq!(second_error.kind(), io::ErrorKind::AddrInUse);
        assert_eq!(
            std::fs::read_to_string(format!("{path}.lock")).unwrap(),
            format!("{}\n", std::process::id())
        );
        drop(first);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{path}.lock"));
    }

    #[test]
    fn best_effort_datagram_drops_when_receiver_is_missing_or_closed() {
        let path = format!("/tmp/soma-best-effort-test-{}.sock", std::process::id());
        let sender = BestEffortDatagram::new(&path).unwrap();
        assert!(!sender.try_send(b"missing"));
        let receiver = UnixDatagram::bind(&path).unwrap();
        assert!(sender.try_send(b"present"));
        drop(receiver);
        let _ = std::fs::remove_file(&path);
        assert!(!sender.try_send(b"closed"));
    }

    #[test]
    fn best_effort_datagram_never_blocks_when_receiver_buffer_is_full() {
        let path = format!(
            "/tmp/soma-best-effort-full-test-{}.sock",
            std::process::id()
        );
        let receiver = UnixDatagram::bind(&path).unwrap();
        let sender = BestEffortDatagram::new(&path).unwrap();
        let mut dropped = false;
        for _ in 0..100_000 {
            if !sender.try_send(&[0_u8; 1024]) {
                dropped = true;
                break;
            }
        }
        assert!(
            dropped,
            "a saturated receive buffer must produce a nonblocking drop"
        );
        drop(receiver);
        let _ = std::fs::remove_file(path);
    }
}
