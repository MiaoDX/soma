# Reachy Official Simulation Comparison Plan

> Status: D0 blocked on local official-simulator prerequisites, 2026-08-25.
> Appetite: at most 4 engineering days before Reachy Mini hardware arrives.
> Parent: `docs/plans/bootstrap-plan.md`, section 5 (Official comparison).

## Plan Ledger

- `status`: D0 `BLOCKED_NEEDS_LOCAL_VALIDATION`; implementation stopped at the required gate
- `current_slice`: simulation-only official comparison while hardware N0 is blocked
- `next_action`: provide an isolated host/container with GObject introspection prerequisites, then rerun the pinned D0 launch and mapping probe
- `blocker`: official v1.9.0 install fails because `gobject-introspection-1.0` development metadata is unavailable; nine-actuator comparability remains unproven
- `evidence`: `docs/measurements/official-simulation-d0-blocker.md`
- `no_touch`: hardware writes, N1 motion, public protocol changes, App framework, Isaac/Genie backend
- `stop_condition`: D0 cannot reproduce the official path or establish a defensible common trace

## Problem

Soma's Reachy simulation path proves its own control semantics, but we do not
yet have reproducible evidence showing how its actuator behavior and lifecycle
semantics compare with the official Reachy simulation implementation. A broad
App or simulator expansion during the hardware wait would spend the available
time on interfaces that are not yet part of Soma's fixed 9-actuator contract.

## Goal

Produce a repeatable simulation-only comparison artifact for Soma MuJoCo and an
isolated, pinned official Reachy simulation using a defensible common actuator
trace, with capability-labelled records and quantitative metrics.

## Appetite And Stop Gates

- Spend no more than four engineering days; preserve the remaining time for
  hardware-arrival preparation and environment failures.
- Use the existing fixed Reachy profile and do not change ControlCore semantics
  or the public Protobuf schema to make comparison easier.
- In the first half-day, pin the official SDK/daemon version and prove it can be
  launched in an isolated environment with the Reachy model.
- Stop the official comparison implementation if that launch or a 9-actuator
  order/unit/sign feasibility check cannot be reproduced in the first half-day;
  record the blocker instead.
- Stop any simulator work that requires broad GPU, container, asset migration,
  vendor-daemon patching, or direct writes to the official simulator's `MjData`.

Before execution, reconcile `docs/status/active/bootstrap.md`: this is a
bounded comparison child slice while hardware N0 remains blocked, not evidence
that N0 or N1 has passed. The official daemon may be a comparison-only
dependency, but never a Soma runtime dependency or concurrent serial owner.

## Day-0 Comparability Audit

Do not build the recorder or score metrics until this audit passes. Pin the
official commit/release, Python/MuJoCo versions, model hash, install and launch
commands, and capture their logs. Verify the official path's exposed command
surface and whether the nine actuator-shaft values can be mapped without
patching the vendor daemon. Record antenna sign conventions, order, units,
limits, initial pose, physics timestep/control decimation, warmup, and readiness.

Create a capability matrix with one of `COMMON`, `DERIVED`, `SOMA_ONLY`,
`OFFICIAL_ONLY`, or `UNAVAILABLE` for every field. Missing official TTL,
sequence, Plant timeline, disposition, or rejection semantics are not parity
failures and must never be synthesized.

## Core Slice

```text
fixed actuator trace
  -> Soma simulation
  -> official Reachy simulation
  -> normalized recorder
  -> metrics and report
```

The scored common trace includes a warmed-up common initial state, a bounded
actuator target with a dwell long enough for motion metrics, and stop/termination
evidence where both paths expose it. Use host recorder monotonic send/receive
timestamps for cross-process latency. Keep expiry, measured hold, reset,
timeline, and stale-command evidence as linked `SOMA_ONLY` acceptance evidence,
not as an official parity score.

## Metrics

| Category | Measures |
| --- | --- |
| Semantics | common command acceptance and stop/lifecycle events, labelled by capability matrix |
| Tracking | target-to-measured RMS/max error, overshoot, settling time, steady-state error |
| Timing | harness-local stimulus-to-observation latency, update interval, median/max; p99 only with declared repeated samples |
| Lifecycle | startup and stop latency when comparable |

