# Soma Rust Policy And Runtime Hardening Plan

> Status: COMPLETE. This plan is the unified execution unit for the identified
> Soma runtime gaps and the policy-inference migration. It does not refactor
> the community Micro Duck training repository.

## Goal

Make Soma's fixed-profile control stack truthful and production-shaped:

- periodic RT ingress/egress remains bounded, fresh, and fail-contained;
- malformed and stale inputs always produce attributable evidence;
- periodic timing has measurable overrun and drop evidence;
- the fixed Open Duck policy ABI is explicit and executable as a contract;
- Rust can host policy inference in a non-RT worker while Python remains the
  reference/oracle path;
- Python and Rust produce equivalent observations, actions, and target safety
  behavior before Rust becomes the deployment default.

## Non-goals and hard boundaries

- No generic robot/policy manifest framework, registry, plugin system, or new
  embodiment abstraction.
- No ONNX inference, async work, Zenoh, blocking I/O, or unbounded allocation
  in `robot-rt`'s periodic section.
- No Open Duck hardware, Reachy hardware actuation, torque/register writes, or
  changes to N0/N1 gates.
- No Micro Duck repository refactor, retraining, checkpoint replacement, or
  training-stack changes.
- No action chunking, interpolation beyond the frozen zero-order hold, or
  generic observation API.

## Approved Decisions

1. Use a reproducible Rust ONNX Runtime binding/native runtime, with `ort` as
   the implementation candidate and an explicit provisioning check.
2. Host the Rust policy worker as a separate process, isolated from both
   `robot-runtime` and periodic `robot-rt`.
3. Make the Rust worker the default deployment route. Retain Python as an
   explicit reference/oracle route for parity, qualification, and diagnosis.
4. Record the fixed Duck `101 -> 14` ABI, ordering, history/phase, default
   pose, scale, slew, checksum, and finite-value rules in a durable ADR or
   equivalent D-04/D-19 update. Do not create a generic ABI.
5. Pin `ort = 2.0.0-rc.11` with dynamic loading and provision native ONNX
   Runtime `1.28.0` outside the repository. Development may use
   `ORT_DYLIB_PATH`; provisioning downloads a platform release from the pinned
   source and verifies its SHA-256. Native runtime binaries are not committed
   or repeated in every Soma software release.

If the backend cannot build and run reproducibly in the supported toolchain,
stop after the spike and return for a new decision; do not silently make Python
the default.

## Target topology

```text
Rust policy worker (non-RT, ONNX)
        or explicit Python reference worker
        -> bounded latest-value handoff
        -> robot-runtime (non-RT authority/transport)
        -> bounded local IPC
        -> robot-rt (periodic admission/apply/evidence only)
        -> fixed MuJoCo/native Plant
```

The Python client remains an oracle, not a second control authority. Both
workers consume the same fixed profile contract and golden fixtures.

## Phases and gates

### A. RT mailbox and failure semantics

Owner: `soma-runtime`, `soma-core`, focused tests.

- Drain ingress to a bounded cap per tick and retain latest valid target;
- preserve ordering of reset/runtime-start messages;
- convert decode failures to `Invalid` rejection evidence;
- make output state sends best-effort (`WouldBlock`, missing/closed receiver)
  with counters instead of terminating RT;
- make generation-zero authority explicit: targets are rejected until a current
  runtime generation is observed;
- preserve timeline, sequence, TTL, measured hold, and requested/admitted/applied
  semantics.

Gate: burst, malformed, receiver-stall, restart, and stale-target tests pass;
Reachy and Duck scenarios remain unchanged.

### B. Periodic timing evidence

Owner: RT binaries and state/diagnostic evidence.

Add bounded counters/fields for tick count, gap, late-by, maximum work duration,
ingress drops, egress drops, and deadline/overrun events. Evidence must be
observable without making hard timing claims from simulation alone.

Gate: timing evidence is encoded in `ActuatorState`; injected receiver-stall
and policy-stall runs prove the loop continues, reports evidence, and reaches
expiry/hold. Existing cadence and gait floors remain the acceptance criteria
unless separately approved.

### C. Fixed policy ABI and Python oracle hardening

Owner: Open Duck profile module, Python client, fixtures/tests.

Freeze one profile-specific contract: observation shape `[1, 101]`, action shape
`[1, 14]`, named actuator order, default pose, normalization, action scale,
27-tick phase, history order, slew limit, checkpoint checksum, and target TTL.

Harden Python against wrong shape, non-finite output, non-finite target, and
out-of-range target. Add fake-runtime fault tests and golden fixtures for reset,
zero/extreme commands, history/phase, and target packing.

Gate: Python oracle passes existing process/fault tests and emits the same
canonical fixture artifacts Rust will consume.

### D. Rust non-RT policy worker spike

Owner: new fixed Open Duck policy module/worker, feature-gated as needed.

Implement the smallest Rust adapter that loads the pinned model, constructs the
fixed observation, runs inference outside RT, applies the same slew/TTL rules,
and hands off the latest target through a bounded channel. Keep model loading,
allocation, and ONNX runtime initialization out of the periodic path.

Gate: reproducible build/runtime provisioning, finite/shape/range checks,
bounded memory behavior, and no new dependency required by the default
headless Rust test command unless explicitly accepted.

