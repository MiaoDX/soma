use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::net::UnixDatagram;
use std::path::Path;

use fs2::FileExt;
use prost::Message;
use soma_core::{
    AppliedCommand, ApplyDisposition as CoreApplyDisposition, CommandResult, ControlTick,
    Lifecycle as CoreLifecycle, PlantHealth as CorePlantHealth, ReachyActuatorTarget,
    RejectionReason as CoreRejectionReason, SourceTimeDomain as CoreSourceTimeDomain,
};
use soma_protocol::v1::{self, rt_request};

pub mod open_duck;

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

pub fn stamp_request_received(
    request: &mut v1::RtRequest,
    received_ns: u64,
    runtime_generation: u64,
) {
    if let Some(rt_request::Request::Target(target)) = request.request.as_mut() {
        target.issued_at_ns = received_ns;
        target.runtime_generation = runtime_generation;
    }
}

pub fn admit_public_request(
    mut request: v1::RtRequest,
    received_ns: u64,
    runtime_generation: u64,
) -> v1::RtRequest {
    match request.request.as_ref() {
        Some(rt_request::Request::Target(_)) => {
            stamp_request_received(&mut request, received_ns, runtime_generation);
            request
        }
        Some(rt_request::Request::Reset(true)) => request,
        _ => ingress_rejection(0, v1::RejectionReason::Invalid),
    }
}

pub fn ingress_rejection(sequence: u64, reason: v1::RejectionReason) -> v1::RtRequest {
    v1::RtRequest {
        request: Some(rt_request::Request::IngressRejection(
            v1::IngressRejection {
                sequence,
                reason: reason as i32,
            },
        )),
    }
}

