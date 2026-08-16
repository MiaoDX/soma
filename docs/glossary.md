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

## robotd

The supervised on-robot deployment unit and service identity that contains `robot-rt`, `robot-runtime`, and their lifecycle supervision. It is not a requirement for one monolithic process.

## Robot Protocol

The ROS-independent public contract used by SDKs, adapters, tools, and external systems. It includes state streams, command streams, RPCs, actions, events, leases, capabilities, and version negotiation.

## Lease

An explicit control-ownership token for a robot capability or control domain. Leases prevent multiple control sources from implicitly competing for authority.

## Plant timeline identity

An opaque identifier for a continuous control or simulation timeline. A reset, restore, replay seek, `robot-rt`/Plant restart, or other discontinuous Plant-state change creates a new identity so stale commands cannot cross timelines. It is distinct from `boot_id`, `runtime_generation`, and resource-scoped `lease_generation`.

## Epoch

Deprecated shorthand for Plant timeline identity. New contracts should use `plant_timeline_id` and must not use "epoch" to mean robot boot, runtime process generation, replay segment, or lease generation.

## ProductModelManifest

The shared canonical description of a robot product/variant, including topology, joints, actuators, sensors, frames, transmissions, nominal parameters, required capabilities, and model asset references. It excludes serial-specific inventory, calibration, live health, and safety-authoritative configuration.

## RobotInstanceManifest

The provisioned identity/configuration of one physical robot, including its serial number, product-model compatibility, installed options, authorized component-set or inventory policy, and provisioning identity references. Live `DeviceInventory` revisions are compared with it but are not embedded in its hash.

## DeviceInventory

The observed installed boards, actuators, sensors, safety controllers, bus mappings, serial numbers, and hardware/firmware revisions of one robot at a particular inventory revision.

## CalibrationSet

A serial-scoped, versioned set of measured offsets, transforms, gains, uncertainty, provenance, and validity constraints. It is activated only against compatible product-model and device-inventory identities.

## ControlProfile

A separately versioned artifact containing tunable controller parameters and non-safety-authoritative operating limits. It cannot relax `SafetyProfile` or lower-authority device and safety-controller limits.

## RobotManifest

The composed runtime view exposed to SDKs and tools. It references the active product model, instance inventory, calibration, control, and safety artifacts rather than acting as another writable source of truth.

## SafetyProfile

An independently governed artifact containing safety-authoritative limits, required safety inputs/devices, derating rules, and fault-to-safe-behavior mappings. It has its own review, signature, activation, audit, and rollback policy.

## RuntimeCapabilitySnapshot

An ephemeral, timestamped view of capabilities that are currently available, degraded, unavailable, or authorization-restricted after considering installed hardware, calibration, active profiles, mode, and faults.

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
