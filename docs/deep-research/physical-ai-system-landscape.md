# Physical AI System Landscape

## Question

How do current robotics platforms expose hardware, control, simulation, SDK, and application layers, and what patterns are useful for Soma?

## Context

Soma is intended to sit below application intelligence and above hardware-specific implementation details. The most useful references are therefore not only robot SDKs, but complete system boundaries: what vendors expose, what they keep internal, how simulation maps to real hardware, and where ROS 2 sits.

## A useful abstraction: L0–L4

| Layer | Meaning | Representative interfaces |
|---|---|---|
| L0 | device, fieldbus, firmware, hardware abstraction | EtherCAT/CAN-FD, motor/IMU/BMS drivers, calibration |
| L1 | joint-level control | position, velocity, torque, stiffness/damping |
| L2 | whole-robot motion | base velocity, gait, trajectory, end-effector control |
| L3 | robot capabilities | navigation, perception, manipulation, teleoperation, recording |
| L4 | application and operations | task orchestration, fleet, OTA, diagnostics, cloud integration |

The important distinction is that an SDK may expose L1 or L2 while the implementation below that boundary remains closed.

## Patterns observed across vendors

### Unitree

Unitree is a strong example of keeping the robot SDK protocol relatively independent from ROS 2. Its modern SDK uses DDS directly and exposes low-level and high-level robot interfaces. ROS 2 interoperability is possible because ROS 2 also uses DDS-compatible infrastructure, but the robot runtime itself is not conceptually organized around ROS nodes.

The low-level control model commonly exposes fields equivalent to:

```text
q
dq
tau
kp
kd
```

This maps naturally to joint impedance control and creates a stable research boundary above the servo-drive implementation.

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

Several vendors expose not only robot SDKs but actuator, hand, or peripheral SDKs. These are useful reminders that L0 has two boundaries:

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

## Adjacent open-source projects for L0/HAL

### Open Dynamic Robot Initiative

ODRI is one of the clearest references for the full lower stack:

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

It demonstrates how much system design exists below a joint-level SDK and is therefore a useful reference for Soma's hardware backend work.

### mjbots / moteus

moteus is a strong actuator-level reference. It combines BLDC servo hardware, FOC firmware, encoder handling, CAN-FD communication, and a host-side API. It shows how a complicated electromechanical subsystem can become a clean programmable actuator boundary.

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
- How should a Product Profile restrict the common Soma protocol for different robot classes?