pub fn runtime_started(generation: u64) -> v1::RtRequest {
    v1::RtRequest {
        request: Some(rt_request::Request::RuntimeStarted(v1::RuntimeStarted {
            generation,
        })),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetDecodeError {
    WrongPositionCount { actual: usize },
}

pub fn decode_target(
    target: v1::ActuatorTarget,
) -> Result<ReachyActuatorTarget, TargetDecodeError> {
    let positions_rad = target
        .positions_rad
        .try_into()
        .map_err(
            |positions: Vec<f32>| TargetDecodeError::WrongPositionCount {
                actual: positions.len(),
            },
        )?;
    Ok(ReachyActuatorTarget {
        positions_rad,
        sequence: target.sequence,
        timeline: target.timeline,
        issued_at_ns: target.issued_at_ns,
        ttl_ns: target.ttl_ns,
        runtime_generation: target.runtime_generation,
    })
}

pub fn update_state(
    state: &mut v1::ActuatorState,
    tick: ControlTick<{ soma_core::ACTUATOR_COUNT }>,
    state_age_ns: u64,
) {
    let (applied_source, applied_sequence) = match tick.applied.command {
        AppliedCommand::Target { sequence } => (v1::AppliedSource::Target as i32, sequence),
        AppliedCommand::MeasuredPositionHold { sequence } => {
            (v1::AppliedSource::MeasuredPositionHold as i32, sequence)
        }
    };
    let (command_sequence, command_disposition, rejection_reason) = match tick.command_result {
        CommandResult::NoCommand => (
            0,
            v1::CommandDisposition::NoCommand,
            v1::RejectionReason::Unspecified,
        ),
        CommandResult::Accepted { sequence } => (
            sequence,
            v1::CommandDisposition::Accepted,
            v1::RejectionReason::Unspecified,
        ),
        CommandResult::Rejected { sequence, reason } => (
            sequence,
            v1::CommandDisposition::Rejected,
            match reason {
                CoreRejectionReason::Timeline => v1::RejectionReason::Timeline,
                CoreRejectionReason::Sequence => v1::RejectionReason::Sequence,
                CoreRejectionReason::Expired => v1::RejectionReason::Expired,
                CoreRejectionReason::Invalid => v1::RejectionReason::Invalid,
                CoreRejectionReason::RuntimeGeneration => v1::RejectionReason::RuntimeGeneration,
            },
        ),
    };
    state.positions_rad.clear();
    state
        .positions_rad
        .extend_from_slice(&tick.measured.positions_rad);
    state.sequence = tick.measured.sequence;
    state.timeline = tick.measured.timeline;
    state.source_timestamp_ns = tick.measured.source_timestamp_ns;
    state.state_age_ns = state_age_ns;
    state.applied_source = applied_source;
    state.applied_sequence = applied_sequence;
    state.expiry_transition = tick.applied.expiry_transition;
    state.command_disposition = command_disposition as i32;
    state.rejection_reason = rejection_reason as i32;
    state.command_sequence = command_sequence;
    state.source_time_domain = match tick.measured.source_time_domain {
        CoreSourceTimeDomain::Simulation => v1::SourceTimeDomain::Simulation,
        CoreSourceTimeDomain::HostMonotonic => v1::SourceTimeDomain::HostMonotonic,
        CoreSourceTimeDomain::Device => v1::SourceTimeDomain::Device,
    } as i32;
    state.runtime_generation = tick.runtime_generation;
    state.runtime_transition = tick.runtime_transition;
    state.lifecycle = match tick.measured.lifecycle {
        CoreLifecycle::Disabled => v1::Lifecycle::Disabled,
        CoreLifecycle::Enabled => v1::Lifecycle::Enabled,
        CoreLifecycle::Stopped => v1::Lifecycle::Stopped,
    } as i32;
    state.health = match tick.measured.health {
        CorePlantHealth::Healthy => v1::PlantHealth::Healthy,
        CorePlantHealth::StaleState => v1::PlantHealth::StaleState,
        CorePlantHealth::CommunicationFault => v1::PlantHealth::CommunicationFault,
        CorePlantHealth::ConfigurationMismatch => v1::PlantHealth::ConfigurationMismatch,
    } as i32;
    state.capture_monotonic_ns = monotonic_ns();
    state.apply_disposition = match tick.apply_disposition {
        CoreApplyDisposition::Submitted => v1::ApplyDisposition::Submitted,
        CoreApplyDisposition::Confirmed => v1::ApplyDisposition::Confirmed,
    } as i32;
}

pub fn encode_state_into(
    state: &mut v1::ActuatorState,
    tick: ControlTick<{ soma_core::ACTUATOR_COUNT }>,
    state_age_ns: u64,
    payload: &mut Vec<u8>,
) -> Result<(), prost::EncodeError> {
    update_state(state, tick, state_age_ns);
    payload.clear();
    state.encode(payload)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TimingEvidence {
    pub tick_count: u64,
    pub tick_gap_ns: u64,
    pub late_by_ns: u64,
    pub max_work_duration_ns: u64,
    pub ingress_drops: u64,
    pub egress_drops: u64,
    pub deadline_overruns: u64,
}

pub fn encode_state_into_with_timing(
    state: &mut v1::ActuatorState,
    tick: ControlTick<{ soma_core::ACTUATOR_COUNT }>,
    state_age_ns: u64,
    timing: TimingEvidence,
    payload: &mut Vec<u8>,
) -> Result<(), prost::EncodeError> {
    update_state(state, tick, state_age_ns);
    state.tick_count = timing.tick_count;
    state.tick_gap_ns = timing.tick_gap_ns;
    state.late_by_ns = timing.late_by_ns;
    state.max_work_duration_ns = timing.max_work_duration_ns;
    state.ingress_drops = timing.ingress_drops;
    state.egress_drops = timing.egress_drops;
    state.deadline_overruns = timing.deadline_overruns;
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
                runtime_generation: 0,
            })),
        };
        stamp_request_received(&mut request, 900, 77);
        let rt_request::Request::Target(target) = request.request.unwrap() else {
            unreachable!()
        };
        let decoded = decode_target(target).unwrap();
        assert_eq!(decoded.issued_at_ns, 900);
        assert_eq!(decoded.positions_rad.len(), 9);
        assert_eq!(decoded.runtime_generation, 77);
        assert!(decoded.is_expired(950));
    }

    #[test]
    fn target_decode_reports_wrong_shape_instead_of_dropping_it() {
        let error = decode_target(v1::ActuatorTarget {
            positions_rad: vec![0.0; 8],
            sequence: 2,
            timeline: 1,
            issued_at_ns: 3,
            ttl_ns: 4,
            runtime_generation: 0,
        })
        .unwrap_err();
        assert_eq!(error, TargetDecodeError::WrongPositionCount { actual: 8 });
    }

    #[test]
    fn ingress_rejection_preserves_available_request_identity() {
        let request = ingress_rejection(17, v1::RejectionReason::Invalid);
        assert_eq!(
            request.request,
            Some(rt_request::Request::IngressRejection(
                v1::IngressRejection {
                    sequence: 17,
                    reason: v1::RejectionReason::Invalid as i32,
                }
            ))
        );
    }

    #[test]
    fn public_ingress_cannot_inject_internal_runtime_messages() {
        let injected = runtime_started(41);
        let admitted = admit_public_request(injected, 100, 42);
        assert_eq!(admitted, ingress_rejection(0, v1::RejectionReason::Invalid));

        let empty = admit_public_request(v1::RtRequest::default(), 100, 42);
        assert_eq!(empty, ingress_rejection(0, v1::RejectionReason::Invalid));
    }

    #[test]
    fn periodic_state_encoding_reuses_preallocated_storage() {
        use soma_core::{AppliedControl, Lifecycle, PlantHealth, ReachyActuatorState};

        let tick = ControlTick {
            measured: ReachyActuatorState {
                positions_rad: [0.1; 9],
                sequence: 1,
                timeline: 2,
                source_timestamp_ns: 3,
                source_time_domain: soma_core::SourceTimeDomain::HostMonotonic,
                lifecycle: Lifecycle::Enabled,
                health: PlantHealth::Healthy,
            },
            command_result: CommandResult::NoCommand,
            applied: AppliedControl {
                positions_rad: [0.1; 9],
                command: AppliedCommand::MeasuredPositionHold { sequence: 1 },
                expiry_transition: false,
            },
            apply_disposition: CoreApplyDisposition::Confirmed,
            runtime_generation: 9,
            runtime_transition: true,
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
        assert_eq!(
            state.apply_disposition,
            v1::ApplyDisposition::Confirmed as i32
        );
    }

    #[test]
    fn periodic_state_encoding_reports_timing_and_drop_evidence() {
        let mut plant = soma_sim::ReachySimPlant::load(
            soma_sim::REACHY_SCENE_PATH,
            std::time::Duration::from_millis(20),
        )
        .unwrap();
        let mut core = soma_core::ControlCore::new();
        let tick = core.tick(&mut plant, None, monotonic_ns()).unwrap();
        let mut state = v1::ActuatorState::default();
        let mut payload = Vec::new();
        let timing = TimingEvidence {
            tick_count: 7,
            tick_gap_ns: 20_100_000,
            late_by_ns: 100_000,
            max_work_duration_ns: 2_000_000,
            ingress_drops: 3,
            egress_drops: 4,
            deadline_overruns: 1,
        };
        encode_state_into_with_timing(&mut state, tick, 0, timing, &mut payload).unwrap();
        let decoded = v1::ActuatorState::decode(payload.as_slice()).unwrap();
        assert_eq!(decoded.tick_count, 7);
        assert_eq!(decoded.tick_gap_ns, 20_100_000);
        assert_eq!(decoded.late_by_ns, 100_000);
        assert_eq!(decoded.max_work_duration_ns, 2_000_000);
        assert_eq!(decoded.ingress_drops, 3);
        assert_eq!(decoded.egress_drops, 4);
        assert_eq!(decoded.deadline_overruns, 1);
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
