# Soma Status

Updated: 2026-08-29

## Current State

The fixed Reachy Mini simulation path is executable and passes its acceptance
scenario. It includes the shared `ControlCore`, pinned MuJoCo Plant, minimal
Protobuf schema, loopback Zenoh runtime, bounded exclusive Unix datagram IPC,
and thin Python scenario client. An optional feature-gated observer provides
lossy read-only MuJoCo and Rerun views without changing the headless path.
The simulation also has a discrete terminal command session for body yaw and
mirrored antenna motion through that same command path.

A reproducible simulation showcase command now drives a deterministic
12-second body-yaw, antenna, and official-kinematics head choreography through
the authoritative command path. It captures one continuous 14-second
fixed-camera MuJoCo VP9 WebM and synchronized Rerun archive, including the
final TTL hold, reset, and stale-timeline rejection evidence, then verifies and
packages them as a static report. The README carries the stable poster and the
live Pages link.

The standalone native N0 probe is implemented and remains strictly read-only.
No native Plant, bus worker, torque enable, or physical motion is implemented.

## Hardware Blocker And Interim Action

N0 cannot run because no Reachy Mini Lite CH343 device (`1a86:55d3`) is
connected. Hardware work remains blocked and no N0/N1 gate is bypassed.

While waiting for the device, the bounded
[official simulation comparison plan](docs/plans/official-simulation-comparison-plan.md)
has completed its simulation-only D0-D3 slice. The private owning command
builds an isolated, pinned Reachy Mini v1.9.0 image with Git LFS assets, runs
Soma and the official daemon separately, records capability-labelled JSONL,
and generates deterministic motion/timing metrics. The representative report
is [here](docs/measurements/official-simulation-representative-report.md);
generated raw evidence remains under ignored `output/`. Hardware N0/N1 gates
remain unchanged.

A follow-up [fixed case suite plan](docs/plans/official-simulation-case-suite-plan.md)
is implemented. The private owning command runs three predeclared yaw,
antenna, and combined cases without outcome-based selection, preserves
per-case lifecycle evidence, and validates two clean runs. The committed
[suite report](docs/measurements/official-simulation-case-suite-report.md)
contains the representative multi-case evidence; generated raw evidence
remains under ignored `output/`.

A cadence audit subsequently found that the original Soma comparison advanced
one 2 ms MuJoCo step per 20 ms control period, so simulation time ran at one
tenth of wall time. The Plant now validates an integral physics schedule at
startup and advances ten substeps per Reachy control period. Product simulation
commands use Rust release binaries. Two corrected clean suites show Soma and
the official backend both settling the commanded yaw and antenna axes in about
80 ms with nearly identical RMS errors. The full root-cause and before/after
evidence is in the [cadence correction report](docs/measurements/official-simulation-cadence-correction-report.md).

When the reviewed device arrives, stop the official daemon before running:

```bash
cargo run --bin soma-reachy-probe --
```

If N0 passes, stop for explicit N1 physical-actuation authorization before any
register write, torque enable, or motion work.

## What Runs Now

```bash
scripts/cargo-mujoco test --workspace
cd python && uv sync && cd ..
scripts/run-sim-scenario
scripts/run-sim-scenario --visualize  # requires a desktop display
scripts/run-sim-teleop
scripts/run-sim-teleop --visualize    # requires a desktop display
scripts/run-sim-teleop --keys ADQE    # bounded integration route
scripts/build-sim-showcase output/simulation-showcase
```

The scenario verifies typed state delivery, actuator movement, TTL expiry to
measured-position hold, reset timeline change, and old-timeline rejection.
Visual mode adds paced MuJoCo motion and a Rerun evidence dashboard. Automated
Xvfb startup, camera interaction, independent sink closure, scenario
continuation, and teardown pass. Live human review of motion and dashboard
readability remains the final gate.

The showcase build creates a new output directory containing `index.html`, a
MuJoCo poster and motion video, `evidence.rrd`, and machine-readable provenance.
Its verifier requires visible 14-second media, all nine requested Rerun streams,
continuous robot transforms through the acceptance tail, and explicit
TTL/reset/rejection evidence before reporting success.

## Active Work

- Completed headless simulation slice: [Open Duck Mini walk policy](docs/plans/open-duck-mini-walk-policy.md). The fixed Duck Plant, isolated transport, Python ONNX client, lineage/fault evidence, and supervised launcher pass Stage 4. Two direct and two process runs are stable; delay and stall cases preserve deadline/hold behavior. The frozen published `BEST_WALK_ONNX.onnx` remains the default. A single composite user-facing rollout with synchronized MJCF/Rerun visualization is active in [the Open Duck showcase plan](docs/plans/open-duck-showcase.md); the frozen straight-walk case remains its internal regression gate.
- Active comparison/hardware state: [bootstrap capsule](docs/status/active/bootstrap.md)
- Preflighted next slice: [official simulation fixed case suite](docs/plans/official-simulation-case-suite-plan.md)
- Completed suite evidence: [official simulation fixed case suite report](docs/measurements/official-simulation-case-suite-report.md)
- Corrected comparison evidence: [simulation cadence correction report](docs/measurements/official-simulation-cadence-correction-report.md)
- Preflighted interim plan: [official simulation comparison](docs/plans/official-simulation-comparison-plan.md)
- Approved hardware plan: [bootstrap plan](docs/plans/bootstrap-plan.md)
- Implemented, manual display review pending: [simulation visualization plan](docs/plans/simulation-visualization-plan.md)
- Implemented, desktop interaction review pending: [simulation command-session plan](docs/plans/reachy-command-session.md)
- Completed and published; enriched choreography locally verified for the next `main` deployment: [simulation showcase plan](docs/plans/simulation-showcase.md)
- Active local review: [Open Duck composite showcase](docs/plans/open-duck-showcase.md)

Open Duck Mini is the only additional robot profile in active scope, and only
as one fixed MuJoCo policy qualification. Generic manifests, Duck hardware, a
third robot, camera/audio paths, an App framework, and additional simulator
backends remain out of scope.