Compare behavior and tolerances, not bitwise-identical physics trajectories.
Align/resample trajectories only after defining a common clock/window; retain
vendor timestamps as metadata. Keep semantic, motion, and runtime-performance
differences distinct in the report. Reliability, retries, disconnect recovery,
CPU, and RSS are out of the core slice.

## Non-Goals

- no Reachy hardware, torque enable, N0/N1 bypass, or official daemon as a Soma
  runtime dependency (the isolated official daemon may be a comparison oracle);
- no new public Robot Protocol, generic simulator interface, or multi-robot
  manifest;
- no complete official/community App compatibility layer;
- no head-pose/IK, camera/audio, ROS, policy runtime, or batch-RL integration;
- no Isaac Sim, Isaac Lab, or Genie Sim backend or feasibility probe;
- no bitwise trajectory equivalence claim.

## Parked Directions

Official/community App support and Isaac/Genie feasibility are separate future
shaping candidates. They are not tasks in this plan. Reopen them only after the
comparison report is complete and a concrete API or asset decision is needed.

## Evidence And Verification

- the Day-0 matrix proves common actuator order, units, signs, initial state,
  and timing assumptions, or produces a blocker report;
- identical trace inputs, warmup, pacing, and readiness conditions are recorded
  for both implementations;
- normalized records include model identity, actuator order, units, host
  timestamps, measured state, and capability/provenance labels;
- the report contains raw artifact paths plus the metric definitions and
  tolerance assumptions;
- private comparison tooling is versioned separately from the public Robot
  Protocol; generated runs belong under execution evidence/output, not human
  architecture docs;
- the existing `scripts/run-sim-scenario` and Rust workspace gates remain
  green;
- official and Soma processes are run separately, never with shared ownership
  of one runtime or serial endpoint.

## Cut Order

1. Remove all App and simulator-backend work (already parked).
2. Remove lifecycle metrics beyond startup/stop.
3. Reduce to target/trajectory/update-interval evidence.
4. Reduce to one fixed trace and one Soma-vs-official simulation report.

## Circuit Breaker

Return to shaping if the two implementations cannot share a defensible
actuator-level trace, common initial state, sign/units mapping, or clock/window.
Stop if the official environment cannot be reproduced in the Day-0 budget or
requires patching its daemon/direct `MjData` access. Do not manufacture a result
from manually matched screenshots or guessed API behavior.

## Stages And Definition Of Done

1. **D0, half day:** isolated official launch, pinned environment, capability
   matrix, and nine-actuator mapping probe.
2. **D1:** one fixed warmed-up motion trace and two thin external adapters;
   no Soma public-contract changes.
3. **D2:** JSONL recorder and deterministic analyzer with fixture tests, metric
   formulas, availability labels, and report generation.
4. **D3:** repeated-run reproducibility check, raw manifests/logs, docs, and
   existing Rust/headless scenario gates.
5. **D4:** reserved buffer; if D0 fails, preserve the blocker report instead of
   expanding scope.

Done means a reviewer can rerun the pinned commands, inspect both raw traces,
see the capability matrix, and distinguish common scored behavior from
Soma-only or unavailable semantics without reading adapter internals.

## Execution Preflight

- `Preflight status`: `DRAFT`
- `Task source`: approved plan and agent-planning-loop review
- `Canonical source`: `docs/plans/official-simulation-comparison-plan.md`
- `Route`: durable `$intuitive-flow`, main session owns stage gates and final
  complete/blocked judgment
- `Goal`: deliver the full simulation-only Soma-versus-official comparison, or
  stop at D0 with reproducible blocker evidence

### Scope

- Run D0 through D4 in order and preserve the four-engineering-day appetite.
- Pin and isolate the official SDK/daemon; do not add it to `soma-client` runtime
  dependencies.
- Reuse the existing Soma simulation launcher, fixed actuator profile, public
  state/command protocol, and headless acceptance scenario.
- Add only private comparison tooling: official and Soma adapters, one canonical
  trace fixture, capability-labelled JSONL records, deterministic metric/report
  generation, focused tests, and one owning run command.
- Store committed fixtures, schemas, tooling, tests, and a small representative
  report in the repository. Store raw/generated runs and environment logs under
  ignored `output/` evidence, with manifests that record exact revisions and
  commands.

### Non-Goals

