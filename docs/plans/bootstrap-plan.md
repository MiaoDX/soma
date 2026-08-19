# Soma Bootstrap Plan

> Status: Reviewed implementation plan. The long-term architecture is complete at a system-boundary level; only Milestones 0 and 1 are current implementation scope. Later milestones reopen when their hardware or product trigger exists.

## Outcome

Soma has two sequential goals:

1. establish a technically defensible, community-backed system direction that can be reviewed with colleagues;
2. implement a narrow runnable spine, then expand downward and outward without replacing its public contracts.

The current lack of hardware is a constraint, not a reason to couple the architecture to one simulator. V0 proceeds top-down from MuJoCo/SIL and a vendor-neutral exposed-interface contract. Physical and bottom-up qualification remain explicit later milestones and cannot be claimed from mocks or vendor documentation.

## Runnable spine

```text
Python SDK                    synthetic 20 Hz cadence source
    |                         (lookup table or sine, no trained
    | Robot Protocol           model required — see D-19)
    | over provisional Zenoh          |
    v                                 v
robot-runtime  <-------------------- command stream
  lease | timing | capability | lifecycle | bounded evidence
    |
    | bounded RT/runtime mailbox contract
    v
robot-rt
  representative ControlCore -> SA-3 -> applied evidence
    |                            ^
    |                            | command TTL + control_mode -> SafeBehavior
    | bounded Plant contract
    v
MuJoCo Plant
    |
    +--> state / events / requested-to-applied lineage
    +--> bounded evidence ring + post-run MCAP export

parallel contract fixture, not a second runtime:
vendor-neutral InterfaceProfile -> adapter conformance
```

This is the smallest slice that exercises the architecture rather than merely demonstrating a simulator. The synthetic cadence source exists specifically to force the timeout, interpolation, and staleness code paths defined in [`policy-runtime-interface.md`](../deep-research/policy-runtime-interface.md) (D-19) to exist before a trained policy or real hardware arrives — it is a control-and-timing fixture, not a step toward a real policy.

## Implementation maturity map

| Direction | Design now | Implement now | Later evidence before a production claim |
| --- | --- | --- | --- |
| Layers and trust | L-2..L5, SA-0..SA-5, Plant and runtime boundaries | Preserve boundaries in module/API layout | Hazard analysis and physical authority validation |
| Time | Clock domains, timeline/generation split, command validity invariants | Time ADR, simulation clock, reset/restart rejection | Target clock/PTP/bus measurements |
| Model | Minimal `ProductModelManifest`, separate instance/calibration/control/safety artifacts | Native MJCF fixture plus validator; second embodiment fixture | Physical inventory/calibration and additional backend mappings |
| RT execution | Restricted cyclic profile and replaceable execution kernel | Minimal native representative graph | Copper adoption decision and target-compute worst-case evidence |
| RT/runtime IPC | Bounded mailbox behavior, ABI/generation/restart rules | Provisional minimal SPSC plus development-machine characterization | Target cache/NUMA/process-failure qualification and dependency reevaluation |
| Distributed protocol | Transport-neutral Robot Protocol, Zenoh provisional default | Minimal state/command/event/lease/capability surface | Triggered Cyclone comparison only if envelope or interoperability requires it |
| Simulation | Same ControlCore/SA-3/Plant semantics, separate Simulation Control | MuJoCo lockstep/reset/fault scenarios | Additional simulators only for a concrete product need |
| Evidence | Requested/admitted/safety-output/applied lineage and replay contract | Bounded in-memory evidence ring plus post-run MCAP export | Live recording, recorder pressure, stateful replay, incident retention and fleet correlation |
| Security | Threats, trust boundaries, artifact and developer-mode invariants | Loopback-only `insecure-local-dev`; no cryptographic implementation | Authentication, signing, trust/update mechanisms and security qualification before any non-local or physical use |
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
| `HOST` | host unit, schema, composition, and packaging checks | validators, canonicalization, API/schema compatibility | deployed scheduling, simulator dynamics, bus/device behavior, physical safety |
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

