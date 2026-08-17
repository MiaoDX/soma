# Runtime and Platform Reference Projects

> Status: Deep Research. Reviewed against upstream sources on 2026-08-17. Upstream projects are moving targets; verify their current release, license, and platform support before adoption.

Evidence snapshot: Copper `8d51149e`, Copper book `ad4fd410`, S-CORE platform `ef2e5f3e`, and S-CORE reference integration `a64b894d`. Adjacent-project summaries were checked against their default-branch HEAD on the review date.

## Question

Which open-source projects should Soma study or reuse for a Rust-first robot runtime, deterministic replay, communication, lifecycle, and safety-oriented platform engineering?

## Context

Soma should not rebuild infrastructure merely to make its architecture look self-contained. It also should not adopt a framework whose abstractions quietly become the Plant contract, safety authority, public Robot Protocol, or release model.

Two projects provide especially useful but different reference points:

- **Copper** is a Rust robot execution runtime with a generated task schedule and runtime-native record/replay.
- **Eclipse S-CORE** is an automotive platform program with modular services, a continuously integrated baseline, and public safety/process evidence.

They are not substitutes:

```text
Copper                              Eclipse S-CORE
------                              ---------------
robot task graph                    platform module set
generated execution plan           lifecycle / IPC / time / persistency
preallocated cycle data             build and reference integration
runtime-native journal              safety and process work products
task-state checkpoints              multi-platform integration evidence
```

## Executive conclusion

Soma should:

1. evaluate Copper as an **execution-kernel candidate** for a bounded part of `robot-rt`;
2. use S-CORE as a **platform-engineering and assurance reference**, not as a dependency bundle;
3. keep MCAP as the ecosystem-facing robot-data container even if a Copper execution journal is retained;
4. keep Soma ownership of Plant semantics, safety authority, command lineage, artifact identity, physical HAL, and the Robot Protocol;
5. use small focused ecosystem components such as Zenoh, iceoryx2, EtherCrab, `embedded-hal`, and Embassy only at the boundaries they fit.

The immediate implementation experiment is not "port Soma to Copper." It is a time-boxed vertical slice that proves whether Copper can host Soma's cycle contract and replay evidence without taking over product semantics.

## Findings: Copper

### What Copper is

