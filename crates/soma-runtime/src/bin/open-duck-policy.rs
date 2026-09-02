use ndarray::Array2;
use ort::{
    session::Session,
    tensor::TensorElementType,
    value::{TensorRef, ValueType},
};
use sha2::{Digest, Sha256};
use soma_runtime::open_duck::{
    decode_state, encode_target, OpenDuckPolicy, OPEN_DUCK_OBSERVATION, OPEN_DUCK_STATE_KEY,
    OPEN_DUCK_TARGET_BYTES, OPEN_DUCK_TARGET_KEY,
};
use std::{
    env, fs,
    path::PathBuf,
    time::{Duration, Instant},
};
use zenoh::Config;

const CHECKPOINT_SHA256: &str = "cb61453a8bcb547ccfdeb4f03ba0fa67ebcf767dcf4aa6e5c9a0d92b302f9b23";

struct Args {
    checkpoint: PathBuf,
    duration: Option<f64>,
    ready_file: Option<PathBuf>,
    vx: f32,
    stall_after: Option<u64>,
    parity_fixture: Option<PathBuf>,
}

fn args() -> Result<Args, String> {
    let mut checkpoint = None;
    let mut duration = None;
    let mut ready_file = None;
    let mut vx = 0.3;
    let mut stall_after = None;
    let mut parity_fixture = None;
    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        let mut value = || it.next().ok_or_else(|| format!("missing value for {arg}"));
        match arg.as_str() {
            "--checkpoint" => checkpoint = Some(PathBuf::from(value()?)),
            "--duration" => duration = Some(value()?.parse().map_err(|_| "invalid duration")?),
            "--ready-file" => ready_file = Some(PathBuf::from(value()?)),
            "--vx" => vx = value()?.parse().map_err(|_| "invalid vx")?,
            "--stall-after" => stall_after = Some(value()?.parse().map_err(|_| "invalid stall-after")?),
            "--parity-fixture" => parity_fixture = Some(PathBuf::from(value()?)),
            "-h" | "--help" => return Err("usage: open-duck-policy --checkpoint PATH [--duration SEC] [--ready-file PATH] [--vx N] [--stall-after N]".into()),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(Args {
        checkpoint: checkpoint.ok_or("--checkpoint is required")?,
        duration,
        ready_file,
        vx,
        stall_after,
        parity_fixture,
    })
}

fn verify_checkpoint(path: &PathBuf) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("read checkpoint: {e}"))?;
    let expected =
        env::var("SOMA_OPEN_DUCK_CHECKPOINT_SHA256").unwrap_or_else(|_| CHECKPOINT_SHA256.into());
    verify_checkpoint_digest(&bytes, &expected)
}

fn verify_checkpoint_digest(bytes: &[u8], expected: &str) -> Result<(), String> {
    let digest = format!("{:x}", Sha256::digest(bytes));
    if digest != expected {
        return Err(format!(
            "checkpoint sha256 mismatch: got {digest}, expected {expected}"
        ));
    }
    Ok(())
}

