# Soma Status

Updated: 2026-08-24

## Current State

The fixed Reachy Mini simulation path is executable and passes its acceptance
scenario. It includes the shared `ControlCore`, pinned MuJoCo Plant, minimal
Protobuf schema, loopback Zenoh runtime, bounded exclusive Unix datagram IPC,
and thin Python scenario client.

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
```

The scenario verifies typed state delivery, actuator movement, TTL expiry to
measured-position hold, reset timeline change, and old-timeline rejection.

## Active Work

- Hardware bootstrap state: [bootstrap capsule](docs/status/active/bootstrap.md)
- Approved hardware plan: [bootstrap plan](docs/plans/bootstrap-plan.md)
- Optional, not yet approved: [simulation visualization plan](docs/plans/simulation-visualization-plan.md)

No other robot profile, generic manifest, camera/audio path, or deferred
production subsystem is active scope.
