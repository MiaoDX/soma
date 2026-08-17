# Soma Bootstrap Plan

> Status: Reviewed implementation plan. The long-term architecture is complete at a system-boundary level; only Milestones 0 and 1 are current implementation scope. Later milestones reopen when their hardware or product trigger exists.

## Outcome

Soma has two sequential goals:

1. establish a technically defensible, community-backed system direction that can be reviewed with colleagues;
2. implement a narrow runnable spine, then expand downward and outward without replacing its public contracts.

The current lack of hardware is a constraint, not a reason to couple the architecture to one simulator. V0 proceeds top-down from MuJoCo/SIL and a G1/G2-class exposed interface boundary. Physical and bottom-up qualification remain explicit later milestones and cannot be claimed from mocks or vendor documentation.

## Runnable spine

```text
Python SDK                    synthetic 20 Hz cadence source
    |                         (lookup table or sine, no trained
    | Robot Protocol           model required — see D-19)
    | over provisional Zenoh          |
    v                                 v
robot-runtime  <-------------------- command stream
  auth boundary | lease | timing | capability | lifecycle | record
    |
    | bounded RT/runtime mailbox contract
    v
robot-rt
  representative ControlCore -> SA-3 -> applied evidence
    |                            ^
    |                            | command_staleness_policy
    |                            | (hold / decay / fallback / inhibit)
    | bounded Plant contract
    v
MuJoCo Plant
    |
    +--> state / events / requested-to-applied lineage
    +--> execution journal + MCAP evidence

parallel contract fixture, not a second runtime:
fake/recorded G1/G2-class boundary -> adapter conformance
```

This is the smallest slice that exercises the architecture rather than merely demonstrating a simulator. The synthetic cadence source exists specifically to force the timeout, interpolation, and staleness code paths defined in [`policy-runtime-interface.md`](../deep-research/policy-runtime-interface.md) (D-19) to exist before a trained policy or real hardware arrives — it is a control-and-timing fixture, not a step toward a real policy.

## Implementation maturity map

| Direction | Design now | Implement now | Later evidence before a production claim |
| --- | --- | --- | --- |
| Layers and trust | L-2..L5, SA-0..SA-5, Plant and runtime boundaries | Preserve boundaries in module/API layout | Hazard analysis and physical authority validation |
| Time | Clock domains, timeline/generation split, command validity invariants | Time ADR, simulation clock, reset/restart rejection | Target clock/PTP/bus measurements |
| Model | Minimal `ProductModelManifest`, separate instance/calibration/control/safety artifacts | Native MJCF fixture plus validator; second embodiment fixture | Physical inventory/calibration and additional backend mappings |
| RT execution | Restricted cyclic profile and replaceable execution kernel | Copper-vs-minimal representative graph spike | PREEMPT_RT and target-compute worst-case evidence |
| RT/runtime IPC | Bounded mailbox behavior, ABI/generation/restart rules | Community-vs-minimal spike and one selected implementation | Target cache/NUMA/process-failure characterization |
| Distributed protocol | Transport-neutral Robot Protocol, Zenoh provisional default | Minimal state/command/event/lease/capability surface | Triggered Cyclone comparison only if envelope or interoperability requires it |
| Simulation | Same ControlCore/SA-3/Plant semantics, separate Simulation Control | MuJoCo lockstep/reset/fault scenarios | Additional simulators only for a concrete product need |
| Evidence | Requested/admitted/safety-output/applied lineage and replay contract | Representative executable replay plus MCAP; recorder interference test | Incident retention, privacy, upload, fleet correlation |
| Security | Threats, trust boundaries, artifact and developer-mode invariants | Signed development artifact verifier and its functional tests | Protocol abuse, fuzzing, DoS, key ceremony, physical debug qualification |
| Hardware | `NativePlantAdapter` vs `ManagedMotionGateway`; capability honesty | Hardware-free adapter conformance only | Actual vendor boundary, then owner-controlled L0/L-1/L-2 qualification |
| Fleet | Release, rollout, recovery, offline-first evidence model | Candidate `ReleaseManifest` identity only | Production signing, canary, rollback, SBOM, operations and support |

