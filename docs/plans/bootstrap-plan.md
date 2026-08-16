# Soma Bootstrap Plan

## Goal

Build Soma from a reference architecture into a measurable, production-oriented robot system while proving both of its essential claims:

1. the same control and runtime contracts work across simulation and physical embodiments;
2. the project can own and validate the lower stack, not only adapt a vendor joint-command SDK.

The bootstrap therefore develops a MuJoCo vertical slice and a public-artifact actuator/electronics vertical slice in parallel. They converge on the same contracts before Soma expands to a bottom-up-qualified whole-robot reference and then to closed commercial compatibility targets.

## Sequencing principles

- Use the canonical L-2–L5 layer vocabulary and keep safety authorities in the separate SA-0–SA-5 namespace.
- Establish the independent safety path, trust-root design, device identity, and signed development update flow before depending on OTA.
- Assess openness per component. A `q/dq/tau/kp/kd` interface is privileged joint command access, not evidence that drive firmware, boot, BMS, fieldbus, or safety is open.
- Develop SIL and physical lower-stack work concurrently so neither an idealized simulator nor vendor middleware silently defines the architecture.
- Qualify every claim at an explicit evidence level and preserve the artifacts needed to reproduce it.
- Use commercial robots after the bottom-up-qualified reference path as boundary/adapter compatibility targets, not substitutes for bottom-up validation.

## Evidence model

Exit criteria use the following evidence tags. They are complementary test environments, not interchangeable marketing maturity levels.

| Evidence | What runs | Claims it can support | Claims it cannot support by itself |
|---|---|---|---|
| `HOST` | static checks and host-side unit, schema, cryptographic, or composition tests | parser/validator behavior, canonicalization, signature-policy logic, API compatibility | deployed scheduling, simulator dynamics, real bus/device behavior, physical safety |
| `SIM` | simulator-native model, backend, or API facsimile | model structure, simulator adapter behavior, scenario generation, throughput | production controller behavior, fieldbus/firmware behavior, physical safety |
| `SIL` | production control/runtime code against a simulated Plant or virtual device | control semantics, lifecycle, timing logic, reset/timeline behavior, deterministic regression, software fault handling | electrical behavior, real bus timing, boot/update recovery, physical dynamics |
| `HIL` | production software with at least one target-equivalent real compute, bus controller, board, drive, sensor, or safety/power component in the loop; remaining components may be emulated | driver and device state machines, watchdogs, timestamps, firmware/update faults, bus and power fault response | unrepresented mechanism dynamics, payload/contact behavior, or thermal/mechanical limits |
| `PHYSICAL` | the controlled physical actuator, mechanism, or whole robot | real dynamics, calibration, power/thermal behavior, physical fault response, operational evidence | certification or general safety assurance beyond the tested configuration |

Review, analysis, and inspection are verification methods rather than execution environments; they should be recorded separately instead of being relabeled as SIL or HIL. Each dynamic result should record the loop composition, including which parts are real, simulated, or emulated, and should reference a repeatable test or scenario ID, hardware and model identity, software/firmware versions, configuration and calibration hashes, measured tolerances, and retained logs/MCAP evidence. A claim must name the strongest relevant evidence actually obtained; for example, a SIL pass must not be presented as proof of E-stop or STO behavior.

## Bottom-Up Reference Profile

Soma keeps a cumulative coverage ledger so a platform cannot pass by declaring few capabilities or leaving every difficult row `unknown`. Before claiming a bottom-up reference, the combined benches and robot must provide verified evidence for at least:

- host identity, boot, signed update, anti-rollback policy, recovery, and boot-reason reporting;
- one representative actuator's electronics, encoder path, device BSP/RTOS, bootloader, firmware/FOC, signed update, rollback or recovery, and fault state;
- raw fieldbus access, master/HAL implementation, cyclic timing, device lifecycle, timestamps, watchdog, and diagnostics;
- serial-scoped calibration provenance, activation compatibility, device readback, and replacement invalidation;
- an independent stop or torque-inhibition final element plus main-compute watchdog behavior, with braking and contactor/energy-isolation roles stated separately;
- power, current, voltage, and thermal evidence plus at least one representative sensor-firmware path.

