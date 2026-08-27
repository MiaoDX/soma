use soma_runtime::{
    bind_owned_datagram, monotonic_ns,
    open_duck::{
        decode_target, OPEN_DUCK_RT_SOCKET, OPEN_DUCK_RUNTIME_SOCKET, OPEN_DUCK_STATE_KEY,
        OPEN_DUCK_TARGET_BYTES, OPEN_DUCK_TARGET_KEY,
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
    let mut buffer = [0_u8; OPEN_DUCK_TARGET_BYTES];
    loop {
        tokio::select! {
            sample = subscriber.recv_async() => { let sample = sample?; let bytes = sample.payload().to_bytes(); if decode_target(bytes.as_ref()).is_some() { let _ = socket.send_to(bytes.as_ref(), OPEN_DUCK_RT_SOCKET).await; } }
            received = socket.recv(&mut buffer) => { let size = received?; if size == OPEN_DUCK_TARGET_BYTES { session.put(OPEN_DUCK_STATE_KEY, buffer[..size].to_vec()).await?; } }
            _ = tokio::signal::ctrl_c() => break,
        }
    }
    let _ = monotonic_ns();
    Ok(())
}
