# Soma Reference Architecture

## Purpose

Soma is a production-oriented reference system for embodied intelligence. It aims to define the stable system contracts that should remain valid across different robot embodiments while allowing hardware, control, simulation, and application implementations to vary.

The central architectural idea is simple:

> Above the plant boundary, a robot should become a software-defined system with explicit contracts for state, command, time, lifecycle, safety, communication, deployment, and observability.

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
   Real hardware and simulators share control and lifecycle contracts, while simulators also expose simulation-only operations such as reset, step, snapshot, and randomization.

6. **Production concerns are first-class architecture**  
   Safety, identity, security, OTA, observability, crash evidence, compatibility, and recovery are designed with the runtime rather than added later.

## High-level system

```text
Applications / Physical AI
        │
        ├── Python SDK
        ├── ROS 2 Adapter
        ├── C/C++ SDK
        └── Fleet / Cloud
        │
Public Robot Protocol
State | Command | RPC | Action | Event | Lease | Capability
        │
robot-runtime (Rust, non-RT)
Auth | Arbitration | Actions | Diagnostics | Recorder | Policy
        │
fixed-layout shared memory / SPSC mailboxes
        │
robot-rt (Rust, RT)
Estimator | Controller | Safety | Watchdog | Plant Interface
        │
        ├── Hardware Plant
        │     HAL → EtherCAT / CAN-FD / devices
        │
        ├── MuJoCo Plant
        └── External Simulation Plant
              Isaac Sim / Isaac Lab / Genesis
```

## Core boundaries

### 1. Hardware Abstraction Layer

The HAL describes real hardware only. It owns the transport and device-specific details that higher layers should not see.

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

The Plant Interface is the stable boundary seen by the real-time control system. Both hardware and physics simulators implement it.

Representative data:

```text
RobotState(k)
RobotCommand(k)
CycleContext(k)
PlantHealth
LifecycleState
```

A controller should not need to know whether the plant is EtherCAT hardware or MuJoCo.

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

The two processes communicate through bounded shared-memory mailboxes. State publication may drop stale samples rather than block the real-time producer. Commands must carry validity, sequence, epoch, and ownership metadata.

## Communication planes

Soma should not force every data class through one middleware.

### RT plane

Direct calls and fixed-size structs only.

### RT ↔ runtime plane

Custom SPSC shared memory with bounded semantics.

### Local bulk-data plane

Shared-memory blob pools or a zero-copy IPC framework such as iceoryx2 for camera frames, point clouds, and other large payloads.

### Distributed robot protocol

Current preferred direction: **Zenoh-first**, with optional gateways for Cyclone DDS and gRPC.

Rationale:

- native Rust ecosystem;
- flexible peer/client/router topology;
- practical LAN/Wi-Fi/WAN deployment;
- avoids making DDS/ROS type systems the architectural source of truth.

Cyclone DDS remains relevant when native DDS interoperability is a product requirement, especially for ROS 2 or Unitree-style ecosystems.

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
epoch_id
sequence
lease_id
source_id
target_tick / target_apply_time
created_time
valid_until
control_mode
payload
```

The system should distinguish **requested**, **accepted**, and **applied** commands so that safety modifications are observable.

## Time model

Time is a first-class contract.

Soma distinguishes:

- `MONOTONIC_ROBOT` — deadlines and watchdogs on physical hardware;
- `SIMULATION` — lockstep, pause, accelerated simulation, reset;
- `UTC/PTP` — multi-computer logs, fleet correlation, external sensors.

Every reset or runtime restart should create a new `epoch_id` so stale commands from a previous timeline can be rejected.

## Simulation architecture

Soma supports several distinct simulation modes rather than forcing one topology.

### SDK/API simulation

External applications connect to a simulator through the same public protocol used for real robots.

### Controller-in-the-loop / SIL

The production control stack runs against a physics plant under simulation time and lockstep scheduling.

### Batch RL / synthetic data

Isaac Lab, Genesis, MJWarp/MJX-style environments may use direct tensor/native APIs for throughput. They reuse observation/action contracts and model metadata, not necessarily the production network path.

### HIL

Real host software communicates with emulated or physical bus/device interfaces to test drivers, watchdogs, timing, and failure recovery.

## Robot model and identity

URDF, MJCF, and USD should not be the sole source of truth. Soma should define a canonical `RobotManifest` containing:

```text
identity
hardware revision
joint table
actuator table
sensor table
frame graph
transmissions
limits
calibration schema
optional modules
supported control modes
model asset references
```

A model compiler can generate or validate URDF/Xacro, MJCF, USD, ROS descriptions, RT joint maps, and policy schemas.

All relevant artifacts should carry stable identifiers such as:

```text
robot_model_id
hardware_revision
model_bundle_id
calibration_id
joint_schema_hash
frame_schema_hash
```

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
- compatibility checks;
- local health gates;
- staged fleet rollout;
- crash evidence and rollback correlation.

A release should represent a tested combination of OS, robot core, device firmware, robot model, configuration schema, and policy compatibility.

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

The software safety path should validate:

- command ownership and validity;
- joint and workspace limits;
- power and thermal envelopes;
- bus and sensor health;
- collision and fall conditions;
- timing and watchdog conditions.

Hardware E-stop, STO, brakes, and power cutoff remain outside the Linux application safety path.

Security should include device identity, signed artifacts, authenticated control sessions, capability-level authorization, auditability, and a long-term secure-boot/update trust chain.

## Open architectural questions

The current reference architecture is a working thesis. Important questions still requiring experiments include:

- Zenoh vs Cyclone DDS under real robot LAN/Wi-Fi/WAN workloads;
- pure-Rust EtherCAT implementations vs mature C/C++ masters;
- PREEMPT_RT latency bounds on candidate compute platforms;
- shared-memory ABI strategy and compatibility;
- canonical robot model representation;
- which safety responsibilities belong in the host versus dedicated safety hardware;
- how much low-level control should be exposed to external research users;
- OTA/recovery strategy across heterogeneous robot electronics.

These questions should be resolved through Deep Research, benchmarks, ADRs, SIL/HIL tests, and eventually physical reference robots.
