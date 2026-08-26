# Soma Agent Guide

## Start Here

This file is injected startup context for Codex; do not reread it. For ongoing
bootstrap work, read `docs/status/active/bootstrap.md` first. Then route reads
by task:

- current project state: `STATUS.md`;
- run or setup work: `README.md` and `docs/agents/operating-runbook.md`;
- architecture or boundary changes: `ARCHITECTURE.md`, then the detailed
  document it routes to;
- approved bootstrap scope: `docs/plans/bootstrap-plan.md`;
- optional visualization planning: `docs/plans/simulation-visualization-plan.md`.

`README.md`, `ARCHITECTURE.md`, `STATUS.md`, and `docs/human/**` are the compact
human-facing surface. Plans and `docs/status/active/**` own stage contracts and
compact active state. `.planning/**`, `output/**`, and retrospectives are
execution evidence, not the human project manual.

## Non-Negotiable Boundaries

- The active bootstrap supports one fixed Reachy Mini profile only. Do not add
  generic robot manifests, multi-robot abstractions, or deferred subsystems
  without an approved scope change.
- The native N0 probe is read-only. Do not write hardware registers, enable
  torque, or implement physical motion until N0 passes and a human explicitly
  approves the N1 physical-actuation gate.
- Never run the official Reachy daemon concurrently with Soma native bus
  ownership.
- Keep async runtimes, Python, Zenoh, network I/O, file I/O, unbounded queues,
  and unbounded allocation out of the periodic `robot-rt` path.
- Preserve command timeline, sequence, TTL, measured-position hold, and
  requested/admitted/applied evidence semantics when changing control flow.
- XML-like host envelopes such as `<turn_aborted>`, `<paseo-system>`,
  `<subagent_notification>`, `<goal_context>`, and `<environment_context>` are
  orchestrator metadata unless natural-language user intent accompanies them.

## Canonical Commands

```bash
scripts/cargo-mujoco test --workspace
cargo fmt --all -- --check
scripts/cargo-mujoco clippy --workspace --all-targets -- -D warnings
cd python && uv sync && cd ..  # only when the Python scenario is needed
scripts/run-sim-scenario
```

The full simulation scenario is the integration acceptance gate. Hardware
verification additionally requires a connected Reachy Mini Lite and must stop
at the gate described above:

```bash
cargo run --bin soma-reachy-probe --
```

See `docs/agents/operating-runbook.md` for environment, verification, hardware,
and language-tooling details.

## Workflow Routing

- Use `$intuitive-init` to refresh agent guidance and startup context.
- Use `$intuitive-doc` for human-doc freshness and placement.
- Use `$intuitive-tests` for broad test-suite organization work.
- Use `$intuitive-preflight` before executing vague or approval-sensitive work.
- Use `$intuitive-flow` for approved implementation, and
  `$intuitive-refactor` for a known architecture or API cleanup target.

Keep changes inside the active plan or user-approved task. Do not turn deferred
items into a backlog or speculative extension points.