## Decision and evidence flow

The [Decision and Research Register](decision-register.md) is the index for open and provisional directions:

```text
decision -> deep research -> bounded experiment -> ADR -> implementation
```

Use an ADR for a durable decision, not for every implementation detail. A reversible default can remain provisional when it has an explicit trigger for reevaluation.

## Evidence model

Evidence labels describe where a claim was actually tested. They are not interchangeable maturity badges.

| Evidence | What runs | Claims it can support | Claims it cannot support by itself |
| --- | --- | --- | --- |
| `HOST` | host unit, schema, cryptographic, composition, and packaging checks | validators, canonicalization, signature policy, API/schema compatibility | deployed scheduling, simulator dynamics, bus/device behavior, physical safety |
| `SIM` | simulator-native model, backend, or API facsimile | asset mapping, simulator adapter behavior, scenario generation, throughput | production runtime path, fieldbus/firmware behavior, physical safety |
| `SIL` | production control/runtime code against a simulated Plant or virtual device | control semantics, lifecycle, timing logic, reset, deterministic regression, software faults | electrical behavior, real bus timing, boot-chain enforcement, physical dynamics |
| `HIL` | production software with target-equivalent compute, bus, board, drive, sensor, or safety/power component in the loop | driver/device state, watchdog, timestamp, firmware/update, bus/power fault behavior | unrepresented mechanism, contact, payload, thermal, or safety behavior |
| `PHYSICAL` | controlled actuator, mechanism, or whole robot | real dynamics, calibration, power/thermal and physical fault response for the tested configuration | certification or untested product/general safety claims |

Each dynamic result records scenario ID, loop composition, hardware/model/release identities, scenario configuration hash, measured tolerances, and retained evidence. `unknown`, `documented`, and `vendor_asserted` never become verified physical evidence by implication.

## Milestone 0: architecture convergence

### Deliverables

- canonical reference architecture, layer/safety vocabulary, and high-level Threat Model;
- Decision Register with priority, status, evidence, reversal trigger, and next action;
- Time, simulation, RT/runtime, protocol, model, safety, OTA, and reference-project research sufficient for V0 ADRs;
- lightweight verification matrix and Performance Envelope template;
- initial `ReleaseManifest` maturity model: `draft -> candidate -> qualified -> released`;
- one agreed runnable-spine scope and explicit non-goals.

### Exit criteria

- canonical docs use one meaning for Plant, timeline, generation, capability, safety authority, and artifact identity;
- every P0 decision is accepted, provisional with a trigger, or assigned a bounded experiment;
- no physical, security-qualified, deterministic-replay, or hard-RT claim exceeds its evidence;
- the runnable spine can begin without selecting physical hardware.

## Milestone 1: no-hardware runnable spine

Milestone 1 has three checkpoints so qualification work does not block the
first executable system, and so the hardest, most research-like piece
(executable stateful replay) cannot silently stall the rest of V0:

| Checkpoint | Purpose | Required result |
| --- | --- | --- |
| **M1a: runs** | prove the architecture can execute end to end | Python lease/command/state through separate runtime/RT processes into MuJoCo, with command lineage and reset/timeline rejection, driven in part by a synthetic cadence source that exercises `command_staleness_policy` (D-19) |
| **M1b: credible** | prove the bounded-engineering V0 claims are measurable and failure-aware | recovery paths, bounded IPC negatives, recorder pressure, adapter conformance, signed verifier and installable candidate |
| **M1c: reproducible** | prove recorded sessions can be replayed and trusted, or state plainly that they cannot yet | executable stateful replay with the four negative cases (omitted state, clock bypass, corrupt log, incompatible build), and the D-09 Copper-vs-native decision |

M1a is the first implementation target. M1b completes the bounded-engineering
half of V0. M1c is scoped separately because stateful replay correctness is a
harder, more open-ended problem than the rest of M1 (see the "Replay
guarantees are a system contract" findings in
[`runtime-and-platform-reference-projects.md`](../deep-research/runtime-and-platform-reference-projects.md));
it is allowed to conclude "not yet, replay is message-level only" without
blocking M1a/M1b from being reviewable and mergeable.

