# Decision and Research Register

> Status: Living planning index. Keep entries short; evidence belongs in Deep Research, decisions in ADRs, and implementation detail in milestone plans.

## Decision path

```text
Decision Register
      |
      v
Deep Research / community evidence
      |
      v
bounded experiment or spike
      |
      v
ADR: accept, reject, or defer
      |
      v
implementation + verification evidence
```

Not every decision needs every step. A reversible implementation default may begin as `Provisional`; a safety, time, public-contract, or trust-boundary decision requires explicit evidence and an ADR before it is treated as stable.

## Status and priority

- **Accepted** — current architecture; change through an ADR.
- **Provisional** — usable default with a named reversal trigger.
- **Research ready** — evidence exists; ADR or experiment is next.
- **Experiment required** — a bounded spike must answer the question.
- **Deferred** — intentionally outside the current implementation milestone.

Priority describes decision order, not eventual product importance:

- **P0** — blocks the no-robot runnable spine.
- **P1** — needed before a real platform or release claim.
- **P2** — later hardware, fleet, or qualification work.

## Register

| ID | Priority | Question | Current posture | Next evidence / stop condition |
| --- | --- | --- | --- | --- |
| D-01 | P0 | Product layers and safety-authority vocabulary | Accepted: L-2..L5 and SA-0..SA-5 remain separate | Maintain conformance across architecture docs and first code modules |
| D-02 | P0 | Hardware-independent control boundary | Accepted: one `ControlCore` and bounded Plant contract for MuJoCo/SIL and later hardware | V0 MuJoCo vertical slice; no embodiment-specific public branch |
| D-03 | P0 | Current implementation direction without robot hardware | Accepted: top-down from MuJoCo plus a vendor-neutral `InterfaceProfile` contract fixture | Runnable spine plus generic adapter conformance; real vendor mapping begins at M2 |
| D-04 | P0 | Runtime process and supervisor boundary | Accepted: supervised `robot-rt` + `robot-runtime`; Foundation defines startup, health, restart and lifecycle semantics | M1a uses a test harness; M1b uses systemd; no custom supervisor daemon |
| D-05 | P0 | RT/runtime IPC implementation | Provisional: minimal SPSC preserves the bounded mailbox behavior for M1a | Characterize on the development machine; derive the final budget from D-20's representative-board end-to-end 1 ms cycle, then compare a mature option only if the measured envelope or maintenance cost requires it |
| D-06 | P0 | Distributed transport | Provisional: Zenoh-first; Cyclone DDS/gRPC remain gateways | Compare only if Zenoh misses its envelope, creates an operational problem, or native DDS becomes required |
| D-07 | P0 | Time and lifecycle semantics | Accepted for M1: timeline/generation split plus `ImmediateTiming` and `TickTargetTiming`; synchronized scheduling is deferred | Time ADR plus reset/restart/stale-command cases; `ScheduledTiming` returns typed unsupported until a synchronized-clock requirement exists |
| D-08 | P0 | V0 robot model strategy | Accepted: minimal `ProductModelManifest` + backend-native assets + validator | Validate native MJCF and one second fixture; no universal compiler |
| D-09 | P0 | RT execution/replay framework | Experiment required: Copper-hosted vs minimal Soma-native representative graph, compared *after* the minimal native graph exists (Lane A), not in parallel with it | V0 decision threshold: if the minimal native graph already satisfies every M1a pass condition, treat Copper as design reference only and defer adoption to M1c; adopt only if it clears every criterion in the Copper adoption spike (`runtime-and-platform-reference-projects.md`) without changing the public Robot Protocol |
| D-10 | P0 | System verification traceability | Accepted: lightweight claim-to-evidence matrix | Each milestone exit claim has a scenario, pass condition, evidence level, and retained artifact |
| D-11 | P0 | Performance requirements without target hardware | Accepted: define Performance Envelopes, not universal numbers | Record workload/platform identity and distributions for every spike |
| D-12 | P1 | Commercial platform boundary | Accepted: classify as `NativePlantAdapter` or `ManagedMotionGateway` using a vendor-neutral `InterfaceProfile` fixture | Generic contract tests now; real vendor mappings and physical qualification only when hardware exists |
| D-13 | P1 | Release integration identity | Accepted: one maturity-bearing `ReleaseManifest`; no parallel integration artifact | Candidate manifest pins source, toolchain, artifacts, tests, exclusions, and evidence |
| D-14 | P1 | Security scope | Accepted: M1 is loopback-only `insecure-local-dev` and implements no cryptographic mechanism | Reopen authentication, TLS, signing, trust/update mechanisms, rotation and revocation before non-local control, external distribution, physical actuation or OTA |
| D-15 | P1 | Python packaging and target matrix | Accepted for V0: Linux x86_64 only | Build/install the x86_64 candidate; add aarch64 only after target selection |
| D-16 | P2 | First physical platform | Deferred until accessible hardware and project need are known | Evaluate exposed boundary, licensing, safety authority, model assets, and recovery before selection |
| D-17 | P2 | Owner-controlled L0/L-1/L-2 path | Deferred, retained as a long-term completeness requirement | Resume with own hardware or an accessible component bench; qualify only observed layers |
| D-18 | P2 | Fleet OTA, production trust, and long-term operations | Deferred after runnable spine and target platform | Product-specific hazard, security, rollout, recovery, and retention evidence |
| D-19 | P0 | Policy/inference to RT interface | Accepted for M1a: freeze interpolation owner, command TTL, `control_mode -> SafeBehavior`, and command lineage; see [`policy-runtime-interface.md`](../deep-research/policy-runtime-interface.md) | Exercise with a synthetic cadence source; defer action chunking and observation alignment until a real policy workload exists |
| D-20 | P0 | Compute latency evidence | Experiment required in two stages: M1a development-machine characterization is provisional; M1b representative-board evidence is qualification | In M1b run `cyclictest` and the end-to-end 1 kHz cycle for 24 h, report max latency/misses/allocations and either-side `SIGKILL`, then derive D-05's final budget from the measured 1 ms cycle |
| D-21 | P0 | V0 public network schema | Provisional: Protobuf is the public Robot Protocol source of truth; transport bindings do not own semantics | Golden compatibility tests plus one major-version rejection scenario; fixed-layout RT messages remain separate and bulk payloads use descriptors/handles |

