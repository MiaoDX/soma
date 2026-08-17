# Soma Deep Research

> Status: Research index. Entries preserve evidence and alternatives; the Decision Register and ADRs determine what is current.

This directory preserves the evidence base behind Soma.

Architecture documents should stay concise and opinionated. Deep Research is intentionally broader: it captures competing approaches, implementation references, failure modes, adjacent-industry practices, and unresolved questions so that architectural decisions remain traceable.

## Current research notes

### System landscape and design space

- [`physical-ai-system-landscape.md`](physical-ai-system-landscape.md) — robotics vendors, SDK exposure levels, and recurring industry patterns.
- [`reference-system-design-space.md`](reference-system-design-space.md) — the broad system design space and current Soma hypotheses.

### Hardware, runtime, and communication

- [`l0-hal-fieldbus.md`](l0-hal-fieldbus.md) — L-2/L-1/L0 decomposition below the Plant, HAL vs Plant, EtherCAT/CAN-FD, RT host, safety, and public reference implementations.
- [`rust-realtime-runtime.md`](rust-realtime-runtime.md) — Rust production readiness, PREEMPT_RT execution profile, allocation/panic/unsafe/FFI boundaries, ABI strategy, and validation plan.
- [`runtime-and-platform-reference-projects.md`](runtime-and-platform-reference-projects.md) — Copper execution/replay, Eclipse S-CORE platform/process lessons, and a practical Rust robotics repository map.
- [`middleware-and-ipc.md`](middleware-and-ipc.md) — RT IPC, shared memory, provisional Zenoh, Cyclone DDS/gRPC gateways, public protocol semantics, and triggered benchmark plan.
- [`robot-protocol-and-data-model.md`](robot-protocol-and-data-model.md) — identity, capabilities, state/command/RPC/action/event/lease semantics, schema evolution, errors, bulk data, and V0 protocol surface.
- [`time-synchronization-and-determinism.md`](time-synchronization-and-determinism.md) — monotonic/PTP/TAI/device/simulation time, lifecycle generations, Plant timeline/tick, EtherCAT DC, sensor timestamp provenance, and deterministic replay.

### Robot model, simulation, and safety

- [`robot-model-manifest-calibration.md`](robot-model-manifest-calibration.md) — product model, instance inventory, calibration, control and SafetyProfile boundaries, simulator overlays, hashes, and PolicyBundle compatibility.
- [`simulation-architecture.md`](simulation-architecture.md) — MuJoCo, Isaac, Genesis, SIL/HIL, simulation time, model identity, PolicyBundle, and conformance testing.
- [`safety-and-fault-architecture.md`](safety-and-fault-architecture.md) — SA-0 through SA-5 safety authority, SafetyProfile governance, standards landscape, safe behavior, fault lifecycle, watchdog hierarchy, and research-mode controls.

### Lifecycle and operations

- [`ota-and-observability.md`](ota-and-observability.md) — release/OTA lifecycle, multi-ECU recovery, health gates, fleet rollout, OTel, MCAP flight recording, crash evidence, and supply-chain concerns.

## Suggested long-term structure

As the evidence base grows, these notes may be split into topic directories:

```text
deep-research/
├── landscape/       # robot companies, SDKs, product architectures
├── hardware/        # buses, device drivers, HAL, RT compute
├── runtime/         # Rust, RT scheduling, IPC, process boundaries
├── middleware/      # Zenoh, DDS, gRPC, shared memory
├── protocol/        # public data model, versioning, capability semantics
├── modeling/        # product/instance manifests, calibration, profiles, assets
├── simulation/      # MuJoCo, Isaac, Genesis, SIL/HIL, replay
├── safety/          # watchdogs, safety boundaries, fault handling
├── deployment/      # OTA, release engineering, security, fleet
└── case-studies/    # focused analysis of specific projects/vendors
```

We prefer a small number of high-signal documents over many empty placeholders. Split a note only when its scope becomes difficult to review or cite.

## Research template

Each substantial note should answer:

1. **Question** — what are we trying to understand?
2. **Context** — why does Soma care?
3. **Findings** — what did we learn?
4. **Alternatives** — what other approaches exist?
5. **Trade-offs** — what do we gain or lose?
6. **Implications for Soma** — what should change in the architecture?
7. **Sources** — primary references where possible.
8. **Open questions** — what still needs experiments or research?

## Relationship to other docs

- `deep-research/` — evidence and exploration;
- `architecture/` — current synthesis and architectural thesis;
- `decisions/` — why a specific choice was made;
- `plans/` — how we intend to build and validate it.
