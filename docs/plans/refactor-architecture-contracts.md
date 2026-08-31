# Refactor Scope: Architecture Contracts

Status: CONTINUE
Owner: `/root`
Source: user-approved architecture optimization review, executed through
`intuitive-flow` and `$intuitive-refactor`.

## Target

Tighten the existing seams between command ingress, `ControlCore`, `Plant`,
the RT/runtime transport, and lifecycle evidence without changing the fixed
Reachy product scope.

## Accepted Checklist

- [ ] Invalid, stale, and dropped requests produce explicit protocol evidence.
- [ ] `Plant` application results distinguish local application from measured
      confirmation and preserve fault detail without a generic interface.
- [ ] Source timestamp, capture monotonic time, simulation/device time domain,
      and runtime generation are explicit where currently implemented.
- [ ] Minimal restart/re-admission lifecycle rules are implemented and tested.
- [ ] `ARCHITECTURE.md`, `STATUS.md`, and relevant protocol docs match reality.

## Parked

- Shared-memory ABI or IPC replacement: needs measured deployment trigger.
- DDS/ROS 2 gateway and generic actuator abstractions: out of current scope.
- Full supervisor process, lease system, OTA, and hardware-specific Plant:
  require their own approved contract or hardware evidence.

## Evidence Ladder

1. Focused Rust unit/contract tests for each changed semantic.
2. `cargo fmt --all -- --check`.
3. `scripts/cargo-mujoco test --workspace`.
4. `scripts/cargo-mujoco clippy --workspace --all-targets -- -D warnings`.
5. `scripts/run-sim-scenario` and changed public protocol/runtime paths.

## Stop Condition

Stop when the checklist is complete, docs are aligned, focused and workspace
proof passes, and remaining ideas are parked. Stop and ask before changing the
public product scope, adding a new dependency, or claiming physical safety.
