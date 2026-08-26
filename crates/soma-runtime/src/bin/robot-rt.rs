use std::io::ErrorKind;
use std::thread;
use std::time::{Duration, Instant};

use prost::Message;
use soma_core::ControlCore;
use soma_protocol::v1::{self, rt_request};
use soma_runtime::{
    bind_owned_datagram, decode_target, encode_state, monotonic_ns, MAX_MESSAGE_SIZE, RT_SOCKET,
    RUNTIME_SOCKET,
};
use soma_sim::{ReachySimPlant, REACHY_SCENE_PATH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let owned = bind_owned_datagram(RT_SOCKET)?;
    let socket = &owned.socket;
    socket.set_nonblocking(true)?;

    let mut plant = ReachySimPlant::load(REACHY_SCENE_PATH)
        .map_err(|error| format!("load Reachy model: {error:?}"))?;
    let mut core = ControlCore::new();
    let period = Duration::from_millis(20);
    let mut next_tick = Instant::now();
    let mut pending = None;
    let mut buffer = [0_u8; MAX_MESSAGE_SIZE];

    loop {
        match socket.recv(&mut buffer) {
            Ok(size) => {
                let Ok(request) = v1::RtRequest::decode(&buffer[..size]) else {
                    continue;
                };
                match request.request {
                    Some(rt_request::Request::Reset(true)) => plant.reset(),
                    Some(rt_request::Request::Target(_)) => {
                        if let Some(target) = decode_target(request, monotonic_ns()) {
                            if plant.validate_positions(target.positions_rad).is_ok() {
                                pending = Some(target);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {}
            Err(error) => return Err(error.into()),
        }

        let now_ns = monotonic_ns();
        let tick = core
            .tick(&mut plant, pending.take(), now_ns)
            .map_err(|error| format!("control tick: {error:?}"))?;
        plant.step();
        let payload = encode_state(tick, 0).encode_to_vec();
        match socket.send_to(&payload, RUNTIME_SOCKET) {
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        next_tick += period;
        thread::sleep(next_tick.saturating_duration_since(Instant::now()));
    }
}
