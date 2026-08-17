# Soma Reference Architecture

> Status: Canonical architecture. This document states the current system thesis; unresolved implementation choices remain provisional until an ADR records evidence and consequences.

## Purpose

Soma is a production-oriented reference system for embodied intelligence. It aims to define the stable system contracts that should remain valid across different robot embodiments while allowing hardware, control, simulation, and application implementations to vary.

The central architectural idea is simple:

> Above the plant boundary, a robot should become a software-defined system with explicit contracts for state, command, time, lifecycle, safety, communication, deployment, and observability.

The canonical assignment of responsibilities is defined in [Layering and Trust Boundaries](layering-and-trust-boundaries.md). Product-stack layers use `L-2` through `L5`; the independent safety-authority chain uses `SA-0` through `SA-5`. These namespaces must not be mixed.

## Architectural principles

1. **Embodiment-independent above the plant boundary**  
   A wheeled base, mobile manipulator, quadruped, or humanoid may have different hardware and control implementations, but should expose common system semantics where practical.

2. **ROS 2 is an ecosystem adapter, not the robot core**  
   The core runtime and protocol must not depend on ROS distributions, message packages, or ROS lifecycle semantics.

3. **Rust-first, not Rust-only**  
   Rust is preferred for the real-time core, runtime, protocol client, and system services. Mature C/C++ vendor libraries, simulation engines, and ROS 2 components may be integrated through narrow FFI or process boundaries.

4. **Hard/soft real-time and distributed software are separate planes**  
   The real-time control loop must not depend on async runtimes, Python, ROS 2, DDS, Zenoh, file I/O, or unbounded allocation.

5. **Simulation is a first-class execution environment**  
   Real hardware and controller-in-the-loop/SIL share the `ControlCore` and Plant contract, while simulators also expose simulation-only operations such as reset, step, snapshot, and randomization. Batch RL shares semantic contracts without being forced through the production process or transport topology.

6. **Production concerns are first-class architecture**  
   Safety, identity, security, OTA, observability, crash evidence, compatibility, and recovery are designed with the runtime rather than added later.

## High-level system

```text
L5  Fleet / cloud operations
L4  Applications, Python/C++ SDKs, ROS 2 adapter, teleoperation
L3  Locomotion, manipulation, perception, navigation, reusable policies
        │
        │ Public Robot Protocol
        │ State | Command | RPC | Action | Event | Lease | Capability
        ▼
robotd supervised deployment unit (spans L2 and L1)
L2  robot-runtime (Rust, non-RT)
      Auth | Arbitration | Actions | Diagnostics | Recorder | OTA
        │ fixed-layout shared memory / SPSC mailboxes
        ▼
L1  robot-rt (Rust, RT)
      ControlCore | Estimator | Controller | SA-3 | Watchdog
        │
        │ bounded Plant Interface
        ▼
L0  ├── physical: HardwarePlant → HAL → EtherCAT / CAN-FD / devices
    │                │
    │                ▼
    │   L-1 Board firmware | bootloaders | FOC | sensors | BMS
    ├── local SIL: deterministic physics Plant, initially MuJoCo
    └── external SIL: ShmPlantProxy → non-RT SimGateway → Isaac / Genesis

L-2 Independent stop / torque / brake / energy-control path,
     safety controller, and trust anchors constrain the physical system
     without depending on robotd
```

## Core boundaries

### 1. Hardware Abstraction Layer

The HAL is an `L0` boundary and describes real hardware only. It owns the host-side transport and device-specific details that higher layers should not see. Board bootloaders, motor-control loops, sensor firmware, and BMS firmware remain at `L-1`; a low-level host API does not make that firmware open or replaceable.

```text
Bus
  ├── EtherCAT
  ├── CAN / CAN-FD
  ├── SPI
  └── Serial

Device
  ├── Servo drive
  ├── IMU
  ├── force/torque sensor
  ├── power / BMS
  └── hand / gripper

Transmission
  ├── gearbox
  ├── differential
  ├── tendon
  └── series-elastic mechanism
```

