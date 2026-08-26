# Soma Status

Updated: 2026-08-25

## Current State

The fixed Reachy Mini simulation path is executable and passes its acceptance
scenario. It includes the shared `ControlCore`, pinned MuJoCo Plant, minimal
Protobuf schema, loopback Zenoh runtime, bounded exclusive Unix datagram IPC,
and thin Python scenario client. An optional feature-gated observer provides
lossy read-only MuJoCo and Rerun views without changing the headless path.
The simulation also has a discrete terminal command session for body yaw and
mirrored antenna motion through that same command path.

The standalone native N0 probe is implemented and remains strictly read-only.
No native Plant, bus worker, torque enable, or physical motion is implemented.

## Blocker And Next Action

N0 cannot run because no Reachy Mini Lite CH343 device (`1a86:55d3`) is
connected. Connect the reviewed device, stop the official daemon, and run:

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
```

The scenario verifies typed state delivery, actuator movement, TTL expiry to
measured-position hold, reset timeline change, and old-timeline rejection.
Visual mode adds paced MuJoCo motion and a Rerun evidence dashboard. Automated
Xvfb startup, camera interaction, independent sink closure, scenario
continuation, and teardown pass. Live human review of motion and dashboard
readability remains the final gate.

## Active Work

- Hardware bootstrap state: [bootstrap capsule](docs/status/active/bootstrap.md)
- Approved hardware plan: [bootstrap plan](docs/plans/bootstrap-plan.md)
- Implemented, manual display review pending: [simulation visualization plan](docs/plans/simulation-visualization-plan.md)
- Implemented, desktop interaction review pending: [simulation command-session plan](docs/plans/reachy-command-session.md)

No other robot profile, generic manifest, camera/audio path, or deferred
production subsystem is active scope.