fn infer(
    session: &mut Session,
    observation: &[f32; OPEN_DUCK_OBSERVATION],
) -> Result<[f32; 14], String> {
    let input = Array2::from_shape_vec((1, OPEN_DUCK_OBSERVATION), observation.to_vec())
        .map_err(|e| e.to_string())?;
    let outputs = session
        .run(ort::inputs!["obs" => TensorRef::from_array_view(&input).map_err(|e| e.to_string())?])
        .map_err(|e| e.to_string())?;
    let (_, values) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|e| e.to_string())?;
    if values.len() != 14 || !values.iter().all(|v| v.is_finite()) {
        return Err("policy output ABI or finite-value mismatch".into());
    }
    values
        .try_into()
        .map_err(|_| "policy output shape mismatch".into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = args().map_err(|e| format!("argument error: {e}"))?;
    verify_checkpoint(&args.checkpoint)?;
    let ort_path = env::var_os("ORT_DYLIB_PATH")
        .ok_or("ORT_DYLIB_PATH must point to the provisioned native ONNX Runtime")?;
    ort::init_from(&ort_path)
        .map_err(|e| {
            format!(
                "load ONNX Runtime from {}: {e}",
                PathBuf::from(&ort_path).display()
            )
        })?
        .commit();
    let mut session = Session::builder()?
        .with_intra_threads(1)?
        .with_inter_threads(1)?
        .commit_from_file(&args.checkpoint)?;
    let input_ok = session.inputs().first().is_some_and(|input| {
        input.name() == "obs"
            && matches!(input.dtype(), ValueType::Tensor { ty: TensorElementType::Float32, shape, .. } if shape.as_ref() == [1, 101])
    });
    let output_ok = session.outputs().first().is_some_and(|output| {
        matches!(output.dtype(), ValueType::Tensor { ty: TensorElementType::Float32, shape, .. } if shape.as_ref() == [1, 14])
    });
    if session.inputs().len() != 1 || session.outputs().len() != 1 || !input_ok || !output_ok {
        return Err("Open Duck model input/output ABI mismatch".into());
    }
    infer(&mut session, &[0.0; OPEN_DUCK_OBSERVATION])
        .map_err(|e| format!("Open Duck model warm-up failed: {e}"))?;
    if let Some(path) = &args.parity_fixture {
        let fixture: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
        let values = fixture["observation"]
            .as_array()
            .ok_or("parity fixture is missing observation")?;
        let observation: [f32; OPEN_DUCK_OBSERVATION] = values
            .iter()
            .map(|value| value.as_f64().map(|value| value as f32))
            .collect::<Option<Vec<_>>>()
            .ok_or("parity fixture observation is not numeric")?
            .try_into()
            .map_err(|_| "parity fixture observation width mismatch")?;
        println!(
            "{}",
            serde_json::to_string(&infer(&mut session, &observation)?)?
        );
        return Ok(());
    }
    let zenoh = zenoh::open(Config::from_json5(r#"{mode:"client",connect:{endpoints:["tcp/127.0.0.1:7448"]},scouting:{multicast:{enabled:false}}}"#)?).await?;
    let subscriber = zenoh.declare_subscriber(OPEN_DUCK_STATE_KEY).await?;
    let publisher = zenoh.declare_publisher(OPEN_DUCK_TARGET_KEY).await?;
    if let Some(path) = &args.ready_file {
        fs::write(path, b"ready\n")?;
    }
    let started = Instant::now();
    let mut policy = OpenDuckPolicy::default();
    let mut emitted = 0_u64;
    let mut states = 0_u64;
    let mut dropped = 0_u64;
    let mut applied = false;
    let mut expiry = false;
    let mut rejected = false;
    let mut max_message_age_ns = 0_u64;
    let mut min_root_height_m = f32::INFINITY;
    let mut max_abs_roll_rad = 0_f32;
    let mut max_abs_pitch_rad = 0_f32;
    let mut first_state_sequence = 0_u64;
    let mut last_state_sequence = 0_u64;
    let mut max_state_sequence_gap = 0_u64;
    let mut max_inference_ns = 0_u64;
    let mut total_inference_ns = 0_u64;
    let mut payload = [0_u8; OPEN_DUCK_TARGET_BYTES];
    loop {
        let sample = subscriber.recv_async().await?;
        let mut latest = sample.payload().to_bytes().to_vec();
        while let Some(sample) = subscriber.try_recv()? {
            dropped = dropped.saturating_add(1);
            latest = sample.payload().to_bytes().to_vec();
        }
        let state = decode_state(&latest).ok_or("invalid Open Duck state payload")?;
        states += 1;
        if first_state_sequence == 0 {
            first_state_sequence = state.sequence;
        }
        if last_state_sequence != 0 {
            max_state_sequence_gap =
                max_state_sequence_gap.max(state.sequence.saturating_sub(last_state_sequence));
        }
        last_state_sequence = state.sequence;
        applied |= state.flags & 1 != 0;
        expiry |= state.flags & 4 != 0;
        rejected |= state.flags & 8 != 0;
        max_message_age_ns = max_message_age_ns.max(state.message_age_ns);
        min_root_height_m = min_root_height_m.min(state.root_height_m);
        max_abs_roll_rad = max_abs_roll_rad.max(state.root_roll_rad.abs());
        max_abs_pitch_rad = max_abs_pitch_rad.max(state.root_pitch_rad.abs());
        if args.stall_after.is_none_or(|limit| emitted < limit) {
            let observation = policy
                .observation(&state, args.vx)
                .ok_or("invalid policy observation")?;
            let inference_started = Instant::now();
            let action = infer(&mut session, &observation)?;
            let inference_ns = inference_started.elapsed().as_nanos() as u64;
            max_inference_ns = max_inference_ns.max(inference_ns);
            total_inference_ns = total_inference_ns.saturating_add(inference_ns);
            let target = policy
                .apply_action(action, &state)
                .ok_or("invalid policy target")?;
            encode_target(&target, &mut payload);
            publisher.put(payload.to_vec()).await?;
            emitted += 1;
        }
        let elapsed = args
            .duration
            .is_some_and(|limit| started.elapsed() >= Duration::from_secs_f64(limit));
        let stall_done = args.stall_after.is_some() && elapsed && applied && expiry;
        if (args.stall_after.is_none() && elapsed && applied) || stall_done {
            let status = if stall_done {
                "stall-complete"
            } else {
                "complete"
            };
            println!(
                concat!(
                "{{\"status\":\"{}\",\"emitted\":{},\"states\":{},\"applied\":{},\"expiry\":{},",
                "\"rejected\":{},\"max_message_age_ns\":{},\"last_requested\":{},",
                "\"last_admitted\":{},\"last_applied\":{},\"min_root_height_m\":{},",
                "\"max_abs_roll_rad\":{},\"max_abs_pitch_rad\":{},\"first_state_sequence\":{},",
                "\"last_state_sequence\":{},\"max_state_sequence_gap\":{},\"max_inference_ns\":{},",
                "\"mean_inference_ns\":{},\"dropped_states\":{},\"runtime_dropped_targets\":{}}}"),
                status,
                emitted,
                states,
                applied,
                expiry,
                rejected,
                max_message_age_ns,
                state.requested_sequence,
                state.admitted_sequence,
                state.applied_sequence,
                min_root_height_m,
                max_abs_roll_rad,
                max_abs_pitch_rad,
                first_state_sequence,
                last_state_sequence,
                max_state_sequence_gap,
                max_inference_ns,
                total_inference_ns / emitted.max(1),
                dropped,
                state.runtime_dropped_targets
            );
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::verify_checkpoint_digest;

    #[test]
    fn checkpoint_digest_is_pinned_and_mismatch_is_attributable() {
        let digest = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(verify_checkpoint_digest(b"abc", digest).is_ok());
        let error = verify_checkpoint_digest(b"changed", digest).unwrap_err();
        assert!(error.contains("checkpoint sha256 mismatch"));
        assert!(error.contains(digest));
    }
}