### F. Documentation closure

Update `ARCHITECTURE.md`, `STATUS.md`, D-04/D-19 references, and the durable ABI
decision with the measured topology, explicit Python role, dependency/runtime
provisioning, and residual limitations. Keep the old Open Duck acceptance plan
as historical evidence; do not duplicate its ledger.

## Verification

```bash
cargo fmt --all -- --check
scripts/cargo-mujoco test --workspace
scripts/cargo-mujoco clippy --workspace --all-targets -- -D warnings
cd python && uv sync && uv run pytest && cd ..
scripts/run-sim-scenario
scripts/run-open-duck-walk policy --case <frozen-case> --repeat 2
```

Additional focused checks must cover latest-wins burst behavior, malformed
ingress/output, generation-zero startup, WouldBlock output, timing counters,
Python/Rust golden parity, model checksum, and cleanup under worker/runtime
failure. Hardware commands remain prohibited by this plan.

## Stop conditions

Stop and return for user review if backend licensing/provisioning is unclear,
parity requires changing the frozen ABI or checkpoint, inference would enter
RT, process isolation provides no measurable benefit and a topology change is
proposed, or any step would weaken existing safety/evidence gates.

## Preflight Contract

Preflight status: DRAFT (approved scope; implementation deferred to the next context)

Task source: user request plus this plan

Canonical source: `docs/plans/soma-rust-policy-and-runtime-hardening-plan.md`

Route: durable `$intuitive-flow`, with `$intuitive-refactor` semantics for the
known RT and policy seams.

Goal: harden Soma's fixed control/runtime contracts and add a separately
process-isolated Rust policy executor that becomes the default after parity.

Scope: phases A-F above, including focused Rust/Python tests, fixed ABI fixtures,
Rust ONNX worker packaging, cross-language parity, timing/fault evidence, and
documentation closure.

Non-goals: generic frameworks; Micro Duck repository changes; retraining or
checkpoint replacement; action chunks/interpolation; hardware, torque,
register, N0/N1, or physical actuation work; inference/async in `robot-rt`.

Entity budget: reuse existing `ControlCore`, `Plant`, Open Duck codecs, Python
policy client, launcher, fixtures, and acceptance harness; remove/merge no
existing public surface unless a duplicate D-04 ledger is superseded; new only
one fixed ABI record/fixture set, one Rust policy worker process, and the
minimum feature-gated ONNX/runtime packaging required to prove the contract;
re-approval required for generic abstractions, public protocol changes,
checkpoint changes, or topology changes.

Context: must-read=`AGENTS.md`, `ARCHITECTURE.md`, `STATUS.md`, this plan,
`docs/plans/open-duck-mini-walk-policy.md`, `docs/plans/decision-register.md`,
`docs/deep-research/policy-runtime-interface.md`,
`crates/soma-core/src/lib.rs`, `crates/soma-runtime/src/lib.rs`, RT binaries,
`crates/soma-sim/src/lib.rs`, `python/soma_client/open_duck_policy.py`, and
existing policy tests; useful=Micro Duck deployment/runtime references already
recorded in the research docs; avoid-unless-needed=training code, historical
outputs, hardware paths, and broad community-repo refactors.

Acceptance:

- SUCCESS: all phases pass their gates; Rust policy worker is reproducibly
  packaged and default; Python/Rust golden parity and fault matrix pass; RT
  remains free of inference/async; required product runs and existing gates
  pass; docs identify residual limitations and the Python oracle route.
- BLOCKED_NEEDS_DECISION: backend provisioning/licensing, ABI change,
  checkpoint replacement, or worker topology change becomes necessary.
- BLOCKED_NEEDS_LOCAL_VALIDATION: required ONNX/native-runtime, simulator,
  process, or manual live proof cannot run in the implementation environment.
- INTERMEDIATE_ONLY: none unless explicitly approved in a later context.
- No regressions: Reachy scenario/teleop, Open Duck fixed acceptance, timeline,
  sequence, TTL, hold, reset, rejection, ownership, N0/N1, and existing public
  protocol semantics remain intact.

Verification: deterministic=`cargo fmt --all -- --check`; `scripts/cargo-mujoco
test --workspace`; `scripts/cargo-mujoco clippy --workspace --all-targets --
-D warnings`; `cd python && uv sync && uv run pytest && cd ..`; focused ABI,
malformed ingress/output, latest-wins, generation, timing, parity, and
allocation tests; integration=`scripts/run-sim-scenario` plus launcher and
process fault matrices; product-run=`scripts/run-open-duck-walk policy --case
<frozen-case> --repeat 2` with Rust default and explicit Python reference;
local-live-manual=required ONNX native-runtime provisioning, simulator
process shutdown/stall runs, and human review of any live evidence; hardware
proof is explicitly out of scope and must not be run.

Execution: main=root supervisor owns scope, gates, and final completion;
worker=bounded workers may own isolated implementation/test slices only;
worker-goal=each worker must name files and return evidence without changing
unrelated surfaces.

To execute: `/goal execute docs/plans/soma-rust-policy-and-runtime-hardening-plan.md with intuitive-flow`

Approval: the user approved the backend direction (`ort` candidate), separate
worker process, Rust default route, Python reference route, and ABI decision.
