# Reachy Mini Simulation Showcase

> Plan status: DONE; published pipeline and enriched choreography verified, 2026-08-29
> Last updated: 2026-08-29
> Control plane: `/root`

## Plan Ledger

- `status`: DONE; artifact pipeline, live report, deterministic choreography, and Pages workflow implemented
- `current_slice`: 12-second multi-axis choreography plus acceptance tail captured on one continuous 14-second MuJoCo/Rerun timeline
- `next_action`: merge the choreography update and confirm the refreshed Pages deployment
- `evidence`: `scripts/verify-sim-showcase`, browser/Rerun screenshots, and the generated ignored `output/simulation-showcase/`
- `no_touch`: public protocol, Plant/RT semantics, robot profile, hardware gates, repository settings
- `stop_condition`: reopen before any hardware claim, additional robot/backend, or interactive report control

## Goal

Publish a reproducible Reachy Mini simulation showcase: a stable README poster
links to a GitHub Pages report containing MuJoCo motion media, a Rerun recording,
the fixed-case comparison summary, and provenance for the exact commit and run.

## Scope

- Add one owning command, `scripts/build-sim-showcase`, which generates the
  complete static report without changing the existing headless scenario.
- Generate a fixed-camera MuJoCo poster and browser-playable WebM from the
  existing fixed Reachy simulation command path.
- Add a deterministic showcase-only choreography covering body yaw, antennae,
  coordinated motion, and safe head nod/tilt targets derived by the pinned
  official Reachy Mini analytical kinematics binding.
- Persist the existing Rerun evidence as a downloadable `.rrd`.
- Generate a self-contained static HTML report with scenario status, media,
  comparison metrics, commit provenance, and artifact links.
- Add a GitHub Actions workflow that builds the report and publishes it through
  GitHub Pages.
- Commit a stable README poster and link it to the Pages report.

## Non-goals

- Reachy hardware access, native bus ownership, torque enable, or motion.
- A new simulator backend, generic renderer abstraction, public snapshot
  protocol, replay service, or production recording/retention system.
- Treating viewer pixels as correctness evidence or changing the authoritative
  `robot-rt` / `ReachySimPlant` path.
- Publishing secrets, mutable CI artifact URLs, or GitHub credentials.

## Entity Budget

Reuse the pinned Reachy model, current scenario, visualization observer,
comparison report, Cargo wrapper, and Python environment. New durable entities
are limited to the owning generation command, fixed report template/assets,
Pages workflow, and the minimum renderer/recording hooks required to create the
declared artifacts. Expanding the public protocol, simulator surface, robot
profiles, or deployment target requires renewed approval.

## Acceptance

1. `scripts/build-sim-showcase <output-dir>` succeeds from a clean checkout and
   produces `index.html`, a nonblank poster, a browser-playable motion file, a
   nonempty `.rrd`, and machine-readable provenance/status metadata.
2. The generated report visibly identifies Reachy Mini, embeds the poster and
   motion, links the `.rrd`, shows the declared Soma/official metrics, and names
   commit SHA plus scenario result.
3. Generated media comes from the authoritative fixed simulation path and uses
   a fixed reviewed camera; choreography commands use the existing public
   command route, while capture remains read-only and cannot write back.
4. The existing headless and interactive visualization commands retain their
   behavior and acceptance assertions.
5. The Pages workflow runs the owning command, validates the report, uploads a
   Pages artifact, and deploys only from the default branch; manual dispatch on
   another branch builds without deployment.
6. README shows the committed poster, accurately labels simulation-only scope,
   and links to the stable Pages report.

## Verification

```bash
scripts/build-sim-showcase output/simulation-showcase
scripts/cargo-mujoco test --workspace
cargo fmt --all -- --check
scripts/cargo-mujoco clippy --workspace --all-targets -- -D warnings
scripts/run-sim-scenario
```

Artifact verification additionally checks nonblank image pixels, exact media
duration, all nine requested Rerun streams, continuous robot transforms through
the acceptance tail, TTL/reset/rejection events, HTML-local link resolution,
metadata content, and absence of hardware claims. The generated HTML requires
a browser screenshot check at desktop and mobile widths.

## Stop Gates

Stop before changing the public command/state schema, `Plant` contract,
real-time semantics, robot profile, hardware gate, or GitHub repository settings.
If native window capture is nondeterministic, replace it with fixed offscreen
rendering rather than weakening the media acceptance criteria.

## Completion

The artifact pipeline, responsive report, README integration, and Pages
workflow are implemented and published. The richer choreography and continuous
Rerun synchronization are locally product-verified; the refreshed deployment
will follow the next merge to `main`.