### Contracts and fixtures

- minimal `ProductModelManifest` and validator;
- backend-native MuJoCo MJCF for a legged or manipulator fixture plus a second embodiment manifest fixture;
- fixed-layout control state/command, Plant, Clock, lifecycle, lease, capability, and command-lineage semantics;
- `InterfaceProfile` only for an adapter's exposed boundary; stable capability names come from the shared `CapabilityCatalog`;
- minimal `ReleaseManifest` that pins build, dependencies, model, scenario, configuration, test profile, exclusions, and evidence identity.

### RT and simulation

- representative Plant -> estimator/controller -> limiter -> SA-3 -> applied sink graph;
- Copper-hosted and minimal Soma-native spike behind the same Plant/task boundary;
- MuJoCo Plant with virtual time, lockstep, pause/step/reset, fault injection, and timeline invalidation;
- allocator detection after activation and Performance Envelope reporting;
- bounded RT/runtime mailbox spike and selected implementation.

### Runtime, protocol, and SDK

- `robot-runtime` with lease/arbitration, lifecycle, capability discovery, timing admission, structured events, recording, and operations modules;
- minimal Robot Protocol for joint/base state, command, events, leases, capabilities, health, and version negotiation;
- provisional Zenoh transport without exposing Zenoh-specific names as the public semantic source of truth;
- Rust client plus a thin Python binding/package;
- reconnect, lease loss, slow-consumer, and runtime-restart behavior in the integrated SDK flow.

### Evidence and replay

- requested, admitted, safety-output, and applied command identity at every stage;
- execution journal/checkpoints for stateful replay and MCAP for tool-neutral incident/interchange data;
- replay manifest covering build, graph, clock, model, scenario, profiles, exclusions, and loss counters;
- omitted-state, clock-bypass/unrecorded-input, corrupt-log, and incompatible-build negative cases;
- recorder enabled/disabled/saturated/disk-pressure comparison against the same workload.

### Hardware-free adapter conformance

A fake or recorded G1/G2-class boundary validates Soma's side of the integration without pretending to emulate vendor internals:

- classify the boundary as `NativePlantAdapter` or `ManagedMotionGateway`;
- declare supported and unsupported capabilities conservatively;
- validate units, frames, modes, lifecycle, command/state mapping, and typed unsupported errors;
- stop requested-to-applied claims at the last observable boundary;
- keep vendor types and assumptions inside the adapter;
- produce only `HOST`, `SIM`, or `SIL` evidence, never `PHYSICAL` evidence.

### Security posture

V0 implements the signed development artifact verifier and tests accepted, altered, unsigned, identity-incompatible, and rollback-disallowed inputs. Broader protocol abuse-case qualification remains explicitly outside this milestone, per the Threat Model; the runtime is not production-security-qualified at Milestone 1.

### Distribution matrix

| Artifact | V0 target | Delivery path | Required proof |
| --- | --- | --- | --- |
| `robot-rt` | Linux x86_64 developer/SIL profile | pinned Cargo workspace build | reproducible build identity, restricted profile, tests and benchmark report |
| `robot-runtime` | Linux x86_64 developer/SIL profile | same candidate release set as `robot-rt` | atomic IPC ABI compatibility and startup/restart scenario |
| Python SDK | one supported CPython range on Linux x86_64 | local wheel produced in CI | clean-environment install and integrated SDK scenario |
| MuJoCo model/scenarios | pinned MuJoCo version | content-addressed release assets | validator and scenario hashes, deterministic/tolerance report |
| Evidence tooling | Linux developer profile | workspace binaries/modules | replay negative cases and MCAP readability |
| Future aarch64/ROS/gateway artifacts | not V0 | added by a target-specific distribution profile | must not change core protocol semantics |

### Demonstration

