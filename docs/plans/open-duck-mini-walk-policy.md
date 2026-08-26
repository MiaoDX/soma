# Open Duck Mini Walk Policy Plan

> Status: Approved plan with execution preflight, 2026-08-26. Implementation
> is intentionally deferred to a new agent window.

## Plan Ledger

- `status`: APPROVED; preflight ready for a new execution window
- `appetite`: gate-bounded; no calendar-duration estimate
- `current_slice`: planning only
- `next_action`: start the preflight's `/goal execute ... with intuitive-flow`
  command in a new agent window
- `no_touch`: Reachy hardware probe and N0/N1 gates; Open Duck hardware;
  generic robot manifests; policy training; ROS 2; a third robot; camera, audio,
  antennas, expression features, and visualization
- `stop_condition`: stop at any gate below; do not start later stages to rescue
  a failed earlier gate or weaken the proof to claim completion

## Decision And Outcome

The user approved reopening the earlier Open Duck deferral on 2026-08-26. This
is one additional fixed simulation profile, not broad robot support. Reachy
Mini remains Soma's primary simulation and hardware path.

The required outcome is:

> A pinned Open Duck Mini v2 ONNX walk policy drives a license-compatible
> pinned MuJoCo model through Soma's Python policy -> loopback Zenoh -> Rust
> runtime -> bounded nonblocking Unix datagram -> periodic Rust RT owner ->
> Rust MuJoCo Plant path, preserving source-frame lineage, timeline, sequence,
> an original deadline, measured-position hold, and requested/admitted/applied
> evidence.

The result qualifies Soma's architecture against a second embodiment and a
50 Hz locomotion policy. It does not claim Open Duck hardware support or that a
simulation fallback is physically safe for a biped.

Before Stage 1, implementation must record this approved exception in D-22 and
the active status capsule. It must not rewrite Reachy's existing support claim
until Open Duck acceptance passes.

## Execution Bounds And Circuit Breaker

Implementation is ordered by evidence gates, not human-time estimates. Each
stage has one owning deliverable and later stages remain blocked until its gate
passes:

| Stage | Deliverable | Hard gate |
| --- | --- | --- |
| 0 | provenance manifest, frozen case, golden contract, direct reference | legal and delayed-reference feasibility |
| 1 | narrow `const N` control-core seam | unchanged Reachy semantics and gates |
| 2 | fixed Duck Plant plus runtime/RT path | synthetic 50 Hz end-to-end control |
| 3 | Python ONNX policy plus fault cases | exact observation/action and deadline behavior |
| 4 | two accepted gait runs, cleanup, regressions, compact docs | complete definition of done |

Stop and return for reshaping if:

- the licensed source set cannot support an independently implemented runner;
- the pinned checkpoint fails the predeclared reference gait floors with the
  intended one-tick delayed/held-action schedule;
- exact policy semantics require copying unlicensed Runtime or Playground code;
- the compiled model cannot map exactly to 14 named policy-controlled joints;
- Python, ONNX Runtime, Zenoh, blocking I/O, an unbounded queue, or unbounded
  allocation must enter the periodic RT section;
- the work requires a runtime robot manifest, device factory, observation DSL,
  profile registry, third robot, or Reachy public contract change;
- the reference case, checkpoint, model parameters, initial state, command, or
  gait thresholds would need tuning after seeing the Soma result;
- completing a gate requires weakening an earlier contract or adding parked
  scope.

## Upstream And Provenance Boundary

Stage 0 starts from these candidates, not from an assumption that they are a
coherent bundle:

- redistributable candidate: `apirrone/Open_Duck_Mini` branch `v2`, commit
  `b23317a485b3cec7d8417f352478778b3475173c`, with its Apache-2.0 license,
  v2 MJCF/assets, and `BEST_WALK_ONNX_2.onnx`;
- checkpoint SHA-256:
  `3c606f9381a1710cc8fecdb7442787dcbfce3ee9bc02a6f1224774ab2b3a1067`;
- inspected ONNX hypothesis: `obs` float tensor `[1, 101]` to
  `continuous_actions` float tensor `[1, 14]`;
- observational-only sources: `Open_Duck_Mini_Runtime` commit
  `32037347dc43186a017f2116bcfde7c461b81f54` and Open Duck Playground.

