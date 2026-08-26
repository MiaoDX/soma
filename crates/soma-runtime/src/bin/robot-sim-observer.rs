use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use prost::Message;
use rerun::blueprint::{Blueprint, StateTimelineView, TextLogView, TimeSeriesView, Vertical};
use rerun::{RecordingStreamBuilder, Scalars, StateChange, TextLog};
use soma_protocol::v1::{self, rt_request};
use soma_runtime::{bind_owned_datagram, monotonic_ns, COMMAND_KEY, STATE_KEY};
#[cfg(feature = "sim-showcase")]
use soma_sim::ReachySimRenderer;
use soma_sim::{
    ReachySimSnapshot, ReachySimViewer, ACTUATOR_NAMES, REACHY_SCENE_PATH, SNAPSHOT_LEN,
};
use zenoh::Config;

enum Evidence {
    Snapshot(Box<ReachySimSnapshot>),
    Command {
        received_ns: u64,
        request: v1::RtRequest,
    },
    State(v1::ActuatorState),
    Malformed {
        observed_ns: u64,
        kind: &'static str,
    },
    Drops {
        observed_ns: u64,
        snapshot: u64,
        rerun: u64,
    },
}

#[derive(Debug, PartialEq)]
enum LogValue {
    Scalar(f64),
    State(String),
    Text(String),
}

#[derive(Debug, PartialEq)]
struct LogRecord {
    time_ns: u64,
    path: String,
    value: LogValue,
}

fn scalar(time_ns: u64, path: impl Into<String>, value: f64) -> LogRecord {
    LogRecord {
        time_ns,
        path: path.into(),
        value: LogValue::Scalar(value),
    }
}

fn state_record(time_ns: u64, path: impl Into<String>, value: impl Into<String>) -> LogRecord {
    LogRecord {
        time_ns,
        path: path.into(),
        value: LogValue::State(value.into()),
    }
}

fn text(time_ns: u64, value: impl Into<String>) -> LogRecord {
    LogRecord {
        time_ns,
        path: "events".into(),
        value: LogValue::Text(value.into()),
    }
}

struct SocketCleanup(String);
impl Drop for SocketCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
        let _ = std::fs::remove_file(format!("{}.lock", self.0));
    }
}

fn accept_snapshot(
    previous: &mut Option<(u64, u64)>,
    snapshot: ReachySimSnapshot,
) -> Option<ReachySimSnapshot> {
    if let Some((timeline, sequence)) = previous {
        if *timeline == snapshot.timeline && snapshot.sequence <= *sequence {
            return None;
        }
    }
    *previous = Some((snapshot.timeline, snapshot.sequence));
    Some(snapshot)
}

fn enum_name<T: TryFrom<i32> + std::fmt::Debug>(value: i32) -> String {
    T::try_from(value)
        .map(|value| format!("{value:?}"))
        .unwrap_or_else(|_| format!("Unknown({value})"))
}

fn map_state(state: &v1::ActuatorState) -> Vec<LogRecord> {
    let time = state.capture_monotonic_ns;
    let mut records = Vec::with_capacity(15);
    for (name, value) in ACTUATOR_NAMES.iter().zip(&state.positions_rad) {
        records.push(scalar(
            time,
            format!("actuators/{name}/measured"),
            *value as f64,
        ));
    }
    records.push(scalar(
        time,
        "state/age_ms",
        state.state_age_ns as f64 / 1e6,
    ));
    records.push(state_record(
        time,
        "state/applied_source",
        enum_name::<v1::AppliedSource>(state.applied_source),
    ));
    records.push(state_record(
        time,
        "state/disposition",
        enum_name::<v1::CommandDisposition>(state.command_disposition),
    ));
    records.push(state_record(
        time,
        "state/rejection_reason",
        enum_name::<v1::RejectionReason>(state.rejection_reason),
    ));
    records.push(state_record(
        time,
        "state/health",
        enum_name::<v1::PlantHealth>(state.health),
    ));
    if state.expiry_transition {
        records.push(text(time, "TTL expiry: measured-position hold applied"));
    }
    if state.command_disposition == v1::CommandDisposition::Rejected as i32 {
        records.push(text(
            time,
            format!(
                "command rejected: {}",
                enum_name::<v1::RejectionReason>(state.rejection_reason)
            ),
        ));
    }
    records
}

