# MuJoCo And Rerun Visualization Plan

> Status: Implemented; deterministic, headless, Xvfb, camera-interaction, and
> independent-sink-closure gates pass. Manual desktop motion/dashboard
> readability review remains pending, 2026-08-24.
> This is an optional simulation developer surface. It does not resume or alter
> the hardware-gated bootstrap work in `docs/status/active/bootstrap.md`.

## Goal

Add one command that runs the existing Python -> Zenoh -> `robot-runtime` ->
`robot-rt` -> `ReachySimPlant` scenario with two complementary live views:

1. a MuJoCo window showing the Reachy model in the generalized state produced
   by the authoritative simulation;
2. a Rerun dashboard showing time-aligned command, measured state, TTL,
   disposition, applied source, health, timeline, latency, and observation-drop
   evidence.

The visualization path must be observational. Its absence, closure, slowness,
or failure must not stop, command, reset, block, or change the scheduling
semantics of the simulation/control path. The existing headless scenario remains
the correctness oracle and default command.

## Architecture Decision

Add one separate, optional `robot-sim-observer` process with one simulation
snapshot ingress and two bounded visualization sinks. `robot-rt` remains the
sole owner of authoritative `MjData`; neither MuJoCo UI nor Rerun can write back.

```text
Python scenario -------- requested command --------+
        |                                          |
        v                                          v
      Zenoh -> robot-runtime -> robot-rt -> ReachySimPlant
        |                              |
        | command + public state       | private, lossy generalized-state tap
        |                              |
        +--------------+---------------+
                       v
              robot-sim-observer
              - one snapshot receiver
              - Zenoh command/state subscriber
              - bounded latest/event fan-out
                    /             \
                   v               v
          MuJoCo renderer      Rerun RecordingStream
          3D physics view      live dashboard/blueprint
          process main thread  isolated logging worker
```

This boundary is deliberately an observer, not another runtime or simulator
authority. `robot-rt` gains one optional best-effort observation outlet; adding
or removing MuJoCo, Rerun, or a later developer sink does not add another
control-process transport.

### Why two views?

MuJoCo is the right view for geometry, constraints, passive joints, camera
navigation, and visible motion. Rerun is the right view for temporal comparison,
state lanes, events, and correlated diagnostics. Rebuilding the full Reachy
mesh scene in Rerun would duplicate MuJoCo without improving control evidence;
using only MuJoCo would hide why a motion was accepted, held, reset, or rejected.

### Why not render inside `robot-rt`?

The pinned `mujoco-rs` passive viewer renders on the process main thread and
synchronizes physics through a shared mutex. Rendering, UI work, asset updates,
and vsync may hold or delay that mutex. Moving the control loop to a worker
thread would couple the 20 ms simulation loop to GUI behavior. Rerun SDK calls
also have no place in that loop. A separate observer process gives both tools a
testable noninterference boundary.

## Evidence And Dependency Policy

- Pin the optional Rerun Rust SDK and viewer integration to `0.36.2`, the
  upstream release published 2026-08-21, rather than accepting an unbounded
  semver upgrade during this milestone.
- Declare the Rust dependency with exact version, default features disabled,
  and only the `sdk` feature enabled; native Rerun UI code belongs to the
  separately supervised viewer executable.
- Do not rely on an arbitrary `rerun` executable from `PATH`. Add
  `rerun-sdk==0.36.2` as a Python visualization extra, let the scenario script
  explicitly start that exact viewer on an ephemeral loopback port, and have
  the Rust observer connect to it. The script owns its PID and cleanup.
- Rerun's official Rust API provides `RecordingStream`, live viewer spawning,
  scalar time series, `StateChange`, and programmatic Blueprints.
- Rerun calls its live stream a recording, but this milestone does not make an
  `.rrd` stream the Soma flight recorder or a deterministic replay artifact.
  MCAP/incident recording and replay semantics remain separate architecture
  work.

Primary upstream references:

- <https://github.com/rerun-io/rerun/releases/tag/0.36.2>
- <https://github.com/rerun-io/rerun/tree/0.36.2/examples/rust/minimal_options>
- <https://github.com/rerun-io/rerun/tree/0.36.2/examples/rust/blueprint>
- <https://github.com/rerun-io/rerun/tree/0.36.2/examples/rust/state_timeline>
- <https://github.com/rerun-io/rerun/blob/0.36.2/docs/content/concepts/logging-and-ingestion/recordings.md>

## Authority And Visualization Matrix

| Evidence | Source of truth | MuJoCo | Rerun |
| --- | --- | --- | --- |
| Requested target/reset/TTL | public Zenoh command observed on the robot host | no | target curves and command events |
| Accepted/rejected disposition | `ControlCore` result in public state | visible only through resulting pose | state lane and rejection event |
| Applied target vs measured hold | `ControlCore` applied source in public state | resulting pose | state lane and expiry event |
| Measured actuator positions | `ReachySimPlant` public state | pose | nine measured curves |
| Full generalized pose/velocity | private `ReachySimPlant` snapshot | all `qpos`/`qvel` | simulation timing/debug metadata only |
| Health and state age | public state | no | state lane and latency curve |
| Plant reset identity | Plant timeline | immediate pose replacement | timeline/reset event |
| Camera/navigation | local MuJoCo viewer state | yes | no |

The current bootstrap does not implement a separate safety-output stage or a
durable requested/admitted/applied journal. The dashboard must display the
evidence that exists and must not invent missing authority layers.

## Scope

1. Add a private fixed-size `ReachySimSnapshot` containing:
   - wire magic and version;
   - exact `nq`/`nv` dimensions;
   - Plant timeline and control-cycle sequence;
   - simulation time and robot-host monotonic capture time;
   - all `qpos[37]` and `qvel[30]` values as little-endian `f64`.
2. Add a read-only `ReachySimPlant::snapshot()` operation. Keep `MjData`
   private and keep `robot-rt` as its only owner.
3. Add opt-in snapshot publication to `robot-rt`:
   - enabled only by an explicit observation argument;
   - at most one snapshot after each authoritative physics step;
   - a nonblocking Unix datagram send with no retry or queue growth;
   - missing receiver, full buffer, and all observer-channel failures drop the
     observation and never fail the control loop.
4. Add a feature-gated `robot-sim-observer` binary:
   - one exclusively owned private snapshot socket;
   - read-only subscriptions to the existing Zenoh command and state keys;
   - bounded, nonblocking fan-out with explicit drop counters;
   - MuJoCo rendering on the process main thread;
   - Rerun initialization before the render loop and logging on an isolated
     worker so one visualization sink cannot stall the other;
   - an explicit loopback Rerun endpoint supplied by the owning script, never
     implicit `PATH` discovery or a detached child process.
5. Add a default Rerun Blueprint with useful views rather than relying on
   manually arranged panels.
6. Extend `scripts/run-sim-scenario` with `--visualize`, which launches both
   MuJoCo and Rerun. Without the flag, build and behavior remain headless and
   unchanged.
7. Add paced visual mode to the existing Python scenario so motion is readable:
   initial pose, yaw motion, TTL hold, reset, and rejection remain visible for
   bounded intervals without changing the assertions or headless timing.
8. Document the commands, authority boundaries, fidelity limits, and expected
   visual result.

Implementation ownership remains narrow:

- `crates/soma-sim`: snapshot type/codec, read-only Plant snapshot, and MuJoCo
  rendering behind a `viewer` feature;
- `crates/soma-runtime`: observer socket/lifecycle, `robot-rt` publication,
  Zenoh observation, Rerun mapping/Blueprint, and `robot-sim-observer` behind a
  `sim-visualization` feature that enables `soma-sim/viewer` and pins the Rerun
  Rust SDK with only the required SDK features;
- `python/pyproject.toml` and `python/uv.lock`: optional `visualization` extra
  containing the matching `rerun-sdk==0.36.2` viewer distribution;
