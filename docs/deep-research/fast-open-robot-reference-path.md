# Fast Open-Robot Reference Path

> Status: Focused deep research, 2026-08-21. This note selects a concrete
> reference robot for the first Soma slice and separates immediate evidence
> from later physical-hardware claims.

## Question

Can Soma's reduced Python -> runtime -> RT -> MuJoCo path produce a compelling
result quickly, without training a policy or importing a large robotics
framework? Which open robot gives that path a credible continuation to real
hardware, and what would block it?

## Recommendation

Make Reachy Mini the only active implementation target, with exactly two
profiles:

1. **`ReachySimPlant`:** the pinned official Reachy MJCF in MuJoCo, driven by
   Soma's own ControlCore and runtime boundary.
2. **`ReachyNativePlant`:** Reachy Mini Lite driven through a Soma-owned Rust
   Dynamixel adapter, with the official daemon and official motor-controller
   wheel absent from the process.

Run the official Reachy daemon and its MuJoCo backend only as a comparison
configuration. BHL, Open Duck Mini, Atom S, SO-101 and the Playground policy
remain evidence from the earlier candidate study, not implementation scope.

## What Was Proven Locally

The following smoke tests ran on the current Linux development machine with
MuJoCo 3.6.0, ONNX Runtime 1.24.4, NumPy 2.4.0, and CPU inference.

### Verified fast asset path

MuJoCo Playground contains ready-to-run policies under
`mujoco_playground/experimental/sim2sim/onnx/`, including Berkeley Humanoid,
Go1, G1, Booster T1, Apollo, and LEAP Hand policies. The Berkeley Humanoid
policy and Menagerie model ran through a minimal headless runner for five
simulated seconds with a 0.5 m/s forward command:

```text
model nq=19 nu=12 sensors=17 assets=13
5s displacement x=2.115 y=0.059 final_z=0.533
finite=True inference_input=52 output=12
```

This is the fastest visible result. It requires MuJoCo, NumPy, ONNX Runtime,
and the pinned model assets; `hidapi` is needed only by the upstream joystick
script and can be omitted when Soma supplies commands.

The model is Menagerie's older `berkeley_humanoid`, not the exact Berkeley
Humanoid Lite physical design. It proves the execution path and gives a useful
demo, but must not be described as a Lite digital twin.

### Exact BHL asset and policy probe

The BHL biped MJCF and `policy_biped_50hz.onnx` also loaded and ran on CPU after
repairing two upstream asset paths. Its published policy contract is 45 inputs
and 12 outputs. A five-second 0.5 m/s rollout produced:

```text
model nq=19 nu=12 nsensordata=52
5s displacement x=2.066 y=-0.751 final_z=-0.053 min_z=-0.086
finite=True input=45 output=12 max_torque=5.0
projected_gravity=[0.120, 0.013, -0.993]
```

The final projected gravity indicates the base remained approximately upright;
the substantial lateral/yaw drift means this is an integration smoke test, not
a locomotion-quality or physical-transfer result.

The current upstream workspace is not clone-and-run at the pinned commits:

- `MujocoEnv` uses an older `source/.../data/mjcf/` path while the asset
  submodule now stores models under `data/robots/.../mjcf/`;
- the generated MJCF declares `meshdir="assets"` and `merged/*.stl`, while the
  checked-in meshes are under the sibling `meshes/` directory.

These are small deterministic repairs, but they are a reason to vendor or fetch
a pinned, tested asset slice rather than depend on BHL's full workspace.

### Native Rust MuJoCo probe

`mujoco-rs 5.0.0+mj-3.9.0` compiled with Rust 1.97.1, loaded a model, and
stepped it successfully. A simulator sidecar is therefore not required for the
first architecture.

Its `auto-download-mujoco` feature downloads the native library for the build,
but the resulting binary still needs the MuJoCo library directory available to
the dynamic loader. The viewer also has main-thread/rendering constraints.
Treat both as packaging and topology details: keep visualization outside the
cyclic control thread and make headless execution the test path.

## Why Berkeley Humanoid Lite

BHL is unusually well aligned with the question Soma needs to answer:

| Property | Evidence | Value to Soma |
| --- | --- | --- |
| Bounded embodiment | 12-DoF biped profile | Small fixed-layout messages and one joint order |
| Native simulation | Exact MJCF and URDF assets | No simulator abstraction work before motion |
| No training prerequisite | Published ONNX checkpoints | Visible policy-driven result on CPU |
| Real rate separation | 50 Hz policy, 250 Hz low-level control, 2000 Hz simulation | Exercises runtime/RT/Plant roles concretely |
| Real hardware continuation | Open low-level CAN repository and UDP policy split | The same observation/action semantics can later cross into hardware |
| Open project | MIT code; non-code assets documented as CC BY-SA 4.0 | Usable with explicit attribution/share-alike handling |

The exact biped policy input is:

```text
velocity command       3
base angular velocity  3
projected gravity      3
joint position error  12
joint velocity        12
previous policy action 12
                       --
                       45
```

The policy emits 12 normalized actions. The published runner scales them by
`0.25`, adds default joint positions, then the low-level layer applies joint PD
and torque limits. Soma should expose typed robot state and a typed
`JointPositionTarget` in radians. It should not expose an opaque 45-element
policy vector as its public robot protocol.

## Actual Blockers And Required Decisions

There is no hardware, training, GPU, ROS 2, or Isaac blocker for the software
happy path. The remaining blockers are narrow contract gaps.

### 1. State is currently too narrow

Joint position alone cannot drive the published BHL policy. The first profile
must carry:

- base orientation quaternion or an equivalently typed projected-gravity input;
- base angular velocity;
- joint positions and velocities in the pinned 12-joint order.

Previous action is policy-runner state, not robot state. Velocity command is a
client/policy input, not a Plant sensor.

### 2. Command semantics must be frozen

For this profile, the cyclic command is a 12-element joint-position target in
radians. `ControlCore` owns PD gains, torque clipping, TTL handling, and the
applied-command result. Do not add torque, velocity, impedance, or generic
command unions until a real need appears.

### 3. Rates must be profile values

Use the published BHL values initially:

```text
policy       50 Hz   (0.020 s)
low-level   250 Hz   (0.004 s)
simulation 2000 Hz   (0.0005 s)
```

These are not universal Soma requirements. In simulation the 250 Hz cycle may
advance four physics substeps. Policy inference remains outside the cyclic Plant
step.

### 4. Timeout behavior is embodiment-specific

Holding the last walking target indefinitely is unsafe. The real BHL low-level
code transitions through passive damping and then idle on stop. The simulation
milestone should implement and visibly report passive damping as its one
fallback, without claiming it will recover balance or make physical hardware
safe.

### 5. Visualization must be explicit but non-authoritative

The current acceptance tests can pass headlessly while showing no result to a
human. Add an optional viewer that consumes snapshots away from the cyclic
thread. The viewer is a demo surface, never a control dependency or test oracle.

### 6. Pin one tested asset set

Pin the BHL workspace commit, asset submodule commit, low-level submodule
commit, `policy_biped_50hz.onnx`, config values, and joint order together. A
small checked-in lock note or constants module is enough; a general model
manifest is still deferred.

### 7. Real hardware has additional gates

Physical BHL work later requires Linux CAN tooling, the correct CAN interface
and actuator configuration, calibration after every power cycle, controlled
bring-up, an independent stop path, and physical safety review. Its actuators
use a single motor-shaft encoder, so absolute joint zero is not retained across
power cycles. None of this should block or be simulated into the first SIL
milestone, and successful MuJoCo motion is not evidence that these gates pass.

## Historical Candidate Comparison (Superseded)

This table preserves the selection evidence that led to Reachy. Its `Decision`
column is historical and does not define current implementation scope.