No single robot must expose every row if a documented supplementary rig qualifies an otherwise inaccessible BMS, sensor, or safety/power path. `unknown`, `vendor_asserted`, and joint-command-only access do not satisfy a required row. The ledger records which rig provides each result and prevents a whole-robot adapter from inheriting evidence it did not earn.

## Phase 0 — Architecture, safety, and trust foundations

### Objectives

- define system boundaries and invariants;
- establish the canonical vocabulary and data contracts;
- select reference paths from component-level evidence rather than whole-platform labels;
- make safety and update trust part of the initial architecture.

### Deliverables

- canonical L-2–L5 reference architecture and Plant/HAL/runtime boundaries;
- time, timeline, lifecycle, error, lease, capability, and command-application models;
- initial `ProductModelManifest`, `RobotInstanceManifest`, `DeviceInventory`, `CalibrationSet`, `ControlProfile`, independently governed `SafetyProfile`, `RuntimeCapabilitySnapshot`, and `PolicyBundle` schemas;
- component-level openness/assurance matrix for each candidate platform;
- preliminary whole-robot primary/fallback selection and a convergence map showing which Phase 1 actuator/bus path carries forward;
- initial hazard analysis, safe-state definitions, independent stop/torque/brake/energy-control concept, and fault-test plan;
- development trust-root and signing-role design, device identity format, anti-rollback and recovery policy;
- signed development artifact verifier and a minimal signed-update proof of concept;
- middleware, IPC, RT scheduling, OTA, recovery, and observability experiment plans;
- ADR process with owners and resolution evidence for open decisions.

### Exit criteria

- `SIL`: wheeled/manipulator and legged manifest fixtures instantiate the same control/runtime contracts without embodiment-specific protocol branches;
- `SIL`: the same fixed-layout control-facing Plant contract can execute against a simulated backend and a stubbed physical HAL;
- `HOST`: a correctly scoped signed development artifact is accepted for its test device identity, while identity-incompatible, modified, unsigned, and policy-disallowed rollback artifacts are rejected with auditable reasons;
- `SIL`: activating an ordinary product model, controller, simulator overlay, or policy cannot change or relax the independently governed `SafetyProfile`;
- every candidate platform has a component matrix covering safety, boot/trust, power/BMS, actuator electronics/firmware, bus/HAL, calibration, runtime, SDK, and simulation assets;
- the first physical bench has a reviewed independent E-stop or power-isolation path that does not depend on Linux, `robot-rt`, middleware, or cloud availability;
- major unresolved decisions have an ADR, bounded experiment, owner, and required evidence level.

## Phase 1 — Parallel vertical slices

### Objective

Build two end-to-end paths against the same contracts:

- **Track A — MuJoCo vertical slice:** validate protocol, runtime, deterministic control, simulation semantics, replay, and SDK behavior;
- **Track B — owner-controlled lower-stack bench:** validate L-1/L0 ownership on a mechanically constrained single joint, then expand to a synchronized multi-axis fixture.

The tracks share schemas, controller interfaces, lifecycle, timestamps, manifests, identity, command validity, observation, and recording. They may use different execution topology below the Plant boundary.

### Shared scope

- Rust `robot-rt` skeleton and fixed-layout RT state/command types;
- bounded shared-memory IPC between `robot-rt` and `robot-runtime`;
- minimal public Robot Protocol and Python SDK;
- requested, admitted, safety-output, and applied command observability, including attributed runtime and safety decisions;
- product model, robot instance/device inventory, configuration, calibration, `ControlProfile`, `SafetyProfile`, and `RuntimeCapabilitySnapshot` identities;
- MCAP flight recording and structured health/events;
- development device identity, signed host/device updates, rollback policy, and recovery metadata.

