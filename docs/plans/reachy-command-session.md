# Reachy Simulation Command Session Plan

> Status: Approved for implementation, 2026-08-25.
> This is a simulation-only developer interaction slice. It does not resume or
> alter the hardware-gated bootstrap work in `docs/status/active/bootstrap.md`.

## Preflight Contract

Preflight status: DRAFT approved by the user on 2026-08-25.

Task source: conversation following cross-robot research, cross-review, and
`intuitive-shape`.

Canonical source: `docs/plans/reachy-command-session.md`.

Route: durable `$intuitive-flow`.

Goal: provide one usable terminal command session that moves the fixed Reachy
Mini simulation through Soma's existing command path while preserving its
timeline, sequence, TTL, hold, and evidence semantics.

Scope:

- add `scripts/run-sim-teleop` as the owning launcher for the local simulation
  stack, optional observational MuJoCo/Rerun views, terminal session, and clean
  shutdown;
- add one focused Python command-session module that bootstraps from public
  state, maintains a complete nine-position target, emits discrete bounded
  commands, follows Plant timeline changes, and reports command outcomes;
- support `A`/`D` body-yaw nudges and `Q`/`E` mirrored antenna nudges;
- add focused tests and update the compact run/status documentation;
- preserve the existing scenario and visualization behavior.

Non-goals: multi-robot adapters; generic robot manifests; generic Joint,
Cartesian, Base, or Gripper targets; capability discovery; resource graphs,
leases, priority, or arbitration; generic RPC, Action, or Event frameworks;
Reachy head-pose control before Stewart-platform IK; hardware motion; official
Reachy daemon integration; MuJoCo or Rerun command write-back; terminal key-up,
hold-to-move, continuous servoing, reset, pause, step, or perturbation.

Entity budget:

- reuse: existing Protobuf `RtRequest`/`ActuatorTarget`/`ActuatorState`, Zenoh
  command and state keys, `robot-runtime`, `robot-rt`, `ReachySimPlant`, and the
  optional read-only observer stack;
- remove/merge: share launcher mechanics with `run-sim-scenario` where a small
  existing helper or narrow extraction clearly reduces duplication; do not
  broaden the launcher into a generic supervisor;
- new: one public launcher because this is a distinct long-lived operator
  workflow, and one focused Python module because command-session state and
  terminal input do not belong in the fixed acceptance scenario;
- expansion triggers: any second embodiment, public target taxonomy, head-pose
  IK, continuous command stream, concurrent command owner, resource authority,
  viewer write-back, or hardware actuation requires a separate proposal and
  approval.

Context:

- must-read: `AGENTS.md`, `docs/status/active/bootstrap.md`, this plan,
  `README.md`, `STATUS.md`, `proto/soma.proto`,
  `python/soma_client/scenario.py`, `scripts/run-sim-scenario`,
  `crates/soma-core/src/lib.rs`, and the runtime command bridge;
- useful: `docs/plans/simulation-visualization-plan.md`, fixed Reachy model
  actuator ordering/limits, and existing runtime/simulation tests;
- avoid-unless-needed: generic protocol research, vendor SDKs, hardware probe
  work, deferred architecture layers, and historical execution evidence.

Acceptance:

- SUCCESS: `scripts/run-sim-teleop --visualize` opens the terminal session and
  observational views; each supported key emits one valid command through the
  existing Python -> Zenoh -> runtime -> real-time -> MuJoCo path; visible
  motion and public state agree; `Ctrl-C` leaves no owned process or socket;
- BLOCKED_NEEDS_DECISION: a correct implementation requires a public protocol
  change, generic abstraction, head-pose IK, continuous key semantics, viewer
  write-back, or hardware control;
- BLOCKED_NEEDS_LOCAL_VALIDATION: desktop-backed keyboard input, visible motion,
  Rerun evidence, and shutdown cannot be exercised in the implementation
  environment;
