# Soma

**A system foundation for embodied intelligence.**

Soma explores a production-grade reference system for Physical AI — a common software and systems foundation that can span different robotic embodiments, from wheeled mobile robots to legged robots and future platforms.

The project focuses on the layers beneath application intelligence: hardware abstraction, real-time control, runtime and communication, simulation, safety, deployment, observability, OTA, and ecosystem adapters.

> Many forms. One foundation.

## Status

Soma is currently in the **research and architecture** phase. The repository will evolve from design documents and comparative research into a working reference implementation validated across simulation and real robot embodiments.

## Design direction

- **Embodiment-independent core** — common contracts above hardware-specific HALs.
- **Rust-first systems stack** — Rust for the real-time core, robot runtime, and SDK client where practical.
- **ROS 2 at the edge** — ROS 2 is an ecosystem adapter, not a dependency of the robot core.
- **Simulation as a first-class backend** — MuJoCo, Isaac Sim/Lab, Genesis, SIL, and HIL participate in the same system contracts.
- **Production from day one** — safety, diagnostics, observability, OTA, security, compatibility, and recovery are architecture concerns, not afterthoughts.

## Documentation

- [Reference architecture](docs/architecture/reference-architecture.md)
- [Layering and trust boundaries](docs/architecture/layering-and-trust-boundaries.md)
- [Security threat model](docs/architecture/security-threat-model.md)
- [Glossary](docs/glossary.md)
- [Architecture decisions](docs/decisions/README.md)
- [Decision and research register](docs/plans/decision-register.md)
- [Bootstrap plan](docs/plans/bootstrap-plan.md)
- [Deep research index](docs/deep-research/README.md)

---

*Soma* (σῶμα) means **body** — the physical embodiment through which intelligence acts on the world.
