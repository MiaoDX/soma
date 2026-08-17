# Physical AI System Landscape

> Status: Deep Research baseline. Vendor and community capabilities change; verify current sources before using a platform claim in an ADR.

## Question

How do current robotics platforms expose hardware, control, simulation, SDK, and application layers, and what patterns are useful for Soma?

## Context

Soma is intended to sit below application intelligence and above hardware-specific implementation details. The most useful references are therefore not only robot SDKs, but complete system boundaries: what vendors expose, what they keep internal, how simulation maps to real hardware, and where ROS 2 sits.

## Canonical stack: L-2–L5

Soma uses the layer vocabulary defined in [Layering and Trust Boundaries](../architecture/layering-and-trust-boundaries.md) across architecture, implementation, platform comparison, and qualification. Safety authorities use the separate `SA-0` through `SA-5` namespace; they must not reuse these layer numbers.

| Layer | Responsibility | Representative components and interfaces |
|---|---|---|
| L-2 | independent safety and trust | independent stop, torque-inhibition, braking and energy-control path, safety MCU/relay, hardware root of trust, recovery authority |
| L-1 | board and device firmware | device BSP/RTOS, bootloader, secure boot measurement, actuator firmware and FOC, sensor firmware, BMS firmware, signed device update |
| L0 | host BSP, HAL, and Plant adaptation | host kernel/platform BSP, fieldbus masters, device discovery/drivers, hardware timestamps, calibration plumbing, hardware and simulation Plant adapters |
| L1 | deterministic control | `robot-rt`, scheduling, state estimation, joint/whole-body control loops, command validation, software safety envelope |
| L2 | robot runtime | `robot-runtime`, lifecycle, identity/authentication, ownership/leases, arbitration, protocol, diagnostics, OTA and recording coordination |
| L3 | reusable capabilities | locomotion, navigation, perception, manipulation, whole-body skills, approved policy execution |
| L4 | SDK and applications | public Python/C++ APIs, ROS 2 gateway, teleoperation, task composition, developer tools, product applications |
| L5 | fleet and cloud operations | provisioning, rollout policy, fleet identity, service observability, incident/data operations, remote administration |

The Plant contract is the control-facing boundary between L1 and the controlled system. A real Plant reaches L0 and the physical layers below it; a simulation Plant replaces that physical path without pretending that simulator RPCs are fieldbus devices. HIL may deliberately emulate L0/L-1 behavior when the driver, bus, firmware, or watchdog path is the subject under test.

The most important distinction is that an SDK may expose an L1-shaped command while the implementation, authority, and recovery path below that boundary remain closed.

## Component-level openness and assurance

"Open robot" and "low-level SDK" are too coarse to guide a bottom-up reference implementation. Openness must be recorded per component, and assurance must describe the evidence actually produced rather than inherit a label from the platform.

A platform comparison should include at least these component rows:

- independent E-stop, STO, safety controller, and power isolation;
- compute boot chain, root keys, recovery, and debug policy;
- power distribution, battery, charger, and BMS;
- actuator electronics, encoder path, and sensor electronics;
- actuator bootloader, FOC/control firmware, and device update path;
- fieldbus protocol, master implementation, device profiles, and diagnostics;
- host HAL, device graph, timestamps, and calibration access;
- RT control, runtime, public SDK, robot model, and simulator assets.

For every row, record evidence instead of a single marketing-derived score:

