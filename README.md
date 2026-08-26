# Soma

**A system foundation for embodied intelligence.**

Soma explores a common software and systems foundation for Physical AI that can
span different robotic embodiments, from wheeled mobile robots to legged robots
and future platforms.

The project focuses on the layers beneath application intelligence: hardware abstraction, real-time control, runtime and communication, simulation, safety, deployment, observability, OTA, and ecosystem adapters.

> Many forms. One foundation.

## Architecture at a glance

![Soma layers and safety authority](docs/architecture/diagrams/soma-stack.svg)

Boundaries are the architecture; the boxes are negotiable. Colour encodes
execution class, not importance: grey is ordinary software, indigo the
non-real-time runtime, teal the deterministic control path, violet
simulation, and ochre the independent safety and trust path that must hold
without Linux, `robotd`, or the network.

![Command lineage](docs/architecture/diagrams/soma-command-lineage.svg)

A policy that behaves oddly is either being constrained or is genuinely bad.
Without these four observable states — requested, admitted, safety-output,
applied — you cannot tell which one, or who to hold responsible.

![Milestones](docs/architecture/diagrams/soma-roadmap.svg)

More diagrams and their relationship to the prose: [`docs/architecture/diagrams/`](docs/architecture/diagrams/).

## Status

Soma is currently in the **research and architecture** phase. The repository will evolve from design documents and comparative research into a working reference implementation validated across simulation and real robot embodiments.

## Design direction

- **Embodiment-independent core** — common contracts above hardware-specific HALs.
- **Rust-first systems stack** — Rust for the real-time core, robot runtime, and SDK client where practical.
- **ROS 2 at the edge** — ROS 2 is an ecosystem adapter, not a dependency of the robot core.
- **Simulation as a first-class backend** — MuJoCo, Isaac Sim/Lab, Genesis, SIL, and HIL participate in the same system contracts.
- **Production-shaped, evidence-led growth** — preserve the boundaries that are
  expensive to undo, and implement qualification, security, compatibility, and
  operations only when a concrete trigger exists.

## Documentation

- [Reference architecture](docs/architecture/reference-architecture.md)
- [Layering and trust boundaries](docs/architecture/layering-and-trust-boundaries.md)
- [Security threat model](docs/architecture/security-threat-model.md)
- [Glossary](docs/glossary.md)
- [Architecture decisions](docs/decisions/README.md)
- [Decision and research register](docs/plans/decision-register.md)
- [Bootstrap plan](docs/plans/bootstrap-plan.md)
- [Deep research index](docs/deep-research/README.md)

## License

[MIT](LICENSE)

---

*Soma* (σῶμα) means **body** — the physical embodiment through which intelligence acts on the world.