- INTERMEDIATE_ONLY: none;
- No regressions: the existing headless and visual fixed scenarios retain
  their command, timing, evidence, supervision, and cleanup behavior.

Verification:

- deterministic: Python unit tests plus workspace format, test, check, and
  clippy gates;
- integration: existing fixed scenario and focused command-session tests;
- product-run: `scripts/run-sim-teleop --help`, a bounded headless command
  session harness, and `scripts/run-sim-teleop --visualize`;
- local-live-manual: on a desktop, press each key repeatedly and in mixed order,
  confirm the expected MuJoCo motion and Rerun requested/admitted/applied
  evidence, then confirm clean `Ctrl-C` teardown;
- optional: Xvfb process-lifecycle validation, which does not replace the
  desktop keyboard and visual review.

Execution: main session owns implementation, supervision, verification, and
final status; worker: none; worker-goal: none.

To execute: `/goal execute docs/plans/reachy-command-session.md with intuitive-flow`

Optional tracking: none.

Approval: approved; changes to scope, interaction semantics, or acceptance
require revision.

## Interaction Contract

The terminal is the only command-originating UI. MuJoCo camera controls and
Rerun navigation remain observational and cannot mutate simulation state.

| Key | Complete-target update | Step |
| --- | --- | --- |
| `A` | decrement `yaw_body` | `0.05 rad` |
| `D` | increment `yaw_body` | `0.05 rad` |
| `Q` | decrement right antenna and increment left antenna | `0.10 rad` each |
| `E` | increment right antenna and decrement left antenna | `0.10 rad` each |
| `Ctrl-C` | stop the session and all processes it owns | n/a |

Keys are case-insensitive discrete events. One recognized key produces at most
one command; key release is not inferred, keyboard repeat is merely repeated
discrete input, and the client does not publish periodically. There is no Space
binding or hidden reset binding.

Each command has a `250 ms` TTL. The target is always all nine actuator
positions in the fixed order: body yaw, Stewart motors 1--6, right antenna,
left antenna. Unchanged positions are preserved. Body yaw is clamped to the
pinned model's finite joint range; antenna accumulation is clamped to a fixed
`[-pi, pi]` interactive envelope. The simulation Plant remains the final
validator of finite values and model limits.

The terminal should remain compact: current timeline and sequence, the latest
measured yaw/antenna values, the pending or last requested target, and the
latest accepted/rejected/applied/hold evidence. It must not claim richer
safety-output, progress, or authority evidence than the current protocol
provides.

## Command-Session Semantics

1. Open a Zenoh client using the same loopback configuration and public keys as
   the fixed scenario.
2. Do not accept input until a healthy state with exactly nine finite positions
   has arrived. Initialize the complete local target from that measured state.
3. Allocate a sequence strictly newer than the state observed on the current
   timeline. Never reuse a sequence within that timeline.
4. For one recognized key, update only the named coordinates, clamp them to the
   fixed interactive bounds, and publish one complete `ActuatorTarget` with the
   current timeline, next sequence, and `250 ms` TTL.
5. Continue receiving state independently of terminal input. Correlate
   disposition and applied evidence by timeline/sequence where the existing
   message permits it; do not treat publish success as command admission.
6. On a Plant timeline change, invalidate the old pending command and sequence
   basis, rebase the complete target to the first valid measured state on the
   new timeline, and only then accept another key.
7. After TTL expiry, show measured-position hold from public state. Do not keep
   resending the last command to mask expiry.
8. Malformed state, unhealthy state, rejected command, Zenoh loss, or runtime
   exit is surfaced clearly. The session must not silently switch timelines,
   synthesize missing state, or continue publishing from stale data.

No public Protobuf or Zenoh-key change is expected. Stop and reopen the plan if
the existing messages cannot honestly support the required terminal evidence.

## Launcher And Lifecycle