fn map_command(received_ns: u64, request: &v1::RtRequest) -> Vec<LogRecord> {
    let mut records = Vec::with_capacity(11);
    match &request.request {
        Some(rt_request::Request::Target(target)) => {
            for (name, value) in ACTUATOR_NAMES.iter().zip(&target.positions_rad) {
                records.push(scalar(
                    received_ns,
                    format!("actuators/{name}/requested"),
                    *value as f64,
                ));
            }
            records.push(scalar(
                received_ns,
                "command/ttl_ms",
                target.ttl_ns as f64 / 1e6,
            ));
            records.push(text(
                received_ns,
                format!(
                    "target sequence={} timeline={}",
                    target.sequence, target.timeline
                ),
            ));
        }
        Some(rt_request::Request::Reset(true)) => {
            records.push(text(received_ns, "reset requested"));
        }
        _ => {}
    }
    records
}

fn elapsed_seconds(start_ns: u64, timestamp_ns: u64) -> f64 {
    timestamp_ns.saturating_sub(start_ns) as f64 / 1e9
}

fn emit(
    rec: &rerun::RecordingStream,
    recording_start_ns: u64,
    records: impl IntoIterator<Item = LogRecord>,
) {
    for record in records {
        rec.set_duration_secs(
            "program_time",
            elapsed_seconds(recording_start_ns, record.time_ns),
        );
        match record.value {
            LogValue::Scalar(value) => {
                let _ = rec.log(record.path, &Scalars::new([value]));
            }
            LogValue::State(value) => {
                let _ = rec.log(record.path, &StateChange::new().with_state([value]));
            }
            LogValue::Text(value) => {
                let _ = rec.log(record.path, &TextLog::new(value));
            }
        }
    }
}

enum RerunDestination {
    Grpc(String),
    File(PathBuf),
}

fn start_rerun(
    destination: RerunDestination,
) -> Result<(SyncSender<Evidence>, JoinHandle<()>), Box<dyn std::error::Error>> {
    let recording_start_ns = monotonic_ns();
    let blueprint = Blueprint::new(Vertical::new([
        TimeSeriesView::new("Body yaw")
            .with_origin("actuators/yaw_body")
            .into(),
        TimeSeriesView::new("Stewart motors")
            .with_contents(["actuators/stewart_*/**"])
            .into(),
        TimeSeriesView::new("Antennae")
            .with_contents(["actuators/*_antenna/**"])
            .into(),
        TimeSeriesView::new("Timing and observer integrity")
            .with_contents([
                "simulation/**",
                "state/age_ms",
                "command/ttl_ms",
                "observer/**",
            ])
            .into(),
        StateTimelineView::new("Control state")
            .with_origin("state")
            .into(),
        StateTimelineView::new("Plant timeline")
            .with_origin("simulation/timeline")
            .into(),
        TextLogView::new("Events").with_origin("events").into(),
    ]));
    let builder = RecordingStreamBuilder::new("soma_simulation").with_blueprint(blueprint);
    let rec = match destination {
        RerunDestination::Grpc(endpoint) => builder.connect_grpc_opts(endpoint)?,
        RerunDestination::File(path) => builder.save(path)?,
    };
    let (sender, receiver) = mpsc::sync_channel::<Evidence>(32);
    let worker = thread::spawn(move || {
        let mut last_timeline = None;
        let mut last_drops = (0, 0);
        while let Ok(evidence) = receiver.recv() {
            match evidence {
                Evidence::Snapshot(snapshot) => {
                    rec.set_duration_secs(
                        "program_time",
                        elapsed_seconds(recording_start_ns, snapshot.capture_monotonic_ns),
                    );
                    let _ = rec.log(
                        "simulation/time_seconds",
                        &Scalars::new([snapshot.simulation_time_ns as f64 / 1e9]),
                    );
                    let _ = rec.log(
                        "simulation/timeline",
                        &StateChange::new().with_state([snapshot.timeline.to_string()]),
                    );
                    let _ = rec.log(
                        "simulation/control_sequence",
                        &Scalars::new([snapshot.sequence as f64]),
                    );
                }
                Evidence::Command {
                    received_ns,
                    request,
                } => emit(&rec, recording_start_ns, map_command(received_ns, &request)),
                Evidence::State(state) => {
                    if last_timeline
                        .replace(state.timeline)
                        .is_some_and(|old| old != state.timeline)
                    {
                        rec.set_duration_secs(
                            "program_time",
                            elapsed_seconds(recording_start_ns, state.capture_monotonic_ns),
                        );
                        let _ = rec.log(
                            "events",
                            &TextLog::new(format!("Plant timeline reset to {}", state.timeline)),
                        );
                    }
                    emit(&rec, recording_start_ns, map_state(&state))
                }
                Evidence::Malformed { observed_ns, kind } => {
                    emit(
                        &rec,
                        recording_start_ns,
                        [text(observed_ns, format!("malformed {kind} observation"))],
                    );
                }
                Evidence::Drops {
                    observed_ns,
                    snapshot,
                    rerun,
                } => {
                    emit(
                        &rec,
                        recording_start_ns,
                        [
                            scalar(observed_ns, "observer/snapshot_drops", snapshot as f64),
                            scalar(observed_ns, "observer/rerun_queue_drops", rerun as f64),
                        ],
                    );
                    if (snapshot, rerun) != last_drops && (snapshot > 0 || rerun > 0) {
                        emit(
                            &rec,
                            recording_start_ns,
                            [text(observed_ns, format!("observer loss: snapshot_drops={snapshot} rerun_queue_drops={rerun}"))],
                        );
                        last_drops = (snapshot, rerun);
                    }
                }
            }
        }
        let _ = rec.flush_blocking();
    });
    Ok((sender, worker))
}

