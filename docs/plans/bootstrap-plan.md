# Soma Bootstrap Plan

## Goal

Build Soma from a reference architecture into a working, measurable, production-oriented robot reference system that can be validated across multiple embodiments and simulation backends.

The bootstrap plan intentionally starts with contracts and simulation before expanding to real hardware and fleet operations.

## Phase 0 — Research and architecture

### Objectives

- define the system boundaries and invariants;
- document the design space and alternatives;
- validate key assumptions with targeted benchmarks;
- establish the canonical vocabulary and data contracts.

### Deliverables

- reference architecture;
- robot/plant/runtime boundary definitions;
- time, lifecycle, error, lease, and capability models;
- initial RobotManifest and PolicyBundle schemas;
- middleware and IPC benchmark plan;
- OTA/observability design notes;
- ADR process.

### Exit criteria

- the architecture can describe at least a wheeled robot and a legged robot without special-case restructuring;
- the same control-facing Plant contract can represent real hardware and simulation;
- major unresolved decisions have experiments or research tasks attached.

## Phase 1 — Minimal reference system

### Objectives

Build the smallest end-to-end system that demonstrates the core boundaries.

### Scope

- Rust `robot-rt` skeleton;
- fixed-layout RT state/command types;
- bounded shared-memory IPC between RT and runtime;
- Rust `robot-runtime`;
- minimal public Robot Protocol;
- Python SDK prototype;
- MuJoCo Plant backend;
- MCAP recording;
- basic structured health/events.

### Demo

A Python application connects to the runtime, acquires a control lease, commands a simulated robot, receives state/health, and records a replayable session while the RT loop remains isolated from Python and middleware behavior.

### Exit criteria

- deterministic headless simulation in CI;
- no ROS 2 dependency in core crates;
- bounded RT/runtime communication under overload;
- stale/expired commands are rejected;
- simulation reset creates a new epoch and invalidates old commands.

## Phase 2 — First physical embodiment

### Objectives

Bring the same core architecture onto real robot electronics.

### Candidate embodiment

Prefer a mechanically simple platform first — for example a small wheeled base or mobile robot — unless a legged platform is already the most practical hardware available.

The goal is to validate the system architecture, not to maximize mechanical complexity.

### Scope

- real HAL implementation;
- CAN-FD and/or EtherCAT integration;
- real-time Linux configuration and latency characterization;
- hardware watchdog and safety integration;
- RobotManifest and calibration pipeline;
- local observability and flight recorder;
- signed release and basic OTA/recovery path.

### Exit criteria

- the same external SDK semantics work against simulation and hardware;
- control-loop timing and jitter are measured on target compute;
- communication loss and process failures lead to defined safe behavior;
- recorded incidents contain enough evidence for root-cause analysis;
- failed software update can recover without reflashing the device manually.

## Phase 3 — Cross-embodiment validation

### Objectives

Prove that Soma is a reference system rather than a one-robot architecture.

### Scope

Add a second embodiment with materially different constraints, for example:

- wheeled base → biped/quadruped;
- AMR → mobile manipulator;
- low-DOF → high-DOF robot.

### Validation questions

- Which interfaces remain unchanged?
- Which HAL/Plant concepts need specialization?
- Can the same Robot Protocol and SDK handle both platforms?
- Does the lifecycle/safety/OTA model generalize?
- Are model and capability discovery sufficient to avoid SKU-specific client code?

### Exit criteria

- two embodiments share the same public protocol family and client SDK;
- common runtime modules remain reusable without embodiment-specific forks;
- platform-specific differences are represented through capabilities, manifests, and backend implementations.

## Phase 4 — Production operations

### Objectives

Turn the reference implementation into an operational platform.

### Scope

- secure boot and device identity;
- production OTA trust chain;
- staged/canary fleet rollout;
- OpenTelemetry-based service observability;
- local MCAP blackbox and incident upload;
- SBOM/provenance and signed artifacts;
- compatibility and LTS policy;
- simulator/HIL release qualification;
- ROS 2 adapters for target distributions.

### Exit criteria

- a release can be built, qualified in simulation/HIL, staged to canary robots, observed, and rolled back;
- an incident can be correlated across release, boot, mission, action, safety event, and robot data;
- robot core operation remains independent of fleet/cloud availability.

## Near-term workstreams

### A. Core contracts

- RT state/command;
- Plant interface;
- simulation control;
- clock/epoch/tick;
- lifecycle/fault model;
- RobotManifest.

### B. Runtime and protocol

- shared-memory IPC;
- lease/arbitration;
- state/action/event protocol;
- Zenoh prototype;
- Python client.

### C. Simulation

- MuJoCo reference backend;
- deterministic scenario runner;
- replay and fault injection;
- Isaac/Genesis adapter contract.

### D. Systems research

- EtherCAT/CAN-FD stack;
- Rust real-time constraints;
- PREEMPT_RT benchmark;
- middleware benchmark;
- OTA/recovery;
- observability and blackbox.

## Definition of success

Soma succeeds if it demonstrates that a common production system architecture can survive changes in robot embodiment, hardware generations, simulation engines, middleware integrations, and application ecosystems without repeatedly redesigning the core runtime.