No license was detected for Runtime or Playground. They may be cited for
publicly observable interface facts, but their code, constants, assets, and
expressive implementation must not be copied, translated, or vendored. Soma's
compatibility ledger and runner must be independently written from allowed
files and documented facts. If that distinction is insufficient to establish
the exact contract, Stage 0 stops unless the user separately approves a
resolved license path.

Stage 0 produces a per-file provenance manifest containing repository, commit,
path, checksum, license, modification, destination, and factual-source notes
for the checkpoint, MJCF, every mesh/texture, and each compatibility patch. A
missing or unapproved entry makes the reference command fail before rollout.

## Narrow Architecture

```text
fixed velocity case
        |
        v
Python Open Duck policy (non-RT: history + ONNX Runtime)
        ^                         |
        | one OpenDuckState       | one OpenDuckTarget
        +------ loopback Zenoh ---+
                       |
                       v
Rust Open Duck runtime (non-RT: validate, coalesce latest, stamp receipt)
                       |
                       | bounded Protobuf / nonblocking Unix datagrams
                       v
Rust Open Duck robot-rt (periodic owner)
  admission + deadline + evidence -> OpenDuckSimPlant -> MuJoCo
```

The only shared abstraction is fixed-size actuator command admission:

- `ActuatorState<const N>`, `ActuatorTarget<const N>`, `Plant<const N>`, and
  `ControlCore<const N>` (or an equally static monomorphized shape);
- Reachy aliases remain fixed at `N = 9` and preserve existing names,
  semantics, external `soma.v1` schema, Zenoh keys, sockets, and binaries;
- Open Duck is fixed at `N = 14` and keeps IMU/contact extraction, model
  validation, reset, and observation facts in its own profile module.

Do not add sensor capabilities to the generic Plant, a dynamic actuator vector,
or a robot metadata layer. Keep the existing size-bounded Protobuf and
nonblocking Unix datagram mechanism; this plan does not migrate Reachy's local
IPC or claim it is a heap-free fixed-layout codec. Preallocate periodic buffers,
bound decoded sizes, and prove no capacity growth after startup.

The runtime publishes one combined experimental Open Duck state, not separate
actuator, policy-frame, and evidence streams. It includes positions,
velocities, gyro, acceleration, contacts, health, timeline, source capture
tick/time, and command evidence. The policy emits at most one target for each
complete state. The runtime coalesces to the latest target and sends at most
one command per RT tick; the RT owner performs a bounded drain and records
drops rather than allowing backlog to renew old commands.

Open Duck uses profile-specific experimental keys, schemas, and socket paths.
Concurrent Reachy/Open Duck operation is not required, but tests must prove
their endpoints cannot cross-wire.

## Fixed Embodiment Contract

The policy-controlled joint hypothesis is exactly 14 positions in this order:

```text
left_hip_yaw, left_hip_roll, left_hip_pitch, left_knee, left_ankle,
neck_pitch, head_pitch, head_yaw, head_roll,
right_hip_yaw, right_hip_roll, right_hip_pitch, right_knee, right_ankle
```

Repository evidence currently conflicts on whether the full v2 model has 15
or 16 actuators; antenna actuators appear outside the 14 policy outputs. Stage
0 must query the compiled MuJoCo model, record every actuator and joint name,
map all 14 policy outputs by name, and declare a fixed value for every excluded
actuator. It must also validate `nq`, `nv`, `nu`, limits, initial pose, sensors,
contacts, and timestep. Index slicing or an unexplained count is a hard stop.

The 101-element observation is also a hypothesis until Stage 0 freezes a named
golden tick:

| Segment | Width | Required semantics |
| --- | ---: | --- |
| body gyroscope | 3 | exact body frame, axes, units, bias |
| body acceleration | 3 | exact frame, axes, units, gravity treatment |
| velocity/head command | 7 | frozen stimulus mapping |
| position relative to default | 14 | named joint order and default pose |
| scaled velocity | 14 | units and scale |
| three action-history frames | 42 | order and reset fill |
| previous motor target | 14 | event ordering and reset value |
| left/right foot contact | 2 | geom pairs, threshold, sample timing |
| gait phase | 2 | frequency, convention, reset |
| **Total** | **101** | |

The compatibility manifest also freezes tensor names, shapes and dtype;
Python, NumPy, ONNX Runtime CPU provider and version; MuJoCo/native library and
compiler settings; Rust toolchain; default pose; action scale; motor-to-target
patch; control/policy cadence; and event order. Start with no target slew or
low-pass filter. Add either only if the licensed contract proves it mandatory,
then record it as a compatibility patch before rollout.

