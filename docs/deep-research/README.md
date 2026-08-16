# Soma Deep Research

This directory preserves the evidence base behind Soma.

Architecture documents should stay concise and opinionated. Deep Research is intentionally broader: it captures competing approaches, implementation references, failure modes, adjacent-industry practices, and unresolved questions so that architectural decisions remain traceable.

## Current research notes

- [`physical-ai-system-landscape.md`](physical-ai-system-landscape.md) — robotics vendors, SDK exposure levels, and recurring industry patterns.
- [`reference-system-design-space.md`](reference-system-design-space.md) — the broad system design space and current Soma hypotheses.
- [`l0-hal-fieldbus.md`](l0-hal-fieldbus.md) — L0 decomposition, HAL vs Plant, EtherCAT/CAN-FD, RT host, safety, and public reference implementations.
- [`middleware-and-ipc.md`](middleware-and-ipc.md) — RT IPC, shared memory, Zenoh, Cyclone DDS, gRPC, public protocol semantics, and benchmark plan.
- [`simulation-architecture.md`](simulation-architecture.md) — MuJoCo, Isaac, Genesis, SIL/HIL, simulation time, model identity, PolicyBundle, and conformance testing.
- [`ota-and-observability.md`](ota-and-observability.md) — release/OTA lifecycle, multi-ECU recovery, health gates, fleet rollout, OTel, MCAP flight recording, crash evidence, and supply-chain concerns.

## Suggested long-term structure

As the evidence base grows, these notes may be split into topic directories:

```text
deep-research/
├── landscape/       # robot companies, SDKs, product architectures
├── hardware/        # buses, device drivers, HAL, RT compute
├── runtime/         # Rust, RT scheduling, IPC, process boundaries
├── middleware/      # Zenoh, DDS, gRPC, shared memory
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
