# Open Duck Mini And MicroDuck

Research date: 2026-08-31

Scope: distinguish the two Duck hardware/software lines that Soma references,
preserve their source locations and implementation lessons, and state what is
and is not carried into Soma. This is reference research, not approval to add
Duck hardware support.

## Executive Summary

`Open Duck Mini` and `MicroDuck` are separate products from related teams:

| Line | Hardware | Primary software shape | Soma relevance |
| --- | --- | --- | --- |
| Open Duck Mini v2 | Open, printable ~42 cm BDX-inspired robot using Feetech smart servos, IMU and foot sensing; Raspberry Pi Zero 2 W runtime | Python-first experiments/runtime; MuJoCo scripts and ONNX policies | Source of Soma's fixed 14-actuator Open Duck Mini v2 simulation and checkpoint work |
| MicroDuck | Pollen Robotics Alpha Duck on Radxa Zero 3W, v2 `imu_to_dxl`, 15 Dynamixel servos | Rust product stack: `robotd`, `duck-control`, Unix JSON-RPC services, systemd, OTA, BLE, media and sensors | Later productisation reference for safety, lifecycle, deployment and service ownership |

Open Duck Mini is a hardware and sim-to-real research hub. MicroDuck is a
later product architecture for different hardware. MicroDuck's 15-servo Alpha
policy contract is not automatically compatible with Open Duck Mini v2's
14-actuator checkpoint.

## Open Duck Mini: Source And Implementation