## Timeline, Deadline, And Fallback

The asynchronous route has an intentional contract: state tick `k` may first
produce an applied target at tick `k + 1`; the action is held until replaced or
expired. Stage 0 tests both a direct zero-delay diagnostic and the required
one-tick delayed/held reference. The delayed route must pass the same absolute
gait floors before Soma integration starts.

Each Open Duck target carries the source state timeline/tick/capture time,
target sequence, and one deadline budget originating at state capture. Runtime
receipt does not mint a new lease: it validates lineage and computes remaining
TTL before forwarding. The RT owner rejects wrong timeline, duplicate or old
sequence, non-finite data, bad profile/length, expired deadline, and invalid
source lineage.

Tests inject delay before inference, during inference, before runtime receipt,
and in local delivery. Every delay consumes the same original budget, and no
queued target may become newly valid after expiry.

The named fallback is `open-duck-sim-measured-position-hold-v1`: on target
expiry the consumer applies the latest measured positions and emits one expiry
transition. This is a simulation liveness/evidence contract only. It is not a
claim that the robot remains upright and must never be reused for physical
actuation without a separate safety decision.

## Frozen Reference Case

Before the first reference rollout, Stage 0 writes and commits one case file.
It freezes:

- all provenance hashes and complete toolchain versions;
- seed and deterministic settings, timestep, 50 Hz control/policy cadence,
  reset pose, command vector, duration, and the zero/one-tick schedules;
- absolute floors for minimum duration, root-height and roll/pitch envelope,
  commanded-direction base displacement, alternating left/right contact
  transitions, non-foot collision count, and stance-foot slip;
- trace fields, fall predicate, deadline, permitted tick lateness/message age,
  drop policy, and teardown conditions.

These absolute floors come from documented examples and robot scale before
observing the checkpoint output; finite state or displacement alone cannot
pass. Only tight reference-to-Soma fixture tolerances may be derived after the
reference runs. Chaotic full-trajectory equality is not the oracle: both paths
must independently meet the same absolute gait floors, while selected named
observation/action ticks must match within the frozen numeric tolerances.

## Execution Stages

### Stage 0: provenance and direct reference gate

- create the provenance manifest and frozen case before rollout;
- independently implement the minimal direct headless runner;
- resolve the full-actuator/14-controlled mapping and excluded values;
- extract the exact 101-field ledger and golden observation/action fixtures;
- run zero-delay diagnostic and two one-tick delayed reference rollouts;
- record latency, contacts, slip, pose, displacement, and repeatability.

Gate: all provenance is compatible, fixtures are exact and finite, and both
delayed runs meet every predeclared gait floor. Otherwise stop without touching
Reachy code.

### Stage 1: static control-core seam

- record the approved D-22/status exception;
- make only actuator arrays, Plant, targets, ticks, and `ControlCore` static in
  `N`, retaining Reachy aliases and behavior;
- prove identical timeline, sequence, TTL, expiry, hold, and rejection
  semantics for `N = 9` and `N = 14`;
- add compile-fail/type checks showing the two profiles cannot cross-wire.

Gate: all existing Reachy tests and scenario behavior remain green; no public
schema, key, socket, binary behavior, manifest, registry, or generic sensor API
changes.

### Stage 2: fixed Open Duck Plant and process path

- add one Open Duck profile module with pinned assets, startup validation,
  reset, named joints, excluded actuator holds, state facts, contacts, and
  exact cadence;
- add fixed Open Duck runtime/RT entry points and experimental external
  messages using the existing bounded transport mechanism;
- publish one combined state and transport one coalesced latest target;
- use ordinary Rust tests, not another standalone runner, for Plant contracts;
- run one synthetic 50 Hz target source through the complete process path.

Gate: startup/model validation, movement, reset, deadline, bounded
message/drop behavior, endpoint isolation, and clean shutdown pass; periodic
buffers show no capacity growth after startup.

### Stage 3: policy and fault integration

- add the thin Python client with pinned CPU ONNX Runtime;
- reproduce the golden observation, history/reset, phase, inference, action
  scale, and motor-target behavior without hardware dependencies;
- run exact fixture comparisons and then the frozen case through Soma;
- inject malformed, non-finite, duplicate, wrong-timeline, stale-source, and
  delayed targets, including delays at each deadline stage;
