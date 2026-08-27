# Open Duck Mini Walk Policy Plan

> Status: Implementation reached the Stage 4 asynchronous gait stop gate.
> A bounded policy-training reshaping experiment was approved on 2026-08-27.

## Plan Ledger

- `status`: BLOCKED on the published policy; bounded training experiment active
- `appetite`: gate-bounded; no calendar-duration estimate
- `current_slice`: Playground GPU smoke complete; long training not started
- `next_action`: report smoke evidence, then calibrate production-shaped PPO
  throughput and memory before a long baseline or latency-aware run
- `no_touch`: Reachy hardware probe and N0/N1 gates; Open Duck hardware;
  generic robot manifests; ROS 2; a third robot; camera, audio, antennas,
  expression features, interactive control, and silent checkpoint replacement
- `stop_condition`: stop at any gate below; do not start later stages to rescue
  a failed earlier gate or weaken the proof to claim completion

## Decision And Outcome

The user approved reopening the earlier Open Duck deferral on 2026-08-26. This
is one additional fixed simulation profile, not broad robot support. Reachy
Mini remains Soma's primary simulation and hardware path.

On 2026-08-27 the user approved a bounded exception to the original
policy-training non-goal. The published checkpoint passes only under same-tick
application and falls after one 2 ms delayed application, despite inference
taking about 0.4 ms. The exception tests whether a newly trained policy can
meet the existing asynchronous contract; it does not relax the gait floors,
change the runtime contract, or qualify a model merely because training
completes.

The pinned Playground commit `b9be205ac64488c23504ca42e5ec790337adeec3`
has no committed lockfile. Its working training environment is reconstructed
with Python 3.12, `playground==0.0.5`, `brax==0.12.4`, `jax==0.6.2`, and the
remaining dependencies restricted to releases available by 2025-08-05. A
10,240-step RTX 3090 smoke completed and its Orbax checkpoint restored. This is
training-path evidence only, not a usable walking policy.

### Training experiment contract

- keep checkpoints, TensorBoard data, caches, and temporary source patches
  outside the Soma worktree until one candidate passes qualification;
- first run a production-shaped memory/throughput calibration with fixed seed
  0; do not infer long-run cost from the reduced 256-environment smoke;
- train one upstream-baseline candidate before changing latency semantics;
- train one latency-aware candidate that represents Soma action/state timing,
  with the exact delay distribution and implementation recorded before launch;
- never replace the frozen published checkpoint silently; candidates remain
  named experiment artifacts until they pass independent qualification;
- evaluate each candidate in the direct zero-delay, direct 2 ms, direct 20 ms,
  and full supervised Soma process paths against the existing posture,
  displacement, contact, lineage, expiry, and cleanup gates.

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
| 0 | provenance manifest, frozen case, golden contract, direct reference | pinned-bundle and same-tick policy feasibility |
| 1 | narrow `const N` control-core seam | unchanged Reachy semantics and gates |
| 2 | fixed Duck Plant plus runtime/RT path | synthetic 50 Hz end-to-end control |
| 3 | Python ONNX policy plus fault cases | exact observation/action and deadline behavior |
| 4 | two accepted gait runs, cleanup, regressions, compact docs | complete definition of done |

Stop and return for reshaping if:

- the pinned compatibility bundle cannot support a thin Soma policy adapter;
- the pinned checkpoint fails the predeclared same-tick reference gait floors;
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

Stage 0 freezes the following approved compatibility bundle:

- checkpoint source: `apirrone/Open_Duck_Mini` branch `v2`, commit
  `b23317a485b3cec7d8417f352478778b3475173c`, with its Apache-2.0 license,
  and `BEST_WALK_ONNX_2.onnx`;
- checkpoint SHA-256:
  `3c606f9381a1710cc8fecdb7442787dcbfce3ee9bc02a6f1224774ab2b3a1067`;
- verified ONNX contract: `obs` float tensor `[1, 101]` to
  `continuous_actions` float tensor `[1, 14]`;
- reference source: `apirrone/Open_Duck_Playground` commit
  `b9be205ac64488c23504ca42e5ec790337adeec3`. Under the user's direction,
  both repositories are treated as Apache-2.0; the Playground model/meshes and
  runner contract combine with the main repository checkpoint as one pinned
  compatibility bundle.

