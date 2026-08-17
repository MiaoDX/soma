# Reference System Design Space

> Status: Deep Research baseline. This document maps alternatives; canonical posture lives in architecture docs and the Decision Register.

## Question

What are the major architectural choices for a production robot reference system, and where should Soma intentionally remain opinionated versus extensible?

## Context

A reference system becomes fragile when it accidentally treats one implementation choice as a permanent architectural truth. Soma should therefore separate **invariants** from **replaceable mechanisms**.

The purpose of this note is to map the important choices we have identified so far and make clear which ones need experiments or ADRs.

## 1. Real-time operating model

### Option A — Linux + PREEMPT_RT

Strengths:

- full Linux ecosystem;
- easier access to network, storage, GPU, vendor libraries;
- practical for x86/ARM robot computers;
- one deployment environment for many subsystems.

Risks:

- real-time behavior depends on kernel, IRQ affinity, CPU isolation, memory locking, driver behavior, firmware, and hardware;
- it is not sufficient to say "PREEMPT_RT" without measuring worst-case latency on target compute.

### Option B — Linux host + dedicated RTOS/MCU control

Strengths:

- strong isolation for actuator/safety loops;
- predictable embedded timing;
- Linux failures do not necessarily stop low-level control immediately.

Risks:

- more distributed state and update complexity;
- additional protocol boundaries;
- duplicated diagnostics and version management.

### Soma direction

Support both. The Plant/HAL boundary should not assume that all control is executed on the Linux host.

## 2. Primary implementation language

### Rust-first

Benefits:

- memory safety and data-race prevention;
- strong type modeling for lifecycle and protocol contracts;
- good fit for runtime/network tooling;
- one language can cover systems code and SDK core.

Risks:

- vendor robotics ecosystems remain largely C/C++;
- no universal stable Rust dynamic ABI;
- real-time safety still requires explicit allocation and blocking discipline;
- some industrial libraries may require unsafe FFI wrappers.

### Soma direction

Use Rust as the preferred implementation language but define strict FFI boundaries and avoid a requirement for a pure-Rust dependency tree.

## 3. Fieldbus and device transport

### EtherCAT

Best fit when:

- many synchronized actuators;
- deterministic cyclic process data;
- distributed clocks matter;
- high-DOF robots need predictable topology.

Questions for Soma:

- IgH vs SOEM vs commercial master vs Rust implementation;
- NIC/driver compatibility;
- DC synchronization quality;
- recovery behavior under slave faults.

### CAN-FD

Best fit when:

- integrated smart actuators or peripherals;
- moderate device count/bandwidth;
- simple wiring and embedded controllers matter.

Soma should build on Linux SocketCAN where appropriate rather than inventing a host CAN stack.

### Soma direction

Bus is a backend implementation detail below Device and Plant abstractions. A robot profile may use multiple buses simultaneously.

## 4. RT ↔ runtime IPC

### Custom fixed-layout SPSC shared memory

Strengths:

- semantics can be made extremely explicit;
- bounded memory and latency;
- minimal dependencies;
- ideal for a small number of critical channels.

Risks:

- requires careful memory layout, Plant-timeline/runtime generation, sequence, cache coherence, and crash recovery design;
- internal ABI compatibility must be versioned.

### iceoryx2 or general zero-copy IPC

Strengths:

- broader pub/sub and request/response primitives;
- zero-copy large payload support;
- Rust-oriented implementation.

Risks:

- more general machinery than the RT command/state boundary needs;
- still requires measurement for the exact workload.

### Soma direction

Keep a fixed-capacity SPSC-style behavioral contract for the critical RT boundary. A bounded spike compares a minimal mailbox with a mature community implementation; choose the smaller implementation that meets the contract and measured envelope. Evaluate iceoryx2 separately for non-RT local bulk data and multi-process high-bandwidth flows.

## 5. Distributed middleware

### Cyclone DDS

Strengths:

- mature DDS semantics and QoS;
- excellent fit for robot LAN pub/sub;
- direct affinity with ROS 2 and existing DDS-based robot SDKs.

Risks:

- more complex discovery and network behavior;
- Rust integration is less direct than Rust-native middleware;
- easy to accidentally make DDS IDL the entire system's type model.

### Zenoh

Strengths:

- Rust-first;
- flexible peer/client/router topology;
- good fit across process, LAN, Wi-Fi, and routed networks;
- bridges exist for ROS 2/DDS ecosystems.

