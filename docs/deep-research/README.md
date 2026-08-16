# Soma Deep Research

This directory preserves the evidence base behind Soma.

Architecture documents should stay concise and opinionated. Deep Research is intentionally broader: it captures competing approaches, implementation references, failure modes, adjacent-industry practices, and unresolved questions so that architectural decisions remain traceable.

## Suggested structure

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

The directory will grow gradually. We prefer a small number of high-signal documents over many empty placeholders.

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
