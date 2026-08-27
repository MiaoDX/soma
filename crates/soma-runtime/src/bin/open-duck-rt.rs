use soma_core::{ActuatorTarget, AppliedCommand, CommandResult, ControlCore};
use soma_runtime::{
    bind_owned_datagram, monotonic_ns,
    open_duck::{
        decode_target, encode_state, OpenDuckState, OPEN_DUCK_RT_SOCKET, OPEN_DUCK_RUNTIME_SOCKET,
        OPEN_DUCK_STATE_BYTES, OPEN_DUCK_TARGET_BYTES,
    },
};
use soma_sim::{OpenDuckSimPlant, OPEN_DUCK_ACTUATOR_COUNT};
use std::io::ErrorKind;
use std::thread;
use std::time::{Duration, Instant};

const PERIOD: Duration = Duration::from_millis(2);
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let owned = bind_owned_datagram(OPEN_DUCK_RT_SOCKET)?;
    owned.socket.set_nonblocking(true)?;
    let mut plant =
        OpenDuckSimPlant::load(PERIOD).map_err(|e| format!("load Duck model: {e:?}"))?;
    let mut core = ControlCore::<OPEN_DUCK_ACTUATOR_COUNT>::new();
    let mut pending: Option<ActuatorTarget<OPEN_DUCK_ACTUATOR_COUNT>> = None;
    let mut bytes = [0_u8; OPEN_DUCK_TARGET_BYTES];
    let mut state_bytes = [0_u8; OPEN_DUCK_STATE_BYTES];
    let mut next = Instant::now();
    let mut ticks = 0_u64;
    let mut requested_sequence = 0;
    let mut admitted_sequence = 0;
    let mut admitted_capture_ns = 0;
    let mut pending_capture_ns = 0;
    let mut rejection_latched = false;
    let mut expiry_latched = false;
    loop {
        match owned.socket.recv(&mut bytes) {
            Ok(size) => {
                if let Some(target) = decode_target(&bytes[..size]) {
                    requested_sequence = target.sequence;
                    pending_capture_ns = target.capture_monotonic_ns;
                    pending = Some(ActuatorTarget {
                        positions_rad: target.positions_rad,
                        sequence: target.sequence,
                        timeline: target.timeline,
                        issued_at_ns: target.capture_monotonic_ns,
                        ttl_ns: target.ttl_ns,
                    });
                } else {
                    rejection_latched = true;
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => return Err(e.into()),
        }
        let now = monotonic_ns();
        let tick = core
            .tick(&mut plant, pending.take(), now)
            .map_err(|e| format!("Duck control tick: {e:?}"))?;
        plant.advance_physics_step();
        ticks += 1;
        if let CommandResult::Accepted { sequence } = tick.command_result {
            admitted_sequence = sequence;
            admitted_capture_ns = pending_capture_ns;
        }
        if matches!(tick.command_result, CommandResult::Rejected { .. }) {
            rejection_latched = true;
        }
        expiry_latched |= tick.applied.expiry_transition;
        if ticks % 10 == 1 {
            let facts = plant.policy_facts();
            let (applied_sequence, mut flags) = match tick.applied.command {
                AppliedCommand::Target { sequence } => (sequence, 1),
                AppliedCommand::MeasuredPositionHold { .. } => {
                    (0, 2 | (u32::from(expiry_latched) * 4))
                }
            };
            if rejection_latched {
                flags |= 8;
            }
            let state = OpenDuckState {
                positions_rad: facts.positions_rad,
                velocities_rad_s: facts.velocities_rad_s,
                gyro_rad_s: facts.gyro_rad_s,
                acceleration_m_s2: facts.acceleration_m_s2,
                feet_contacts: facts.feet_contacts,
                root_height_m: facts.root_height_m,
                root_roll_rad: facts.root_roll_rad,
                root_pitch_rad: facts.root_pitch_rad,
                sequence: tick.measured.sequence,
                timeline: tick.measured.timeline,
                capture_monotonic_ns: now,
                requested_sequence,
                admitted_sequence,
                applied_sequence,
                message_age_ns: if applied_sequence == 0 {
                    0
                } else {
                    now.saturating_sub(admitted_capture_ns)
                },
                flags,
            };
            encode_state(&state, &mut state_bytes);
            let _ = owned.socket.send_to(&state_bytes, OPEN_DUCK_RUNTIME_SOCKET);
            expiry_latched = false;
            rejection_latched = false;
        }
        next += PERIOD;
        thread::sleep(next.saturating_duration_since(Instant::now()));
    }
}