## Milestone 1: no-robot runnable spine

Milestone 1 has three independently reviewable checkpoints. Foundation freezes
only the contracts needed by M1a; target qualification belongs to M1b; live
recording, framework adoption and stateful replay cannot block either one.

### Foundation contracts

- minimal `ProductModelManifest`, backend-native MuJoCo MJCF, a second embodiment fixture, and a validator;
- fixed-layout RT control state/command and mailbox lifecycle, ABI, generation, overflow and restart semantics;
- Protobuf as the provisional V0 source of truth for the public network schema, with golden compatibility fixtures and major-version rejection; RT messages remain fixed-layout, while bulk data crosses the public protocol by descriptor or handle;
- one exclusive `whole_body` lease with acquire, renew, release, expiry, conflict and stale-generation behavior; resource-graph leases are deferred;
- `ImmediateTiming` and `TickTargetTiming`; `ScheduledTiming` is represented but returns typed unsupported in M1;
- `control_mode -> SafeBehavior`, interpolation owner, command TTL/staleness policy, and requested/admitted/safety-output/applied lineage; policy chunking and observation alignment are deferred;
- a supervisor contract for startup order, health, restart and lifecycle transitions, implemented by an M1a test harness and by systemd in M1b; no custom supervisor daemon;
- vendor-neutral `InterfaceProfile` and shared `CapabilityCatalog` fixtures;
- minimal `ReleaseManifest` that pins build, dependencies, model, scenario, configuration, test profile, exclusions and evidence identity.

### M1a: runs

Deliverables:

- separate `robot-runtime` and `robot-rt` processes connected by a provisional minimal SPSC mailbox;
- a minimal native Plant -> estimator/controller -> limiter -> SA-3 -> applied sink graph and a MuJoCo Plant with virtual time, lockstep, pause/step/reset, fault injection and timeline invalidation;
- minimal Protobuf Robot Protocol over provisional Zenoh, a Rust client and thin Python binding;
- a bounded in-memory evidence ring with requested-to-applied lineage and post-run MCAP export;
- Linux x86_64 developer/SIL packaging under the loopback-only `insecure-local-dev` profile.

Required scenarios and exit gate:

- the Python SDK acquires the exclusive lease, commands MuJoCo and receives state/evidence end to end;
- reset rejects a stale timeline; the synthetic cadence source exercises the frozen interpolation, TTL and `SafeBehavior` path;
- lease acquire/renew/release/expiry/conflict/stale-generation and both supported timing modes pass; scheduled timing fails with a typed unsupported result;
- schema goldens, major-version rejection and model-validator positive/negative fixtures pass;
- supervisor test harness demonstrates ordered startup, process-death detection, motion inhibition and explicit restart/reacquisition;
- development-machine IPC characterization reports latency distributions, allocations, overflow and crash behavior without claiming target qualification.

### M1b: credible

Deliverables:

- systemd units implementing the Foundation supervisor contract;
- runtime restart/reconnect, slow-consumer and bounded IPC recovery evidence;
- vendor-neutral adapter-contract conformance using `InterfaceProfile`;
- one clean-install Linux x86_64 candidate;
- representative-board qualification: `cyclictest` plus the 1 kHz end-to-end cycle over a 24-hour soak, recording maximum latency, deadline misses, allocations and process-failure behavior.

Required scenarios and exit gate:

- runtime restart preserves the Plant timeline, invalidates the old lease and requires explicit reacquisition;
- overflow, stale generation and incompatible mailbox ABI cannot enable motion;
- the target board completes the 24-hour qualification and establishes the final IPC budget from the measured end-to-end 1 ms cycle; no fixed microsecond threshold is assumed;
- the generic adapter fixture proves mappings, capabilities, typed unsupported behavior and honest evidence limits;
- the candidate installs and runs the M1a scenario in a clean environment.