### Track A scope — MuJoCo

- MuJoCo Plant backend;
- deterministic headless scenario runner;
- pause, step, reset, accelerated time, and timeline invalidation;
- command expiry, lease loss, overload, and sensor/actuator fault injection;
- record/replay and conformance fixtures reusable by later backends.

### Track B scope — public-artifact actuator bench

- select an actuator path using the component matrix, preferring the Phase 2 primary robot's actuator, bus, and update stack so the bench work carries forward;
- when a generic moteus/ODRI-class contract rig is chosen instead, label it as a supplementary reference rig and budget the second L-1/L0 bring-up explicitly;
- mechanically constrain and power-limit the initial single-joint setup;
- own or reproducibly build the actuator firmware and host HAL for the layers being claimed;
- integrate CAN-FD and/or EtherCAT timing, device state, raw diagnostics, and hardware timestamps;
- implement calibration, watchdog, boot/update/recovery, and an independent bench stop/torque-inhibition or power-isolation final element selected by the bench hazard analysis;
- expand from one joint to at least three simultaneously controlled axes without changing the public Plant state/command contract.

### Demonstrations

1. A Python application acquires a lease, commands the MuJoCo robot, receives state and health, and records a replayable session while the RT loop remains isolated from Python and middleware behavior.
2. The same application and protocol family command a constrained physical joint and then a multi-axis fixture through a real HAL, with unsupported privileges reported through capability discovery.

### Exit criteria

- `SIM`: the MuJoCo asset passes joint/frame identity, unit, transmission, actuator/sensor mapping, and initial-state checks against the canonical product model;
- `SIL`: the headless MuJoCo scenario repeats within declared state, timing, and event-order tolerances in CI;
- `SIL`: RT/runtime communication remains bounded under client and middleware overload, and stale, expired, or wrong-timeline commands are rejected;
- `SIL`: simulation reset changes the control-timeline identity and rejects old-timeline commands; historical recordings remain intact and segmented for replay, while the accepted time/lifecycle ADR defines how boot, runtime restart, replay, and lease generations relate to that identity;
- `PHYSICAL`: one actuator and a fixture with at least three simultaneously controlled axes use the same versioned Plant state/command semantics as the MuJoCo path;
- `HIL` and `PHYSICAL`: bus loss, process death, stale commands, and watchdog expiry reach the declared safe bench state within measured bounds;
- `HIL`: signed host and device artifacts are verified during boot/activation, altered or disallowed rollback artifacts are rejected, and an interrupted update recovers without vendor-only reflashing; a test that exercises only the updater cannot claim boot-chain enforcement;
- `PHYSICAL`: a retained trace correlates requested, admitted, safety-output, and applied commands, including runtime/safety/lower-authority reasons, with hardware timestamps, bus health, power/thermal data, boot reason, and artifact hashes;
- `PHYSICAL`: the multi-axis fixture measures cycle jitter/skew and synchronization loss while executing at least one coordinated or mechanically coupled motion;
- every claimed L-2/L-1/L0 capability is backed by the component matrix and test evidence; gaps remain explicit and are not inferred from joint-command access;
- the core crates have no ROS 2 dependency.

## Phase 2 — Bottom-up-qualified whole-robot reference

### Objective