- `python/soma_client/scenario.py`: optional bounded visual pacing only;
- `scripts/run-sim-scenario`: optional process orchestration only.

No new crate, public Protobuf message, public Zenoh key, simulator service,
generic renderer trait, or general observability framework is justified for
this fixed local milestone.

## Expected Visual Result

### MuJoCo window

- complete Reachy Mini model, floor, lighting, and existing scene camera;
- local orbit, pan, and zoom controls that affect only the camera;
- initial pose, visible `yaw_body` motion, TTL measured-position hold, and an
  immediate reset to the new timeline without cross-reset interpolation;
- no actuator sliders, perturbation, pause, step, reset, or control write-back.

### Rerun dashboard

Use the robot-host monotonic clock as the common temporal axis. Snapshot and
state messages carry producer capture time; requested commands use observer
receive time in the same host clock domain.

The default Blueprint contains:

1. requested and measured actuator positions, grouped into body yaw, six
   Stewart motors, and two antennae;
2. state age and requested TTL duration;
3. state lanes for applied source, command disposition, rejection reason, and
   Plant health;
4. Plant timeline and simulation-time reset evidence;
5. text/event rows for target, expiry transition, reset, rejection, malformed
   observation, and sink drops;
6. observer snapshot-drop and Rerun-queue-drop counters.

Raw 37-coordinate `qpos` and 30-coordinate `qvel` are transported for faithful
MuJoCo rendering but are not expanded into 67 default Rerun plots. Selected
debug values may be logged only when they add evidence not already represented
by the named actuator state.

## Non-goals

- Rerun reconstruction of the full MJCF mesh scene or a second 3D robot view;
- authoritative `.rrd`/MCAP flight recording, deterministic replay, retention,
  upload, or catalog integration;
- remote/network visualization or a public snapshot compatibility promise;
- viewer-originated target, reset, stepping, pause, perturbation, or physics
  ownership;
- camera/audio streams, ROS, Foxglove, or a generic simulator API;
- supervisor recovery or stable observer continuity across `robot-rt` restart;
- offscreen rendering as a required CI path;
- any Reachy hardware, N0, N1, native bus, or official-daemon work.

## Execution Plan

### 1. Snapshot Contract

- Define fixed dimensions from the pinned model rather than dynamic vectors.
- Implement checked encode/decode with an exact datagram length, magic,
  version, dimension, and finite-value validation.
- Include producer monotonic capture time so simulation snapshots align with
  the existing public state's `capture_monotonic_ns`.
- Expose only a copied snapshot from `ReachySimPlant`; do not expose `MjData`,
  mutable model access, or viewer-specific handles.
- Unit-test round trip, malformed length, wrong version/dimensions, non-finite
  values, and full `f64` preservation.

Stop if the pinned model no longer has `(nq=37, nv=30, nu=9)`; that is a model
profile change, not a visualization compatibility case.

### 2. Best-effort Producer

- Add an explicit `robot-rt` observation option and nonblocking sender.
- Capture after `plant.step()` so the visible pose is the completed physics
  step, while timeline/sequence identify the producing control cycle.
- Treat every observation send failure as a drop. Do not retry, sleep,
  reconnect synchronously, grow a queue, or return an error from the loop.
- Keep existing public state publication and 20 ms scheduling semantics
  unchanged.
- Test no receiver, closed receiver, and saturated receiver behavior without
  blocking or terminating the producer.

### 3. Simulation Observer And Sink Isolation

- Bind the private snapshot socket with exclusive ownership and clean it up on
  normal exit.
- Decode command/state Protobuf from read-only Zenoh subscriptions; do not
  republish, admit, acknowledge, or mutate those messages.
- Use bounded `try_send`/latest-value paths between ingress and sinks. Never
  wait for MuJoCo rendering or Rerun logging while receiving new observations.
- Count and surface snapshot ingress drops, malformed frames, and Rerun queue
  drops. Do not silently present a complete trace after loss.
