use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::net::UnixDatagram;
use std::path::Path;

use fs2::FileExt;
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

pub struct OwnedDatagram {
    pub socket: UnixDatagram,
    _lock: File,
}

pub fn bind_owned_datagram(path: &str) -> io::Result<OwnedDatagram> {
    let lock_path = format!("{path}.lock");
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)?;
    lock.try_lock_exclusive().map_err(|_| {
        io::Error::new(io::ErrorKind::AddrInUse, format!("{path} is already owned"))
    })?;
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

pub fn decode_target(request: v1::RtRequest, now_ns: u64) -> Option<ReachyActuatorTarget> {
    let rt_request::Request::Target(target) = request.request? else {
        return None;
    };
    let positions_rad = target.positions_rad.try_into().ok()?;
    Some(ReachyActuatorTarget {
        positions_rad,
        sequence: target.sequence,
        timeline: target.timeline,
        issued_at_ns: now_ns,
        ttl_ns: target.ttl_ns,
    })
}

pub fn encode_state(tick: ControlTick, state_age_ns: u64) -> v1::ActuatorState {
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
    v1::ActuatorState {
        positions_rad: tick.measured.positions_rad.to_vec(),
        sequence: tick.measured.sequence,
        timeline: tick.measured.timeline,
        timestamp_ns: tick.measured.timestamp_ns,
        state_age_ns,
        applied_source,
        applied_sequence,
        expiry_transition: tick.applied.expiry_transition,
        command_disposition: command_disposition as i32,
        rejection_reason: rejection_reason as i32,
        health: match tick.measured.health {
            CorePlantHealth::Healthy => v1::PlantHealth::Healthy,
            CorePlantHealth::StaleState => v1::PlantHealth::StaleState,
            CorePlantHealth::CommunicationFault => v1::PlantHealth::CommunicationFault,
            CorePlantHealth::ConfigurationMismatch => v1::PlantHealth::ConfigurationMismatch,
        } as i32,
        capture_monotonic_ns: monotonic_ns(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_requires_exactly_nine_positions_and_uses_rt_receive_time() {
        let request = v1::RtRequest {
            request: Some(rt_request::Request::Target(v1::ActuatorTarget {
                positions_rad: vec![0.0; 9],
                sequence: 2,
                timeline: 1,
                issued_at_ns: 123,
                ttl_ns: 50,
            })),
        };
        let decoded = decode_target(request, 900).unwrap();
        assert_eq!(decoded.issued_at_ns, 900);
        assert_eq!(decoded.positions_rad.len(), 9);
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
        drop(first);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{path}.lock"));
    }
}
