# Glossary

This glossary keeps Soma terminology precise as the architecture evolves.

## Embodiment

The physical form of a robot: for example a wheeled base, mobile manipulator, quadruped, or humanoid.

## HAL

Hardware Abstraction Layer. The layer that hides real device and transport details such as EtherCAT, CAN-FD, servo drives, IMUs, and power electronics from higher-level robot software.

## Plant

The controlled physical or simulated system seen by the real-time controller. A Plant may be backed by real hardware, MuJoCo, Isaac Sim, Genesis, or another simulator.

## Plant Interface

The control-facing contract shared by real and simulated Plants. It exposes state, commands, timing context, lifecycle, and health without leaking bus- or simulator-specific implementation details.

## Simulation Control Interface

A simulator-only contract for operations such as reset, step, pause, snapshot/restore, randomization, ground truth, and fault injection.

## robot-rt

The real-time Soma process responsible for deterministic scheduling, Plant I/O, control integration, command validation, software safety, and watchdogs.

## robot-runtime

The non-real-time Soma process responsible for the public protocol, leases, actions, capability discovery, diagnostics, recording, policy lifecycle, communication, and operations integration.

## Robot Protocol

The ROS-independent public contract used by SDKs, adapters, tools, and external systems. It includes state streams, command streams, RPCs, actions, events, leases, capabilities, and version negotiation.

## Lease

An explicit control-ownership token for a robot capability or control domain. Leases prevent multiple control sources from implicitly competing for authority.

## Epoch

A unique identifier for a continuous robot or simulation timeline. Reset, restart, or equivalent discontinuities create a new epoch so stale commands from previous timelines can be rejected.

## RobotManifest

The canonical description of a robot product/variant, including identity, joints, actuators, sensors, frames, transmissions, limits, calibration schema, optional modules, and supported control modes.

## PolicyBundle

A deployable policy artifact containing a model plus its observation/action contracts, normalization, rates, compatibility metadata, checksums, and signatures.

## SIL

Software-in-the-loop. Production software components run against a simulated Plant without real robot electronics.

## HIL

Hardware-in-the-loop. Real compute, buses, controllers, or electronics participate in a test setup while other parts of the robot are simulated or emulated.

## Flight Recorder

A bounded local blackbox that continuously records recent high-frequency robot state, commands, events, and metadata, freezing relevant windows when faults or incidents occur.

## ReleaseManifest

A versioned description of a tested robot software release and the compatible combination of OS, runtime, firmware, robot model, policies, and schemas.

## ROS 2 Adapter

An integration layer that maps Soma Robot Protocol semantics to ROS 2 topics, services, actions, TF, Nav2, MoveIt, ros2_control, and related ecosystems. ROS 2 is intentionally outside the Soma core runtime.
