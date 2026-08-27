use std::io::ErrorKind;
use std::thread;
use std::time::{Duration, Instant};
use soma_core::{ActuatorTarget, ControlCore};
use soma_runtime::{bind_owned_datagram, monotonic_ns, open_duck::{decode_target, OPEN_DUCK_RT_SOCKET, OPEN_DUCK_RUNTIME_SOCKET, OPEN_DUCK_TARGET_BYTES}};
use soma_sim::{OpenDuckSimPlant, OPEN_DUCK_ACTUATOR_COUNT};

const PERIOD: Duration = Duration::from_millis(2);
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let owned = bind_owned_datagram(OPEN_DUCK_RT_SOCKET)?; owned.socket.set_nonblocking(true)?;
    let mut plant = OpenDuckSimPlant::load(PERIOD).map_err(|e| format!("load Duck model: {e:?}"))?;
    let mut core = ControlCore::<OPEN_DUCK_ACTUATOR_COUNT>::new(); let mut pending: Option<ActuatorTarget<OPEN_DUCK_ACTUATOR_COUNT>> = None;
    let mut bytes = [0_u8; OPEN_DUCK_TARGET_BYTES]; let mut next = Instant::now();
    loop {
        match owned.socket.recv(&mut bytes) { Ok(size) => if let Some(target) = decode_target(&bytes[..size]) { pending = Some(ActuatorTarget { positions_rad: target.positions_rad, sequence: target.sequence, timeline: target.timeline, issued_at_ns: target.capture_monotonic_ns, ttl_ns: target.ttl_ns }); }, Err(e) if e.kind() == ErrorKind::WouldBlock => {}, Err(e) => return Err(e.into()) }
        let now = monotonic_ns(); let _tick = core.tick(&mut plant, pending.take(), now).map_err(|e| format!("Duck control tick: {e:?}"))?; plant.advance_physics_step();
        next += PERIOD; thread::sleep(next.saturating_duration_since(Instant::now()));
        if next.elapsed().is_zero() { let _ = &OPEN_DUCK_RUNTIME_SOCKET; }
    }
}