| Field | Question answered |
|---|---|
| Interface access | Can Soma observe, command, configure, diagnose, or only consume a high-level service? |
| Design access | Are protocol definitions, source, schematics, manufacturing files, and calibration procedures available under usable terms? |
| Build reproducibility | Can the exact running artifact be rebuilt and matched to its provenance? |
| Update authority | Can the team safely flash, roll back, and recover the component without a vendor-only tool or service? |
| Trust ownership | Who controls signing keys, identity provisioning, anti-rollback policy, recovery keys, and debug enablement? |
| Replaceability | Can the component or implementation be replaced without bypassing an undocumented safety or commissioning dependency? |
| Assurance evidence | Which claims have [HOST, SIM, SIL, HIL, or PHYSICAL evidence](../plans/bootstrap-plan.md#evidence-model), with traceable manifests, logs, and test reports? |

Each cell should distinguish `verified`, `documented`, `vendor_asserted`, `unknown`, and `not_applicable`, and attach the source artifact or test ID. These states are not collapsed into one platform score: a component can have public source but no reproducible build, an open protocol but vendor-only update authority, or physical test evidence without trust-key ownership.

Do not collapse qualification evidence, installed hardware, live health, caller authorization, and trust-key ownership into one public boolean manifest. Soma uses four related views:

- an offline `EvidenceMatrix` records sources, build/update authority, trust ownership, and qualification evidence;
- `RobotInstanceManifest` plus provisioning and safety policy record authorized component policy, while revisioned `DeviceInventory` carries L0/device read-back and attestation without exposing sensitive key material;
- `RuntimeCapabilitySnapshot` reports what is currently available or degraded, with source inventory/profile hashes, timestamp, sequence, and validity;
- request-time authorization decides whether a particular caller may exercise an otherwise available capability.

Unknown or unverified evidence cannot support an openness or assurance claim. It does not prove that a component is physically absent or non-functional. Sensitive facts such as recovery-key custody belong in access-controlled evidence and provisioning records, not an ordinary SDK capability boolean.

### Low-level commands are not lower-stack openness

A command containing fields such as:

```text
q
dq
tau
kp
kd
```

is a useful joint-control boundary, but it proves only that the caller can request a joint state or impedance-like action. It does **not** establish access to or authority over:

- actuator electronics, encoder processing, current loops, or FOC firmware;
- device bootloaders, firmware signing keys, anti-rollback, or recovery;
- bus master implementation, device state machines, or raw diagnostics;
- BMS, power sequencing, contactors, STO, or the independent E-stop path;
- calibration provenance or the exact requested-to-admitted-to-safety-output-to-applied command path, including attributable runtime and safety decisions.

Soma should therefore describe this as **privileged joint command access**, not as an open L-1/L0 stack. It is still valuable as a compatibility boundary, but it cannot validate firmware, trust-root, fieldbus, or independent-safety work.

## Patterns observed across vendors

### Unitree

Unitree is a strong example of keeping the robot SDK protocol relatively independent from ROS 2. Its modern SDK uses DDS directly and exposes low-level and high-level robot interfaces. ROS 2 interoperability is possible because ROS 2 also uses DDS-compatible infrastructure, but the robot runtime itself is not conceptually organized around ROS nodes.

The privileged joint-control model commonly exposes fields equivalent to:

```text
q
dq
tau
kp
kd
```

This maps naturally to joint impedance control and creates a stable research boundary above the servo-drive implementation. Under the component matrix above, it is not evidence that the drive firmware, boot chain, power system, or independent safety implementation is open.

Unitree's MuJoCo and Isaac-oriented tooling is also instructive: simulation aims to reproduce the externally visible robot control/state semantics rather than duplicating every piece of real hardware transport. This is a useful precedent for Soma's Plant Interface.

### Agibot / Zhiyuan

Zhiyuan shows a different pattern: products with different purposes expose different depths of control.

Research-oriented platforms expose significantly more of the low-level development chain, while application and commercial products emphasize motion, navigation, interaction, task, and diagnostic capabilities. Its software ecosystem also demonstrates plugin-style communication abstraction rather than assuming one mandatory transport for all subsystems.

This supports a Soma principle: **a single protocol family does not require every robot SKU to expose the same privilege level**. Capability discovery and authorization should define what each product exposes.

### LimX Dynamics

LimX provides a useful reference for research-focused robots because the ecosystem connects low-level control, robot descriptions, simulators, reinforcement-learning workflows, and real-hardware deployment.

The key lesson is that sim-to-real consistency depends on more than message names. Observation ordering, action semantics, control rates, robot model identifiers, and policy runtime assumptions must also be versioned.

### Booster Robotics

Booster follows a similar research-platform approach with low-level state/command interfaces and reinforcement-learning deployment flows. This reinforces the idea that policy deployment deserves its own artifact contract rather than being treated as copying a model file onto a robot.

### Deep Robotics

Deep Robotics demonstrates a multi-product approach where research platforms may expose lower-level motion SDKs while industrial products emphasize navigation, robot-server, Android, station, and management APIs. The lesson for Soma is that public SDK shape should be product-oriented, while the internal system architecture can remain common.

### Fourier and other actuator-centric ecosystems

Several vendors expose not only robot SDKs but actuator, hand, or peripheral SDKs. These are useful reminders that the lower stack has two distinct boundaries:

- a **device boundary**, where a motor controller becomes a programmable actuator;
- a **robot boundary**, where many actuators and sensors become one coherent Plant.

Soma should model both without exposing device-specific transport to the whole control stack.

## International references

### Boston Dynamics Spot

Spot is an example of a production robot SDK emphasizing ownership, authentication, command feedback, E-stop integration, payloads, and higher-level robot behavior rather than making raw actuator control the default user experience.

The strongest lesson is that **control ownership is a first-class API concept**. A production robot should not rely on multiple publishers writing to the same command topic and hoping arbitration emerges from convention.

### Franka

Franka demonstrates a clean research/industrial manipulator boundary with a well-defined real-time control API and external ROS integrations. It is a useful example of exposing low-level control without requiring ROS to be the control system itself.

### Universal Robots

UR separates several integration modes: real-time data/control, dashboard/management interfaces, ROS drivers, and controller-side extension mechanisms. This supports Soma's decision not to force one API shape to solve real-time control, application integration, and system operations simultaneously.

### PAL Robotics / Clearpath

These platforms illustrate ROS-centric application ecosystems. They are valuable for compatibility targets, but also show why Soma should isolate ROS distribution and package-version concerns at the adapter boundary rather than embedding them into the robot core.

### RBY1 / Reachy-style network SDKs

Network-first robot APIs using gRPC or similar service interfaces demonstrate that application developers can work effectively with robots without ROS being the primary public interface. This is particularly relevant for Soma's Python SDK and high-level action model.

## Adjacent open-source projects for L-1/L0 and HAL

### Open Dynamic Robot Initiative

ODRI is one of the clearest references for a substantial open actuator-control and L-1/L0 slice:

```text
motor / encoder / FOC
        ↓
servo electronics
        ↓
master board / communication
        ↓
robot driver
        ↓
controller
```

It demonstrates how much system design exists below a joint-level SDK and is therefore a useful reference for Soma's L-1/L0 hardware backend work.

### mjbots / moteus

moteus is a strong actuator-level reference. It combines BLDC servo hardware, FOC firmware, encoder handling, CAN-FD communication, and a host-side API. It shows how a complicated electromechanical subsystem can become a clean programmable actuator boundary and provides a practical target for exercising L-1/L0 ownership rather than only consuming an L1 command API.

### SOEM / IgH EtherCAT

SOEM and IgH/EtherLab represent two major styles of Linux EtherCAT integration. They are useful for understanding the gap between fieldbus implementation and a robot-level HAL. Soma should benchmark them, along with Rust alternatives, rather than selecting based solely on implementation language.

### SocketCAN / CANopen

Linux SocketCAN provides a strong transport abstraction for CAN and CAN-FD. Higher layers such as CANopen/CiA 402 can be built above it. This is a good example of keeping bus transport separate from device semantics.

## Simulation patterns

The industry exhibits at least four distinct simulation use cases:

1. **SDK-compatible robot simulation** — applications connect to a simulator using robot-like APIs.
2. **Controller-in-the-loop / SIL** — production controller code runs against a physics plant.
3. **Batch learning and synthetic data** — high-throughput direct simulator APIs are more important than network-path fidelity.
4. **HIL** — real host/device/bus software interacts with emulated or physical electronics.

A common mistake is to force these modes through one middleware and one simulator topology. Soma should instead share contracts and model identity while allowing execution topology to differ.

## What the landscape suggests for Soma

### Stable public boundary

The public SDK should expose robot concepts, not internal node names or device-specific messages.

### Tiered control exposure

Different products may expose:

- read-only low-level state;
- safe whole-robot motion;
- privileged joint control;
- internal device access.

The architecture should support all levels without making all of them public by default.

### ROS 2 at the ecosystem edge

Several successful platforms demonstrate that robot APIs can be useful without ROS being the system core. Soma should provide a strong ROS 2 bridge while preserving a smaller independent runtime.

### Simulation contract over simulator uniformity

Real hardware, MuJoCo, Isaac, and Genesis should agree on joint/frame/model/time/action semantics. They do not need identical transport or process topology.

### Production APIs need ownership and feedback

Lease, command validity, progress, cancellation, safety intervention, and explicit error semantics are as important as the command payload itself.

## Open questions

- Which level of L1 control should the first public Soma SDK expose?
- How much DDS compatibility is required for interoperability with existing robot ecosystems?
- Which vendor patterns remain valid when scaling from a single research robot to a managed fleet?
- Which stable `CapabilityCatalog` entries and adapter `InterfaceProfile` declarations are needed to restrict the common protocol for different robot classes?

## Primary references

- Unitree SDK2 — https://github.com/unitreerobotics/unitree_sdk2
- Unitree MuJoCo — https://github.com/unitreerobotics/unitree_mujoco
- AgiBot X1 host-to-DCU controller source — https://github.com/AgibotTech/agibot_x1_infer/tree/main/src/module/dcu_driver_module/xyber_controller/xyber_api
- Open Dynamic Robot Initiative — https://open-dynamic-robot-initiative.github.io/
- ODRI master board — https://github.com/open-dynamic-robot-initiative/master-board
- mjbots moteus — https://github.com/mjbots/moteus
- Berkeley Humanoid Lite — https://github.com/HybridRobotics/Berkeley-Humanoid-Lite
- Reachy 2 core — https://github.com/pollen-robotics/reachy2_core
- Poulpe actuator firmware — https://github.com/pollen-robotics/firmware_Poulpe
- Poulpe EtherCAT controller — https://github.com/pollen-robotics/poulpe_ethercat_controller
