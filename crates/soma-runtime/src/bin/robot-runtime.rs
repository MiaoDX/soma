use std::io::ErrorKind;

use prost::Message;
use soma_protocol::v1;
use soma_runtime::{
    bind_owned_datagram, ingress_rejection, monotonic_ns, stamp_request_received, COMMAND_KEY,
    MAX_MESSAGE_SIZE, RT_SOCKET, RUNTIME_SOCKET, STATE_KEY,
};
use tokio::net::UnixDatagram;
use zenoh::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let owned = bind_owned_datagram(RUNTIME_SOCKET)?;
    owned.socket.set_nonblocking(true)?;
    let socket = UnixDatagram::from_std(owned.socket.try_clone()?)?;
    let config = Config::from_json5(
        r#"{
            mode: "router",
            listen: { endpoints: ["tcp/127.0.0.1:7447"] },
            scouting: { multicast: { enabled: false } }
        }"#,
    )?;
    let session = zenoh::open(config).await?;
    let subscriber = session.declare_subscriber(COMMAND_KEY).await?;
    let mut buffer = [0_u8; MAX_MESSAGE_SIZE];

    loop {
        tokio::select! {
            sample = subscriber.recv_async() => {
                let sample = sample?;
                let bytes = sample.payload().to_bytes();
                if bytes.len() > MAX_MESSAGE_SIZE {
                    let payload = ingress_rejection(0, v1::RejectionReason::Invalid).encode_to_vec();
                    socket.send_to(&payload, RT_SOCKET).await?;
                    continue;
                }
                let mut request = match v1::RtRequest::decode(bytes.as_ref()) {
                    Ok(request) => request,
                    Err(_) => ingress_rejection(0, v1::RejectionReason::Invalid),
                };
                stamp_request_received(&mut request, monotonic_ns());
                let payload = request.encode_to_vec();
                match socket.send_to(&payload, RT_SOCKET).await {
                    Ok(_) => {}
                    Err(error) if error.kind() == ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
            }
            received = socket.recv(&mut buffer) => {
                let size = received?;
                let mut state = v1::ActuatorState::decode(&buffer[..size])?;
                state.state_age_ns = monotonic_ns().saturating_sub(state.capture_monotonic_ns);
                session.put(STATE_KEY, state.encode_to_vec()).await?;
            }
            _ = tokio::signal::ctrl_c() => break,
        }
    }
    Ok(())
}