Risks:

- Soma must define more of its own public protocol semantics;
- needs real benchmarks for robot-specific traffic and failure modes.

### gRPC

Strengths:

- excellent cross-language RPC and enterprise integration;
- strong tooling and API ergonomics.

Risks:

- insufficient as the only transport for high-rate pub/sub and large sensor streams.

### Soma direction

Zenoh-first public/distributed data plane, with DDS and gRPC adapters where they provide ecosystem value. This is a provisional, reversible V0 choice; a Zenoh/Cyclone comparison is triggered by a missed Performance Envelope, an operational problem, or a native DDS requirement rather than required before implementation.

## 6. Public schema

### ROS messages as source of truth

Rejected as the default direction because it couples the product protocol to ROS package/version semantics.

### DDS IDL as source of truth

Attractive for DDS-first platforms, but less suitable if Soma wants protocol independence and multiple transport profiles.

### Protobuf for public API

Strong schema evolution and language tooling. Suitable for RPC, events, actions, state metadata, and most SDK objects.

Limitations:

- not ideal for fixed-layout RT structures;
- large payloads should be referenced rather than copied into generic messages.

### Soma direction

Maintain distinct but mapped representations:

- fixed-layout RT schema;
- versioned public network schema;
- bulk-data descriptors.

## 7. Python exposure

### Native Python implementation of protocol

Advantages:

- simple packaging for pure-Python applications;
- direct access to Python middleware bindings.

Downside:

- duplicates reconnect, lease, timeout, schema, and security logic.

### Rust client + PyO3

Advantages:

- one core client implementation;
- protocol/security behavior shared across CLI/Python/C ABI;
- high-performance parsing and I/O.

Downside:

- wheel/build complexity;
- async runtime integration must be carefully designed.

### Soma direction

Prefer a Rust client as the source of truth with an async-first Python API through PyO3. Provide a synchronous convenience layer separately.

## 8. ROS 2 integration

Three possible boundaries:

1. ROS 2 below the robot runtime;
2. ROS 2 as the robot runtime;
3. ROS 2 above the public robot protocol.

Soma chooses **3**.

This isolates:

- ROS distribution lifecycle;
- `rclcpp` / `rclpy` dependencies;
- Nav2/MoveIt/ros2_control versions;
- ROS message package evolution.

The trade-off is maintaining a high-quality bridge rather than inheriting the ROS graph directly.

## 9. Simulation strategy

### One simulator as the canonical environment

Simple but fragile. No single simulator is optimal for control regression, visual simulation, synthetic data, and massive RL workloads.

### Multiple backends behind shared contracts

Soma direction:

- MuJoCo — control regression, SIL, CI, low-latency simulation;
- Isaac Sim/Lab — RTX sensors, complex scenes, synthetic data, RL;
- Genesis — research/batch learning backend;
- HIL — driver, fieldbus, watchdog, and failure testing.

The shared artifacts should be control semantics, product-model identity, and the contributing hashes in the composed runtime `RobotManifest`, not one universal simulator execution path.

## 10. Robot model representation

### URDF-only

Useful for ROS and kinematics, but insufficient as the universal representation for hardware mappings, calibration, product variants, safety limits, policy schemas, and simulator-specific overlays.

### Soma direction

Define a canonical shared `ProductModelManifest`, compose it with separately governed robot-instance inventory, calibration, control, and safety artifacts, and generate/validate ecosystem representations such as URDF, MJCF, and USD. The public `RobotManifest` is a runtime projection rather than another writable source file.

This is a major design area requiring dedicated research.

## 11. Safety boundary

A production robot needs at least three layers:

```text
Application intent
      ↓
software safety / command authority
      ↓
real-time controller / watchdog
      ↓
hardware safety path: E-stop / STO / brakes / power control
```

The Linux process must never be treated as the sole safety authority.

Soma must keep "software stop" and "hardware emergency stop" semantically distinct.

## 12. OTA and release engineering

A robot is not a single firmware artifact. Releases can include:

- bootloader;
- OS/kernel/rootfs;
- robot-rt/runtime;
- MCU/FPGA firmware;
- product model, robot instance/device inventory, calibration and control profiles;
- independently governed SafetyProfile;
- models/policies;
- configuration and skills.

Soma should model a release as a tested compatibility set rather than independent files.