| Candidate | Fast visible result | Exact open hardware path | Dependency/risk | Decision |
| --- | --- | --- | --- | --- |
| Playground Berkeley Humanoid | Best: bundled CPU ONNX policy was verified locally | No; model predates exact Lite design | Small sim2sim slice, but not a physical twin | Day-zero demo only |
| Berkeley Humanoid Lite biped | Good after two asset-path repairs | Best: exact assets, policies, CAN low-level | Full workspace pulls Isaac/Torch/CUDA; asset license needs care | Primary Soma profile |
| Reachy Mini Lite | Excellent: official MuJoCo backend and the same Python SDK as hardware | Strong for expressive head/body/antenna motion, camera, and audio | 50 Hz managed position control does not exercise dynamic locomotion; hardware CAD is non-commercial | Best quick physical interaction candidate |
| FashionStar Atom S | Immediate on hardware via bundled poses/WebBLE; no verified simulator | UART protocol and servo SDKs are documented | No official URDF/MJCF found; Atom SDK page is empty and open-platform page is unfinished | Cheap native-bus experiment, not current primary |
| SO-101 | Easy model viewing and position control | Strong and inexpensive via LeRobot/Feetech | Gripper normalization differs between hardware and MJCF | First-hardware fallback |
| ToddlerBot | Bundled replay motions give an immediate demo | Strong project with MuJoCo and deployment code | Large dependency surface; CAD is CC BY-NC-SA 4.0 | Secondary showcase only |
| Open Duck Mini | Exact MuJoCo model and published ONNX walking policies; sim2real path documented | Strongest bottom-up path: printable body, Feetech bus, Raspberry Pi runtime, IMU and foot sensing | Not a turnkey product; printing/assembly/calibration required; runtime and Playground repos lack detected license metadata; docs still say unfinished | Best physical dynamic-control reference if we accept building it |

### Reachy Mini changes the hardware recommendation

Reachy Mini is more open and more complete than the initial candidate pass
captured. Its main SDK and daemon are Apache-2.0 and include:

- an official MuJoCo model/backend and documented `reachy-mini-daemon --sim`;
- one SDK against simulation and both Lite and Wireless hardware;
- a 50 Hz robot backend with immediate high-frequency targets;
- exact URDF/MJCF and mesh assets in the SDK repository;
- camera, microphone, speaker, 6-DoF Stewart-platform head, body yaw, and two
  antenna joints;
- a public Rust motor-controller implementation for its nine Dynamixel motors.

A direct model probe at SDK commit
`20bc9eedc81ddc552235d222ca7e39205b2c2481` loaded and stepped successfully in
the local MuJoCo installation (`nq=37`, `nv=30`, `nu=9`). This makes Reachy Mini
the strongest candidate for a quick *physical, visible, interactive* result.

There are two caveats. First, the hardware design is CC BY-NC-SA rather than a
commercially permissive license. Second, the separate public motor-controller
repository has no detected license file or GitHub license metadata at commit
`dff1c536a75735a950564e18240d6a67b056819c`; its reuse terms need clarification
even though the main SDK which depends on it is Apache-2.0.

The earlier candidate study favored treating Reachy's daemon as a managed
motion gateway because that was the quickest demo. That recommendation is now
superseded: the active plan uses only a direct MuJoCo Plant and a native Lite
Plant. The official daemon is retained solely as a simulation/hardware
comparison oracle.

### Bottom-up Reachy assessment

Reachy permits a meaningful bottom-up host stack, but not ownership of every
layer in the actuator. The practical ownership boundary is:

| Layer | Soma can own? | Reachy-specific constraint |
| --- | --- | --- |
| Client protocol, command TTL, scheduling, runtime | Yes | None beyond the chosen profile contract |
| `ControlCore` limits, interpolation, timeout and state publication | Yes | The first profile should remain position-oriented |
| Stewart kinematics and joint-to-motor mapping | Yes | Six motors drive a platform with six passive ball joints; it is not six independent Cartesian joints |
| Dynamixel TTL transport and register access | Yes, technically | The bus is standard Dynamixel Protocol 2.0 at 1 Mbaud; Soma needs exclusive serial-port ownership |
| Smart-servo position/current loop | No | The XC330/XL330 firmware and internal safety limits remain vendor-controlled |
| Power electronics, wiring and physical stop behavior | Not fully | These are properties of the purchased hardware and must be validated separately |