## Deep-research coverage

| Direction | Primary research | Remaining decision work |
| --- | --- | --- |
| Runtime and RT | `rust-realtime-runtime.md`, `runtime-and-platform-reference-projects.md` | D-05, D-09 |
| Time and replay | `time-synchronization-and-determinism.md` | D-07 ADR and V0 scenario |
| Simulation consistency | `simulation-architecture.md` | MuJoCo conformance and tolerance evidence |
| Protocol and middleware | `robot-protocol-and-data-model.md`, `middleware-and-ipc.md` | D-06 transport confirmation; D-21 schema compatibility evidence |
| Model and calibration | `robot-model-manifest-calibration.md` | D-08 validator spike |
| Safety and security | `safety-and-fault-architecture.md`, `security-threat-model.md` | Target-specific hazard/security decisions |
| Hardware and fieldbus | `l0-hal-fieldbus.md`, `physical-ai-system-landscape.md` | D-16 and D-17 when hardware exists |
| OTA and evidence | `ota-and-observability.md` | D-13 candidate manifest, later D-18 |
| Policy/inference interface | `policy-runtime-interface.md` | D-19 M1a contract, then deferred chunking/alignment when a real workload exists |
| Compute latency | none yet; D-20 produces it | M1a development characterization, then M1b 24-hour representative-board qualification |

## Register discipline

- Do not add component backlog items here; put implementation work in the bootstrap plan.
- Do not duplicate research conclusions; link the evidence.
- Every `Provisional` entry has a measurable reversal trigger.
- Every `Deferred` entry says what reopens it.
- Close an experiment with an ADR, including negative results; do not leave a permanent spike branch as the decision record.