### M1c: reproducible

Deliverables and exit gate:

- live recording with bounded degradation and explicit disk-pressure/loss behavior;
- the D-09 Copper-vs-native adoption decision, measured against the working M1a graph;
- executable stateful replay with replay-manifest completeness and the omitted-state, clock-bypass, corrupt-log and incompatible-build negative cases.

M1c may conclude that V0 supports only message-level replay and defer stateful
replay with a named trigger. That result does not reopen M1a or M1b.

### Security posture

M1 implements no authentication, TLS, artifact signing, trust store, secure boot,
anti-rollback, key rotation or revocation. The `insecure-local-dev` profile binds
only to loopback; `source_id` is evidence attribution, not authenticated
identity. Security design reopens before any non-local control, external
distribution, physical actuation or OTA path is enabled.

### Distribution matrix

| Artifact | V0 target | Delivery path | Required proof |
| --- | --- | --- | --- |
| `robot-rt` | Linux x86_64 developer/SIL profile | pinned Cargo workspace build | reproducible build identity, restricted profile, tests and benchmark report |
| `robot-runtime` | Linux x86_64 developer/SIL profile | same candidate release set as `robot-rt` | atomic IPC ABI compatibility and startup/restart scenario |
| Python SDK | one supported CPython range on Linux x86_64 | local wheel produced by the workspace build | clean-environment install and integrated SDK scenario |
| MuJoCo model/scenarios | pinned MuJoCo version | content-addressed release assets | validator and scenario hashes, deterministic/tolerance report |
| Evidence tooling | Linux developer profile | workspace binaries/modules | bounded-ring behavior and post-run MCAP readability |
| Future aarch64/ROS/gateway artifacts | not V0 | added by a target-specific distribution profile | must not change core protocol semantics |

### Checkpoint demonstrations

M1a:

```text
start candidate release
 -> Python SDK connects and discovers capabilities
 -> acquires lease and commands MuJoCo
 -> receives state, health, and command-decision evidence
 -> reset creates a new Plant timeline and rejects stale command
 -> runtime restart invalidates old lease but not the Plant timeline
 -> SDK reconnects and explicitly reacquires authority
 -> bounded evidence exports to MCAP after the run
```

M1b:

```text
install candidate on the representative board
 -> systemd starts both processes in contract order
 -> run the M1a scenario and restart/reacquisition path
 -> complete the 24-hour 1 kHz qualification
 -> publish the final target-derived IPC budget and adapter conformance
```

M1c:

```text
run the representative graph with live recording pressure
 -> make losses and completeness explicit
 -> compare native execution with Copper and record the adoption decision
 -> restore/re-execute a valid session; reject all declared negative cases
```

## Milestone verification matrix

This is an index, not a test DSL. Scenario implementation details live beside the code when implementation begins.