The HAL should not depend on ROS 2 types.

### 2. Plant Interface

The Plant Interface is the stable boundary seen by the real-time control system. Hardware and physics simulators implement the same bounded contract so the same `ControlCore`, controller, and `SA-3` Safety Supervisor can execute against them.

Representative data:

```text
RobotState(k)
RobotCommand(k)
CycleContext(k)
PlantHealth
LifecycleState
```

A controller should not need to know whether the plant is EtherCAT hardware or MuJoCo. A local simulator may implement the Plant in process when its timing and allocation behavior are bounded. An external simulator must be reached through an RT-safe shared-memory proxy and a non-RT gateway; Python, network RPC, simulator callbacks, and unbounded engine work must not execute on the periodic `robot-rt` path.

### 3. Simulation Control Interface

Simulation has capabilities that real hardware does not. These must remain separate from the normal robot control API.

```text
reset
step
pause
snapshot / restore
set_seed
randomize
spawn / remove objects
ground-truth access
fault injection
```

This separation prevents simulation-only capabilities from leaking into deployable application code.

## Runtime separation

`robotd` is the supervised deployment unit and on-robot service identity, not a monolithic-process requirement. Its baseline deployment consists of `robot-rt`, `robot-runtime`, and supervision that owns startup order, health monitoring, restart policy, and coordinated lifecycle transitions. The supervisor may be an init-system configuration or a dedicated process, but it must preserve the same externally visible failure and recovery semantics.

### `robot-rt`

Responsibilities:

- deterministic control scheduling;
- plant I/O;
- state estimation where required for the control loop;
- whole-body / joint control integration;
- software safety envelope;
- watchdogs and command validation.

Constraints:

- no Tokio or general async runtime in the control loop;
- no ROS 2, DDS, Zenoh, Python, network RPC, or blocking file I/O;
- no unbounded queues;
- no dynamic allocation in the periodic critical path;
- fixed-size RT data structures;
- measured worst-case timing on target hardware.

### `robot-runtime`

Responsibilities:

- public protocol;
- identity and authentication;
- control leases and arbitration;
- actions and long-running tasks;
- capability discovery;
- diagnostics and recording;
- policy lifecycle;
- external communication;
- OTA and observability coordination.

The two processes communicate through bounded shared-memory mailboxes. State publication may drop stale samples rather than block the real-time producer. Commands must carry timing-mode/local-deadline, sequence, Plant-timeline, runtime-generation, and ownership metadata. Loss or restart of `robot-runtime` must be handled by local `robot-rt` and lower safety authorities without unexpected motion.

## Communication planes

Soma should not force every data class through one middleware.

### RT plane

Direct calls and fixed-size structs only.

### RT ↔ runtime plane

Bounded shared-memory mailboxes with fixed-layout messages. The behavioral contract is stable; a time-boxed spike will choose between a minimal SPSC implementation and a mature community implementation without changing the public Robot Protocol.

### Local bulk-data plane

Shared-memory blob pools or a zero-copy IPC framework such as iceoryx2 for camera frames, point clouds, and other large payloads.

### Distributed robot protocol

Current provisional direction: **Zenoh-first**, with optional gateways for Cyclone DDS and gRPC.

Rationale:

- native Rust ecosystem;
- flexible peer/client/router topology;
- practical LAN/Wi-Fi/WAN deployment;
- avoids making DDS/ROS type systems the architectural source of truth.

Cyclone DDS remains relevant when native DDS interoperability is a product requirement, especially for ROS 2 or Unitree-style ecosystems.

V0 does not require a transport bake-off before implementation. A comparative benchmark is triggered if Zenoh misses a declared Performance Envelope, creates an operational problem, or native DDS interoperability becomes a product requirement.

## Public Robot Protocol

The public contract should be independent of ROS 2.

It should explicitly model:

- **State streams** — joint state, IMU, odometry, power, health;
- **Command streams** — velocity, low-level research commands, teleoperation;
- **RPC** — short transactions and configuration;
- **Actions** — cancellable long-running operations with progress;
- **Events** — faults, lifecycle transitions, lease loss, safety intervention;
- **Leases** — control ownership and arbitration;
- **Capabilities** — feature and hardware discovery;
- **Version negotiation** — protocol and schema compatibility.

Commands should include at least:

```text
robot_id
plant_timeline_id
sequence
lease_id
source_id
timing target: immediate / synchronized time / Plant-timeline tick + phase
required clock domain and sync quality where scheduled
client_created_time (evidence only)
server_receive_time and derived local deadline
control_mode
payload
```

The system should distinguish **requested**, **admitted**, **safety-output**, and **applied** commands. Admission records `SA-4` protocol/identity/lease/mode/timing decisions; safety output records `SA-3` validation, clipping, or substitution; applied evidence records what reached the Plant/HAL and any observable lower-authority constraint.

## Time model

Time is a first-class contract.

Soma distinguishes robot-local monotonic deadlines, resettable simulation time, and synchronized/calendar correlation time. Canonical clock-domain names and conversion rules belong to the Time ADR; architecture overviews should not duplicate an enum that can drift.

A reset, snapshot restore, replay seek, `robot-rt`/Plant restart, or other discontinuous Plant-state change creates a new opaque `plant_timeline_id` so stale commands cannot cross timelines. Robot boot, `robot-runtime` restart, and lease succession use separate `boot_id`, `runtime_generation`, and resource-scoped `lease_generation`; restarting only `robot-runtime` does not imply that the Plant timeline changed.

## Simulation architecture

Soma supports several distinct simulation modes rather than forcing one topology.

### SDK/API simulation

External applications connect to a simulator through the same public protocol used for real robots.

### Controller-in-the-loop / SIL

The production control stack runs against a physics plant under simulation time and lockstep scheduling.

### Batch RL / synthetic data

Isaac Lab, Genesis, MJWarp/MJX-style environments may use direct tensor/native APIs for throughput. They reuse observation/action schemas, model identity and coordinate conventions, control semantics, normalization/clipping, latency assumptions, and policy-bundle compatibility. They are not required to instantiate `robotd`, Zenoh, or one production IPC graph per environment.

### HIL

Real host software communicates with emulated or physical bus/device interfaces to test drivers, watchdogs, timing, and failure recovery.

## Robot model and identity

URDF, MJCF, and USD should not be the sole source of truth. Soma should define a shared canonical `ProductModelManifest` and compose it with artifacts that have different owners and trust domains:

```text
ProductModelManifest       topology, nominal parameters, frames, transmissions, assets
RobotInstanceManifest      one robot's provisioned identity/configuration
DeviceInventory            observed installed boards/devices and firmware revisions
CalibrationSet             serial-specific measured corrections
ControlProfile             controller tuning and ordinary operational limits
SafetyProfile              independently governed safety-authoritative limits/behavior
```

V0 uses a minimal `ProductModelManifest` alongside backend-native URDF, MJCF, and USD artifacts. A validator checks shared identity, joints, frames, units, transmissions, and declared compatibility; it does not attempt lossless cross-format generation. A model compiler is a later option only where repeated, proven transformations justify it. The public `RobotManifest` is a composed runtime projection for SDK discovery, not another writable source of truth. Separately, L0 reports inventory and health while L2 produces an ephemeral `RuntimeCapabilitySnapshot`; that snapshot references the activation-set hashes but does not enter the product-model or `RobotManifest` hash.

All relevant artifacts should carry stable identifiers such as:

```text
robot_model_id
hardware_revision
product_model_bundle_id / hash
robot_instance_manifest_hash
device_inventory_hash
calibration_set_hash
control_profile_hash
safety_profile_hash
joint_schema_hash
frame_schema_hash
```

Instance identifiers such as `robot_serial` must not change the shared product-model hash. A model, controller, policy, or simulator update cannot implicitly change the active `SafetyProfile`.

## Policy deployment