All plan non-goals remain binding. In particular: no hardware access or writes,
public protocol/ControlCore/Plant changes, vendor-daemon patching, direct
official `MjData` access, App support, head-pose/IK/media work, or Isaac/Genie
probe/backend.

### Entity Budget

- `reuse`: `scripts/run-sim-scenario`, fixed Reachy actuator constants/assets,
  existing Protobuf/Zenoh client path, Python test environment, root docs and
  canonical cargo gates
- `remove/merge`: none required; keep generated evidence out of human docs and
  production packages
- `new`: one private comparison tooling directory/package, one owning script,
  one canonical trace/schema, two thin adapters, analyzer/report generator,
  focused fixtures/tests, and ignored `output/` entry; each exists only to keep
  official dependencies and comparison semantics outside production code
- `expansion triggers`: any public interface, generic simulator abstraction,
  second trace family, vendor patch, App/backend work, hardware use, or
  dependency added to production packages requires re-approval

### Context Package

- `must-read`: this plan; `AGENTS.md` injected guidance;
  `docs/status/active/bootstrap.md`; `ARCHITECTURE.md`;
  `docs/architecture/layering-and-trust-boundaries.md` sections on Plant and
  external simulators; `docs/plans/bootstrap-plan.md` sections 2, 3, and 5;
  `crates/soma-sim/src/lib.rs`; `crates/soma-runtime/src/bin/robot-rt.rs`;
  `python/soma_client/scenario.py`;
  `crates/soma-sim/assets/reachy-mini/UPSTREAM.md`
- `useful`: `docs/deep-research/fast-open-robot-reference-path.md` Reachy
  sections and `docs/deep-research/simulation-architecture.md` only when D0
  needs upstream/interface context
- `avoid-unless-needed`: unrelated deep research, visualization internals,
  deferred production architecture, other robot profiles, and historical
  execution evidence

### Acceptance

- `SUCCESS`: D0 capability audit passes; one canonical trace runs separately
  against both pinned implementations; raw records validate against the private
  schema; deterministic analyzer tests pass; the generated report states metric
  formulas/tolerances/provenance and labels every field by capability; a clean
  rerun reproduces the report within declared tolerances.
- `BLOCKED_NEEDS_DECISION`: D0 shows only a vendor patch, direct `MjData` write,
  public Soma contract change, or non-actuator command surface can establish
  comparability; stop and return the evidence plus options.
- `BLOCKED_NEEDS_LOCAL_VALIDATION`: the official simulator product run cannot
  execute in the available host environment; implementation is not complete or
  merge-ready until the pinned live gate runs.
- `INTERMEDIATE_ONLY`: none; a D0 blocker report is a valid stopped outcome,
  not a successful comparison.
- `No regressions`: existing Rust gates and `scripts/run-sim-scenario` pass;
  public protocol and current demo behavior remain unchanged.

### Verification

- `deterministic`: `cargo fmt --all -- --check`;
  `scripts/cargo-mujoco test --workspace`; comparison analyzer/schema/fixture
  tests; `git diff --check`
- `integration`: run both adapters independently against recorded fixtures;
  validate JSONL/manifests; generate the report twice and compare stable fields
- `product-run`: run the new owning comparison command from a clean process
  state, producing both traces and the report; rerun
  `scripts/run-sim-scenario`
- `local-live-manual`: required pinned official daemon/SDK + MuJoCo launch and
  9-actuator round trip in D0; inspect capability matrix, signs/units, warmup,
  cadence, and report claims against raw traces
- `optional`: visual trajectory inspection; never substitutes for typed traces
  and metric checks

### Execution

- `main`: own D0 stop gate, plan ledger/capsule updates, process isolation,
  scope cuts, verification, commits, and final complete/blocked judgment
- `worker`: none required by default; bounded read-only upstream inspection or
  independent report review may be delegated without changing ownership
- `worker-goal`: none
- `To execute`: `/goal execute docs/plans/official-simulation-comparison-plan.md with intuitive-flow`
- `Approval`: `LGTM`, `approve`, or starting the named goal in the new context
  approves this contract; do not rerun shaping or the planning loop unless D0
  triggers a stop decision

## Handoff

Execute the entire plan through `$intuitive-flow` in a new context. The plan is
not an approval to start hardware work or to implement another simulator.