- Closing the MuJoCo window disables that sink but leaves Rerun logging active
  until scenario cleanup. Closing/disconnecting Rerun disables that sink but
  leaves MuJoCo rendering active.
- Observer failure or exit never changes scenario success; the script still
  lets the scenario finish, preserves its result, and returns a separate
  nonzero composite result when visualization startup or supervision failed.

### 4. MuJoCo Sink

- Enable the pinned `mujoco-rs` viewer feature only for optional builds.
- Construct and render on the observer process main thread.
- Drain available snapshots, use only the newest valid snapshot, copy
  `qpos`/`qvel`/time into viewer-owned data, call `forward()`, and render.
- Reject non-increasing sequence within one timeline. On timeline change,
  immediately replace visible state and clear prior sequence.
- Do not use the passive viewer's bidirectional merge against authoritative
  Plant data; no UI state can cross the process boundary.

### 5. Rerun Sink

- Pin the Rust SDK to Rerun `0.36.2` behind the optional
  `sim-visualization` feature with default/native-viewer features disabled.
- Connect one `RecordingStream` to the exact loopback endpoint passed by the
  script and send a deterministic Blueprint before live data. Do not call SDK
  `spawn()`, which would search `PATH` and detach a viewer process by default.
- Map command/state/snapshot evidence according to the authority matrix. Use
  stable entity paths and actuator names from the fixed Reachy profile.
- Use robot-host monotonic duration as the primary Rerun timeline, with control
  sequence and Plant timeline logged as explicit evidence rather than treating
  resettable simulation time as a globally monotonic clock.
- Emit categorical changes through `StateChange`, numerical series through
  `Scalars`, and reset/expiry/rejection/drop evidence through structured state
  and text events.
- Treat viewer disconnect, backpressure, or logging error as a sink-local
  failure. Flush only during observer shutdown, never on the ingest path.

### 6. Product Command, Pacing, And Docs

- Parse only `--visualize`; reject unknown scenario-script arguments with usage
  text.
- Add `rerun-sdk==0.36.2` as a non-default Python visualization extra so the
  existing headless `uv sync` does not install or start Rerun.
- In visual mode, sync that extra, select an unused loopback port, verify the
  viewer reports version `0.36.2`, start `uv run --extra visualization rerun`
  on that port, and retain its PID. Start the observer against the explicit
  endpoint, wait for snapshot/Zenoh/Rerun readiness, then start the existing
  runtime processes and paced Python scenario.
- Pace the visual scenario approximately as: initial 1 s, yaw motion 2 s, TTL
  hold 1 s, reset 2 s. Waits happen in the Python client after observed state
  transitions and do not alter TTL or control-loop timing.
- Preserve current headless execution speed and assertions when `--visualize`
  is absent.
- Ensure cleanup terminates only processes it started and reports orphaned
  observer/Rerun processes or sockets as failure evidence.
- Update `README.md` with headless and combined visualization commands, the
  expected two-window result, and the authority/fidelity boundary.

## Acceptance Criteria

1. `scripts/run-sim-scenario` remains headless, fast, and produces the current
   passing result without requiring a display or starting Rerun.
2. `scripts/run-sim-scenario --visualize` opens both views and the paced scenario
   still passes all existing assertions.
3. MuJoCo visibly shows yaw motion, measured-position hold, and immediate reset;
   camera interaction cannot affect simulation state.
4. Rerun shows requested versus measured actuator curves aligned with applied
   source, disposition, expiry, reset/timeline, rejection, health, state age,
   and observation-drop evidence using the supplied Blueprint.
5. The accepted command, TTL expiry transition, reset, and old-timeline
   rejection can each be located in Rerun and correlated with the measured yaw
   curve and MuJoCo motion.
6. Missing observer, early MuJoCo close, Rerun close/disconnect, slow rendering,
   full buffers, malformed datagrams, and either sink's failure cannot block or
   terminate `robot-rt`, `robot-runtime`, or the Python scenario.
