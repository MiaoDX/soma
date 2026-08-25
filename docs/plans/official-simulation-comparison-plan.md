# Reachy Official Simulation Comparison Plan

> Status: Planned child slice; execute only after preflight reconciles the
> active bootstrap capsule, 2026-08-25.
> Appetite: at most 4 engineering days before Reachy Mini hardware arrives.
> Parent: `docs/plans/bootstrap-plan.md`, section 5 (Official comparison).

## Plan Ledger

- `status`: planned; awaiting preflight and active-capsule reconciliation
- `current_slice`: simulation-only official comparison while hardware N0 is blocked
- `next_action`: run `$intuitive-preflight`, then execute the full plan through `$intuitive-flow`
- `blocker`: official simulator environment and nine-actuator comparability are unproven
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

## Handoff

After approval, execute the entire plan through `$intuitive-flow`. The plan is
not an approval to start hardware work or to implement another simulator.
