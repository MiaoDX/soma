# Soma Architecture

Soma's current executable slice is a Rust-first control path for one fixed
Reachy Mini profile. A thin Python client exists only to drive the end-to-end
acceptance scenario.

```text
Python scenario client
        |
        | Protobuf over loopback Zenoh
        v
robot-runtime (Rust, non-RT)
        |
        | bounded exclusive Unix datagrams
        v
robot-rt (Rust control owner)
        |
        | Plant trait
        v
ReachySimPlant (MuJoCo)
```

The same bounded `Plant` and `ControlCore` contracts are intended to serve a
future native Reachy hardware Plant. That path is not implemented: the current
hardware surface stops at the standalone read-only N0 probe.

The local MuJoCo adapter validates simulation cadence when it loads a model.
One control period must be a positive integral number of physics timesteps;
advancing the Plant then executes exactly that many substeps. This simulation
contract stays outside the generic hardware-facing `Plant` trait, but every
future MuJoCo robot adapter must pass the same startup precheck so a mismatched
time scale cannot run silently.

## Implementation Language Selection

Language follows ownership and execution guarantees, not the feature label
(`sim`, `teleop`, or `client`). The default rule is:

> Rust owns mechanisms and authoritative robot semantics; Python expresses
> intent, composes workflows, and consumes public interfaces.

Choose Rust when a module:

- runs in the periodic control path or must provide bounded timing, memory, or
  failure behavior;
- owns Plant I/O, command admission, sequence/timeline/TTL handling, limits,
  hold behavior, safety decisions, or lifecycle authority;
- is a long-running robot-side runtime or system service whose failure must be
  contained without relying on a Python process; or
- is shared by simulation and future hardware and therefore must preserve the
  deployable control contract.

Choose Python when a module:

- is an L4 SDK/application, teleoperation surface, experiment, task
  orchestrator, data tool, or developer-facing workflow;
- submits requests and reads evidence through the public Robot Protocol without
  making safety or control decisions; or
- intentionally provides cross-language end-to-end proof of that protocol.

These rules do not prohibit a Python dependency behind a non-RT process
boundary, nor do they require rewriting a mature simulator or vendor library.
They do prohibit Python, async work, middleware, blocking I/O, or unbounded
allocation from entering `robot-rt`'s periodic path. A proposed exception must
name the boundary, timing/resource evidence, failure containment, and why the
existing adapter/process seam is insufficient.

Current mapping:

| Module | Language | Reason |
| --- | --- | --- |
| `ReachySimPlant`, `ControlCore`, `robot-rt` | Rust | authoritative Plant and deterministic control semantics |
| `robot-runtime` and protocol bridge | Rust | robot-side non-RT runtime and lifecycle authority |
| `python/soma_client/scenario.py` | Python | black-box protocol acceptance client |
| `python/soma_client/command_session.py` | Python | L4 terminal teleoperation client |
| `python/soma_client/showcase.py` | Python | deterministic offline showcase targets using pinned official analytical kinematics |

Therefore the current demos are not Python simulations: MuJoCo simulation and
control remain Rust; Python supplies two intentionally separate clients, while
the showcase helper only plans complete nine-actuator targets for the scenario
client. The official kinematics binding does not own physics, admission, or
actuation. Core semantics belong in Rust tests. Python scenarios should verify
the public protocol and complete process topology rather than reimplement those
semantics.

## Code Map

| Path | Responsibility |
| --- | --- |
| `crates/soma-core` | Fixed Reachy state/target types, Plant boundary, command admission, expiry, and hold behavior |
| `crates/soma-sim` | Pinned MuJoCo Reachy Plant |
| `crates/soma-protocol` and `proto/` | Minimal command, state, and reset wire schema |
| `crates/soma-runtime` | `robot-rt`, `robot-runtime`, loopback Zenoh, and local datagram boundary |
| `crates/soma-probe` | Read-only Reachy Mini Lite N0 audit |
| `python/soma_client` | Thin integration-scenario client |

## Contracts And Proof Boundaries

- `robot-rt` owns the control loop; Python, Zenoh, async work, and blocking I/O
  stay outside its periodic path.
- Commands are admitted by Plant timeline, runtime generation, increasing
  sequence, and local TTL. Malformed ingress and RT validation failures become
  explicit rejection evidence; they never silently replace the active target.
  Expiry transitions to the latest measured-position hold.