- join every requested sequence to admission/rejection and applied/hold
  evidence, including source tick and message age.

Gate: observation/action fixtures match, fault cases cannot renew stale
validity, RT tick lateness and message age stay within the frozen case, and an
inference stall reaches the named hold with attributable evidence.

### Stage 4: acceptance and closure

- expose one owning `scripts/run-open-duck-walk` launcher with reference,
  synthetic, clean-policy, repeat, and injected-stall modes;
- run two clean Soma rollouts against the frozen case and retain compact
  machine-readable metrics;
- run canonical workspace and Reachy regression gates;
- prove cleanup for normal completion, signals, policy/runtime failure, and
  launch failure;
- update only the compact human docs, decision register, active status, and
  this plan to reflect the result actually proven.

Gate: the definition of done is complete. Do not substitute partial support,
visual judgment, or weakened thresholds.

## Acceptance And Verification

Acceptance requires all of the following:

- auditable, license-compatible per-file provenance and an independently
  implemented reference route;
- exact 14-joint mapping inside the validated full model, with every excluded
  actuator fixed and checked;
- exact 101-value golden fixture, ONNX output fixture, units, frames, reset,
  cadence, event order, dtype, and toolchain pins;
- two delayed reference and two complete Soma runs meeting the same
  predeclared uprightness, displacement, alternating-contact, non-collision,
  low-slip, duration, and repeatability floors;
- original-deadline consumption across capture, inference, middleware, local
  delivery, and RT admission, with no stale backlog revival;
- complete source/request/admission/application lineage and named hold evidence;
- bounded message sizes/drain, no periodic buffer growth, bounded tick lateness
  and message age, clean process/endpoint teardown;
- unchanged Reachy public protocol, runtime behavior, simulations, hardware
  gates, and canonical tests;
- no Open Duck hardware, training, visualization, interactive control, generic
  robot/policy framework, simultaneous-profile feature, or third embodiment.

Stage 0 replaces placeholders with exact commands and numbers before Stage 1.
The final gate includes at least:

```bash
cargo fmt --all -- --check
scripts/cargo-mujoco test --workspace
scripts/cargo-mujoco clippy --workspace --all-targets -- -D warnings
scripts/run-sim-scenario
scripts/run-sim-teleop --keys ADQE
scripts/run-open-duck-walk reference --case <frozen-case>
scripts/run-open-duck-walk policy --case <frozen-case> --repeat 2
scripts/run-open-duck-walk stall --case <frozen-case>
git diff --check
```

Hardware commands and visual acceptance are absent by design.

## Parked And Rejected

Parked until this headless outcome passes: MuJoCo/Rerun visualization,
interactive velocity input, simultaneous Reachy/Open Duck launch, showcase
regeneration, generic policy bundles, broad launcher refactors, optional
filtering/slew, exhaustive malformed-model matrices, hardware, training, ROS 2,
and any additional robot.

Rejected for this plan: copying or translating unlicensed Runtime/Playground
implementation; choosing thresholds after reference output; treating two
identical deterministic runs as sufficient walking proof; a second IPC
technology; a runtime manifest; dynamic robot descriptions; generic sensor or
observation interfaces; weakening Reachy semantics; or claiming stale-frame
safety without source lineage.

## User-Review Gates

Return to the user before:

- accepting a partial outcome or relaxing frozen gait floors;
- using Runtime/Playground implementation or any unresolved asset/license;
- changing the checkpoint/model pair after Stage 0 starts;
- changing Reachy's external schema, keys, behavior, hardware gates, or N0/N1;
- replacing the named hold, adding hardware/training/generic abstractions, or
  turning a parked item into mandatory scope.

## Definition Of Done And Handoff

The plan is done only when the pinned policy produces the required reference
and complete Soma headless gait runs, the stall and deadline tests prove no
stale revival, provenance/contracts/metrics are recorded, Reachy remains green,
cleanup is complete, and compact docs state exactly what is supported.

The preflight below is the handoff for a new window. Execute it through
`$intuitive-flow`; Stage 0 is a hard gate and the first execution phase, not a
substitute for the approved outcome.

## Execution Preflight