[Copper](https://github.com/copper-project/copper-rs) is an Apache-2.0 Rust runtime for static robot task graphs. A RON graph declares sources, tasks, sinks, connections, resources, missions, logging, and runtime settings. The `#[copper_runtime]` macro consumes that graph and generates an application-specific runtime and schedule.

Its useful architectural properties are:

- graph topology and message types are known at compile time;
- the hot path uses preallocated message storage organized as a `CopperList`;
- foreground `process()` work is separated from less deterministic lifecycle/background work;
- a mockable `RobotClock` keeps task time behind a runtime-owned interface;
- Linux, normal host simulation, and `no_std` MCU examples use the same task model;
- graph, generated schedule, and observed execution can be inspected with `just dag`, `just plan`, and `just plan-log`;
- serial, asynchronous-log, background-task, and pipelined execution modes are explicit rather than hidden behind a general callback executor.

This is much closer to Soma's `robot-rt` problem than a conventional distributed pub/sub framework. Copper is not itself a complete robot platform: it does not replace product manifests, command authority, independent hardware safety, OTA, fleet management, or a public SDK protocol.

### Maturity posture

Copper has a `v1.0.0` tag, extensive working examples, a component catalog, runtime replay tests, and active documentation. The reviewed `master` README requires Rust 1.95 or newer. These are positive adoption signals, not production or safety qualification.

The project is evolving quickly: source already includes features such as MCAP export and distributed replay that older roadmap text may still call planned. Pin an evaluated commit/release and its generated decoder/toolchain. Treat Copper's published benchmark as a useful test design and comparative data point, but rerun the workload on Soma's target CPU, kernel, storage, graph, and logging profile.

### What a `.copper` log contains

The unified log format is an append-oriented, application-native journal. The current runtime defines separate section types for:

```text
StructuredLogLine  indexed structured text events
CopperList         per-cycle task messages and processing metadata
FrozenTasks        serialized task-state keyframes
RuntimeLifecycle   mission/configuration/runtime lifecycle records
```

A `CopperList` carries a cycle ID plus the generated tuple of typed messages. Message metadata includes task processing time, optional time-of-validity, status, and, for distributed Copper-aware paths, origin metadata.

The normal host logger uses preallocated memory-mapped slabs and binary encoding. Every task output is logged by default, although logging can be disabled or reduced per task. This design keeps execution recording inside the runtime instead of adding an external subscriber to each data stream.

The format is intentionally tied to generated application types: a generated log reader knows the exact message tuple needed to decode the bytes. That is an advantage for faithful runtime replay and a disadvantage for long-lived, tool-neutral data access.

### Replay model

Copper's strongest idea is not the file extension. It is the combination of schedule, time, messages, mutable task state, and output comparison:

```text
static graph + generated schedule
              |
recorded CopperLists + task-state keyframes + runtime clock
              |
restore state -> inject recorded boundary inputs -> re-execute tasks
              |
new replay log -> compare messages, metadata, and keyframes
```

The mechanisms are concrete:

- `Freezable::freeze` and `thaw` serialize task state that affects later cycles;
- keyframes are captured every configured number of CopperLists (default: 100);
- `RobotClockMock` advances replay with recorded time rather than wall time;
- simulation/replay callbacks choose which sources or sinks are replaced by recorded behavior;
- replay writes a separate output log rather than mutating the source evidence;
- Copper's determinism tests encode complete CopperLists and keyframes and compare them byte for byte across repeated record runs and resimulation.

The source also contains causal distributed-replay machinery based on recorded subsystem/instance/CopperList origin edges. This is relevant to future multi-compute robots, but it should not expand Soma's first replay milestone.

### `.copper` versus MCAP

These formats solve different problems and can coexist.

| Concern | `.copper` | MCAP |
| --- | --- | --- |
| Primary role | Runtime execution journal | Ecosystem-facing time-series container |
| Type knowledge | Generated application-specific Rust types | Embedded schemas and channel metadata |
| Internal task state | `FrozenTasks` keyframes | Must be modeled as explicit channels/attachments |
| Schedule/process evidence | CopperList ID and task process metadata | Custom schema required |
| Replay style | Restore/re-execute generated task graph | Reader republishes or feeds an application-defined harness |
| Tool ecosystem | Copper readers/debugger | Foxglove and broader robotics tooling |
| Long-term portability | Coupled to Copper format and decoder | Better interchange boundary |

Copper now includes an optional MCAP exporter in `cu29-export`. For Soma, the conservative design is:

```text
if Copper is adopted:
    .copper = execution journal and task-state replay
    MCAP    = incident bundle, SDK data, visualization, interchange
else:
    Soma journal/checkpoints + MCAP = equivalent two-level evidence model
```

Do not make a custom execution journal the only durable record of a field incident.

### Replay guarantees are a system contract

Copper provides useful mechanisms, but "bit-for-bit replay" is not automatic for arbitrary task code. The guarantee depends on all relevant nondeterminism being controlled:

- a stateful task can inherit the default empty `Freezable` implementation and silently omit state;
- system time, thread races, network reads, device state, and unseeded randomness bypass replay unless routed through runtime-owned resources;
- disabling task logging or excluding large handle contents can remove information needed by a later replay;
- new code, configuration, model artifacts, dependencies, compiler flags, or CPU floating-point behavior may intentionally change outputs;
- a static schedule does not by itself provide PREEMPT_RT setup, CPU/IRQ isolation, memory locking, or a worst-case deadline proof;
- Copper's test and safety-case artifacts are evidence about Copper's scoped examples, not certification of a Soma product.

Copper provides a seeded `CuRng` resource and explicit handle-content policies, which are good patterns. Soma still needs an artifact/replay manifest that pins at least:

```text
runtime and application build identity
task graph/configuration signature
ProductModel / robot instance / calibration / control / SafetyProfile
policy/model artifacts
target architecture and relevant numeric profile
recording exclusions and loss counters
clock and Plant timeline semantics
```

### Fit with Soma

The best fit is a bounded execution layer:

```text
Soma-owned lifecycle and activation
  Soma Plant / controller / software-safety task contracts
      Copper-generated in-process execution (candidate)
  Soma-owned HAL and independent safety path

Soma robot-runtime
  authority / actions / protocol / OTA / recorder
      bounded IPC
  robot-rt
```

Copper should not become the public application protocol, cross-process authority model, or release identity. A source/sink adapter should translate between Copper messages and Soma's Plant/RT IPC contracts.

### Copper adoption spike

Build one representative graph, not a toy arithmetic pipeline:

```text
mock/recorded Plant state
 -> estimator
 -> controller
 -> command limiter
 -> software-safety gate
 -> applied-command sink
```

Acceptance criteria:

1. 1 kHz execution with no cyclic allocation and target PREEMPT_RT measurements.
2. Requested, admitted, safety-output, and applied command identities remain visible.
3. Plant timeline/tick and Soma time-domain semantics survive recording and replay.
4. A stateful estimator/controller restores from keyframes correctly.
5. Seeded randomness and one injected external fault replay identically.
6. Record A, record B, and replay A produce the expected exact/tolerance comparisons.
7. Missing state, disabled logging, corrupted log, and incompatible build identity fail visibly.
8. Copper can be removed behind the same Soma task/Plant contract without changing the public protocol.

Adopt only after this spike. Until then, use Copper as executable design evidence.

## Findings: Eclipse S-CORE

### What S-CORE is

[Eclipse S-CORE](https://github.com/eclipse-score) (Safe Open Vehicle Core) is a code-first open-source platform effort for onboard automotive ECUs. It was founded in September 2024 and develops platform modules under a functional-safety-oriented process.

S-CORE's own documentation is explicit about the boundary:

> S-CORE is not a ready-to-integrate series product. It is a generic foundation for commercial distributions.
>
> Responsibility for ASPICE, ISO 21434 (cybersecurity), and ISO 26262 (functional safety) compliance of the final system always remains with the series project.

That is also the correct posture for Soma: open components and process evidence reduce work, but the robot product still owns integration, hazards, assumptions of use, validation, and release approval.

### High-signal repositories

| Repository | What it demonstrates | Soma use |
| --- | --- | --- |
| [`score`](https://github.com/eclipse-score/score) | Platform architecture, requirements, decisions, tool evaluation, safety and management plans | Reference for public engineering evidence and decision boundaries |
| [`reference_integration`](https://github.com/eclipse-score/reference_integration) | Pinned multi-repo baseline, target images, FIT/ITF, QEMU/Docker tests, known integration exceptions | Reference for a continuously tested compatibility set |
| [`process_description`](https://github.com/eclipse-score/process_description) | Requirements, architecture, implementation, verification, safety/security/change processes | Tailor a small Soma process instead of inventing terminology |
| [`lifecycle`](https://github.com/eclipse-score/lifecycle) | Launch manager, run targets, dependency ordering, recovery, external watchdog, heartbeat/deadline/logical supervision; Rust and C++ client APIs | Strong reference for `robot-supervisor` and health contracts |
| [`communication`](https://github.com/eclipse-score/communication) | LoLa zero-copy shared-memory IPC, service discovery, proxy/skeleton APIs, mixed QM/ASIL concerns | Compare with iceoryx2/custom SHM; study criticality isolation, not AUTOSAR surface |
| [`time`](https://github.com/eclipse-score/time) | Type-separated system/steady/high-resolution/PTP vehicle clocks, mock overrides, time daemon/slave | Reference for Soma's typed clock domains and test overrides |
| [`orchestrator`](https://github.com/eclipse-score/orchestrator) | Rust action graph, sequence/concurrency/select/catch, worker mapping, C++ FFI, Linux/QNX | Reference for non-RT action orchestration, not the servo scheduler |
| [`persistency`](https://github.com/eclipse-score/persistency) | Rust key-value storage, snapshots, restore, Bazel/Cargo paths | Reference for durable configuration/update journals after maturity review |
| [`toolchains_rust`](https://github.com/eclipse-score/toolchains_rust) | Pinned Ferrocene toolchains, Linux/QNX targets, coverage and Miri artifacts under Bazel | Concrete evidence for a future safety-oriented Rust toolchain profile |
| [`itf`](https://github.com/eclipse-score/itf) | Target-oriented integration tests across Docker and QEMU | Reference for Soma SIL/HIL runner interfaces |

Repository maturity is uneven. For example, the current `logging` top-level README is still a generic C++/Rust Bazel template, while lifecycle, communication, time, and reference integration expose much more concrete functionality. Treat every repository as an independently reviewed candidate, not as a maturity guarantee inherited from the organization.

### The `known_good` pattern

`reference_integration/known_good.json` pins each participating repository by commit and also records:

- Bazel patches applied by integration;
- extra build/test configuration;
- explicitly excluded test targets;
- language/coverage metadata;
- a timestamp for the integrated set.

The reference workspace then builds target images and runs cross-module tests on Linux and QNX-oriented configurations. This is more honest than a list of nominally compatible versions: patches, exclusions, and known issues are visible integration facts.

Soma is initially better served by a Cargo workspace/monorepo than by copying S-CORE's Bazel multirepo structure. It should still adopt the semantic pattern:

```text
ReleaseManifest (draft -> candidate -> qualified -> released)
  exact source and dependency identities
  toolchain and target profile
  model/configuration artifact identities
  required patches or deviations
  test profile and explicit exclusions
  generated evidence and known limitations
```

This single manifest complements, rather than replaces, `Cargo.lock`, container digests, and signed release artifacts. Soma should not introduce a parallel integration artifact; maturity state distinguishes an under-test candidate from a promoted release.

### Lifecycle and health are separate from task execution

S-CORE separates:

- process launch and termination;
- named run-target activation;
- startup/shutdown dependency ordering;
- failure recovery policy;
- application "running" notification;
- heartbeat, deadline, and logical supervision;
- external watchdog integration.

That is a useful correction to runtime-centric designs. Copper can schedule a robot graph, but Soma still needs a supervisor that can answer:

```text
Which processes should exist in this mode?
Which dependency is ready?
Was the expected control-flow checkpoint reached?
Should a process restart, a run target change, or motion stay inhibited?
Who owns the external watchdog contract?
```

The first Soma implementation can be smaller than S-CORE's Launch Manager, but these semantics should be explicit rather than scattered through systemd units and ad hoc heartbeat topics.

### Typed time as an API boundary

S-CORE's time module exposes distinct compile-time clock domains and mock backends. Its automotive `VehicleTime` is not Soma's API, but the pattern reinforces Soma's existing direction:

- local monotonic time for deadlines;
- high-resolution/raw sources for measurement only where justified;
- synchronized PTP/TAI domain with health status;
- wall time for human/fleet correlation;
- simulation/mock override for tests.

Soma must additionally retain Plant timeline/tick and device capture-time provenance, which are robot-specific gaps not solved by S-CORE.

### Rust safety evidence: useful but scoped

In March 2026 S-CORE accepted a project decision that Rust was ready for ASIL-B components. The decision cites Ferrocene, coverage/tooling, Clippy/rustfmt integration, and planned safety-critical Rust coding guidance.

The same decision states an important limitation: the assessment is specific to **QNX 8 with Ferrocene**; other platforms require their own evaluation. Soma's likely Linux/PREEMPT_RT profile therefore cannot inherit this conclusion.

What Soma can reuse is the shape of the argument:

```text
qualified/pinned compiler and library subset
coding rules and static analysis
coverage tooling and qualification rationale
Miri/sanitizer strategy
target OS/toolchain evidence
module-level language decision
integrator-owned final safety case
```

### What not to copy from S-CORE

Do not copy these before a Soma vertical slice proves the need:

- Adaptive AUTOSAR APIs and automotive service abstractions;
- a Bazel multi-repository program merely for organizational symmetry;
- the entire ASPICE/ISO work-product set without product-risk tailoring;
- C++ base-library layers that Rust or the standard library already provides;
- QNX-specific build and certification assumptions;
- module templates or incubating repositories with no validated Soma use case.

The useful principle is "continuously consistent stack," not "maximum platform surface." Soma should validate its selected modules together from the start, while keeping the selected set deliberately small.

## Adjacent Rust project map

The following shortlist has an identifiable role in Soma. It is not an endorsement list.

| Project | Strongest reference value | Candidate role in Soma | Main caution |
| --- | --- | --- | --- |
| [Copper](https://github.com/copper-project/copper-rs) | Static robot graph, generated schedule, runtime-native replay | `robot-rt` execution-kernel spike | Framework contract and custom log coupling |
| [Dora](https://github.com/dora-rs/dora) | Multi-language distributed dataflows, Arrow, Zenoh SHM, lifecycle/monitoring, `.drec` record/replay | Compare for non-RT perception/AI pipelines and developer tooling | Explicitly soft real-time; replay is not Copper-style task-state checkpointing |
| [iceoryx2](https://github.com/eclipse-iceoryx/iceoryx2) | Rust-core zero-copy lock-free IPC and service patterns | Local large-payload pool or runtime/service IPC benchmark | More general and complex than a fixed SPSC RT mailbox |
| [Zenoh](https://github.com/eclipse-zenoh/zenoh) | Rust-native routed pub/sub plus query/reply and SHM | Robot-local distributed plane, laptop/site/fleet path | Not the cyclic servo transport; define Robot Protocol above it |
| [Rerun](https://github.com/rerun-io/rerun) | Multimodal recording, visualization, and time-aware inspection | Developer observability and incident exploration | Visualization state is not authoritative execution evidence |
| [MCAP Rust](https://github.com/foxglove/mcap) | Standard robotics container implementation | Flight recorder/interchange substrate | Does not define replay semantics or task checkpoints |
| [EtherCrab](https://github.com/ethercrab-rs/ethercrab) | Pure-Rust EtherCAT master | Benchmark candidate for L0 bus integration | Must prove topology/device support and worst-case timing against mature masters |
| [`embedded-hal`](https://github.com/rust-embedded/embedded-hal) | Portable traits for MCU drivers | HAL boundary for selected secondary controllers | Too low-level to define robot Plant semantics |
| [Embassy](https://github.com/embassy-rs/embassy) | Async embedded executor, timers, networking, MCU ecosystem | Non-hard-safety MCU firmware and peripheral tasks | Async convenience is not a hard-RT proof |
| [`r2r`](https://github.com/sequenceplanner/r2r) | Async Rust API over ROS 2 `rcl` and generated C types | ROS 2 gateway option when native ROS installation is acceptable | Build/runtime remains coupled to sourced ROS 2 libraries |
| [`ros2-client`](https://github.com/Atostek/ros2-client) | Pure-Rust ROS 2 client over RustDDS | Research option for a more Rust-native gateway | Smaller compatibility surface than official `rcl`; interoperability needs testing |

### Copper versus Dora

These are the two most relevant Rust runtime-level comparisons:

| | Copper | Dora |
| --- | --- | --- |
| Primary unit | In-process typed task graph | Process/operator dataflow |
| Topology | Static/generated, with explicit missions | Declarative and increasingly dynamic/distributed |
| RT posture | Zero-allocation critical path; generated schedule | Soft real-time tuning and per-node process controls |
| Language model | Rust task API, experimental Python task support | Rust/Python/C/C++ nodes and operators |
| Data model | Generated typed CopperList | Apache Arrow messages |
| Replay | Runtime messages + clock + task-state keyframes | Dataflow message record/replay and node substitution |
| Best Soma question | Can it execute the bounded control graph? | Can it simplify perception/AI pipeline operations? |

Neither should be asked to own independent safety authority or the public Robot Protocol.

## Alternatives

### Adopt Copper as the complete robot platform

This maximizes reuse but overextends Copper into lifecycle, safety, protocol, OTA, and product-model responsibilities it does not claim to own.

### Build all execution and replay machinery in Soma

This preserves exact control but repeats substantial work: graph validation, schedule generation, preallocated cycle storage, state checkpoints, log tooling, and replay navigation.

### Use only ROS 2/DDS and rosbag2/MCAP

This provides the largest robotics ecosystem but does not by itself supply a bounded in-process servo schedule or stateful deterministic re-execution.

### Recommended hybrid

```text
Soma contracts and safety authority
  |
  +-- Copper candidate for bounded in-process control execution
  +-- custom fixed SPSC IPC at the hard RT boundary
  +-- Zenoh for distributed robot/runtime communication
  +-- MCAP for flight data and ecosystem tooling
  +-- ROS 2 gateway at the edge
  +-- S-CORE-inspired lifecycle, candidate release identity, and evidence
```

Each dependency remains replaceable behind a Soma-owned contract.

## Trade-offs

| Choice | Gain | Cost/risk |
| --- | --- | --- |
| Copper spike | Reuses a coherent scheduler/replay design and tests real code early | Integration effort may show its graph/log model is too invasive |
| Dual `.copper` + MCAP evidence | Faithful execution replay plus open tooling | Storage, identity, retention, and correlation complexity |
| S-CORE-inspired baseline | Makes compatibility and exclusions reviewable | Requires disciplined release metadata before fleet scale |
| Small Rust component set | Reduces unsafe/C++ surface and duplicated infrastructure | Several projects are young and need target-specific qualification |
| ROS 2 only at edge | Protects core contracts and version cadence | Gateway ownership and interoperability testing remain Soma work |

## Implications for Soma

### Must do

1. Define a replay manifest independent of any log container.
2. Enumerate every nondeterministic input and mutable state owner in the RT graph.
3. Preserve requested/admitted/safety-output/applied command lineage through recording.
4. Build the Copper adoption spike and compare it with a minimal Soma-native loop.
5. Create a machine-readable integrated release baseline with exact identities, test profile, exclusions, and known limitations.
6. Keep lifecycle/health supervision outside the execution graph contract.
7. Validate every selected dependency as one stack on target Linux and hardware.

### Better to have

- periodic task-state checkpoints and seekable replay;
- generated task-graph/schedule visualization;
- Copper-to-MCAP export or equivalent correlation tooling;
- heartbeat, deadline, and logical supervision APIs;
- deterministic seeded resource APIs for randomness and other external effects;
- QEMU/SIL target images patterned after S-CORE integration tests;
- a future Ferrocene/toolchain evaluation when a safety case requires it.

### Not necessary for the first vertical slice

- distributed causal replay across multiple computers;
- a general dynamic plugin ABI;
- S-CORE's full Bazel multirepo and process stack;
- Adaptive AUTOSAR compatibility;
- one runtime spanning RT control, perception, cloud, and fleet;
- bitwise equivalence across different CPU architectures or simulator backends.

## Primary sources

### Copper

- Copper repository: https://github.com/copper-project/copper-rs
- Copper book: https://copper-project.github.io/copper-rs-book/
- Runtime overview: https://copper-project.github.io/copper-rs/Copper-Runtime-Overview/
- Task lifecycle and keyframes: https://copper-project.github.io/copper-rs/Task-Lifecycle/
- RON configuration: https://copper-project.github.io/copper-rs/Copper-RON-Configuration-Reference/
- Bare-metal development: https://copper-project.github.io/copper-rs/Baremetal-Development/
- Benchmarks: https://copper-project.github.io/copper-rs/Benchmarks/
- Logging and replay: https://copper-project.github.io/copper-rs-book/logging-replay.html
- Export formats and MCAP: https://copper-project.github.io/copper-rs-book/export-formats.html
- Determinism test (reviewed commit): https://github.com/copper-project/copper-rs/blob/8d51149e5e162e252b4c8627341236f187b2562a/examples/cu_caterpillar/src/determinism.rs
- Runtime replay non-regression test (reviewed commit): https://github.com/copper-project/copper-rs/blob/8d51149e5e162e252b4c8627341236f187b2562a/core/cu29_runtime/tests/replay_anytime.rs

### Eclipse S-CORE

- S-CORE organization: https://github.com/eclipse-score
- Platform documentation: https://eclipse-score.github.io/score/main/
- Platform introduction and integrator boundary: https://github.com/eclipse-score/score/blob/main/docs/index.rst
- Consistent-stack decision: https://github.com/eclipse-score/score/blob/main/docs/design_decisions/DR-001-strat.md
- Rust ASIL-B readiness decision and scope: https://github.com/eclipse-score/score/blob/main/docs/design_decisions/DR-001-arch.md
- Reference integration: https://github.com/eclipse-score/reference_integration
- Integrated `known_good.json`: https://github.com/eclipse-score/reference_integration/blob/main/known_good.json
- Lifecycle and health: https://github.com/eclipse-score/lifecycle
- Communication/LoLa: https://github.com/eclipse-score/communication
- Time: https://github.com/eclipse-score/time
- Process description: https://github.com/eclipse-score/process_description
- Rust toolchains: https://github.com/eclipse-score/toolchains_rust

### Adjacent Rust ecosystem

- Dora: https://github.com/dora-rs/dora
- iceoryx2: https://github.com/eclipse-iceoryx/iceoryx2
- Zenoh: https://github.com/eclipse-zenoh/zenoh
- Rerun: https://github.com/rerun-io/rerun
- MCAP: https://github.com/foxglove/mcap
- EtherCrab: https://github.com/ethercrab-rs/ethercrab
- embedded-hal: https://github.com/rust-embedded/embedded-hal
- Embassy: https://github.com/embassy-rs/embassy
- r2r: https://github.com/sequenceplanner/r2r
- ros2-client: https://github.com/Atostek/ros2-client

## Open questions

1. Can Copper's generated task API express the Soma Plant cycle without weakening the four-stage command lineage?
2. What is the smallest replay manifest that can reject an incompatible build/artifact set before re-execution?
3. Which boundary inputs should be logged in `.copper`, MCAP, or both without creating two authorities?
4. Does Copper's memory-mapped logger preserve the required worst-case RT behavior under sustained dirty-page pressure on target hardware?
5. Can large handle-backed payload policies be made replay-complete without unacceptable storage bandwidth?
6. Is Dora useful enough above `robot-runtime` to justify a second execution framework, or should Soma use plain processes plus Zenoh?
7. Which S-CORE health/lifecycle semantics belong in Soma V0 versus later fleet-hardening phases?
8. At what organizational scale should the single `ReleaseManifest` evolve from a Cargo-monorepo source list to a multi-repo `known_good` source set?