- `Plant::apply_positions` reports whether application is merely submitted or
  synchronously confirmed by the Plant. MuJoCo control writes are submitted;
  later measured state remains the motion evidence. State also labels its source
  timestamp domain (`simulation`, host monotonic, or device) separately from
  the runtime capture timestamp.
- Simulation reset changes the Plant timeline, preventing old commands from
  crossing the reset.
- Each `robot-runtime` start announces an opaque runtime generation. The RT
  owner drops active authority and sequence admission on a generation change;
  targets from an older generation are rejected until a current-generation
  target is admitted. The transition is published in state evidence.
- The current local datagram IPC is a bootstrap mechanism. Bounded shared
  memory remains a target architecture decision that requires measurement and
  an implementation trigger.
- Native hardware work is read-only until N0 passes, and torque-enabled motion
  additionally requires explicit human N1 authorization.

## Approved Open Duck Qualification

Open Duck Mini is an implemented, fixed simulation-only qualification profile.
Its 50 Hz ONNX policy keeps inference outside a 500 Hz Duck periodic RT/physics
path:

```text
Duck state -> Rust ONNX policy (default) -----> loopback Zenoh -> robot-runtime
           -> Python ONNX policy (oracle) ----^                    |
                                                                  v
                                  Duck Plant <- robot-rt <- bounded IPC
```

This path uses asynchronous latest-value semantics. A captured state is sent
to inference immediately; the resulting target is applied on the first
available RT tick and held with a bounded zero-order hold until replaced or
expired. There is no artificial one-policy-frame delay. Capture-to-application
age, source lineage, deadline consumption, and stale rejection are evidence,
not assumptions about lockstep scheduling.

The 2 ms Duck RT tick advances one MuJoCo physics step and emits a policy frame
every tenth tick. Reachy's existing 20 ms RT tick and ten-substep Plant advance
do not change. Running Duck with Reachy's 20 ms batching would force each
policy response into the next policy frame and is therefore rejected.

The process split provides ownership and failure-isolation boundaries, but is
not inherently a safety proof. After the Duck path produces timing and failure
evidence, D-04 requires a focused comparison with Rust-hosted ONNX inference
and a single Rust process that isolates async work from its periodic RT thread.
No alternative may move async work, middleware, inference, or unbounded
allocation into the periodic section.

The fixed policy ABI is observation `[1, 101]` to action `[1, 14]`, with the
named 14-actuator order and default pose defined by `soma_runtime::open_duck`.
It uses a 27-tick phase, three-action history, 0.25 action scale, 5.24 rad/s
slew limit, 40 ms target TTL, and checkpoint SHA-256
`cb61453a8bcb547ccfdeb4f03ba0fa67ebcf767dcf4aa6e5c9a0d92b302f9b23`.
Python validates this contract as the reference/oracle path. Rust now provides
the equivalent fixed observation and target adapter outside RT; ONNX Runtime
native provisioning remains an explicit deployment prerequisite and is not
loaded by `robot-rt`. The Rust worker reports ready only after its target
publisher has a matching runtime subscriber, so startup discovery cannot age
early state-derived targets past their TTL. Rejection reasons and rejected
target age remain attributable in the fixed-profile metrics evidence.

The Pages showcase reuses the pinned direct reference runner for one 12-second
composite `vx`/`vy`/yaw command reel. It records complete generalized state and
uses the pinned MJCF loader to reconstruct read-only Mesh3D geometry and body
transforms on the same Rerun timeline as policy evidence. This does not add a
second Plant authority or change the frozen straight-walk acceptance case.

## Deeper Documents

- [Reference architecture](docs/architecture/reference-architecture.md) owns
  the long-term system thesis and marks unimplemented target surfaces.
- [Layering and trust boundaries](docs/architecture/layering-and-trust-boundaries.md)
  owns the `L-2` through `L5` and `SA-0` through `SA-5` vocabulary.
- [Security threat model](docs/architecture/security-threat-model.md) owns the
  qualified-deployment invariants and the current local-development exception.
- [Glossary](docs/glossary.md) owns stable cross-module terminology.
- [Policy/runtime interface](docs/deep-research/policy-runtime-interface.md)
  compares reference scheduling choices and owns the low-rate policy timing
  rationale.