Move from the lower-stack fixture to a whole-robot embodiment that exposes enough of the relevant mechanics, electronics, firmware, bus, HAL, model, and calibration path to validate Soma bottom-up. Candidates with substantial public lower-stack artifacts include [Berkeley Humanoid Lite](https://github.com/HybridRobotics/Berkeley-Humanoid-Lite) and its [Recoil motor controller](https://github.com/T-K-233/Recoil-Motor-Controller-BESC) for a legged path, or [Reachy 2](https://github.com/pollen-robotics/reachy2_core) with public [Poulpe firmware](https://github.com/pollen-robotics/firmware_Poulpe) and [EtherCAT controller](https://github.com/pollen-robotics/poulpe_ethercat_controller) for a manipulation path. Each candidate still requires component-level verification; these sources do not by themselves establish an open safety, power, trust-key, or full OTA chain.

A remaining closed component does not invalidate the platform, but it limits the layers and claims the platform can qualify. Those limits must be published.

### Scope

- whole-robot real HAL and synchronized multi-device control;
- target-compute RT configuration, latency, IRQ, and resource characterization;
- robot instance identity, device inventory, model and calibration pipeline;
- independent stop/torque/brake/energy-control path and platform safety integration;
- whole-robot MuJoCo model and matching SIL scenarios;
- local observability, crash-safe flight recording, signed OTA, and recovery;
- locomotion or manipulation capability sufficient to exercise coordinated control rather than isolated joints.

### Exit criteria

- `SIL` and `PHYSICAL`: the same external SDK semantics and capability discovery run a matched scenario against simulation and the whole robot;
- `PHYSICAL`: control-loop timing, jitter, synchronization, power, and thermal behavior are measured on target compute and stored with the platform manifest;
- `HIL`: communication loss, controller/runtime failure, sensor invalidity, and every interrupted-update state reach the defined inhibited/recovery state with retained evidence; destructive power-cut update tests may remain on a constrained target-equivalent rig;
- `PHYSICAL`: risk-assessed communication, control, and sensor-fault scenarios on the whole robot reach their defined states within measured bounds without requiring destructive update testing on a moving machine;
- `PHYSICAL`: model and calibration identity are verified at activation, and incompatible or stale artifacts are rejected before actuation;
- `HIL`: failed host and claimed device-firmware updates recover without vendor-only reflashing or loss of device identity;
- recorded incidents include enough requested-to-applied command, bus, firmware, boot, power, thermal, lifecycle, and safety evidence for root-cause analysis;
- all component-matrix rows have a known status, and source/build/update/key ownership is demonstrated for every lower layer described as open;
- the cumulative Bottom-Up Reference Profile has verified coverage for every required row, including any supplementary rig evidence;
- no private vendor daemon or undocumented service is required below a layer Soma claims to own.

## Phase 3 — Closed commercial boundary compatibility

### Objective

Prove that Soma can integrate useful commercial robots without confusing SDK compatibility with bottom-up ownership.

### Scope

- add at least one commercial platform whose accessible boundary may be a privileged joint API, whole-robot motion API, vendor daemon, or ROS 2 integration;
- classify the adapter as either a `NativePlantAdapter`, which enters Soma below `robot-rt` and can exercise Soma `SA-3`, or a `ManagedMotionGateway`, which terminates above the Plant boundary and retains vendor motion/safety authority;
- terminate proprietary protocol and product-specific types inside the correctly classified adapter rather than calling every boundary a HAL;
- publish conservative capabilities and authorization for the accessible boundary;
- preserve vendor safety, commissioning, firmware, and recovery authority rather than bypassing it;
- reuse protocol, SDK, lifecycle, recording, and matched SIL scenarios wherever the accessible boundary permits.

### Exit criteria

- `PHYSICAL`: the common SDK runs its declared supported subset against the bottom-up-qualified reference and commercial platform, while unsupported operations return stable typed errors;
- the commercial adapter publishes explicit capability and component-matrix limits, including unverified drive firmware, boot keys, BMS, raw bus, E-stop, or STO access;
- proprietary types, transport, and vendor daemon assumptions do not leak into common controller, runtime, protocol, or client packages;
- `SIL` and `PHYSICAL`: a matched conformance scenario produces comparable command, feedback, lifecycle, and incident records at the exposed boundary;
- vendor safety and update mechanisms remain authoritative where Soma lacks ownership, and tests do not claim validation of inaccessible lower layers;
- a `ManagedMotionGateway` claims only the command, feedback, lifecycle, and safety-intervention semantics observable at its exposed boundary, never Soma Plant or `SA-3` coverage;
- adding the commercial platform requires an adapter and Product Profile, not a fork of the common runtime or SDK.

## Phase 4 — Fleet production and operations

### Objective

Scale and harden the safety, identity, signing, update, observability, and compatibility mechanisms established in Phases 0–2. Phase 4 does not introduce the trust chain for the first time.

### Scope

- manufacturing identity provisioning and ownership lifecycle;
- separation and protection of development, release, recovery, and fleet authorization roles;
- production secure-boot enforcement, anti-rollback, key rotation/revocation, and recovery ceremonies;
- staged/canary fleet rollout with compatibility manifests and automated rollback policy;
- OpenTelemetry-based service observability and local crash-safe MCAP flight recorder;
- incident upload, retention, access control, and release/boot/mission/action correlation;
- SBOM, provenance, reproducible builds, vulnerability response, and signed artifacts;
- compatibility, deprecation, and LTS policy;
- SIM/SIL/HIL/PHYSICAL release-qualification gates;
- ROS 2 adapters for selected distributions.

### Exit criteria

- a release is reproducibly built and signed, passes its declared SIM/SIL/HIL/PHYSICAL gates, rolls through a canary cohort, and can be halted or rolled back without losing local robot safety;
- a compromised, revoked, expired, incompatible, or rollback-disallowed artifact is rejected with an auditable reason;
- identity provisioning, key rotation, recovery, ownership transfer, and decommissioning are exercised in runbooks and representative tests;
- an incident can be correlated across hardware/model identity, release, boot, mission, lease, command, applied action, and safety event;
- robot core operation and independent safe-state mechanisms remain available when fleet/cloud services are unavailable;
- a commercial boundary adapter can participate only within its declared update, identity, observability, and safety capabilities.

## Near-term workstreams

### A. Contracts and assurance

- canonical L-2–L5 vocabulary;
- RT state/command and requested-to-applied lineage;
- Plant, HAL, device-management, clock/timeline, lifecycle/fault, lease, and capability contracts;
- `ProductModelManifest`, `RobotInstanceManifest`, `DeviceInventory`, `CalibrationSet`, `ControlProfile`, independently governed `SafetyProfile`, `RuntimeCapabilitySnapshot`, and component matrix;
- evidence schema and conformance report format.

### B. Safety and trust bootstrap

- hazard and safe-state analysis;
- independent E-stop/power-isolation bench design;
- development signing root and role separation;
- device identity, signed update, anti-rollback, recovery, and boot-reason reporting.

### C. MuJoCo/SIL vertical slice

- `robot-rt`, `robot-runtime`, shared-memory IPC, and Zenoh prototype;
- Python client and lease/arbitration;
- deterministic scenario runner, replay, and fault injection;
- MuJoCo Plant backend and Isaac/Genesis adapter contract.

### D. Owner-controlled lower-stack vertical slice

- actuator/electronics selection through the component matrix;
- single-joint safety fixture, then synchronized multi-axis bench;
- CAN-FD/EtherCAT HAL, firmware build/update, timestamps, calibration, and diagnostics;
- HIL and physical watchdog, bus-fault, power-fault, update-recovery, and flight-recorder tests.

### E. Platform conformance

- reusable SIM/SIL/HIL/PHYSICAL scenarios;
- capability and Product Profile validation;
- bottom-up whole-robot qualification;
- closed commercial boundary/adapter compatibility after the qualified reference path.

## Definition of success

Soma succeeds if one versioned production architecture survives changes in robot embodiment, hardware generations, simulators, middleware integrations, and application ecosystems while preserving explicit ownership boundaries. Its evidence must distinguish what was proven in simulation, software, electronics, and physical hardware, and it must never infer lower-stack openness or safety authority from an SDK-shaped joint command.