Likely direction:

- A/B Linux rootfs;
- dual-bank/MCUboot-style firmware update;
- signed manifests and TUF/Uptane-inspired trust;
- local robot-level health gate before mark-good;
- staged fleet rollout.

## 13. Observability

A single logging stack is insufficient.

Soma should separate:

- OpenTelemetry-style metrics/logs/traces for system behavior;
- MCAP for high-frequency robot data and flight recording;
- coredumps/pstore/MCU retention data for crash evidence.

The RT process should only emit bounded counters and fixed-size events.

## 14. Build-vs-buy philosophy

Soma should build where the contract is part of the product's identity, and reuse where mature infrastructure exists.

### Likely build

- Plant contract;
- RT state/command model;
- control authority/lease semantics;
- product-model and composed RobotManifest contracts;
- robot-instance, calibration, control, and SafetyProfile artifact boundaries;
- public Robot Protocol;
- SDK ergonomics;
- safety/runtime integration;
- release compatibility model.

### Likely reuse/adapt

- EtherCAT master where mature;
- SocketCAN;
- cryptographic libraries;
- RAUC/MCUboot/TUF implementations;
- Zenoh/DDS/gRPC;
- OpenTelemetry;
- MCAP;
- simulation engines.

## 15. Runtime and platform reference implementations

Two open-source references sharpen this build-vs-reuse boundary without replacing Soma's architecture.

### Copper: execution kernel candidate

[Copper](https://github.com/copper-project/copper-rs) is a Rust runtime for static robot task graphs with generated scheduling, preallocated cycle messages, runtime-native recording, mockable time, task-state keyframes, and deterministic resimulation tests.

It is a credible candidate for bounded in-process execution inside `robot-rt`. Soma must still own the Plant contract, four-stage command lineage, safety authority, RT/runtime IPC, public protocol, and release/artifact model. Adoption requires a representative 1 kHz spike rather than inference from framework benchmarks.

### Eclipse S-CORE: platform and evidence reference

[Eclipse S-CORE](https://github.com/eclipse-score) is not a robot runtime and explicitly is not a ready-to-integrate series product. Its highest-value lessons for Soma are:

- lifecycle and health supervision are explicit platform services, separate from application/task execution;
- typed local, steady, high-resolution, and PTP-synchronized clock APIs reduce cross-domain mistakes;
- the reference integration pins a `known_good.json` set of module commits, patches, test configuration, exclusions, and known issues;
- Linux/QNX target images and cross-module feature tests continuously prove an integrated set rather than nominal version compatibility;
- requirements, decisions, tool evaluations, safety assumptions, and integrator responsibilities are public artifacts;
- its 2026 Rust ASIL-B readiness decision shows the expected evidence shape while limiting the conclusion to QNX 8 with Ferrocene.

Soma should copy the **continuously consistent stack** principle through a small candidate `ReleaseManifest`. It should not copy Adaptive AUTOSAR APIs, the Bazel multirepo structure, C++ base-library layers, or a full automotive process before the first vertical slice needs them.

The detailed repository map, Copper replay analysis, adjacent Rust projects, and adoption gates live in [`runtime-and-platform-reference-projects.md`](runtime-and-platform-reference-projects.md).

## Decisions that need experiments before ADRs

1. Zenoh production confirmation, with a Zenoh vs Cyclone DDS benchmark only when a defined trigger is met.
2. RT shared-memory implementation and measured overhead.
3. Rust EtherCAT vs mature C master comparison.
4. PREEMPT_RT on target x86/ARM compute.
5. Python/PyO3 async ergonomics and packaging.
6. MuJoCo in-process vs separate-process SIL topology.
7. iceoryx2 vs custom blob pool for local sensor payloads.
8. ProductModelManifest source-of-truth format and composed runtime RobotManifest.
9. Copper-hosted versus minimal Soma-native RT execution/replay spike.
10. Machine-readable candidate `ReleaseManifest` derived from S-CORE's `known_good` pattern.

## Architectural invariant to preserve

Soma should be able to replace any of these mechanisms without redesigning the whole system:

```text
EtherCAT master
CAN adapter
middleware
simulator
ROS distribution
Python version
GPU platform
cloud backend
```

The stable value of Soma should live in its contracts, lifecycle, safety model, compatibility rules, and developer experience rather than in a particular middleware choice.