The public motor controller confirms that this is not merely a REST wrapper. It
does synchronous reads/writes for all nine IDs, exposes torque enable, Stewart
current targets, operating modes, raw register reads/writes, voltage/error checks,
retries, and a timestamped position cache. Its loop period is configurable, but
the official backend uses 50 Hz. That makes it a useful implementation
reference for the private Reachy bus worker. One embodiment does not justify a
public `ActuatorBus`/`ActuatorDevice` hierarchy, making Dynamixel the universal
actuator API, or claiming hard-real-time qualification. The separately
published motor-controller repository
also has no detected license metadata, so Soma should initially implement an
adapter against the protocol rather than copy the code until reuse terms are
clarified.

The Rust repository is first-party in the practical product sense: it is under
the `pollen-robotics` GitHub organization, its README says it is used by the
Reachy Mini project, and the main SDK declares the released
`reachy_mini_motor_controller` wheel as a dependency. It is a companion native
extension, not firmware running inside a motor and not evidence that the
controller board is user-reprogrammable.

The source does not state a single explicit reason for choosing Rust. The
implementation shows the likely engineering reasons: the serial port is owned
by one native controller, synchronous Dynamixel I/O is kept out of Python's
GIL, commands are accepted through a bounded Tokio channel, and the wheel is
built for Linux ARM/x86, macOS and Windows. The Rust layer also centralizes
retries, port errors, motor discovery, voltage ramp-up and shutdown. This is a
good reliability and packaging boundary for a Python SDK. It is not a
hard-real-time kernel: the loop uses Tokio timing, a normal OS thread and a
configurable 50-Hz read cycle, while the Python backend also runs a 50-Hz
update loop.

The active implementation has exactly two Soma Plants:

1. `ReachySimPlant`: Soma drives the pinned Reachy MJCF directly.
2. `ReachyNativePlant`: a Soma-owned worker opens the USB serial port,
   performs the same position/state exchange directly, and runs the same
   profile-level limits and timeout policy. The official daemon must be stopped;
   both controllers cannot safely contend for one Dynamixel bus.

The official daemon is run separately against both MuJoCo and Lite. This makes
the comparison measurable without turning the vendor gateway into a third Soma
implementation profile.

### Sensors and transferability

Reachy is representative of serial smart-servo interaction robots: device IDs,
sync reads/writes, position targets, telemetry, retries, and a roughly 50--100
Hz application loop. It is not representative of high-torque BLDC modules,
CAN-FD/EtherCAT drives, torque-controlled humanoid legs, or whole-body dynamic
locomotion. The portable Soma concept is therefore the actuator and state
contract, not the Dynamixel API.

The media design should influence Soma without expanding the first RT mailbox:

- actuator positions, command acknowledgement, and timestamps belong in the typed
  runtime/control stream;
- camera frames belong in a separate bulk sensor stream carrying sensor ID,
  frame ID, capture timestamp, format and calibration metadata (or a shared
  memory handle), never pixel payloads in the cyclic mailbox;
- microphone/audio is another asynchronous media stream;
- IMU is an optional typed sensor stream. Reachy Wireless reads a BMI088 at the
  50 Hz backend loop, while Lite and MuJoCo report no IMU, so an IMU must not be
  a mandatory Reachy profile field.

The resulting active scope is Reachy-only: direct simulation first, native Lite
second, and the official stack as a comparison. BHL and all other robots remain
historical candidate evidence. Defer a general sensor framework, ROS 2,
multi-robot manifests, and generalized torque interfaces until a concrete
post-Reachy need exists.

### Soma Rust controller versus Reachy Rust controller

The two Rust efforts overlap at the device boundary but have different jobs:

| Responsibility | Reachy motor-controller crate | Soma implementation for Reachy |
| --- | --- | --- |
| Public API | PyO3 methods used by the Reachy daemon | Protobuf/Zenoh client and runtime API |
| Process role | Native device-I/O component inside the product daemon | `robot-runtime` plus a separate `robot-rt` process |
| Hardware scope | Fixed IDs and register mappings for 9 Reachy motors | A Reachy-specific `HardwarePlant`/HAL adapter behind a stable Plant contract |
| Cyclic work | Read positions and execute queued motor commands | ControlCore, state validity, limits, timeout, safe behavior, and Plant interaction |
| Kinematics | Not the main responsibility; Python backend performs head kinematics/IK | Owned by the selected Reachy profile or adapter, with explicit Plant semantics |
| Safety authority | Device discovery, voltage/error checks, retries and torque state | Soma software safety and command authority; physical lower safety remains external |
| Simulation reuse | Separate MuJoCo backend in the Reachy daemon | The same Soma ControlCore should run against MuJoCo and native hardware Plant |

The official path is effectively:

```text
Reachy SDK -> Python daemon/kinematics -> Rust motor loop -> Dynamixel bus
```

The intended native Soma path is:

```text
Python client -> robot-runtime -> bounded mailbox -> robot-rt/ControlCore
    -> Reachy HardwarePlant -> Rust Dynamixel adapter -> bus
```

The current Soma bootstrap plan does not yet implement this line; the plan now
replaces the former BHL bootstrap target rather than claiming an existing
hardware path is reusable.

There are two implementation milestones and one external comparison:

1. `ReachySimPlant`: load the checked-in Reachy MJCF and exercise Soma
   ControlCore without any vendor daemon.
2. `ReachyNativePlant`: stop the official daemon and let a Soma-owned Rust
   adapter own the serial port. This is the actual bottom-up test.

The official daemon is run separately with MuJoCo and Lite to provide the
comparison baseline.

The native milestone needs one architecture qualification. Soma's `robot-rt`
contract forbids blocking I/O and unbounded timing in its periodic path, while
the Reachy controller uses serial reads with a 10 ms port timeout and Tokio
timers. A direct call from `robot-rt` to that driver would weaken the stated
boundary. The conservative design is a dedicated L0 bus/I/O worker with
bounded mailboxes and measured command/state age; only after measurement should
we decide whether the Reachy 50 Hz profile can share the RT process. This is a
real difference from the official product stack, not an implementation detail
to hide behind the Rust language choice.

### Reachy Mini Lite versus Open Duck Mini

These robots should not be treated as interchangeable candidates. Reachy is a
commercially purchasable, expressive head robot with a maintained daemon, SDK,
MuJoCo backend, camera/audio stack, and a public motor-controller layer. Open
Duck Mini v2 is a roughly 42 cm, 15-actuator biped whose repository publishes
the CAD/print assets, URDF/MJCF, two ONNX walking policies, a MuJoCo Playground
environment, a Raspberry Pi Zero 2 W runtime, and instructions for Feetech
motor identification and sim-to-real transfer. Its stated BOM target is under
USD 400, but it is a build project rather than a boxed robot.

| Decision criterion | Reachy Mini Lite | Open Duck Mini v2 |
| --- | --- | --- |
| First visible result | Best: run the official daemon in simulation, then plug in Lite | Good in simulation; physical result requires printing, assembly, wiring and calibration |
| Bottom-up ownership | Host stack to Dynamixel bus; vendor servo firmware remains opaque | Host stack, direct Feetech bus, onboard runtime and policy loop are all replaceable; servo firmware still vendor-controlled |
| Dynamic-control relevance | Low: Stewart head/body yaw, position-oriented, official loop 50 Hz | High: biped locomotion, 15 actuators, IMU/foot observations, 50 Hz policy-to-servo loop and motor identification |
| Sensor architecture | Camera, microphone, speaker; IMU only on Wireless, not Lite | BNO055 IMU and foot sensing are part of the walking runtime; camera/audio are optional unfinished expression features |
| Simulation evidence | Maintained official MuJoCo model and same SDK as hardware | Exact robot MJCF/URDF plus ONNX policies and a documented sim2real workflow, but some docs are explicitly outdated or unfinished |
| Hardware availability | Buy a finished Lite | Buy parts / print and assemble; no standard finished unit was verified |
| License clarity | SDK Apache-2.0; hardware CC BY-NC-SA; motor-controller reuse terms unresolved | Main repo Apache-2.0; runtime and Playground repositories have no detected license metadata; CAD/parts provenance should be checked before redistribution |
| Main risk | It validates product integration more than Soma's dynamic RT boundary | Build and calibration time, plus less mature/documented runtime |

