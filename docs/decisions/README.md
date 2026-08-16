# Architecture Decision Records

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

- RT/runtime process separation;
- Zenoh-first distributed protocol;
- ROS 2 as an external adapter;
- canonical RobotManifest;
- MuJoCo as the first reference simulation backend;
- OTA trust/update stack;
- observability data planes.

Do not write an ADR before the underlying question is sufficiently researched or experimentally validated.