fn start_zenoh(
    sender: SyncSender<Evidence>,
) -> (mpsc::Receiver<()>, Arc<AtomicBool>, JoinHandle<()>) {
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let running = Arc::new(AtomicBool::new(true));
    let worker_running = running.clone();
    let worker = thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("create observer Zenoh runtime");
        runtime.block_on(async move {
            let config = Config::from_json5(r#"{ mode: "client", connect: { endpoints: ["tcp/127.0.0.1:7447"] }, scouting: { multicast: { enabled: false } } }"#).unwrap();
            let session = match zenoh::open(config).await { Ok(session) => session, Err(_) => return };
            let commands = match session.declare_subscriber(COMMAND_KEY).await { Ok(value) => value, Err(_) => return };
            let states = match session.declare_subscriber(STATE_KEY).await { Ok(value) => value, Err(_) => return };
            let _ = ready_tx.send(());
            while worker_running.load(Ordering::Relaxed) {
                tokio::select! {
                    sample = commands.recv_async() => if let Ok(sample) = sample { let observed_ns = monotonic_ns(); let bytes = sample.payload().to_bytes(); let event = v1::RtRequest::decode(bytes.as_ref()).map(|request| Evidence::Command { received_ns: observed_ns, request }).unwrap_or(Evidence::Malformed { observed_ns, kind: "command" }); let _ = sender.try_send(event); },
                    sample = states.recv_async() => if let Ok(sample) = sample { let observed_ns = monotonic_ns(); let bytes = sample.payload().to_bytes(); let event = v1::ActuatorState::decode(bytes.as_ref()).map(Evidence::State).unwrap_or(Evidence::Malformed { observed_ns, kind: "state" }); let _ = sender.try_send(event); },
                    _ = tokio::time::sleep(Duration::from_millis(20)) => {},
                }
            }
        });
    });
    (ready_rx, running, worker)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (socket_path, destination, frames_dir) = match args.as_slice() {
        [socket_flag, path, rerun_flag, endpoint]
            if socket_flag == "--snapshot-socket" && rerun_flag == "--rerun-endpoint" =>
        {
            (path.clone(), RerunDestination::Grpc(endpoint.clone()), None)
        }
        #[cfg(feature = "sim-showcase")]
        [socket_flag, path, rerun_flag, archive, frames_flag, frames]
            if socket_flag == "--snapshot-socket"
                && rerun_flag == "--rerun-file"
                && frames_flag == "--frames-dir" =>
        {
            std::fs::create_dir_all(frames)?;
            (
                path.clone(),
                RerunDestination::File(PathBuf::from(archive)),
                Some(PathBuf::from(frames)),
            )
        }
        _ => return Err("usage: robot-sim-observer --snapshot-socket PATH --rerun-endpoint URL\n       robot-sim-observer --snapshot-socket PATH --rerun-file FILE --frames-dir DIR".into()),
    };
    let readiness_path = format!("{socket_path}.ready");
    let owned = bind_owned_datagram(&socket_path)
        .map_err(|error| format!("bind showcase snapshot socket {socket_path}: {error}"))?;
    let _cleanup = SocketCleanup(socket_path);
    owned.socket.set_nonblocking(true)?;
    let (rerun, rerun_worker) = start_rerun(destination)?;
    let (zenoh_ready, zenoh_running, zenoh_worker) = start_zenoh(rerun.clone());
    zenoh_ready
        .recv_timeout(Duration::from_secs(10))
        .map_err(|_| "observer Zenoh subscriptions were not ready")?;
    let mut viewer = if frames_dir.is_none() {
        Some(
            ReachySimViewer::launch(REACHY_SCENE_PATH)
                .map_err(|error| format!("launch MuJoCo viewer: {error:?}"))?,
        )
    } else {
        None
    };
    #[cfg(feature = "sim-showcase")]
    let mut renderer = frames_dir
        .as_ref()
        .map(|_| ReachySimRenderer::launch(REACHY_SCENE_PATH))
        .transpose()
        .map_err(|error| format!("launch EGL MuJoCo renderer: {error:?}"))?;
    let mut first_capture_ns = None;
    let mut last_frame_ns = 0_u64;
    let mut frame_index = 0_u32;
    std::fs::write(&readiness_path, b"ready")?;
    let _readiness_cleanup = SocketCleanup(readiness_path);
    let mut buffer = [0_u8; SNAPSHOT_LEN + 1];
    let mut previous = None;
    let mut rerun_drops = 0_u64;
    let mut snapshot_drops = 0_u64;

    loop {
        let mut newest = None;
        loop {
            match owned.socket.recv(&mut buffer) {
                Ok(size) => {
                    if let Ok(snapshot) = ReachySimSnapshot::decode(&buffer[..size]) {
                        if let Some((timeline, sequence)) = previous {
                            if timeline == snapshot.timeline && snapshot.sequence > sequence + 1 {
                                snapshot_drops += snapshot.sequence - sequence - 1;
                            }
                        }
                        if let Some(snapshot) = accept_snapshot(&mut previous, snapshot) {
                            newest = Some(snapshot);
                        } else {
                            snapshot_drops += 1;
                        }
                    } else {
                        snapshot_drops += 1;
                        let _ = rerun.try_send(Evidence::Malformed {
                            observed_ns: monotonic_ns(),
                            kind: "snapshot",
                        });
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) => return Err(error.into()),
            }
        }
        if let Some(snapshot) = newest {
            first_capture_ns.get_or_insert(snapshot.capture_monotonic_ns);
            if let Some(active) = &mut viewer {
                active
                    .render(&snapshot)
                    .map_err(|error| format!("render MuJoCo: {error:?}"))?;
            }
            #[cfg(feature = "sim-showcase")]
            if let (Some(active), Some(directory)) = (&mut renderer, &frames_dir) {
                if frame_index == 0
                    || snapshot.capture_monotonic_ns.saturating_sub(last_frame_ns) >= 66_000_000
                {
                    active
                        .render_png(
                            &snapshot,
                            directory.join(format!("frame-{frame_index:04}.png")),
                        )
                        .map_err(|error| format!("render showcase frame: {error:?}"))?;
                    last_frame_ns = snapshot.capture_monotonic_ns;
                    frame_index += 1;
                }
            }
            let capture_complete = frames_dir.is_some()
                && first_capture_ns.is_some_and(|start| {
                    snapshot.capture_monotonic_ns.saturating_sub(start) >= 6_000_000_000
                });
            if let Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) =
                rerun.try_send(Evidence::Snapshot(Box::new(snapshot)))
            {
                rerun_drops += 1;
            }
            let _ = rerun.try_send(Evidence::Drops {
                observed_ns: monotonic_ns(),
                snapshot: snapshot_drops,
                rerun: rerun_drops,
            });
            if capture_complete {
                break;
            }
        }
        if let Some(active) = &mut viewer {
            if !active.running() {
                viewer = None;
            }
        }
        if rerun_drops > 0 && rerun_drops % 100 == 1 {
            eprintln!("Rerun queue drops: {rerun_drops}");
        }
        thread::sleep(Duration::from_millis(10));
    }
    zenoh_running.store(false, Ordering::Relaxed);
    zenoh_worker.join().map_err(|_| "Zenoh observer panicked")?;
    drop(rerun);
    rerun_worker.join().map_err(|_| "Rerun writer panicked")?;
    if frames_dir.is_some() && frame_index < 2 {
        return Err("showcase capture produced fewer than two frames".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(timeline: u64, sequence: u64) -> ReachySimSnapshot {
        ReachySimSnapshot {
            timeline,
            sequence,
            simulation_time_ns: 0,
            capture_monotonic_ns: 0,
            qpos: [0.0; 37],
            qvel: [0.0; 30],
        }
    }

    #[test]
    fn rejects_stale_sequence_and_accepts_timeline_change() {
        let mut previous = None;
        assert!(accept_snapshot(&mut previous, snapshot(1, 2)).is_some());
        assert!(accept_snapshot(&mut previous, snapshot(1, 2)).is_none());
        assert!(accept_snapshot(&mut previous, snapshot(1, 1)).is_none());
        assert!(accept_snapshot(&mut previous, snapshot(2, 1)).is_some());
    }

    #[test]
    fn rerun_program_time_starts_at_zero_instead_of_host_uptime() {
        assert_eq!(elapsed_seconds(20_000_000_000, 20_000_000_000), 0.0);
        assert_eq!(elapsed_seconds(20_000_000_000, 21_500_000_000), 1.5);
        assert_eq!(elapsed_seconds(20_000_000_000, 19_999_999_999), 0.0);
    }

    #[test]
    fn command_mapping_uses_observer_time_and_fixed_requested_paths() {
        let request = v1::RtRequest {
            request: Some(rt_request::Request::Target(v1::ActuatorTarget {
                positions_rad: (0..9).map(|value| value as f32).collect(),
                sequence: 7,
                timeline: 3,
                issued_at_ns: 1,
                ttl_ns: 250_000_000,
            })),
        };
        let records = map_command(900, &request);
        assert_eq!(records.len(), 11);
        assert_eq!(records[0], scalar(900, "actuators/yaw_body/requested", 0.0));
        assert_eq!(
            records[8],
            scalar(900, "actuators/left_antenna/requested", 8.0)
        );
        assert_eq!(records[9], scalar(900, "command/ttl_ms", 250.0));
        assert_eq!(records[10], text(900, "target sequence=7 timeline=3"));
    }

    #[test]
    fn state_mapping_uses_producer_time_and_surfaces_categorical_events() {
        let state_message = v1::ActuatorState {
            positions_rad: vec![0.5; 9],
            sequence: 8,
            timeline: 3,
            timestamp_ns: 10,
            state_age_ns: 2_500_000,
            applied_source: v1::AppliedSource::MeasuredPositionHold as i32,
            applied_sequence: 7,
            expiry_transition: true,
            command_disposition: v1::CommandDisposition::Rejected as i32,
            rejection_reason: v1::RejectionReason::Timeline as i32,
            health: v1::PlantHealth::Healthy as i32,
            capture_monotonic_ns: 1_234,
        };
        let records = map_state(&state_message);
        assert!(records.contains(&scalar(1_234, "state/age_ms", 2.5)));
        assert!(records.contains(&state_record(
            1_234,
            "state/applied_source",
            "MeasuredPositionHold"
        )));
        assert!(records.contains(&state_record(1_234, "state/disposition", "Rejected")));
        assert!(records.contains(&state_record(1_234, "state/rejection_reason", "Timeline")));
        assert!(records.contains(&state_record(1_234, "state/health", "Healthy")));
        assert!(records.contains(&text(1_234, "TTL expiry: measured-position hold applied")));
        assert!(records.contains(&text(1_234, "command rejected: Timeline")));
    }
}