| Claim | Scenario | Evidence | Pass condition | Retained artifact | Milestone |
| --- | --- | --- | --- | --- | --- |
| Shared model semantics span embodiments | `model-validator-fixtures` | `HOST` | two embodiments validate; duplicate/missing joint, unknown/cyclic frame, invalid unit, non-finite value, actuator mismatch and native-asset identity mismatch are rejected | manifest fixtures + validator report | M1a |
| Public schema changes are explicit | `protobuf-compatibility` | `HOST` | golden V0 payloads remain compatible | golden fixtures + compatibility report | M1a |
| Incompatible public schemas fail visibly | `protocol-major-version-rejection` | `HOST`/`SIL` | SDK/runtime integration rejects an unsupported major version before commands are admitted | integration trace | M1a |
| Same control path drives simulation | `sdk-mujoco-roundtrip` | `SIL` | SDK -> runtime -> RT -> MuJoCo -> state/evidence completes | MCAP + release/scenario IDs | M1a |
| Timeline prevents stale control | `reset-with-inflight-command` | `SIL` | reset changes timeline; prior command is rejected with reason | event and command-lineage record | M1a |
| A low-rate command source has a defined 1 kHz behavior | `cadence-source-decimation` | `SIL` | synthetic 20 Hz source uses the frozen interpolation owner and TTL; expiry selects `control_mode -> SafeBehavior` and records lineage | command-lineage record + D-19 contract identity | M1a |
| M1 lease scope is complete and exclusive | `whole-body-lease-lifecycle` | `HOST`/`SIL` | acquire, renew, release, expiry and conflict work; stale generation is rejected | lease trace | M1a |
| M1 timing scope is explicit | `timing-mode-admission` | `HOST`/`SIL` | immediate and tick-target commands execute; scheduled timing returns typed unsupported | admission trace | M1a |
| Supervision preserves recovery semantics | `supervisor-process-death` | `SIL` | the harness detects either process death, inhibits motion and requires explicit restart/reacquisition | lifecycle trace | M1a |
| RT/runtime IPC is bounded and restart-safe | `ipc-overflow-generation-abi` | `HOST`/`SIL` | no cyclic allocation/blocking; stale or incompatible region cannot enable motion; development-machine distributions are reported as provisional | test and characterization report | M1a |
| Runtime generations and leases recover explicitly | `runtime-restart-reconnect` | `SIL` | Plant timeline persists; old lease fails; SDK reacquires | lifecycle/lease trace | M1b |
| Slow consumers cannot block RT | `subscriber-overload` | `SIL` | bounded drop/age counters increase; RT envelope holds | benchmark + counters | M1b |
| Production supervision matches the contract | `systemd-supervisor-conformance` | `HOST`/`SIL` | systemd startup, process death, restart and explicit reacquisition match the M1a harness semantics | unit files + lifecycle trace | M1b |
| Target loop behavior is qualified | `target-board-24h` | `HIL` | representative board completes 24 h at 1 kHz with max latency, misses, allocations and crash behavior reported | qualification report + final IPC budget | M1b |
| Replay is executable and honest | `stateful-replay-negative-cases` | `HOST`/`SIL` | valid replay compares as declared; omission/corruption/incompatibility fails visibly | source and replay journals + manifest | M1c |
| Live recording cannot perturb control silently | `recorder-pressure` | `SIL` | RT envelope holds or failure is explicit; losses invalidate completeness claim | enabled/disabled/pressure report | M1c |
| Vendor boundary stays contained | `interface-profile-conformance` | `HOST`/`SIM` | generic classification, mappings, capabilities, typed unsupported errors and evidence limits pass | conformance report | M1b |
| Ordinary artifacts cannot relax safety profile | `artifact-authority-separation` | `HOST`/`SIL` | model/control/policy/scenario changes leave active `SafetyProfile` identity and bounds unchanged | activation/evidence trace | M1b |
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
  +-- whole_body lease / timing / capability admission
  +-- Protobuf golden compatibility / major rejection
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
[PLANNED M1a] bounded evidence / post-run MCAP export

[PLANNED M1c] live recording and executable replay
  +-- omitted state / clock bypass
  +-- corrupt log / incompatible build

[DEFERRED] protocol security abuse, fuzz, sustained DoS and penetration tests
```

Implementation modules that should retain small inline ASCII diagrams once code exists:

- lifecycle/generation types: reset and restart state transitions;
- RT/runtime IPC: header validation, generation handshake and overwrite/drop behavior;
- runtime admission: requested -> admitted -> rejected decision flow;
- M1c replay service: record -> checkpoint -> restore -> re-execute -> compare;
- adapter boundary: vendor input/output mapping and last observable evidence stage.

## Performance Envelope

Do not put universal numbers in the architecture without a workload and platform. Every benchmark records:

```text
profile:
  CPU / kernel / scheduler / simulator / build
  graph and message payloads
  client/subscriber count
  evidence mode; recorder mode and storage only when recording is in scope
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

