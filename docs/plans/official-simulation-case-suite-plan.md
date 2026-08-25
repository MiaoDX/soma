# Official Simulation Fixed Case Suite Plan

> Status: Agent-planning-loop converged; awaiting user approval, 2026-08-25.
> Parent evidence: `docs/plans/official-simulation-comparison-plan.md`.
> Appetite: at most 2 engineering days; simulation-only.

## Plan Ledger

- `status`: planning loop converged; awaiting user approval
- `current_slice`: extend the completed one-trace comparison into a fixed three-case evidence suite
- `next_action`: after user approval, prepare an execution-ready preflight for the whole plan
- `no_touch`: hardware, N0/N1, public protocol, production runtime dependencies, generic trace framework, result-driven case selection
- `stop_condition`: stop if a case cannot share the fixed nine-actuator public command/state mapping or requires unsafe/undefined Stewart targets

## Problem

The completed comparison is fully automated once invoked, but it exercises one
combined yaw-and-antenna trace. That is too narrow to show whether observed
differences are specific to one actuator group. Selecting cases after seeing
their metrics would weaken the evidence by making the representative result
outcome-dependent.

## Goal

Run and report a small, predeclared suite of fixed Reachy simulation cases that
separates yaw, antenna, and combined behavior. Preserve every case result and
designate the combined case as representative by declared coverage, never by
its measured outcome.

## Scope

- Add one committed `suite.json` containing three named cases in explicit order:
  `yaw-step`, `mirrored-antennas`, and `combined-yaw-antennas`.
- Make that file the sole owner of actuator order, deltas, representative case,
  warmup, observation cadence, and dwell.
- Make both existing adapters consume the selected case from that suite file.
- Run each case from independent Soma and official process lifecycles with the
  same readiness, warmup, pacing, and relative initial-state procedure.
- Preserve all successful, failed, and timed-out case artifacts under ignored
  `output/`, including case ID, suite hash, execution order, commands, process
  logs, stage status, and failure reason.
- Extend the existing analyzer/report path to include every case plus a compact
  case matrix. Do not rank cases by metrics.
- Commit one representative multi-case report generated from a clean suite run.

## Non-Goals

- no automatic outcome-based selection, scoring, ranking, tie-breaking, or
  optimization of cases;
- no generic case registry, plugin architecture, simulator abstraction, or
  second robot profile;
- no standalone Stewart-shaft case until a separate plan defines coupled-safe
  targets and acceptance;
- no pandas, NumPy, Jinja, schema library, or new test framework;
- no hardware, public Protobuf, ControlCore, Plant, daemon patch, direct
  official `MjData`, App, media, IK-product, Isaac, or Genie work;
- no automatic scheduling or background execution. The owning command remains
  a manually invoked reproducible product gate.

## Execution Design

### Canonical Suite

One committed `suite.json` is validated by a new dependency-free stdlib loader.
It contains the schema version, fixed actuator order, shared timing, explicit
ordered case list, stable IDs, exactly nine finite deltas per case, and the
predeclared representative case. Unknown keys and duplicate case IDs are
rejected. The suite identity is SHA-256 over the exact committed file bytes;
the schema version is therefore part of the hash.

| Case | Non-zero deltas | Declared role |
| --- | --- | --- |
| `yaw-step` | `yaw_body = +0.15` | isolate body-yaw tracking |
| `mirrored-antennas` | `right = +0.15`, `left = -0.15` | isolate SDK/MJCF antenna signs and tracking |
| `combined-yaw-antennas` | union of the two cases | representative case by actuator-group coverage |

The current duplicate constants in `common.py` and drifted `trace.json` are
deleted or derived from `suite.json`. No trace values remain hard-coded in an
adapter, analyzer, test, or script.

### Isolation And Ordering

The owning script uses the explicit array order in `suite.json`. For every case
it:

1. starts a fresh Soma stack, waits for the fixed profile, warms up, records the
   relative trace, and stops the stack;
2. starts a fresh official pinned container, waits for backend readiness, warms
   up, records the same relative trace through the public SDK, and removes the
   container;
3. validates both JSONL streams and generates the per-case metrics/report;
4. writes a run-owned `case-status.json` containing stage timestamps, readiness,
   actual warmup, command, shutdown outcome, and any failure reason;
5. records failures without silently removing the case from the suite report.

No simulator state is reused across cases. Case ordering cannot change the
declared representative case.