`scripts/run-sim-teleop` owns only processes and filesystem artifacts it
creates. Its default mode runs the simulation/runtime and terminal without a
display dependency. `--visualize` additionally starts the existing pinned
Rerun viewer and read-only MuJoCo/Rerun observer route. `-h`/`--help` documents
both modes, controls, TTL, discrete input behavior, and `Ctrl-C` cleanup;
unknown or incompatible options fail with usage and exit code 2.

Startup must wait for the same runtime sockets and loopback Zenoh readiness as
the fixed scenario before starting the Python session. Visual mode must retain
the pinned dependency, explicit loopback endpoint, and observer readiness
rules. Any startup failure prints the relevant owned logs and tears down all
started children. Normal exit, `Ctrl-C`, `TERM`, Python failure, runtime exit,
and visualization failure must reap owned process groups and remove owned
sockets/ready files without affecting unrelated processes.

Avoid copying the full scenario launcher into a second drifting script. During
implementation, choose the smallest local reuse that keeps both public scripts
readable and independently testable; do not introduce a general process
orchestration framework.

## Focused Tests

Add behavior-level tests for:

- bootstrap rejects wrong-length/non-finite state and copies all nine measured
  positions into the initial target;
- each key changes only its specified coordinates and preserves the other
  values;
- yaw and antenna targets clamp at their fixed bounds;
- sequence allocation is strictly increasing within one timeline;
- a timeline change discards stale pending state and rebases target/sequence;
- accepted, rejected, target-applied, and TTL-to-measured-hold evidence is
  represented accurately without inventing acknowledgements;
- unrecognized input emits no command and one recognized event emits exactly
  one command;
- launcher help/error handling, readiness failure, child failure, signal
  handling, and socket/process teardown.

Tests should exercise the command-session state machine without requiring a
live terminal, desktop, or sleeps. A bounded input harness may feed key events
to the public command for integration proof; production behavior remains an
interactive terminal session.

## Required Verification

```bash
scripts/cargo-mujoco fmt --all -- --check
scripts/cargo-mujoco test --workspace
scripts/cargo-mujoco check --workspace
scripts/cargo-mujoco clippy --workspace --all-targets -- -D warnings
scripts/run-sim-scenario
scripts/run-sim-scenario --visualize
scripts/run-sim-teleop --help
scripts/run-sim-teleop <bounded-input-harness-options>
scripts/run-sim-teleop --visualize
```

The implementation must replace `<bounded-input-harness-options>` with the
smallest documented non-interactive test route rather than leaving a
placeholder in the finished product documentation.

The final desktop-backed run must verify `A`, `D`, `Q`, and `E` individually
and in mixed order; visible yaw and mirrored antenna motion; requested versus
measured values and disposition/applied/expiry evidence in Rerun; continued
terminal operation after TTL hold; and clean shutdown with no owned processes
or sockets left behind. Xvfb may prove window startup and teardown only.

## Stop Gates

- Stop if terminal input, Python, Zenoh, visualization, or process supervision
  would enter or block the periodic `robot-rt` path.
- Stop if interaction requires weakening timeline, sequence, TTL,
  measured-position hold, or requested/admitted/applied evidence semantics.
- Stop if MuJoCo or Rerun must become authoritative or write back.
- Stop if correct head motion requires bypassing Stewart-platform kinematics;
  this slice intentionally controls only body yaw and antennas.
- Stop if a generic command/target/session/robot abstraction is proposed only
  for a future embodiment.
- Stop before any official-daemon coexistence or physical robot actuation.

## Definition Of Done

The focused tests and all deterministic gates pass; the existing fixed
scenario remains unchanged in headless and visual modes; the public teleop
command proves discrete bounded motion through Soma's existing path; desktop
review confirms MuJoCo motion and Rerun evidence; every owned process and
socket is cleaned up; compact human docs describe the command and its limits;
and this plan is marked implemented without changing the blocked hardware
capsule.