A policy should be deployed as a versioned `PolicyBundle`, not as an isolated model file.

A bundle includes:

- model artifact;
- observation and action schemas;
- normalization and clipping;
- history length;
- policy rate and control decimation;
- required robot model and capabilities;
- runtime compatibility;
- checksum and signature.

Policy compatibility does not authorize a safety change. `SafetyProfile` remains a separately reviewed, signed, activated, audited, and rolled-back artifact.

## ROS 2 boundary

ROS 2 belongs above the public Robot Protocol.

```text
ROS application
      │
robot_ros2_bridge
      │
robot client / Robot Protocol
      │
robot-runtime
```

The bridge maps robot semantics to `sensor_msgs`, `nav_msgs`, TF, ROS actions, Nav2, MoveIt, and optional ros2_control integration. Different ROS distributions may be shipped as separate adapters or containers without rebuilding the robot core.

## OTA and lifecycle operations

The runtime participates in a broader operations plane that includes:

- signed release manifests;
- OS A/B update and rollback;
- MCU/FPGA firmware lifecycle;
- model and policy bundles;
- robot-instance inventory, calibration, control, and independently governed safety profiles;
- compatibility checks;
- local health gates;
- staged fleet rollout;
- crash evidence and rollback correlation.

A release should represent a tested compatibility set across OS, robot core, device firmware, product model, robot instance/device inventory, calibration, control profile, SafetyProfile, configuration schema, and policy. Transporting these artifacts in one release does not merge their signing roles or activation authority.

## Observability

Soma separates four kinds of evidence:

1. **Metrics** — long-term health and SLOs;
2. **Structured events/logs** — lifecycle and fault context;
3. **Traces** — SDK/action/task execution chains;
4. **High-frequency robot data** — MCAP flight recorder and replay.

The RT core emits only bounded counters and fixed-size events into a preallocated ring. Formatting, OTLP export, storage, and upload happen outside the real-time process.

Crash evidence should include userspace coredumps, kernel pstore/ramoops, and MCU reset/fault records.

## Security and safety

Safety is an independent authority, not just command clamping.

Safety mechanisms use the `SA-0` through `SA-5` namespace defined by [Layering and Trust Boundaries](layering-and-trust-boundaries.md). The `SA-0` independent stop/torque/brake/energy-control path and `SA-1` independent safety control remain outside the normal Linux control path; `SA-3` is the `robot-rt` Safety Supervisor and `SA-4` is runtime authority/mode policy.

The software safety path should validate:

- command ownership and validity;
- joint and workspace limits;
- power and thermal envelopes;
- bus and sensor health;
- collision and fall conditions;
- timing and watchdog conditions.

Hardware E-stop, STO, brakes, and power cutoff remain outside the Linux application safety path.

Security should include device identity, signed artifacts, authenticated control sessions, capability-level authorization, auditability, and a long-term secure-boot/update trust chain. The high-level threats, trust boundaries, and invariants are defined in [Security Threat Model](security-threat-model.md); detailed mechanisms remain ADR and implementation work.

## Open architectural questions

The current reference architecture is a working thesis. Important questions still requiring experiments include:

- whether Zenoh needs a Cyclone DDS comparison after measured V0 workloads or a native DDS requirement triggers it;
- pure-Rust EtherCAT implementations vs mature C/C++ masters;
- PREEMPT_RT latency bounds on candidate compute platforms;
- shared-memory ABI strategy and compatibility;
- whether repeated model transformations justify a compiler beyond the V0 manifest validator;
- which safety responsibilities belong in the host versus dedicated safety hardware;
- how much low-level control should be exposed to external research users;
- OTA/recovery strategy across heterogeneous robot electronics;
- the policy/inference-to-RT boundary: timeout policy, interpolation ownership, observation-time alignment, and action chunking — see [`policy-runtime-interface.md`](../deep-research/policy-runtime-interface.md) and `D-19`.

These questions should be resolved through Deep Research, benchmarks, ADRs, SIL/HIL tests, and eventually physical reference robots.