This comparison explains the earlier alternatives but no longer proposes an
implementation sequence. Open Duck's locomotion scope and BHL's dynamic-control
scope are both excluded by D-22; neither should add interfaces, assets, tests,
or acceptance criteria to the Reachy bootstrap.

### Atom S is component-open, but not yet stack-complete

FashionStar sells Atom S as a CNY 1,699, 10-DoF, 3D-printed humanoid using ten
RA8-U25H-M UART bus servos and a Seeed XIAO ESP32-S3. Its official documentation
publishes the UART packet protocol, joint IDs, angle read/write, synchronized
commands, damping, stop, telemetry, Python and embedded servo SDKs, and links to
STP/STL downloads. These are enough to write a native Soma hardware Plant.

However, the current official documentation does not support treating Atom S
as a ready open-source robot stack:

- the Atom SDK page has no content;
- the page for other open-source platforms says `To be continued`;
- no official Atom S URDF, MJCF, simulator, locomotion policy, or host robot
  SDK was found in the documented resources;
- the linked `servodevelop/wiki` GitHub repository was unavailable during this
  research pass, and the CAD download's explicit reuse license was not visible
  on the documentation page;
- the default experience is an ESP32/Web Bluetooth action player and pose
  editor, not a feedback-driven walking controller.

The vendor calls the kit “completely open source,” but the evidence presently
supports a narrower claim: its mechanics are downloadable and its actuator bus
is well documented. Atom S is viable when the purpose is to prove Soma can own
a UART servo loop cheaply. It is not the shortest sim-to-real route because
Soma would first have to construct and validate the model, joint convention,
calibration, state loop, and safe fallback.

SO-101 is similarly simple to place on a desk and actuate, but it mostly
validates position I/O and calibration. BHL better validates the process
boundary and multi-rate dynamic control that motivated Soma. Reachy Mini better
validates a quick interactive physical product. These comparisons no longer
select the active robot; Reachy Mini is now fixed by D-22.

## Active Execution Sequence

The canonical sequence is maintained in
[`bootstrap-plan.md`](../plans/bootstrap-plan.md). In short: run the read-only
native N0 probe when hardware is available; implement the direct Reachy MuJoCo
Plant and Soma process path; approve N1 before torque enable; bring up the
private Reachy bus worker in staged motion; then compare Soma and the official
stack in both simulation and Lite hardware.

## Explicit Non-Work

Do not implement another robot, a managed vendor Plant, a public generic
actuator hierarchy, head-pose/IK commands, media streaming, domain
randomization, training, hardware CAN, ROS 2, policy registries, or a general
asset manifest in this slice.

## Sources

Primary sources were inspected at the commits listed below on 2026-08-20.