The representative graph starts at 1 kHz, but the rate is a test workload, not a permanent requirement for every embodiment. M1a development-machine results characterize the implementation; M1b derives the final mailbox budget from the representative board's end-to-end 1 ms cycle measurements. Copper and live-recorder deltas are measured later in M1c.

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
| Model activation | duplicate/missing joint, unknown/cyclic frame, invalid unit, non-finite value, actuator mismatch or native-asset identity mismatch | `model-validator-fixtures` | fail before actuation with a typed reason |
| Replay | omitted state, nondeterminism, corruption, incompatible build | `stateful-replay-negative-cases` | mismatch/rejection; never silent success |
| Recorder | queue saturation, dirty-page pressure, full disk | `recorder-pressure` | bounded degradation and loss counters; no false completeness |
| Vendor adapter | unsupported feature or hidden vendor authority | `interface-profile-conformance` | typed unsupported result and bounded evidence claim |
| Protocol attack | malformed/replayed/excessive client traffic | deferred to M4 by decision | M1 makes no production-security claim; Threat Model preserves required invariant |

No M1 failure mode is both silently handled and claimed as verified without a planned test. Deferred security cases block a production-security claim, not the no-robot architecture demonstration.

## What already exists

The repository currently contains research and architecture documents only: no workspace, executable code, test framework, CI, package, or release pipeline exists yet. Soma should reuse community mechanisms and retain only product-specific contracts:

| Need | Existing reference or implementation | Soma-owned remainder |
| --- | --- | --- |
| Physics/SIL | MuJoCo, PX4/ArduPilot SITL patterns | Plant, time/reset, command lineage and conformance semantics |
| RT graph and replay | Copper | M1c adoption spike, Plant/authority integration and honest replay manifest |
| Distributed transport | Zenoh; Cyclone DDS gateway ecosystem | Robot Protocol semantics, lease/timing/capability policy |
| Local bulk IPC | iceoryx2 candidate | measured choice and payload lifetime policy |
| Evidence container | MCAP and ecosystem tools | execution state/checkpoints, identities and completeness rules |
| Lifecycle/integration evidence | Eclipse S-CORE patterns | small Soma lifecycle contract and maturity-bearing `ReleaseManifest` |
| Host/device updates | RAUC/OSTree/Mender, MCUboot, TUF/Uptane patterns | product compatibility, authority, health and recovery policy |
| ROS ecosystem | ROS 2 bridges and standard messages | mapping at the edge without core dependency |
| Vendor surfaces | Unitree/AgiBot and other public SDK/sim assets | honest boundary classification and capability conformance |

Soma does not need to own a general middleware, universal model language, automotive platform API, logging container, update framework, or simulator engine.

## NOT in scope for Milestone 1

- physical robot, actuator, fieldbus, E-stop, STO, brake, power, thermal, secure-boot, or actuation hard-RT qualification: M1b's representative compute board does not provide physical-system evidence;
- a universal URDF/MJCF/USD compiler or lossless conversion layer: V0 validates native assets;
- complete vendor emulators or vendor-internal behavior: `InterfaceProfile` fixtures validate only Soma's boundary;
- a mandatory Zenoh/Cyclone bake-off: comparison is trigger-based;
- independent update/health/observer/recorder daemons: these start as `robot-runtime` modules;
- full protocol security abuse, penetration, fuzz, DoS, production key, or physical debug testing: deferred to M4;
- Isaac, Genesis, device/actuator HIL rigs, batch-RL infrastructure, ROS distribution matrix, fleet rollout, cloud service, long-term retention, or UI tooling;
- universal plugin ABI, cross-version internal RT IPC compatibility, or multi-repository integration platform;
- safety certification or claims about inaccessible vendor safety and lower-stack implementation.

## Parallel implementation lanes

Contracts are a short sequential gate; implementation then separates into bounded lanes.

