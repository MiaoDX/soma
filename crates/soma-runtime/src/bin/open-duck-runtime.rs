use soma_runtime::{
    bind_owned_datagram, monotonic_ns,
    open_duck::{
        decode_state, decode_target, encode_state, OPEN_DUCK_RT_SOCKET, OPEN_DUCK_RUNTIME_SOCKET,
        OPEN_DUCK_STATE_BYTES, OPEN_DUCK_STATE_KEY, OPEN_DUCK_TARGET_KEY,
    },
};
use tokio::net::UnixDatagram;
use zenoh::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let owned = bind_owned_datagram(OPEN_DUCK_RUNTIME_SOCKET)?;
    owned.socket.set_nonblocking(true)?;
    let socket = UnixDatagram::from_std(owned.socket.try_clone()?)?;
    let session = zenoh::open(Config::from_json5(r#"{mode:"router",listen:{endpoints:["tcp/127.0.0.1:7448"]},scouting:{multicast:{enabled:false}}}"#)?).await?;
    let subscriber = session.declare_subscriber(OPEN_DUCK_TARGET_KEY).await?;
    let mut buffer = [0_u8; OPEN_DUCK_STATE_BYTES];
    let mut published = [0_u8; OPEN_DUCK_STATE_BYTES];
    let mut dropped_targets = 0_u64;
    loop {
        tokio::select! {
            sample = subscriber.recv_async() => {
                let sample = sample?;
                let bytes = sample.payload().to_bytes();
                let mut latest = decode_target(bytes.as_ref()).map(|_| bytes.to_vec());
                while let Some(sample) = subscriber.try_recv()? {
                    let bytes = sample.payload().to_bytes();
                    if decode_target(bytes.as_ref()).is_some() {
                        if latest.is_some() {
                            dropped_targets = dropped_targets.saturating_add(1);
                        }
                        latest = Some(bytes.to_vec());
                    }
                }
                if let Some(latest) = latest {
                    let _ = socket.send_to(&latest, OPEN_DUCK_RT_SOCKET).await;
                }
            }
            received = socket.recv(&mut buffer) => {
                let size = received?;
                if let Some(mut state) = decode_state(&buffer[..size]) {
                    state.runtime_dropped_targets = dropped_targets;
                    encode_state(&state, &mut published);
                    session.put(OPEN_DUCK_STATE_KEY, published.to_vec()).await?;
                }
            }
            _ = tokio::signal::ctrl_c() => break,
        }
    }
    let _ = monotonic_ns();
    Ok(())
}
