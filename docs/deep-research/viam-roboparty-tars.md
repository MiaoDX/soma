# Viam, RoboParty, And TARS-AI

Research date: 2026-08-31

Scope: review the public organizations requested for Soma's reference catalog,
select projects related to robot runtime, SDK, simulation, control, deployment,
and edge/cloud AI, and record the architectural lessons and non-goals.

## Executive Summary

These organizations represent three distinct reference shapes:

| Organization | High-signal projects | Architectural shape | Soma relevance |
| --- | --- | --- | --- |
| [Viam Robotics](https://github.com/viamrobotics) | `rdk`, `api`, `viam-rust-sdk`, `micro-rdk`, `dynamixel`, `visualization` | General robot server with typed resources/modules, protobuf/gRPC, multi-language SDKs, cloud connectivity and embedded variant | L2/L4 resource model, module seam, public protocol, remote operations; not a periodic control-core template |
| [RoboParty](https://github.com/Roboparty) | `UFO`, `roboparty_train`, `roboparty_deploy`, `roboparty_firmware`, `Party_OS`, `rpo_hardware` | Open humanoid product line joining Isaac Lab training, ROS 2 deployment, CAN/CAN-FD hardware, board images and teleoperation | Policy artifact/config discipline, sim-to-real, calibration and deployment evidence; not a drop-in Soma runtime |
| [TARS-AI Community](https://github.com/TARS-AI-Community) | `TARS-AI`, `docs` | Community open hardware recreation with Raspberry Pi edge software and optional PC companion server for STT/TTS/LLM/vision | L4/L5 interaction, graceful edge/cloud split and community documentation; far outside Soma's RT scope |

The main conclusion is complementary rather than competitive. Viam is the
strongest reference for a user-facing extensible robot platform; RoboParty is
the strongest of these three for a humanoid policy-to-hardware deployment
pipeline; TARS-AI is a useful reference for an expressive, network-assisted
application robot. None justifies replacing Soma's fixed Plant/ControlCore
contract or adding generic robot manifests before a concrete need exists.

## Viam Robotics

### Projects reviewed

- [`rdk`](https://github.com/viamrobotics/rdk), current `main` snapshot
  `f54637a589479bcfd122fe0af6998a8598c13433`: Go `viam-server`, built-in
  components/services, module loading, robot client, motion/reference-frame,
  data/logging and cloud-facing server surfaces.
- [`api`](https://github.com/viamrobotics/api): protobuf/gRPC API definitions.
- [`viam-rust-sdk`](https://github.com/viamrobotics/viam-rust-sdk), snapshot
  `92180bae41a1b519243519106976dcb182530436`: alpha Rust client, tonic-based
  dialing, generated protobuf APIs and optional WebRTC transport.
- [`micro-rdk`](https://github.com/viamrobotics/micro-rdk), snapshot
  `b03697b75c7dac7d9b1c591734c7fabcb8a80654`: Rust/ESP32 server for
  resource-constrained microcontrollers, with installer and module templates.
- [`dynamixel`](https://github.com/viamrobotics/dynamixel): Go Dynamixel
  interface; useful as a device-driver reference, but old and protocol/API
  specific.
- [`visualization`](https://github.com/viamrobotics/visualization): motion and
  spatial-data visualization utilities.

### Architecture

The RDK presents a robot as a server containing named resources. A resource has
an API/type and a concrete model; custom resources are packaged as modules and
communicate with the parent server over local gRPC/Unix sockets. The same API
is exposed through Go, Python, TypeScript, C++, Flutter, Java and Rust SDKs.
Configuration and module startup/reconfiguration are runtime concerns, and
the platform also integrates cloud management, data capture and remote access.

`micro-rdk` carries the resource/module idea to an ESP32 rather than trying to
run the full Go server. This is a useful example of keeping the public resource
contract stable while changing the implementation tier.

### Lessons for Soma

- A typed resource/module seam can make L2 capability composition and SDK
  extension practical without putting plugins in `robot-rt`.
- Protobuf as a public schema and gRPC/WebRTC as adapters are a credible
  multi-language boundary; Soma should keep this at runtime/L4, not the cyclic
  Plant mailbox.
- Viam's configuration, module lifecycle and cloud data concerns belong to a
  future `robot-runtime`/operations layer if Soma needs them.
- The resource abstraction is deliberately broad and dynamic. Soma should not
  import it into the fixed profile: generic registries and runtime-discovered
  devices would weaken current static ordering and evidence guarantees.

### Caveats

The RDK repository is AGPLv3, while the Rust SDK is marked alpha and not a
hard-real-time implementation. The platform's remote/server model, resource
reconfiguration and cloud connectivity are useful design references but do not
provide evidence for bounded periodic execution, safety authority, or physical
actuation correctness.

## RoboParty

### Projects reviewed

- [`UFO`](https://github.com/Roboparty/UFO), snapshot
  `0b89378de02f77712072d983bc43ec2a46455dc0`: source-available humanoid
  learning framework. Main focuses on MJLab training, RobotState motion import,
  tracking/goal/reward inference and ONNX export; deployment is on a separate
  branch/runtime.
- [`roboparty_train`](https://github.com/Roboparty/roboparty_train), snapshot
  `92008a6317d6d0efe8b58abc2fb3c630c75992c0`: Isaac Lab 5.1/RSL-RL workspace
  for RPO locomotion, AMP, BeyondMimic, Parkour, motion retargeting and MuJoCo
  sim2sim. It uses `robolab` and `rsl_rl` submodules.
- [`roboparty_deploy`](https://github.com/Roboparty/roboparty_deploy), snapshot
  `a8a0f1557cc5d085234b8bac248a8f543342f531`: GPLv3 ROS 2 Humble deployment
  for RPO/Roboto on Orange Pi 5 Plus and D-Robotics RDK X5, with C++ drivers,
  Python SDK, inference nodes, camera/depth, CAN/CAN-FD motors and IMU.
- [`roboparty_firmware`](https://github.com/Roboparty/roboparty_firmware):
  board-image and firmware build systems.
- [`Party_OS`](https://github.com/Roboparty/Party_OS): humanoid operating-system
  image/product integration.
- [`rpo_hardware`](https://github.com/Roboparty/rpo_hardware): mechanical,
  PCB, BOM and manufacturing assets.

### Architecture

RoboParty separates training from deployment. Training is Python/Isaac Lab and
RSL-RL, with explicit robot configuration, motion retargeting and ONNX export.
Deployment is a ROS 2 workspace with independent motor, IMU, inference and
utility packages. The documented hardware path includes 500 Hz IMU input,
CAN/CAN-FD motor configuration, zero calibration, generated Python bindings,
real-time kernel options, and explicit start/stop/init/zero services.

UFO is especially clear about a policy not being portable merely because the
file is ONNX: robot XML, controlled joints, observation dimensions, action
dimensions and robot-specific goal/reward semantics must agree. New robot
bring-up remains experimental and automatic retargeting is not provided.

### Lessons for Soma

- Keep model identity, joint order, observation layout, action dimensions and
  calibration tied to one PolicyBundle; this reinforces Soma's existing
  provenance and compatibility rules.
- Separate training, inference and hardware deployment artifacts. The
  `roboparty_train`/`roboparty_deploy` split is a useful lifecycle pattern even
  though Soma currently uses a smaller MuJoCo + Python policy path.
- Explicit zeroing, motor init/deinit, generated bindings and hardware safety
  warnings are good evidence requirements for a future physical Plant.
- Sim2sim and policy export should be treated as compatibility tests, not as
  proof of physical safety or controller timing.

### Caveats

The stack is ROS 2 and Isaac Lab oriented, with large GPU/toolchain and board
dependencies. `roboparty_deploy` documents direct real-hardware control and
calibration operations; Soma's N0/N1 gate must remain stricter until a human
authorizes actuation. RoboParty's organization contains many product-specific
repositories, so these findings are about the listed high-signal projects, not
an exhaustive audit of every repository.

## TARS-AI Community

### Projects reviewed

- [`TARS-AI`](https://github.com/TARS-AI-Community/TARS-AI), snapshot
  `7593f8c8b63c35e3c07cc98665970fe55cee2c23`: community recreation of the
  TARS robot, with 3D-print files, Raspberry Pi-oriented application code,
  skills/modules, speech, music, memory, character and web surfaces.
- [`docs`](https://github.com/TARS-AI-Community/docs), snapshot
  `7a0904c6b583a12ed720e57a07b5cdc5405f2d41`: documentation hub and hardware
  v2 printing guidance.

The repository also contains `TARS-AI_Server`, a PC/edge companion that
offloads STT, TTS, LLM, vision, image generation, music generation and
embeddings from a Raspberry Pi over HTTP. The Pi remains the interaction and
device endpoint while heavier AI runs on a nearby computer.

### Lessons for Soma

- This is a clear L4/L5 pattern: keep device interaction local and make heavy
  intelligence an optional remote service with an explicit degraded mode.
- Skills/modules and a companion server are useful references for future
  application composition, but must never acquire Plant or safety authority.
- Community-facing hardware/docs and attribution are part of product quality;
  they should be tracked separately from RT and protocol qualification.

### Caveats

The project is CC BY-NC 4.0, so its hardware/software artifacts are not a
default source for commercial Soma reuse. It is not a locomotion controller,
robot safety stack, or simulator conformance reference. Its networked AI
services are intentionally outside the periodic path.

## Cross-Reference With Soma

| Soma seam | Best reference here | What to borrow | What not to copy |
| --- | --- | --- | --- |
| L0/L1 Plant and safety | RoboParty deployment/calibration evidence; Viam device examples | explicit hardware config, calibration, typed device APIs | ROS 2 or generic resources inside `robot-rt` |
| L2 runtime/modules | Viam RDK/module lifecycle | resource identity, module isolation, multi-language SDK boundary | dynamic plugin discovery in the fixed RT profile |
| L3 policy artifacts | RoboParty UFO/train/deploy | model/config/joint/observation compatibility and export metadata | assuming any ONNX checkpoint is cross-robot portable |
| L4/L5 apps and AI | TARS companion server; Viam cloud/app surfaces | optional remote AI, degraded local mode, operations and documentation | network/LLM/media dependencies in the periodic process |

The current Soma decision remains unchanged: Reachy is the primary robot
profile, and Open Duck Mini v2 is one fixed simulation qualification. These
three organizations enrich the reference catalog; they do not add a new robot
profile, generic manifest system, ROS 2 dependency, or cloud control plane.

## Sources And Method

Sources are the organizations' public repositories and README/design files,
reviewed at the snapshots listed above on 2026-08-31. We selected projects that
expose robot architecture, policy/runtime boundaries, hardware deployment,
simulation, or application/operations patterns. We did not audit every repo,
run their stacks, or infer safety/commercial claims from repository popularity.

## Open Questions

- Which Viam resource/module lifecycle ideas remain useful once Soma's leases,
  capabilities and safety authority are implemented?
- Which RoboParty policy metadata fields should be added to Soma's future
  PolicyBundle without creating a generic robot registry?
- Whether TARS-style remote AI belongs in a future Soma application repository
  rather than the robot-runtime repository.