| Step | Modules | Depends on |
| --- | --- | --- |
| Foundation | shared types, Protobuf schema, model fixtures, lease/timing/supervisor contracts, scenario IDs, release identity, RT/runtime IPC (D-05) | M0 decisions |
| Lane A: control and simulation | `robot-rt`, Plant, MuJoCo, minimal native execution graph, RT benchmark | Foundation |
| Lane B: runtime and SDK | `robot-runtime`, protocol, Rust/Python client | Foundation |
| Lane C (M1c): execution evidence | Copper-vs-native comparison (D-09), stateful replay and live recorder | M1a's minimal graph |
| Lane D: adapter conformance | adapter fixtures, `InterfaceProfile`, capability cases | Foundation |
| M1a integration | end-to-end command, reset, contract tests and post-run MCAP export | A + B |
| M1b qualification | recovery, adapter conformance, target-board envelope and packaging | M1a + D |

Execution order:

```text
Foundation (shared contracts, RT/runtime IPC)
   |
   +--> Lane A: minimal native graph --+
   +--> Lane B ------------------------+--> M1a --> M1b qualification
   +--> Lane D -----------------------------------+
                                         |
                                         +--> M1c: recorder / Copper / replay
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

- [x] **T0 (P0)** — Repo hygiene — add LICENSE, `.gitignore`, and architecture diagrams ahead of the first workspace commit.
  - Surfaced by: Architecture Review — a public repository about to receive a Rust workspace had neither a license nor a `.gitignore`.
  - Files: `LICENSE`, `.gitignore`, `docs/architecture/diagrams/`.
  - Verify: LICENSE present; `cargo build` artifacts do not appear in `git status`; documentation links resolve.
- [ ] **T0b (P0)** — Development-machine characterization (D-20) — measure the provisional SPSC and 1 kHz loop without blocking M1a on target selection.
  - Surfaced by: Architecture Review — implementation feedback is needed early, while physical qualification belongs after a representative board is selected.
  - Files: benchmark/allocation/crash harness and local results.
  - Verify: latency distributions, allocations, overflow, ABI/generation rejection and `SIGKILL` behavior are reported as development evidence only.
- [ ] **T1 (P1)** — Decisions — record P0 ADRs that have evidence and write bounded protocols for experiment-dependent decisions.
  - Surfaced by: Architecture Review — research and decisions lacked a consistent convergence path.
  - Files: `docs/decisions/`, `docs/plans/decision-register.md`, relevant Deep Research.
  - Verify: every P0 row is accepted, provisional with trigger, or has a measurable experiment; spike results later close their own ADRs.
- [ ] **T2 (P1)** — Foundation — create the minimal workspace, Protobuf public schema, shared contracts, model fixtures, validator, supervisor harness and candidate release identity.
  - Surfaced by: Scope and Code Quality Review — runnable spine must precede the broad long-term module surface.
  - Files: initial workspace modules and CI/build configuration.
  - Verify: clean build; schema goldens and major-version rejection; lease/timing/supervisor cases; model positive fixtures and all named negative fixtures; no ROS/core dependency.
- [ ] **T3 (P1)** — RT/SIM — implement the representative control graph and MuJoCo Plant with virtual time, including a synthetic cadence source exercising command TTL and `control_mode -> SafeBehavior` (D-19).
  - Surfaced by: Architecture Review — hardware-independent Plant/ControlCore claim needs executable evidence; the policy-to-RT boundary (D-19) had no executable evidence at all.
  - Files: `robot-rt`, simulation modules, scenarios, and the frozen D-19 `SafetyProfile` fields.
  - Verify: lockstep/reset/fault tests, allocation detector, RT Performance Envelope, and the `cadence-source-decimation` scenario.
- [ ] **T4 (P1)** — Runtime/SDK — implement minimal protocol, leases, capabilities, lifecycle, Zenoh transport and Python path.
  - Surfaced by: Test Review — normal and recovery SDK flows cross all critical components.
  - Files: `robot-runtime`, protocol/client modules, Python package.
  - Verify: SDK roundtrip, reconnect, lease loss, slow consumer and runtime restart scenarios.
- [ ] **T5 (P0)** — RT/runtime IPC (Foundation) — land a provisional minimal SPSC mailbox and preserve the bounded mailbox contract.
  - Surfaced by: Code Quality and Performance Review — M1a requires two separate processes, so this belongs in Foundation; target evidence later decides whether the implementation remains appropriate.
  - Files: RT/runtime IPC module and benchmark.
  - Verify: overflow, generation, ABI, restart, allocation and development-machine latency characterization; M1b sets the final target budget.
- [ ] **T5b (P1)** — Target qualification (M1b, D-20) — run `cyclictest` and the end-to-end 1 kHz cycle on a selected representative board for 24 hours.
  - Files: qualification harness and `docs/measurements/` report.
  - Verify: maximum latency, misses, allocations and process-failure behavior are reported; the final D-05 budget is derived from the measured 1 ms cycle.
- [ ] **T6 (P2)** — Execution evidence (M1c) — qualify live recording, decide Copper adoption and attempt executable stateful replay. May conclude with a deferral decision rather than a working replay implementation.
  - Surfaced by: Test and Performance Review — message playback alone cannot support deterministic replay or RT isolation claims; `runtime-and-platform-reference-projects.md` shows replay completeness is an application contract, not a framework guarantee.
  - Files: execution/replay, MCAP and runtime recording modules.
  - Verify: valid replay plus omitted-state, clock-bypass, corrupt-log, incompatible-build and storage-pressure cases; or a recorded ADR explaining what is deferred and why.
- [ ] **T7 (P2)** — Adapter boundary — build the vendor-neutral `InterfaceProfile` conformance fixture.
  - Surfaced by: Test Review — hardware independence should be exercised before hardware arrives.
  - Files: adapter contract fixtures and conformance scenarios.
  - Verify: classification, mappings, capabilities, typed errors and evidence limits.
- [ ] **T8 (P2)** — Distribution — build and install the V0 candidate artifact set in clean CI environments.
  - Surfaced by: Architecture Review — binaries and SDKs need an explicit delivery path.
  - Files: CI, packaging and candidate `ReleaseManifest` generation.
  - Verify: reproducible identities, atomic runtime/RT set, clean Python wheel install and scenario execution.

No separate `TODOS.md` items are created by this review. Deferred work already has a milestone, trigger, and context here or a decision entry in the register; duplicating it into a generic backlog would make ownership less clear.

## Definition of success

M1a succeeds when the separate-process Python-to-MuJoCo path runs with the frozen schema, lease, timing, supervisor, model and policy-boundary contracts. M1b succeeds when one installable Linux x86_64 candidate demonstrates recovery, generic adapter conformance and representative-board 24-hour qualification. M1c succeeds when live recording is bounded, Copper adoption is decided and recorded sessions replay honestly with all four negative cases failing visibly — or when stateful replay is explicitly deferred and V0 is limited to message-level replay. Every checkpoint states what was measured, what failed visibly, what remains provisional and what has not been physically or security qualified.

The longer plan succeeds when those contracts survive later vendor hardware, owner-controlled lower layers, additional simulators, and fleet operations without hiding inaccessible authority or replacing the runnable spine with a platform-specific fork.

## Plan status

M0 documentation has converged: canonical vocabulary is consistent across
architecture, safety, and threat-model documents, and every P0 decision in
the [Decision and Research Register](decision-register.md) is either
accepted, provisional with a reversal trigger, or assigned a bounded
experiment. `D-05`, `D-09`, and `D-20` remain experiment-backed by design —
implementation of T3–T6 is expected to close
them, not the other way around. This plan is ready for M1a implementation;
it is not a claim that every open question has already been answered.