The minimal runtime bundle is vendored at pinned commits with per-file
provenance: only the flat-terrain MJCF includes, referenced runtime meshes,
checkpoint, and license/upstream records. Soma's policy client remains a thin
adapter and does not vendor the full Playground application, training
environment, print/CAD files, or unused scenes. A maintainer-only update script
may refresh the bundle explicitly; normal build, CI, and runtime never download
it.

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
  500 Hz admission + deadline + evidence -> OpenDuckSimPlant -> MuJoCo
              |
              +-- every 10th tick -> 50 Hz policy frame
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

Duck rate separation is fixed: `robot-rt` and the MuJoCo Plant run every 2 ms
(500 Hz), advancing exactly one physics step; every tenth completed RT tick
produces one 50 Hz policy frame. Reachy's existing 20 ms RT period and ten-
substep Plant schedule remain unchanged.

The runtime publishes one combined experimental Open Duck state at 50 Hz, not
separate actuator, policy-frame, and evidence streams. It includes positions,
velocities, gyro, acceleration, contacts, health, timeline, source capture
tick/time, and command evidence. The policy emits at most one target for each
complete state. The runtime coalesces to the latest target and sends at most
one command per 2 ms RT tick; the RT owner performs a bounded drain and records
drops rather than allowing backlog to renew old commands. This faster RT tick
is required for apply-as-soon-as-admitted semantics; batching ten physics
steps into a 20 ms Duck tick would recreate the rejected fixed-frame delay.

Open Duck uses profile-specific experimental keys, schemas, and socket paths.
Concurrent Reachy/Open Duck operation is not required, but tests must prove
their endpoints cannot cross-wire.

## Fixed Embodiment Contract

The policy controls exactly 14 positions in this order:

```text
left_hip_yaw, left_hip_roll, left_hip_pitch, left_knee, left_ankle,
neck_pitch, head_pitch, head_yaw, head_roll,
right_hip_yaw, right_hip_roll, right_hip_pitch, right_knee, right_ankle
```

The canonical Playground model compiles to `nq = 21`, `nv = 20`, and `nu = 14`.
It is used directly as the compatibility model instead of deriving a new
14-actuator model from the main repository's 16-actuator variant. Stage 0 must
still query and record every actuator and joint name and validate limits,
initial pose, sensors, contacts, and timestep. Index slicing or an unexplained
count is a hard stop.

The verified 101-element observation contract is frozen by a named golden tick:

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
patch; RT/physics/policy cadence; and event order. The official motor target
slew limit of `5.24 rad/s` is applied once per 20 ms policy update and is part
of the compatibility contract. The disabled
low-pass action filter is not enabled in Soma unless a separately frozen
checkpoint contract requires it.

## Timeline, Deadline, And Fallback

The normal route is asynchronous latest-value control, not a fixed one-policy-
tick pipeline. State tick `k` is published immediately, Python infers and sends
its target without phase padding, and `robot-rt` applies an admitted target on
the first available RT tick. Observation-to-application latency is therefore
measured and bounded rather than defined as exactly 20 ms. The most recent
target is held with zero-order hold until replaced or expired.

The direct reference preserves the official same-tick ordering: observe,
infer, update the motor target, then continue physics. An injected 20 ms delay
is a transport/fault diagnostic only. It must preserve lineage, consume the
original deadline, and reach the named fallback when expired; it does not need
to meet the normal gait floor.

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
- seed and deterministic settings, 2 ms RT/physics cadence, 50 Hz policy
  cadence and decimation,
  reset pose, command vector, duration, the normal apply-as-soon-as-admitted
  schedule, and the injected-delay schedule;
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
- verify the complete 14-actuator name/order mapping;
- extract the exact 101-field ledger and golden observation/action fixtures;
- run the same-tick reference rollouts and injected-delay robustness cases;
- record latency, contacts, slip, pose, displacement, and repeatability.

Gate: all provenance is compatible, fixtures are exact and finite, and two
same-tick reference runs meet every predeclared gait floor. Injected-delay
cases must pass timing, lineage, expiry, and boundedness assertions. Otherwise
stop without touching Reachy code.

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
  reset, named joints, state facts, contacts, 2 ms RT/physics cadence, and
  10-tick policy-frame decimation;