- [MuJoCo Playground](https://github.com/google-deepmind/mujoco_playground),
  commit `e74217bb89c77a74ba02e4789263991864375799`; see its
  [sim2sim directory](https://github.com/google-deepmind/mujoco_playground/tree/main/mujoco_playground/experimental/sim2sim)
  and bundled ONNX policies. Apache-2.0.
- [MuJoCo Menagerie Berkeley Humanoid](https://github.com/google-deepmind/mujoco_menagerie/tree/main/berkeley_humanoid),
  the model used by the Playground day-zero probe. BSD-3-Clause for this model.
- [Berkeley Humanoid Lite](https://github.com/HybridRobotics/berkeley-humanoid-lite),
  commit `984741a3623c93b0583ccfdc479f1f8b1c4d900e`; see the
  [50 Hz biped config](https://github.com/HybridRobotics/berkeley-humanoid-lite/blob/main/configs/policy_biped_50hz.yaml)
  and [MuJoCo runner](https://github.com/HybridRobotics/berkeley-humanoid-lite/blob/main/source/berkeley_humanoid_lite/berkeley_humanoid_lite/environments/mujoco.py).
- [BHL Assets](https://github.com/HybridRobotics/Berkeley-Humanoid-Lite-Assets),
  submodule commit `fc90fedd008b1e56a22e3c5221548d6b24f49707`.
- [BHL Lowlevel](https://github.com/HybridRobotics/Berkeley-Humanoid-Lite-Lowlevel),
  submodule commit `652777cc7c49884e7cd7ddfada758dc1979bf627`;
  contains policy preprocessing, CAN control, calibration, and stop behavior.
- [Berkeley Humanoid Lite paper](https://arxiv.org/abs/2504.17249) and
  [project documentation](https://berkeley-humanoid-lite.gitbook.io/berkeley-humanoid-lite-docs).
- [mujoco-rs](https://github.com/davidhozic/mujoco-rs), version
  `5.0.0+mj-3.9.0`; see its [installation guide](https://mujoco-rs.readthedocs.io/en/v5.0.x/installation.html).
  MIT OR Apache-2.0.
- [SO-ARM100/SO-101](https://github.com/TheRobotStudio/SO-ARM100), commit
  `7629d2ad9853d10fb903093a33ef6114099d97e5`, and
  [LeRobot](https://github.com/huggingface/lerobot), commit
  `713a409faedd73bb5597481b8885f17fbee23330`. Apache-2.0.
- [ToddlerBot](https://github.com/hshi74/toddlerbot), commit
  `e337f3b177b4b53abff70b31d1695a7b66cc6d2e`. Software MIT; hardware/CAD
  design CC BY-NC-SA 4.0.
- [Open Duck Mini](https://github.com/apirrone/Open_Duck_Mini). Apache-2.0.
- [Reachy Mini SDK](https://github.com/pollen-robotics/reachy_mini), commit
  `20bc9eedc81ddc552235d222ca7e39205b2c2481`; see the
  [simulation guide](https://huggingface.co/docs/reachy_mini/platforms/simulation/get_started),
  [Lite hardware specification](https://huggingface.co/docs/reachy_mini/platforms/reachy_mini_lite/hardware),
  and [architecture guide](https://huggingface.co/docs/reachy_mini/SDK/core-concept).
  Main software Apache-2.0; hardware design CC BY-NC-SA.
- [Reachy Mini motor controller](https://github.com/pollen-robotics/reachy-mini-motor-controller),
  commit `dff1c536a75735a950564e18240d6a67b056819c`. Public Rust source; no
  repository license was detected during this pass.
- [FashionStar Atom S specification](https://fashionstar.com.hk/wiki/zh/humanoid/atom-s/datasheet-atom-s/),
  [quick start](https://fashionstar.com.hk/wiki/zh/humanoid/atom-s/quick-start-atom-s/),
  [open-resource status](https://fashionstar.com.hk/wiki/zh/humanoid/atom-s/open-resource-atom-s/),
  and [Atom SDK status](https://fashionstar.com.hk/wiki/zh/sdk/humanoid/atom-sdk/).
- [FashionStar UART/RS-485 protocol](https://fashionstar.com.hk/wiki/zh/uart-servo/protocols/uart-rs485-protocol/)
  and [Python servo SDK guide](https://fashionstar.com.hk/wiki/zh/sdk/servo/python-sdk/).

## Remaining Uncertainty

- The exact BHL rollout should be captured visually after the asset repairs and
  checked against upstream intended behavior; the numerical probe alone does
  not establish gait quality.
- BHL asset/checkpoint redistribution must preserve the applicable MIT and CC
  BY-SA 4.0 notices; legal review is needed before distributing a product
  bundle.
- The final `mujoco-rs` version and native-library packaging should be pinned
  only when the Rust workspace exists; the successful probe establishes
  feasibility, not a permanent dependency choice.
- Physical BHL readiness cannot be inferred until an actual bill of materials,
  controller computer, CAN adapter, actuator firmware, and safe test setup are
  selected.
- Reachy Mini motor-controller reuse terms and Atom S CAD/firmware licenses need
  explicit confirmation before redistribution or commercial use.
- Atom S has not been physically probed; its documented 5-10 ms UART command
  spacing, synchronized write behavior, feedback throughput, and fault handling
  need measurement on a purchased unit before choosing a control-cycle budget.