```text
start candidate release
 -> Python SDK connects and discovers capabilities
 -> acquires lease and commands MuJoCo
 -> receives state, health, and command-decision evidence
 -> reset creates a new Plant timeline and rejects stale command
 -> runtime restart invalidates old lease but not the Plant timeline
 -> SDK reconnects and explicitly reacquires authority
 -> session records, restores state, re-executes, and compares output
```

## Milestone verification matrix

This is an index, not a test DSL. Scenario implementation details live beside the code when implementation begins.

| Claim | Scenario | Evidence | Pass condition | Retained artifact | Milestone |
| --- | --- | --- | --- | --- | --- |
| Shared model semantics span embodiments | `model-two-fixtures` | `HOST` | two different embodiments validate without public protocol branches | manifest fixtures + validator report | M1a |
| Same control path drives simulation | `sdk-mujoco-roundtrip` | `SIL` | SDK -> runtime -> RT -> MuJoCo -> state/evidence completes | MCAP + release/scenario IDs | M1a |
| Timeline prevents stale control | `reset-with-inflight-command` | `SIL` | reset changes timeline; prior command is rejected with reason | event and command-lineage record | M1a |
| A low-rate command source can safely drive the 1 kHz loop | `cadence-source-decimation` | `SIL` | synthetic 20 Hz source drives the loop; a missed tick reaches the declared `command_staleness_policy` fallback and is recorded with attribution | command-lineage record + D-19 policy identity | M1a |
| Runtime generations and leases recover explicitly | `runtime-restart-reconnect` | `SIL` | Plant timeline persists; old lease fails; SDK reacquires | lifecycle/lease trace | M1b |
| Slow consumers cannot block RT | `subscriber-overload` | `SIL` | bounded drop/age counters increase; RT envelope holds | benchmark + counters | M1b |
| RT/runtime IPC is bounded and restart-safe | `ipc-overflow-generation-abi` | `HOST`/`SIL` | no cyclic allocation/blocking; stale or incompatible region cannot enable motion | test and benchmark report | M1b |
| Recorder does not perturb control silently | `recorder-pressure` | `SIL` | RT envelope holds or failure is explicit; losses invalidate completeness claim | enabled/disabled/pressure report | M1b |
| Replay is executable and honest | `stateful-replay-negative-cases` | `HOST`/`SIL` | valid replay compares as declared; omission/corruption/incompatibility fails visibly | source and replay journals + manifest | M1c |
| Vendor boundary stays contained | `adapter-contract-fixture` | `HOST`/`SIM` | classification, mappings, capabilities, typed unsupported errors and evidence limits pass | conformance report | M1b |
| Ordinary artifacts cannot relax safety profile | `artifact-authority-separation` | `HOST`/`SIL` | model/control/policy/scenario changes leave active `SafetyProfile` identity and bounds unchanged | activation/evidence trace | M1b |
| Development verifier rejects invalid artifacts | `signed-artifact-negative-cases` | `HOST` | modified, unsigned, incompatible and disallowed rollback inputs are rejected audibly | verifier report | M1b |
| A physical boundary behaves as declared | target-specific matched scenario | `PHYSICAL` | supported subset works and inaccessible authority is not claimed | platform qualification bundle | M2+ |

## Test coverage view

There is no code or test framework yet, so executable coverage is currently 0. The matrix above defines planned behavioral coverage, not achieved coverage.

```text
CODE / SYSTEM PATHS                              USER / OPERATOR FLOWS

[PLANNED M1] Python SDK                          [PLANNED M1] normal command session
  +-- connect / discover                           +-- acquire lease -> command -> observe
  +-- reconnect / runtime generation               +-- reset -> stale command rejected
  +-- lease loss / explicit reacquire               +-- runtime restart -> recover explicitly
  +-- slow consumer / gap visibility                +-- unsupported capability -> typed error
          |
[PLANNED M1] Robot Protocol
  +-- lease / timing / capability admission
  +-- requested -> admitted evidence
          |
[PLANNED M1] bounded RT IPC
  +-- overflow / generation / ABI mismatch
          |
[PLANNED M1] ControlCore + SA-3
  +-- safety-output -> applied evidence
          |
[PLANNED M1] MuJoCo Plant
  +-- step / reset / injected fault
          |
[PLANNED M1] recording and executable replay
  +-- omitted state / clock bypass
  +-- corrupt log / incompatible build

[DEFERRED] protocol security abuse, fuzz, sustained DoS and penetration tests
```

