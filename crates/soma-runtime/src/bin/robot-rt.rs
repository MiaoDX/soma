use std::io::ErrorKind;
use std::thread;
use std::time::{Duration, Instant};

use prost::Message;
use soma_core::{CommandInput, ControlCore, RejectionReason};
use soma_protocol::v1::{self, rt_request};
use soma_runtime::{
    bind_owned_datagram, decode_target, encode_state_into, monotonic_ns, BestEffortDatagram,
    MAX_MESSAGE_SIZE, RT_SOCKET, RUNTIME_SOCKET,
};
use soma_sim::{ReachySimPlant, REACHY_SCENE_PATH};

const CONTROL_PERIOD: Duration = Duration::from_millis(20);
const MAX_INGRESS_PER_TICK: usize = 8;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let observation = match (args.next().as_deref(), args.next()) {
        (None, None) => None,
        (Some("--observe-socket"), Some(path)) => {
            if args.next().is_some() {
                return Err("usage: robot-rt [--observe-socket PATH]".into());
            }
            Some(BestEffortDatagram::new(path)?)
        }
        _ => return Err("usage: robot-rt [--observe-socket PATH]".into()),
    };
    let owned = bind_owned_datagram(RT_SOCKET)?;
    let socket = &owned.socket;
    socket.set_nonblocking(true)?;

    let mut plant = ReachySimPlant::load(REACHY_SCENE_PATH, CONTROL_PERIOD)
        .map_err(|error| format!("load Reachy model: {error:?}"))?;
    let mut core = ControlCore::new();
    let mut next_tick = Instant::now();
    let mut pending = CommandInput::None;
    let mut buffer = [0_u8; MAX_MESSAGE_SIZE];
    let mut state = v1::ActuatorState {
        positions_rad: Vec::with_capacity(9),
        ..Default::default()
    };
    let mut payload = Vec::with_capacity(MAX_MESSAGE_SIZE);
    let mut _ingress_drops = 0_u64;
    let mut _egress_drops = 0_u64;

    loop {
        let mut drained = 0;
        while drained < MAX_INGRESS_PER_TICK {
            match socket.recv(&mut buffer) {
                Ok(size) => {
                    drained += 1;
                    let Ok(request) = v1::RtRequest::decode(&buffer[..size]) else {
                        pending = CommandInput::Rejected {
                            sequence: 0,
                            reason: RejectionReason::Invalid,
                        };
                        continue;
                    };
                    match request.request {
                        Some(rt_request::Request::Reset(true)) => plant.reset(),
                        Some(rt_request::Request::Target(target)) => {
                            let sequence = target.sequence;
                            pending = match decode_target(target) {
                                Ok(target)
                                    if plant.validate_positions(target.positions_rad).is_ok() =>
                                {
                                    CommandInput::Target(target)
                                }
                                Ok(_) | Err(_) => CommandInput::Rejected {
                                    sequence,
                                    reason: RejectionReason::Invalid,
                                },
                            }
                        }
                        Some(rt_request::Request::IngressRejection(rejection)) => {
                            pending = CommandInput::Rejected {
                                sequence: rejection.sequence,
                                reason: RejectionReason::Invalid,
                            };
                        }
                        Some(rt_request::Request::RuntimeStarted(started)) => {
                            pending = CommandInput::RuntimeStarted {
                                generation: started.generation,
                            };
                        }
                        _ => {}
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(_) => {
                    _egress_drops += 1;
                    break;
                }
            }
        }
        if drained == MAX_INGRESS_PER_TICK {
            _ingress_drops = _ingress_drops.saturating_add(1);
        }

        let now_ns = monotonic_ns();
        let tick = core
            .tick_ingress(
                &mut plant,
                std::mem::replace(&mut pending, CommandInput::None),
                now_ns,
            )
            .map_err(|error| format!("control tick: {error:?}"))?;
        plant.advance_control_period();
        if let Some(observation_socket) = &observation {
            let snapshot = plant.snapshot(monotonic_ns()).encode();
            observation_socket.try_send(&snapshot);
        }
        encode_state_into(&mut state, tick, 0, &mut payload)?;
        match socket.send_to(&payload, RUNTIME_SOCKET) {
            Ok(_) => {}
            Err(error) if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::WouldBlock) => {
                _egress_drops = _egress_drops.saturating_add(1);
            }
            Err(_) => {
                _egress_drops = _egress_drops.saturating_add(1);
            }
        }

        next_tick += CONTROL_PERIOD;
        thread::sleep(next_tick.saturating_duration_since(Instant::now()));
    }
}
