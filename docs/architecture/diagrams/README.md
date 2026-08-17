# Architecture Diagrams

These SVGs are the canonical visual summary of the reference architecture and
the current plan. If a diagram and the prose in `docs/architecture/` or
`docs/plans/` disagree, treat it as a documentation bug and fix whichever one
is wrong — do not assume either is automatically correct.

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

Regenerate rather than hand-edit where practical; these were produced as flat
SVG (no external font or script dependency) so they render identically in
light and dark GitHub themes and can be opened directly in a browser.