Both adapters sample the latest observed state on 20 ms monotonic deadlines for
exactly 100 planned samples during the two-second dwell. Each record includes
the planned sample index and deadline; lateness and actual observation intervals
remain descriptive timing evidence. A commanded actuator must move at least
0.05 rad from its warmed-up initial value or the case is invalid. Commanded
indexes are derived from non-zero suite deltas, never hard-coded.

The suite runner is intentionally not fail-fast. Daemon readiness and each
adapter have bounded timeouts; a case failure produces status/log evidence and
later cases still run. The final suite report is always attempted, and the
owning command exits nonzero after report generation if any declared case did
not succeed. The default output is a unique run directory; an explicit output
path must not already exist, preventing stale evidence reuse.

### Reporting

Reuse the existing analyzer and Markdown renderer. The suite report includes:

- complete case inventory and suite hash;
- success/failure/timeout status for every declared case;
- per-case raw artifact paths, model identities, capabilities, sample counts,
  timing, and actuator metrics;
- a compact matrix comparing the three cases without collapsing unlike
  semantic, tracking, and runtime measures into one score;
- descriptive yaw-only versus combined yaw deltas and antenna-only versus
  combined antenna deltas, so interaction effects are visible without becoming
  a selector score;
- an explicit note that `combined-yaw-antennas` is representative because it
  covers both commanded actuator groups, not because it performed best.

## Stages

1. **S0, evidence contract and canonical input:** replace duplicate trace
   ownership with one ordered `suite.json`; add strict loader, hash, malformed
   suite tests, structured case status, timeout/exit semantics, and clean-output
   enforcement.
2. **S1, adapters:** parameterize both existing adapters by manifest and prove
   each emits the same case ID, suite hash, order, units, 100-sample deadline
   schedule, movement validity, and relative targets. Run one isolated live case
   before proceeding.
3. **S2, orchestration/report:** add the bounded shell loop, per-case clean
   lifecycle, failure preservation, case matrix, and representative label.
4. **S3, reproducibility:** run the full suite twice, compare case inventory and
   stable report fields, commit one representative report, and run existing
   repository gates.

## Acceptance

- One owning command runs all three cases without user choices after launch.
- Both adapters consume the same committed suite file; no hard-coded trace
  deltas remain elsewhere.
- Every case starts from independent processes and records readiness, warmup,
  initial positions, command and observation timestamps, and shutdown outcome.
- All declared cases appear in the report even when one fails or times out;
  later cases continue, the report is attempted, and the command exits nonzero
  after report generation if any case failed.
- The report contains all per-case metrics and labels the combined case by
  predeclared coverage, with no metric-derived selection.
- Two clean runs have identical suite hash, case order, representative label,
  capability labels, status schema, and exactly 100 planned samples per
  successful implementation/case. Numerical metric deltas and host timing are
  reported descriptively; they are not post-hoc pass/fail or selection gates.
- Every commanded actuator moves at least 0.05 rad in each successful case;
  failure to demonstrate motion invalidates that case.
- Public protocol, ControlCore, Plant, and existing one-scenario behavior are
  unchanged.

## Verification

```bash
PYTHONPATH=. python3 -m unittest comparison.official_sim.test_analyze
python3 -m compileall -q comparison/official_sim
bash -n scripts/run-official-sim-comparison
scripts/run-official-sim-comparison output/official-sim-suite-run1
scripts/run-official-sim-comparison output/official-sim-suite-run2
cargo fmt --all -- --check
scripts/cargo-mujoco test --workspace
scripts/run-sim-scenario
git diff --check
```

The repeated-run verifier hard-gates only stable structural fields and movement
validity. It emits numerical metric deltas for review but must not use a
byte-for-byte report comparison or metric-derived selection because host timing
and physics trajectories are expected to vary.

## Stop Gates

- Stop and return evidence if either public adapter cannot execute one of the
  three declared cases without a daemon patch, direct `MjData` access, or public
  Soma contract change.
- Stop before adding a Stewart-specific case, a fourth case, a generic case
  abstraction, or outcome-based selector; those require a new scope decision.
- Stop if repeated runs cannot preserve case inventory, suite identity, initial
  state assumptions, or the declared actuator mapping.

## Evidence Retention

The existing one-trace representative report remains immutable evidence of the
completed parent plan. This plan commits a new suite report at
`docs/measurements/official-simulation-case-suite-report.md` and updates current
status to link the suite report without deleting the parent evidence.

## Handoff

After planning-loop approval, prepare an execution-ready preflight and run the
whole plan through `$intuitive-flow`. Do not reopen the completed one-trace plan
or rewrite its evidence as though it had not completed.