Implementation modules that should retain small inline ASCII diagrams once code exists:

- lifecycle/generation types: reset and restart state transitions;
- RT/runtime IPC: header validation, generation handshake and overwrite/drop behavior;
- runtime admission: requested -> admitted -> rejected decision flow;
- replay service: record -> checkpoint -> restore -> re-execute -> compare;
- adapter boundary: vendor input/output mapping and last observable evidence stage.

## Performance Envelope

Do not put universal numbers in the architecture without a workload and platform. Every benchmark records:

```text
profile:
  CPU / kernel / scheduler / simulator / build
  graph and message payloads
  client/subscriber count
  recorder mode and storage
  warm-up, duration, repetitions

RT cycle:
  target rate, p50/p99/p99.9/max, deadline misses, cyclic allocations
RT IPC:
  latency distribution, sample age, drops/overwrites, CPU cost
runtime/protocol:
  throughput, latency distribution, queue age, reconnect and overload behavior
recorder:
  sustained write rate, queue depth/loss, disk-pressure degradation, RT delta
simulation:
  step rate, real-time factor, state/event tolerance and repeatability
```

The representative Copper/minimal spike starts at 1 kHz, but the rate is a test workload, not a permanent requirement for every embodiment.

## Milestone 2: first accessible robot boundary

Trigger: suitable hardware becomes accessible and a product/research need justifies integration.

- evaluate the actual boundary, license, model assets, commissioning, update/recovery, and safety authority;
- implement either a `NativePlantAdapter` or `ManagedMotionGateway` without forking the common runtime or SDK;
- rerun the M1 adapter and SDK scenarios against hardware for the supported subset;
- measure target compute, network, timestamp, and observable command behavior;
- publish unknown and inaccessible lower-layer capabilities instead of inferring them;
- do not claim Soma `SA-3` coverage for a managed vendor motion boundary.

## Milestone 3: owner-controlled lower stack

Trigger: Soma has its own hardware, an accessible component bench, or a concrete need to own L0/L-1/L-2.

- mechanically constrained actuator then synchronized multi-axis fixture;
- owned or reproducibly built firmware/HAL for each claimed layer;
- fieldbus timing, calibration, device lifecycle, watchdog, update/recovery, and diagnostics;
- independent stop/torque-inhibition or energy-control path selected by hazard analysis;
- HIL and PHYSICAL evidence for bus loss, process death, timestamp faults, update interruption, power/thermal limits, and command application;
- whole-robot integration only after component evidence can support the intended ownership claim.

This milestone may begin from different layers. It does not require rewriting the M1 public protocol, time, Plant, evidence, or SDK contracts.

## Milestone 4: production and fleet qualification

- production provisioning, signing roles, secure boot, anti-rollback, rotation/revocation, recovery and decommissioning;
- protocol abuse, fuzzing, penetration, sustained DoS, and physical debug-interface qualification;
- reproducible multi-target builds, SBOM/provenance and vulnerability response;
- candidate qualification across required SIM/SIL/HIL/PHYSICAL gates;
- canary rollout, halt/rollback, offline operation, incident retention/access/privacy and support runbooks;
- compatibility, deprecation, ROS distribution and LTS policies.

## Failure modes

