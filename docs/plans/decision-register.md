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

- **P0** — blocks the no-hardware runnable spine.
- **P1** — needed before a real platform or release claim.
- **P2** — later hardware, fleet, or qualification work.

## Register

| ID | Priority | Question | Current posture | Next evidence / stop condition |
| --- | --- | --- | --- | --- |
| D-01 | P0 | Product layers and safety-authority vocabulary | Accepted: L-2..L5 and SA-0..SA-5 remain separate | Maintain conformance across architecture docs and first code modules |
| D-02 | P0 | Hardware-independent control boundary | Accepted: one `ControlCore` and bounded Plant contract for MuJoCo/SIL and later hardware | V0 MuJoCo vertical slice; no embodiment-specific public branch |
| D-03 | P0 | Current implementation direction without hardware | Accepted: top-down from MuJoCo and G1/G2-class exposed boundaries | Runnable spine plus hardware-free adapter conformance |
| D-04 | P0 | Runtime process boundary | Accepted: supervised `robot-rt` + `robot-runtime`; operations begin as runtime modules | Restart/generation/lease integration scenario |
| D-05 | P0 | RT/runtime IPC implementation | Experiment required: preserve mailbox behavior, compare minimal SPSC with a mature community option | Choose the smallest implementation meeting boundedness, restart, ABI, and Performance Envelope criteria |
| D-06 | P0 | Distributed transport | Provisional: Zenoh-first; Cyclone DDS/gRPC remain gateways | Compare only if Zenoh misses its envelope, creates an operational problem, or native DDS becomes required |
| D-07 | P0 | Time and lifecycle semantics | Research ready: `ROBOT_MONOTONIC`, simulation time, synchronized time, timeline/generation split | Time ADR plus reset/restart/stale-command scenario |
| D-08 | P0 | V0 robot model strategy | Accepted: minimal `ProductModelManifest` + backend-native assets + validator | Validate native MJCF and one second fixture; no universal compiler |
| D-09 | P0 | RT execution/replay framework | Experiment required: Copper-hosted vs minimal Soma-native representative graph | Adopt only if public contracts remain independent and replay/RT criteria pass |
| D-10 | P0 | System verification traceability | Accepted: lightweight claim-to-evidence matrix | Each milestone exit claim has a scenario, pass condition, evidence level, and retained artifact |
| D-11 | P0 | Performance requirements without target hardware | Accepted: define Performance Envelopes, not universal numbers | Record workload/platform identity and distributions for every spike |
| D-12 | P1 | Commercial platform boundary | Accepted: classify as `NativePlantAdapter` or `ManagedMotionGateway` | Hardware-free contract tests now; physical qualification only when hardware exists |
| D-13 | P1 | Release integration identity | Accepted: one maturity-bearing `ReleaseManifest`; no parallel integration artifact | Candidate manifest pins source, toolchain, artifacts, tests, exclusions, and evidence |
| D-14 | P1 | Security scope | Accepted: high-level Threat Model now; broad protocol abuse testing deferred | Functional verifier tests in V0; security qualification before production claim |
| D-15 | P1 | Python packaging and target matrix | Research ready | Build/install spike for the first Linux x86_64/aarch64 development profiles |
| D-16 | P2 | First physical platform | Deferred until accessible hardware and project need are known | Evaluate exposed boundary, licensing, safety authority, model assets, and recovery before selection |
| D-17 | P2 | Owner-controlled L0/L-1/L-2 path | Deferred, retained as a long-term completeness requirement | Resume with own hardware or an accessible component bench; qualify only observed layers |
| D-18 | P2 | Fleet OTA, production trust, and long-term operations | Deferred after runnable spine and target platform | Product-specific hazard, security, rollout, recovery, and retention evidence |

## Deep-research coverage

| Direction | Primary research | Remaining decision work |
| --- | --- | --- |
| Runtime and RT | `rust-realtime-runtime.md`, `runtime-and-platform-reference-projects.md` | D-05, D-09 |
| Time and replay | `time-synchronization-and-determinism.md` | D-07 ADR and V0 scenario |
| Simulation consistency | `simulation-architecture.md` | MuJoCo conformance and tolerance evidence |
| Protocol and middleware | `robot-protocol-and-data-model.md`, `middleware-and-ipc.md` | D-06 confirmation, schema choice |
| Model and calibration | `robot-model-manifest-calibration.md` | D-08 validator spike |
| Safety and security | `safety-and-fault-architecture.md`, `security-threat-model.md` | Target-specific hazard/security decisions |
| Hardware and fieldbus | `l0-hal-fieldbus.md`, `physical-ai-system-landscape.md` | D-16 and D-17 when hardware exists |
| OTA and evidence | `ota-and-observability.md` | D-13 candidate manifest, later D-18 |

## Register discipline

- Do not add component backlog items here; put implementation work in the bootstrap plan.
- Do not duplicate research conclusions; link the evidence.
- Every `Provisional` entry has a measurable reversal trigger.
- Every `Deferred` entry says what reopens it.
- Close an experiment with an ADR, including negative results; do not leave a permanent spike branch as the decision record.