The hardware hub is [`apirrone/Open_Duck_Mini`](https://github.com/apirrone/Open_Duck_Mini),
reviewed at commit `b23317a485b3cec7d8417f352478778b3475173c` (branch `v2`).
Its README describes a roughly 42 cm printable robot, a sub-$400 BOM target,
Onshape CAD, print/assembly guides, actuator identification with Rhoban BAM,
and a separate Raspberry Pi Zero 2 W runtime repository. The `experiments/`
tree is predominantly Python and includes MuJoCo policy playback, motor-control
experiments, Placo walks, RL training and data/observation utilities.

The training/reference environment is [`apirrone/Open_Duck_Playground`](https://github.com/apirrone/Open_Duck_Playground),
reviewed at commit `b9be205ac64488c23504ca42e5ec790337adeec3` (2025-08-05).
It is a Python `uv` project built around MuJoCo Playground/MJX. The
`open_duck_mini_v2` package contains MJCF scenes, sensors, rewards,
standing/walking tasks, joystick, ONNX export and `mujoco_infer.py`. The
reference runner decimates 2 ms MuJoCo steps to a 50 Hz policy, builds a
101-value observation, runs ONNX, applies `default_actuator + action * 0.25`,
and rate-limits targets to 5.24 rad/s.

The exact compatibility facts are pinned in
[`open-duck-stage0-provenance.md`](../status/active/open-duck-stage0-provenance.md):
the flat scene has `nq=21`, `nv=20`, `nu=14`, the checkpoint outputs 14
actions, and the input includes IMU, command, joint state, three action-history
frames, previous motor target, contacts and gait phase.

Strengths are the visible path from CAD/BOM to simulation, training, exported
checkpoint and physical build/runtime, plus a community workflow. Risks are
working-repository quality, hard-coded values, incomplete documentation, and
historical model/runner actuator and observation mismatches. Pin exact files;
branch names alone are insufficient provenance.

## MicroDuck: The Later Product Stack

The product repository is [`pollen-robotics/microduck`](https://github.com/pollen-robotics/microduck),
reviewed at `590b986bd8c0d50ae02cb3ea2f59c463b6828168` (2026-08-27). Its design
scopes v1 to one Alpha hardware configuration: Radxa Zero 3W, v2
`imu_to_dxl`, one UART, fifteen Dynamixel servos and a 50 Hz control loop.
`duck-control` is a library with no sockets or Tokio; it contains bus I/O,
observations, kinematics, ONNX policy execution and safety. `robotd` wraps it
with the loop and `robot.*` JSON-RPC API.

The product topology adds `configd`, `updaterd`, `btd`, `padd`, `mediad` and
`tofd`, plus `robotctl`. `robotd` is the only motor writer; update releases are
signed, atomically switched and health-gated with rollback. This answers
exclusive ownership, startup/shutdown, policy prerequisites, deadline health,
authorization and recovery concerns that the Open Duck experiments do not try
to solve.

## Comparison With Soma

| Concern | Open Duck Mini / Playground | MicroDuck | Soma today |
| --- | --- | --- | --- |
| Source identity | `apirrone` repos; v2 model/checkpoint lineage | `pollen-robotics/microduck`; Alpha lineage | Open Duck v2 assets pinned separately from Reachy |
| Main language | Python experiments/runtime | Rust daemon/control library | Rust control/Plant/runtime; Python external policy/client |
| Simulation authority | Python MuJoCo/MJX runner | Product hardware daemon; MuJoCo is reference support | Rust `OpenDuckSimPlant` and `ReachySimPlant` |
| Policy boundary | Same Python process as simulator; 50 Hz | ONNX inside `robotd` at 50 Hz | Open Duck inference outside the 500 Hz `robot-rt` path |
| Low-level rate | 2 ms physics, 20 ms policy | 20 ms hardware control loop | 2 ms Open Duck RT/physics, 20 ms policy transport |
| Failure semantics | Runner/script behavior | Exclusive safety writer, deadman, finite/range checks, fall/health | Timeline, sequence, TTL, measured-position hold and lineage evidence |
| Product operations | Build guides and separate runtime | OTA, rollback, systemd, BLE, media, config, recovery | Simulation launcher and evidence only |

Soma combines narrow lessons from both while keeping its proof boundary
explicit: Open Duck compatibility is an isolated simulation qualification;
MicroDuck is an operational architecture reference. Soma does not claim Duck
hardware support or MicroDuck production maturity.

## Implications For Soma

1. Keep `Open Duck Mini v2`, `Open Duck Playground`, and `MicroDuck Alpha` as
   distinct names in every provenance record and report.
2. Preserve the fixed 14-actuator/101-input Open Duck compatibility tests; do
   not import MicroDuck's 15-servo Alpha constants into that profile.
3. Borrow MicroDuck's exclusive bus writer, safety-owned I/O, finite/range
   checks, deadline health and update/recovery gates for any future hardware
   slice.
4. Do not copy MicroDuck's monolithic `robotd` topology by default. Any
   Rust-hosted policy or bus worker must preserve Soma's periodic no-blocking,
   bounded-allocation contract and be justified by measurement.
5. Add future community references as case studies with source pins,
   implementation shape, artifact caveats, compatibility facts and explicit
   Soma implications.

## Sources

- [Open Duck Mini](https://github.com/apirrone/Open_Duck_Mini), commit `b23317a485b3cec7d8417f352478778b3475173c`.
- [Open Duck Playground](https://github.com/apirrone/Open_Duck_Playground), commit `b9be205ac64488c23504ca42e5ec790337adeec3`.
- [MicroDuck architecture](https://github.com/pollen-robotics/microduck/blob/590b986bd8c0d50ae02cb3ea2f59c463b6828168/docs/design/architecture.md).
- [MicroDuck robotd design](https://github.com/pollen-robotics/microduck/blob/590b986bd8c0d50ae02cb3ea2f59c463b6828168/docs/design/robotd-design.md).
- Soma [Open Duck provenance](../status/active/open-duck-stage0-provenance.md) and [architecture](../../ARCHITECTURE.md).

## Open Questions

- Confirm artifact-level license terms for exact Open Duck meshes, checkpoints
  and runtime before redistribution beyond the directed provenance decision.
- If Duck hardware becomes active scope, verify physical actuator/IMU/sensor
  assumptions against the pinned Playground model.
- Re-run the policy boundary comparison required by the Open Duck plan before
  moving inference from Python into Rust.