| Path | Realistic failure | Planned test | Planned handling / visibility |
| --- | --- | --- | --- |
| SDK connection | network loss or runtime restart | `runtime-restart-reconnect` | connection state and generation change are visible; explicit reacquire |
| Lease | ownership revoked or stale renewal | same integrated scenario | command stops; typed lease event/error |
| Command admission | old timeline, deadline, mode, or sequence | `reset-with-inflight-command` plus unit cases | rejected with attributable decision evidence |
| RT IPC | producer crash, overflow, corrupt/incompatible header | `ipc-overflow-generation-abi` | motion remains inhibited; counters/reason visible |
| Control/Plant | invalid state or missed cycle | MuJoCo fault scenarios | SA-3 safe behavior and structured event; lower safety required on hardware |
| Model activation | wrong joint/frame/artifact identity | validator and activation tests | fail before actuation with mismatch reason |
| Replay | omitted state, nondeterminism, corruption, incompatible build | `stateful-replay-negative-cases` | mismatch/rejection; never silent success |
| Recorder | queue saturation, dirty-page pressure, full disk | `recorder-pressure` | bounded degradation and loss counters; no false completeness |
| Vendor adapter | unsupported feature or hidden vendor authority | `adapter-contract-fixture` | typed unsupported result and bounded evidence claim |
| Artifact verifier | altered, unsigned, incompatible or rollback-disallowed input | `signed-artifact-negative-cases` | preserve current state and audit rejection |
| Protocol attack | malformed/replayed/excessive client traffic | deferred to M4 by decision | M1 makes no production-security claim; Threat Model preserves required invariant |

No M1 failure mode is both silently handled and claimed as verified without a planned test. Deferred security cases block a production-security claim, not the no-hardware architecture demonstration.

## What already exists

The repository currently contains research and architecture documents only: no workspace, executable code, test framework, CI, package, or release pipeline exists yet. Soma should reuse community mechanisms and retain only product-specific contracts:

| Need | Existing reference or implementation | Soma-owned remainder |
| --- | --- | --- |
| Physics/SIL | MuJoCo, PX4/ArduPilot SITL patterns | Plant, time/reset, command lineage and conformance semantics |
| RT graph and replay | Copper | adoption spike, Plant/authority integration and honest replay manifest |
| Distributed transport | Zenoh; Cyclone DDS gateway ecosystem | Robot Protocol semantics, lease/timing/capability policy |
| Local bulk IPC | iceoryx2 candidate | measured choice and payload lifetime policy |
| Evidence container | MCAP and ecosystem tools | execution state/checkpoints, identities and completeness rules |
| Lifecycle/integration evidence | Eclipse S-CORE patterns | small Soma lifecycle contract and maturity-bearing `ReleaseManifest` |
| Host/device updates | RAUC/OSTree/Mender, MCUboot, TUF/Uptane patterns | product compatibility, authority, health and recovery policy |
| ROS ecosystem | ROS 2 bridges and standard messages | mapping at the edge without core dependency |
| Vendor surfaces | Unitree/AgiBot and other public SDK/sim assets | honest boundary classification and capability conformance |

Soma does not need to own a general middleware, universal model language, automotive platform API, logging container, update framework, or simulator engine.

## NOT in scope for Milestone 1

- physical robot, actuator, fieldbus, E-stop, STO, brake, power, thermal, secure-boot, or hard-RT qualification: no hardware exists;
- a universal URDF/MJCF/USD compiler or lossless conversion layer: V0 validates native assets;
- complete G1/G2 emulators or vendor-internal behavior: adapter fixtures validate only Soma's boundary;
- a mandatory Zenoh/Cyclone bake-off: comparison is trigger-based;
- independent update/health/observer/recorder daemons: these start as `robot-runtime` modules;
- full protocol security abuse, penetration, fuzz, DoS, production key, or physical debug testing: deferred to M4;
- Isaac, Genesis, HIL, batch-RL infrastructure, ROS distribution matrix, fleet rollout, cloud service, long-term retention, or UI tooling;
- universal plugin ABI, cross-version internal RT IPC compatibility, or multi-repository integration platform;
- safety certification or claims about inaccessible vendor safety and lower-stack implementation.

## Parallel implementation lanes

Contracts are a short sequential gate; implementation then separates into bounded lanes.