- add fixed Open Duck runtime/RT entry points and experimental external
  messages using the existing bounded transport mechanism;
- publish one 50 Hz combined state from every tenth RT tick and transport one
  coalesced latest target;
- use ordinary Rust tests, not another standalone runner, for Plant contracts;
- run one synthetic 50 Hz target source through the complete 500 Hz RT path.

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
- add optional MuJoCo and Rerun observers using the existing Reachy Mini
  lossy, read-only snapshot pattern; observer loss or closure must not affect
  the headless rollout;
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
- exact 14-joint mapping inside the validated 14-actuator model;
- exact 101-value golden fixture, ONNX output fixture, units, frames, reset,
  cadence, event order, dtype, and toolchain pins;
- two same-tick reference and two complete Soma runs meeting the predeclared
  uprightness, displacement, alternating-contact, non-collision, low-slip,
  duration, and repeatability floors; injected-delay cases must meet timing,
  lineage, expiry, and boundedness assertions;
- original-deadline consumption across capture, inference, middleware, local
  delivery, and RT admission, with no stale backlog revival;
- complete source/request/admission/application lineage and named hold evidence;
- bounded message sizes/drain, no periodic buffer growth, bounded tick lateness
  and message age, clean process/endpoint teardown;
- unchanged Reachy public protocol, runtime behavior, simulations, hardware
  gates, and canonical tests;
- no Open Duck hardware, interactive control, generic robot/policy
  framework, simultaneous-profile feature, or third embodiment; MuJoCo and
  Rerun visualization remain optional observational outputs.

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

Hardware commands and visual judgment as an acceptance oracle are absent by
design. The optional visualization path is exercised for startup, observation,
independent closure, and teardown.

## Parked And Rejected

Parked until this headless outcome passes: interactive velocity input,
simultaneous Reachy/Open Duck launch, showcase regeneration, generic policy
bundles, broad launcher refactors, optional filtering, exhaustive
malformed-model matrices, hardware, training beyond the approved bounded
experiment, ROS 2, and any additional robot.
The first accepted Duck rollout also provides optional MuJoCo and Rerun
visualization, following the existing Reachy Mini observer path; visualization
is off the cyclic control path and is never the gait oracle.

Rejected for this plan: copying the full Playground application as Soma's
runtime; choosing thresholds after reference output; treating two
identical deterministic runs as sufficient walking proof; a second IPC
technology; a runtime manifest; dynamic robot descriptions; generic sensor or
observation interfaces; weakening Reachy semantics; or claiming stale-frame
safety without source lineage.

## Cross-Process Review Trigger

The first Duck implementation retains the existing Python policy -> loopback
Zenoh -> Rust runtime -> bounded Unix datagram -> Rust RT topology. This is a
deliberate isolation choice, not a claim that more processes are inherently
safer or more reliable. After Stage 4 records inference latency, message age,
drops, deadline expiry, process failure, and teardown evidence, run a focused
architecture review comparing at least:

- the current isolated Python policy process;
- ONNX Runtime hosted by Rust in `robot-runtime`, with async/middleware work
  still outside `robot-rt`;
- one Rust process with isolated async runtime and periodic RT thread, using a
  bounded latest-value handoff.

The review must compare failure containment, scheduling jitter, stale-data
proof, ownership and shutdown, dependency/runtime complexity, observability,
and whether process isolation adds a demonstrated safety property. It must not
move inference or async work into the periodic RT section. This is the D-04
reversal trigger; it is not part of the Duck Stage 0-4 implementation.

## User-Review Gates

Return to the user before:

- accepting a partial outcome or relaxing frozen gait floors;
- vendoring the full Playground application or changing the approved minimal
  bundle/provenance boundary;
