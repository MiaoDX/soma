# Architecture Diagrams

These SVGs are derived visual overviews of the reference architecture and
current plan. Markdown in `docs/architecture/` and `docs/plans/` is
authoritative. If a diagram disagrees with that prose, update the diagram.

- [`soma-stack.svg`](soma-stack.svg) — responsibility layers (L-2..L5), the
  `robotd` deployment unit, the three Plant implementations, and the
  independent SA-0..SA-5 safety authority chain. Companion to
  [Layering and trust boundaries](../layering-and-trust-boundaries.md).
- [`soma-command-lineage.svg`](soma-command-lineage.svg) — the
  requested / admitted / safety-output / applied command lineage, which
  authority owns each transition, and what gets recorded. Companion to the
  "Requested, admitted, safety-output, applied" section of the
  [reference architecture](../reference-architecture.md).
- [`soma-roadmap.svg`](soma-roadmap.svg) — M0 through M4, what each milestone
  gates or triggers, and what is explicitly out of scope for Milestone 1.
  Companion to the [bootstrap plan](../../plans/bootstrap-plan.md).

The diagrams are manually maintained flat SVGs with no external font or script
dependency, so they render consistently and can be opened directly in a
browser. Keep them concise; contract detail belongs in the Markdown sources.