| Step | Modules | Depends on |
| --- | --- | --- |
| Foundation | shared types, model fixtures, scenario IDs, release identity, RT/runtime IPC (D-05) | M0 decisions |
| Lane A: control and simulation | `robot-rt`, Plant, MuJoCo, minimal native execution graph, RT benchmark | Foundation |
| Lane B: runtime and SDK | `robot-runtime`, protocol, Rust/Python client | Foundation |
| Lane C: execution evidence | Copper-vs-native comparison (D-09), replay, MCAP, recorder | Lane A's minimal graph, not Foundation directly |
| Lane D: adapter conformance | adapter fixtures, `InterfaceProfile`, capability cases | Foundation |
| Integration | end-to-end recovery, reset, overload, Performance Envelope, packaging | A + B + C + D |

Execution order:

```text
Foundation (shared types, RT/runtime IPC)
   |
   +--> Lane A: minimal native graph -----+--> Lane C: Copper comparison --+
   +--> Lane B ---------------------------|                                +--> integration and M1 qualification
   +--> Lane D ---------------------------+--------------------------------+
```

RT/runtime IPC (D-05) moved into Foundation rather than its own lane: M1a's
definition requires commanding MuJoCo through separate `robot-runtime` and
`robot-rt` processes, so the mailbox is on the M1a critical path, not an
optional comparison alongside it.

Lane C is deliberately sequential after Lane A's minimal native graph exists,
not parallel with it. D-09's adoption criteria (see the Copper adoption
spike in [`runtime-and-platform-reference-projects.md`](../deep-research/runtime-and-platform-reference-projects.md))
require comparing Copper against a working Soma-native baseline; comparing it
against a graph that does not exist yet is not a comparison. Lanes A/B share
fixed-layout types but should not edit those types independently after the
Foundation checkpoint.

## Implementation Tasks

- [x] **T0 (P0)** — Repo hygiene — add LICENSE, `.gitignore`, and baseline doc lint/link-check CI ahead of the first workspace commit.
  - Surfaced by: Architecture Review — T2 assumes CI configuration exists; a public repository about to receive a Rust workspace had neither a license nor a `.gitignore`.
  - Files: `LICENSE`, `.gitignore`, `docs/architecture/diagrams/`.
  - Verify: LICENSE present; `cargo build` artifacts do not appear in `git status`; documentation links resolve.
- [ ] **T0b (P0)** — Target-compute baseline (D-20) — measure 1 kHz max latency, allocation-free execution, and SPSC-mailbox crash behavior on a representative embedded board.
  - Surfaced by: Architecture Review — the two-process Linux/PREEMPT_RT assumption the whole design rests on was going untested through all of M1 because "no hardware" was read as "no robot," when it does not require a robot to test.
  - Files: `docs/measurements/`, a standalone `cyclictest`/allocation-detector harness (not part of the `robot-rt` workspace).
  - Verify: 24 h soak reports **max** latency (not p99) on the target board; SPSC ring survives `SIGKILL` on either side; results published and referenced by D-05 and D-09 thresholds.
- [ ] **T1 (P1)** — Decisions — record P0 ADRs that have evidence and write bounded protocols for experiment-dependent decisions.
  - Surfaced by: Architecture Review — research and decisions lacked a consistent convergence path.
  - Files: `docs/decisions/`, `docs/plans/decision-register.md`, relevant Deep Research.
  - Verify: every P0 row is accepted, provisional with trigger, or has a measurable experiment; spike results later close their own ADRs.
- [ ] **T2 (P1)** — Foundation — create the minimal workspace, shared contracts, model fixtures, validator and candidate release identity.
  - Surfaced by: Scope and Code Quality Review — runnable spine must precede the broad long-term module surface.
  - Files: initial workspace modules and CI/build configuration.
  - Verify: clean build, validator cases, two embodiment fixtures, no ROS/core dependency.
- [ ] **T3 (P1)** — RT/SIM — implement the representative control graph and MuJoCo Plant with virtual time, including a synthetic cadence source exercising `command_staleness_policy` (D-19).
  - Surfaced by: Architecture Review — hardware-independent Plant/ControlCore claim needs executable evidence; the policy-to-RT boundary (D-19) had no executable evidence at all.
  - Files: `robot-rt`, simulation modules, scenarios, `SafetyProfile` staleness-policy field.
  - Verify: lockstep/reset/fault tests, allocation detector, RT Performance Envelope, and the `cadence-source-decimation` scenario.