- changing the checkpoint/model pair after Stage 0 starts;
- changing Reachy's external schema, keys, behavior, hardware gates, or N0/N1;
- replacing the named hold, expanding training beyond the bounded experiment,
  adding hardware/generic abstractions, or
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
- Deepen only fixed-size actuator admission to const N; then add the fixed Duck Plant, experimental transport, Python policy client, deadline/fault evidence, launcher, optional MuJoCo/Rerun observers, tests, and compact docs.
Non-goals: Open Duck hardware; policy training beyond the bounded experiment recorded above; interactive control; simultaneous profiles; ROS 2; generic manifests, registries, device factories, sensor capabilities, observation DSLs, policy bundles, or a third robot; Reachy N0/N1 or public-contract changes. Optional MuJoCo/Rerun observation is in scope but never authoritative.
Entity budget: reuse=ControlCore semantics, soma-core Plant boundary, soma-sim MuJoCo patterns, existing Protobuf/Zenoh/nonblocking Unix datagram transport, Python client, launcher supervision, and Reachy observer patterns; remove/merge=one combined Duck state, one owning launcher, ordinary Plant tests instead of extra runners; new=one provenance/compatibility manifest and frozen case, one fixed Duck profile/Plant, experimental Duck schemas and two process entry points, one Python policy client, one launcher because each is required by the end-to-end proof; expansion triggers=any new abstraction, protocol change, license obligation, checkpoint/model replacement, physical fallback, parked feature, or relaxed gate requires user re-approval.
Context: must-read=AGENTS.md, docs/plans/open-duck-mini-walk-policy.md, docs/status/active/bootstrap.md, STATUS.md, ARCHITECTURE.md, docs/agents/operating-runbook.md, docs/plans/decision-register.md, crates/soma-core/src/lib.rs, crates/soma-runtime/src/lib.rs, crates/soma-runtime/src/bin/robot-rt.rs, crates/soma-sim/src/lib.rs, proto/soma.proto, python/pyproject.toml; useful=docs/deep-research/fast-open-robot-reference-path.md, docs/deep-research/policy-runtime-interface.md, docs/deep-research/robot-model-manifest-calibration.md; avoid-unless-needed=other historical plans, output traces, retrospectives, and Playground training/full-application code beyond the pinned runner/model compatibility surface.

Acceptance:
- SUCCESS: every Stage 0-4 gate and the plan Definition of Done passes, including two same-tick reference rollouts, two complete Soma gait rollouts, injected-delay and original-deadline stall/fault proof, optional MuJoCo/Rerun observation, clean teardown, exact provenance/contracts, and unchanged Reachy gates.
- BLOCKED_NEEDS_DECISION: any stop condition or expansion trigger in the canonical plan; do not silently reshape, tune, replace the pinned bundle, or accept partial support.
- BLOCKED_NEEDS_LOCAL_VALIDATION: required MuJoCo/ONNX/native product runs or canonical Reachy simulation gates cannot execute in the available environment.
- INTERMEDIATE_ONLY: none.
- No regressions: existing Reachy soma.v1 schema, keys, sockets, binaries, simulations, teleop, hardware probe, N0/N1 gates, timeline/sequence/TTL/hold semantics, and workspace quality gates remain unchanged and green.

Verification: deterministic=cargo fmt --all -- --check; scripts/cargo-mujoco test --workspace; scripts/cargo-mujoco clippy --workspace --all-targets -- -D warnings; focused golden fixture, 9/14 type/semantic, model mapping, malformed/stale/deadline, allocation-capacity, cleanup tests; integration=scripts/run-sim-scenario and scripts/run-sim-teleop --keys ADQE plus experimental endpoint isolation, synthetic 50 Hz path, and Xvfb observer startup/independent-closure/teardown; product-run=scripts/run-open-duck-walk reference --case <frozen-case>, scripts/run-open-duck-walk policy --case <frozen-case> --repeat 2, scripts/run-open-duck-walk policy --case <frozen-case> --visualize, scripts/run-open-duck-walk stall --case <frozen-case>; local-live-manual=required native MuJoCo and CPU ONNX Runtime runs plus MuJoCo/Rerun desktop review in the execution environment, with BLOCKED_NEEDS_LOCAL_VALIDATION if unavailable; optional=none.
Execution: main=root session owns the durable goal, stage gates, worker inspection, integration, stop/complete judgment, and final diff; worker=bounded read-only provenance/reference review or isolated verification only when it improves control; worker-goal=never replaces or completes the root goal.
To execute: /goal execute docs/plans/open-duck-mini-walk-policy.md with intuitive-flow
Optional tracking: none
Approval: already approved in the source conversation; starting the To execute command in the new window authorizes implementation under this contract.
```