```text
Preflight status: DRAFT
Task source: approved plan plus user direction to remove calendar estimates
Canonical source: docs/plans/open-duck-mini-walk-policy.md
Route: durable $intuitive-flow
Goal: Add one fixed Open Duck Mini v2 simulation profile whose pinned ONNX walk policy passes the independent reference and complete Soma paths without weakening Reachy or creating a generic robot framework.

Scope:
- Execute Stages 0-4 in order, honoring every hard stop gate.
- Establish license-compatible provenance, the exact model/policy contract, and a frozen gait case before changing shared code.
- Deepen only fixed-size actuator admission to const N; then add the fixed Duck Plant, experimental transport, Python policy client, deadline/fault evidence, launcher, tests, and compact docs.
Non-goals: Open Duck hardware; policy training; visualization; interactive control; simultaneous profiles; ROS 2; generic manifests, registries, device factories, sensor capabilities, observation DSLs, policy bundles, or a third robot; Reachy N0/N1 or public-contract changes.
Entity budget: reuse=ControlCore semantics, soma-core Plant boundary, soma-sim MuJoCo patterns, existing Protobuf/Zenoh/nonblocking Unix datagram transport, Python client and launcher supervision patterns; remove/merge=one combined Duck state, one owning launcher, ordinary Plant tests instead of extra runners; new=one provenance/compatibility manifest and frozen case, one fixed Duck profile/Plant, experimental Duck schemas and two process entry points, one Python policy client, one launcher because each is required by the end-to-end proof; expansion triggers=any new abstraction, protocol change, license obligation, checkpoint/model replacement, physical fallback, parked feature, or relaxed gate requires user re-approval.
Context: must-read=AGENTS.md, docs/plans/open-duck-mini-walk-policy.md, docs/status/active/bootstrap.md, STATUS.md, ARCHITECTURE.md, docs/agents/operating-runbook.md, docs/plans/decision-register.md, crates/soma-core/src/lib.rs, crates/soma-runtime/src/lib.rs, crates/soma-runtime/src/bin/robot-rt.rs, crates/soma-sim/src/lib.rs, proto/soma.proto, python/pyproject.toml; useful=docs/deep-research/fast-open-robot-reference-path.md, docs/deep-research/policy-runtime-interface.md, docs/deep-research/robot-model-manifest-calibration.md; avoid-unless-needed=other historical plans, output traces, retrospectives, Runtime/Playground implementation beyond cited factual compatibility inspection.

Acceptance:
- SUCCESS: every Stage 0-4 gate and the plan Definition of Done passes, including two delayed reference rollouts, two complete Soma gait rollouts, original-deadline stall/fault proof, clean teardown, exact provenance/contracts, and unchanged Reachy gates.
- BLOCKED_NEEDS_DECISION: any stop condition or expansion trigger in the canonical plan; do not silently reshape, tune, copy unlicensed material, or accept partial support.
- BLOCKED_NEEDS_LOCAL_VALIDATION: required MuJoCo/ONNX/native product runs or canonical Reachy simulation gates cannot execute in the available environment.
- INTERMEDIATE_ONLY: none.
- No regressions: existing Reachy soma.v1 schema, keys, sockets, binaries, simulations, teleop, hardware probe, N0/N1 gates, timeline/sequence/TTL/hold semantics, and workspace quality gates remain unchanged and green.

Verification: deterministic=cargo fmt --all -- --check; scripts/cargo-mujoco test --workspace; scripts/cargo-mujoco clippy --workspace --all-targets -- -D warnings; focused golden fixture, 9/14 type/semantic, model mapping, malformed/stale/deadline, allocation-capacity, cleanup tests; integration=scripts/run-sim-scenario and scripts/run-sim-teleop --keys ADQE plus experimental endpoint isolation and synthetic 50 Hz path; product-run=scripts/run-open-duck-walk reference --case <frozen-case>, scripts/run-open-duck-walk policy --case <frozen-case> --repeat 2, scripts/run-open-duck-walk stall --case <frozen-case>; local-live-manual=required native MuJoCo and CPU ONNX Runtime runs in the execution environment, with BLOCKED_NEEDS_LOCAL_VALIDATION if unavailable; optional=none.
Execution: main=root session owns the durable goal, stage gates, worker inspection, integration, stop/complete judgment, and final diff; worker=bounded read-only provenance/reference review or isolated verification only when it improves control; worker-goal=never replaces or completes the root goal.
To execute: /goal execute docs/plans/open-duck-mini-walk-policy.md with intuitive-flow
Optional tracking: none
Approval: already approved in the source conversation; starting the To execute command in the new window authorizes implementation under this contract.
```