- [ ] **T4 (P1)** — Runtime/SDK — implement minimal protocol, leases, capabilities, lifecycle, Zenoh transport and Python path.
  - Surfaced by: Test Review — normal and recovery SDK flows cross all critical components.
  - Files: `robot-runtime`, protocol/client modules, Python package.
  - Verify: SDK roundtrip, reconnect, lease loss, slow consumer and runtime restart scenarios.
- [ ] **T5 (P0)** — RT/runtime IPC (Foundation) — compare the minimal SPSC mailbox against a mature community option using the D-05 decision threshold, then land one bounded mailbox.
  - Surfaced by: Code Quality and Performance Review — preserve behavior without committing to unnecessary custom infrastructure; Architecture Review — M1a requires two separate processes, so this belongs in Foundation, not an optional parallel lane.
  - Files: RT/runtime IPC module and benchmark.
  - Verify: overflow, generation, ABI, restart, allocation and latency evidence against the D-20 target-compute baseline.
- [ ] **T6 (P2)** — Replay/Recorder (M1c) — qualify executable stateful replay and recorder interference. May conclude with a deferral decision rather than a working implementation; see M1c.
  - Surfaced by: Test and Performance Review — message playback alone cannot support deterministic replay or RT isolation claims; `runtime-and-platform-reference-projects.md` shows replay completeness is an application contract, not a framework guarantee.
  - Files: execution/replay, MCAP and runtime recording modules.
  - Verify: valid replay plus omitted-state, clock-bypass, corrupt-log, incompatible-build and storage-pressure cases; or a recorded ADR explaining what is deferred and why.
- [ ] **T7 (P2)** — Adapter boundary — build the hardware-free G1/G2-class conformance fixture.
  - Surfaced by: Test Review — hardware independence should be exercised before hardware arrives.
  - Files: adapter contract fixtures and conformance scenarios.
  - Verify: classification, mappings, capabilities, typed errors and evidence limits.
- [ ] **T8 (P2)** — Distribution — build and install the V0 candidate artifact set in clean CI environments.
  - Surfaced by: Architecture Review — binaries and SDKs need an explicit delivery path.
  - Files: CI, packaging and candidate `ReleaseManifest` generation.
  - Verify: reproducible identities, atomic runtime/RT set, clean Python wheel install and scenario execution.

No separate `TODOS.md` items are created by this review. Deferred work already has a milestone, trigger, and context here or a decision entry in the register; duplicating it into a generic backlog would make ownership less clear.

## Definition of success

M1a succeeds when the separate-process Python-to-MuJoCo path runs with command lineage, correct reset/timeline behavior, and a synthetic cadence source exercising `command_staleness_policy`. Milestone 1 succeeds at M1b when one versioned, installable no-hardware candidate also demonstrates recovery, boundedness, recorder isolation, and hardware-free vendor-boundary conformance. M1c succeeds when recorded sessions replay honestly with all four negative cases failing visibly — or, if that proves harder than scoped, when the plan states plainly that V0 replay is message-level only and stateful replay is deferred with a named trigger. It must be clear what was measured, what failed visibly, what remains provisional, and what has not been physically or security qualified.

The longer plan succeeds when those contracts survive later vendor hardware, owner-controlled lower layers, additional simulators, and fleet operations without hiding inaccessible authority or replacing the runnable spine with a platform-specific fork.

## Plan status

M0 documentation has converged: canonical vocabulary is consistent across
architecture, safety, and threat-model documents, and every P0 decision in
the [Decision and Research Register](decision-register.md) is either
accepted, provisional with a reversal trigger, or assigned a bounded
experiment. `D-05`, `D-09`, `D-19`, and `D-20` remain `Experiment required` /
`Research ready` by design — implementation of T3–T5 is expected to close
them, not the other way around. This plan is ready for M1a implementation;
it is not a claim that every open question has already been answered.
