# Architecture Decision Records

> Status: Process definition. Open and provisional questions are indexed in the [Decision and Research Register](../plans/decision-register.md); this directory contains durable decisions after their evidence is sufficient.

This directory records decisions that materially shape Soma.

Deep Research captures evidence and alternatives. The reference architecture captures the current system thesis. An ADR records a concrete decision, its context, trade-offs, and consequences.

## Suggested format

```text
# ADR-0001: Decision title

Status: Proposed | Accepted | Superseded
Date: YYYY-MM-DD

## Context

What problem are we deciding?

## Decision

What are we choosing?

## Alternatives considered

What else was evaluated?

## Consequences

What becomes easier, harder, or constrained?

## Validation

What evidence, benchmark, or test supports the decision?
```

## Candidate early ADRs

- canonical L-2 through L5 platform layers and SA-0 through SA-5 safety-authority namespace;
- RT/runtime process separation;
- Zenoh-first distributed protocol;
- ROS 2 as an external adapter;
- canonical ProductModelManifest and composed runtime RobotManifest;
- product-model, robot-instance, calibration, control, and SafetyProfile artifact boundaries;
- canonical time domains and Plant timeline/runtime/lease generation semantics;
- MuJoCo as the first reference simulation backend;
- minimal manifest validator versus any later model-generation tooling;
- provisional Zenoh default and its measurable reevaluation triggers;
- single maturity-bearing ReleaseManifest for integrated-set identity;
- security trust boundaries and target-specific mechanism choices;
- OTA trust/update stack;
- observability data planes.

Do not write an ADR before the underlying question is sufficiently researched or experimentally validated.
