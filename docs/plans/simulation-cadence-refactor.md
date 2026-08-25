# Simulation Cadence Refactor

## Plan Ledger

- `status`: ACTIVE
- `current_slice`: align wall-clock control periods with MuJoCo physics time
- `next_action`: implement startup schedule validation and control-period advancement
- `evidence`: existing fixed official simulation case suite, then two clean reruns
- `no_touch`: public protocol, ControlCore semantics, hardware, generic robot manifests, simulator backends
- `stop_condition`: stop if cadence correctness requires a public contract or robot-profile abstraction

## Refactor Gate

Root cause evidence shows the Reachy model has a 2 ms MuJoCo timestep while
`robot-rt` runs every 20 ms and currently advances one physics step. The same
MJCF and actuator parameters are used by Soma and the official backend; the
official backend advances ten physics steps per 50 Hz control update.

Target: make simulation cadence a validated `soma-sim` adapter contract. A
MuJoCo Plant must reject a control period that is not a positive integral
multiple of its physics timestep, and advancing one control period must execute
exactly that validated number of substeps. The generic hardware-facing `Plant`
trait remains unchanged.

The canonical product simulation launchers will use Rust release binaries.
Release mode is an execution-quality change, not the fix for simulation-time
alignment.

## Acceptance

- The pinned Reachy model validates 20 ms control / 2 ms physics as ten substeps.
- Invalid, zero, non-finite, shorter-than-physics, and non-integral schedules fail before execution.
- One runtime control tick advances exactly one 20 ms simulation interval.
- The schedule validator is reusable by a future MuJoCo robot adapter without introducing a robot registry or manifest.
- Product simulation scripts build and launch `target/release` binaries.
- Two clean fixed-suite runs pass with stable structure, 100 samples per stream, valid movement, and refreshed quantitative evidence.
- Existing ControlCore, protocol, scenario, formatting, tests, and Clippy gates remain green.

## Verification

```bash
cargo fmt --all -- --check
scripts/cargo-mujoco test --workspace
scripts/cargo-mujoco clippy --workspace --all-targets -- -D warnings
scripts/run-sim-scenario
scripts/run-official-sim-comparison output/official-sim-cadence-run1
scripts/run-official-sim-comparison output/official-sim-cadence-run2
PYTHONPATH=. python3 -m comparison.official_sim.analyze \
  --suite comparison/official_sim/suite.json \
  --verify-runs output/official-sim-cadence-run1 output/official-sim-cadence-run2 \
  --json output/official-sim-cadence-repeatability.json
git diff --check
```
