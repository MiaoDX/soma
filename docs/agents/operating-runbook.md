# Agent Operating Runbook

This runbook holds repository-specific setup and verification detail that does
not belong in the root startup contract.

## Environment

Rust is the primary stack. The workspace is rooted at the repository root, and
`.cargo/config.toml` gives Cargo and rust-analyzer the repository-local MuJoCo
download path. Use `scripts/cargo-mujoco` for commands that also need the
MuJoCo shared library at runtime:

```bash
scripts/cargo-mujoco test --workspace
```

Python is a small scenario client, not the primary development environment.
Create `python/.venv` only when running or changing that client:

```bash
cd python
uv sync
cd ..
```

## Verification

Use the smallest relevant check while iterating, then run the complete gates
for changes that affect shared behavior:

```bash
scripts/cargo-mujoco test --workspace
cargo fmt --all -- --check
scripts/cargo-mujoco clippy --workspace --all-targets -- -D warnings
scripts/run-sim-scenario
```

`scripts/run-sim-scenario` builds both runtime processes, starts the local
Zenoh path, runs the Python client, and verifies movement, TTL expiry to
measured-position hold, timeline reset, and old-timeline rejection.

## Hardware Gate

The Reachy probe is intentionally read-only:

```bash
cargo run --bin soma-reachy-probe --
cargo run --bin soma-reachy-probe -- --device /dev/ttyUSB0
```

Run it only with the official daemon stopped. A missing device is an external
blocker, not a reason to weaken identity, configuration, torque-state, or
exclusive-ownership checks. Stop before any register write, torque enable, or
physical motion unless N0 has passed and a human has approved N1.

## Language Tooling

Serena is configured in `.serena/project.yml` with Rust first and Python
second. Codex starts it through the user-level MCP entry registered by
`serena setup codex`; a new Codex session is required to expose the tools.

Validate the local project integration with:

```bash
serena project health-check .
serena project index .
```

If `rust-analyzer` is missing, prefer the active toolchain's component. A
standalone binary on `PATH` is also supported when the configured rustup mirror
does not publish the matching component. Keep machine-local executable paths
out of `.serena/project.yml`.
