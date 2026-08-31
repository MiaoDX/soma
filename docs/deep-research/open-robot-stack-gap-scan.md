# Open Robot Stack Gap Scan

Research date: 2026-08-31

## Research Brief

- **Question:** Which open projects that build a robot, or a substantial robot
  software stack, have we missed in Soma's reference catalog?
- **Included:** Public projects with at least two of hardware, low-level
  control, simulation, learning, deployment, SDK, operations, or application
  layers visible in first-party repositories.
- **Excluded:** Single-purpose algorithms, vendor-only SDKs without a system
  boundary, abandoned toy repos, and projects mentioned only by search snippets.
- **Completion test:** Identify representative projects across printable
  robots, humanoids, quadrupeds, robot servers, learning/data platforms and
  application stacks; record why each matters and whether it changes Soma.

## Executive Summary

The strongest gaps in the previous catalog were not another simulator or
fieldbus. They were complete, buildable robot ecosystems:

| Project | Shape | Why it matters to Soma | Recommendation |
| --- | --- | --- | --- |
| [StanfordQuadruped](https://github.com/stanfordroboticsclub/StanfordQuadruped) | Raspberry Pi Python quadruped: joystick, gait/stance/swing controllers, IK, PWM hardware | Clear small-robot control decomposition and a concrete end-to-end DIY baseline | Add as historical/educational control reference; Pupper v1 is EOL |
| [Mini Pupper ROS](https://github.com/mangdangroboticsclub/mini_pupper_ros) | Apache-2.0 ROS 2 platform with hardware, simulation, SLAM/Nav2, vision, fleet and dance packages | One of the closest examples of a complete small quadruped product stack | Add as ROS2/product-stack case study, not as Soma RT core |
| [Poppy Humanoid](https://github.com/poppy-project/poppy-humanoid) | Long-lived 3D-printed humanoid with Dynamixel hardware, Python tooling, SD image and community docs | Important precedent for open hardware, calibration, education and lifecycle | Add as historical open-hardware platform; license and age constrain reuse |
| [BotBrain](https://github.com/botbotrobotics/BotBrain) | MIT ROS2 brain plus 3D-printable enclosure, web UI, teleop, navigation, fleet and monitoring | Strong L2/L4 operational and multi-robot reference | Add as application/operations reference; not a safety/control authority |
| [LeRobot](https://github.com/huggingface/lerobot) | Apache-2.0 Python robot-learning platform: hardware adapters, datasets, policies, teleop and evaluation | Strongest missing reference for data/policy artifact workflows and extensible hardware adapters | Add as L3/L4 learning/data reference; keep outside `robot-rt` |

Together with the existing Open Duck, MicroDuck, Reachy, RoboParty and Viam
cases, this gives Soma coverage from actuator/control loops through robot
servers, learning pipelines, fleet UX and community hardware. It does not
support adding a generic robot registry or ROS 2 dependency to the current
fixed profiles.

## Findings

### Stanford Pupper

The [StanfordQuadruped](https://github.com/stanfordroboticsclub/StanfordQuadruped)
repository is MIT licensed and currently carries an end-of-life notice for
Pupper v1 while the team prepares Pupper v3. The checked-in v1 stack is still a
valuable reference: `run_robot.py` runs a loop connecting joystick input over a
UDP socket to a controller and hardware interface. The controller separates a
gait scheduler, stance controller, swing controller and inverse kinematics;
the hardware layer maps joint targets to PWM through `pigpiod` on a Raspberry
Pi. The README documents trot/walk/rest state transitions and the
cartesian-foot-to-joint-angle flow.

This is a compact example of a Python-first control stack where the control
algorithm and hardware transport are explicit modules, but timing and safety
are not treated as hard real-time guarantees. Its UDP input path and
software-generated PWM are useful historical contrasts to Soma's typed,
bounded command path and Plant seam. It also demonstrates the value of a clear
controller diagram and build documentation even when the implementation is
small.

### MangDang Mini Pupper

[Mini Pupper ROS](https://github.com/mangdangroboticsclub/mini_pupper_ros) is an
Apache-2.0 ROS 2 Humble platform for Mini Pupper 1/2. The repository includes
bringup, hardware and description packages, simulation, controllers,
navigation/Nav2, SLAM, person tracking, recognition, fleet coordination, dance,
music and pre-built Ubuntu images. It supports Raspberry Pi-oriented robot
deployment and documents a complete ROS workspace flow.

Its architecture is application/platform oriented rather than a deterministic
control-kernel design. The value to Soma is the breadth of a small commercial
robot stack: package boundaries, simulation and hardware parity, fleet
operations, prebuilt images, and user-facing capabilities. The risk is that
ROS graph/lifecycle and package composition do not by themselves establish
exclusive actuator ownership, bounded periodic execution or a safety case.

### Poppy Humanoid

[Poppy Humanoid](https://github.com/poppy-project/poppy-humanoid) is an
open-source, 3D-printed humanoid platform maintained as a research and
education ecosystem. Its README documents 25 Robotis Dynamixel actuators,
Raspberry Pi 3/4 images, Python installation, CAD/3D-print releases and
assembly/BOM guidance. Hardware is CC BY-SA and software is GPLv3; the Poppy
name is trademarked.

Poppy is less relevant as a modern runtime than as a durable open-hardware
program: mechanical files, electronics/actuator assumptions, calibration,
community support and educational affordances all outlive individual control
implementations. It is a useful reminder that hardware provenance and
assembly/calibration documentation are product interfaces too. GPLv3 and
legacy Python dependencies make direct code reuse unattractive for Soma.

### BotBrain / BotBot Robotics

[BotBrain](https://github.com/botbotrobotics/BotBrain) is MIT licensed and
describes a modular open-source brain for legged robots. It combines a ROS 2
workspace with a web UI and a 3D-printable compute enclosure. The README lists
Unitree Go2/G1, DirectDrive Tita and custom robot support; RealSense cameras,
SLAM/Nav2, mission planning, fleet control, health dashboards, lifecycle/state
machines, priority-based velocity arbitration, dead-man and emergency-stop
flows.

This is a strong L2/L4 reference: it shows how teleoperation, navigation,
robot selection, fleet views, diagnostics and web UX can sit above ROS2 robot
drivers. It also explicitly separates velocity arbitration and dead-man
behavior from application actions. However, README-level feature claims are
not evidence of a bounded control path or independent safety authority. Soma
should borrow the separation of operator UX, missions and health, while keeping
all authoritative command admission and safety below that seam.

### Hugging Face LeRobot

[LeRobot](https://github.com/huggingface/lerobot) is Apache-2.0 and focuses on
end-to-end robot learning rather than one robot product. Its Python-native
`Robot` interface covers supported hardware such as SO-100, Koch, Reachy2,
Unitree G1 and others. It provides teleoperation, recording, training,
evaluation and visualization around a standardized `LeRobotDataset` (Parquet
state/action data plus MP4/images), with policies including ACT, diffusion,
RL and VLA models. Third-party robot, teleoperator and camera packages are
auto-discovered by package naming conventions.

LeRobot is the most important missing reference for Soma's L3 policy/data
surface. Its dataset schema and policy metadata are useful inputs to a future
Soma recording/export format, and its hardware adapter interface is a good
example of an L4 research API. It deliberately optimizes for accessibility
and breadth, not periodic safety: Python, PyTorch, video and plugin discovery
must stay outside Soma's `robot-rt`. A LeRobot adapter should submit typed
intent or policy targets through `robot-runtime`, never own Plant writes.

## Secondary Candidates And Why They Are Not Promoted Yet

- [BotBrain](https://github.com/botbotrobotics/BotBrain) is promoted above
  because it contains both hardware packaging and a broad robot operations
  surface. Its underlying ROS2 dependencies still need a source-level audit
  before any implementation comparison.
- [Poppy Project](https://github.com/poppy-project) has multiple creatures and
  a long history, but this scan only reviewed the humanoid repository. Other
  Poppy variants should be separate case studies if a concrete embodiment is
  considered.
- [AUTinyDane](https://github.com/JensLajordMunk/AUTinyDane), [Spot Micro
  variants](https://github.com/vertueux/smov), and similar DIY quadrupeds are
  useful discovery leads but do not currently show enough maintained,
  multi-layer evidence to outrank Mini Pupper, Pupper or Open Duck.
- [Stanford Pupper v3](https://github.com/stanfordroboticsclub) is a future
  follow-up, not an analyzable source at this scan's boundary; the v1 repo's
  README says its build instructions are still forthcoming.

## What This Changes For Soma

No current architecture or scope decision changes.

| Soma area | New reference lesson | Current action |
| --- | --- | --- |
| L1 control | Pupper's explicit gait/stance/swing/IK decomposition | Keep controller internals behind Plant/ControlCore; document control flow clearly |
| L2 runtime | Mini Pupper and BotBrain package/lifecycle/health surfaces | Consider only after hardware profile and safety gates; no ROS2 import now |
| L3 policy/data | LeRobot dataset, adapter and policy artifact conventions | Feed future PolicyBundle/recording design; keep inference outside periodic RT unless measured and approved |
| Hardware product | Poppy and Pupper build/calibration/community docs | Treat BOM, calibration, license and assembly provenance as first-class evidence |
| L4/L5 UX | BotBrain missions/fleet/health and Mini Pupper navigation surfaces | Possible future application repository, not `robot-rt` or fixed protocol core |

The strongest missing concept was a dedicated policy/data reference. Soma's
existing Open Duck and RoboParty research covers model compatibility and
sim-to-real; LeRobot adds dataset lifecycle, teleoperation recording and
community adapter distribution. These are complementary, not a reason to
replace Soma's current Rust ownership rules.

## Gaps And Limits

This is a bounded GitHub organization/repository scan, not an exhaustive survey
of all open robot projects. GitHub search favors popular and recently active
repositories; some projects are hidden behind personal accounts, external
documentation, submodules or hardware-only CAD sites. We inspected first-party
README/repository metadata for the promoted candidates but did not build or run
their stacks. Feature claims remain documented claims unless backed by source or
tests. License fields should be rechecked before reuse of any artifact.

## Sources

- [StanfordQuadruped](https://github.com/stanfordroboticsclub/StanfordQuadruped), current `master`, MIT, EOL notice for Pupper v1.
- [Mini Pupper ROS](https://github.com/mangdangroboticsclub/mini_pupper_ros), current `ros2-dev`, Apache-2.0.
- [Poppy Humanoid](https://github.com/poppy-project/poppy-humanoid), current `master`, hardware CC BY-SA/software GPLv3.
- [BotBrain](https://github.com/botbotrobotics/BotBrain), current `main`, MIT.
- [LeRobot](https://github.com/huggingface/lerobot), current `main`, Apache-2.0.
- Existing Soma references: [Open Duck and MicroDuck](open-duck-and-microduck.md), [RoboParty/Viam/TARS](viam-roboparty-tars.md), and [fast open-robot reference path](fast-open-robot-reference-path.md).