7. Closing one visualization sink leaves the other sink and scenario running;
   final cleanup leaves no process or owned socket behind.
8. Reset changes the accepted timeline and neither sink interpolates or labels
   old-timeline evidence as part of the new timeline.
9. Wrong wire version, dimensions, datagram length, non-finite generalized
   state, and stale same-timeline sequence are rejected and counted.
10. No visualization snapshot or Rerun type appears in the public Robot
    Protocol, control core, hardware profile, or public Zenoh API.
11. Documentation states that MuJoCo pixels and Rerun streams are observational
    evidence, while scenario assertions and typed state remain authoritative.
12. Visual mode uses matching Rerun SDK/viewer `0.36.2` versions from pinned
    project dependencies and does not depend on a preinstalled `rerun` command.

## Verification

Required deterministic gates:

```bash
scripts/cargo-mujoco fmt --all -- --check
scripts/cargo-mujoco test --workspace
scripts/cargo-mujoco check --workspace
scripts/cargo-mujoco check -p soma-runtime --features sim-visualization \
  --bin robot-rt --bin robot-sim-observer
```

Required headless product gate:

```bash
scripts/run-sim-scenario
```

Required display-backed product gate:

```bash
scripts/run-sim-scenario --visualize
```

The display-backed gate must confirm both windows, readable pacing, MuJoCo yaw
motion/reset, the required Rerun plots/state lanes/events, independent sink
closure, continued scenario completion, and clean process/socket teardown. An
Xvfb launch may prove GUI startup in automation, but it does not replace human
review of motion and dashboard semantics.

Add focused tests for:

- snapshot codec validation and exact generalized-state preservation;
- latest-valid-frame selection and timeline/sequence transitions;
- missing/closed/full socket behavior;
- command/state-to-Rerun entity mapping, timestamps, categorical states, and
  drop counters without requiring a live Rerun viewer;
- bounded fan-out when a sink is stalled or disconnected;
- headless versus visual pacing argument handling and process cleanup;
- Rerun version mismatch, unavailable loopback port, readiness failure, and
  child-process teardown.

Test assertions and the existing Python scenario remain authoritative
correctness evidence; pixels and Rerun rows are observability evidence.

## Risks And Stop Gates

- Stop if snapshot publication can block, allocate without a bound, or make an
  observer error fatal to the control loop.
- Stop if either sink can backpressure the observer ingress or the other sink.
- Stop if the pinned Rerun SDK cannot provide the Blueprint/state/timeline APIs
  without changing the workspace toolchain or default headless dependency path.
- Stop if the matching optional Rerun viewer cannot be started, supervised, and
  reaped by the scenario script without relying on global installation state.
- Stop if `mujoco-rs` cannot run its event loop on the observer process main
  thread in the supported desktop environment.
- Stop on snapshot/model dimension mismatch rather than truncating or filling
  state.
- Stop if observer timestamps imply stronger causal ordering than the available
  command/state evidence supports.
- Stop if `--visualize` changes default headless behavior or makes scenario
  success depend on a display.
- Stop and reopen scope before adding public/remote compatibility, flight
  recording, replay, catalog/upload, or bidirectional viewer control.

## Parked Work

- Choose `.rrd` export, MCAP flight recording, retention, and replay semantics
  in a separate recording plan; a live Rerun stream is not that decision.
- Consider Rerun 3D transforms or meshes only if the MuJoCo/Rerun split leaves a
  concrete diagnostic gap.
- Consider offscreen screenshots/video only after live viewing is useful.
- Consider public simulator ground truth or remote tooling only with a separate
  protocol and trust-boundary decision.
- Resume the hardware bootstrap only when the Reachy Mini Lite N0 prerequisite
  is physically available; this visualization plan does not change that blocker.

## Definition Of Done

The implementation, focused tests, unchanged headless scenario, combined
display-backed run, default Rerun Blueprint, independent sink failure checks,
README update, and clean teardown all pass. The plan can then be marked complete
in this file without changing the blocked hardware capsule.
