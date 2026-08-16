# Safety and Fault Architecture

> Status: Deep Research. Goal: define which safety responsibilities belong in hardware, servo drives, `robot-rt`, runtime, and application layers before Soma exposes low-level control externally.

## Question

How should Soma structure safety and fault management across very different embodiments — wheeled bases, AMRs, legged robots, mobile manipulators, and future humanoids — without pretending that one software clamp or one standard covers every robot?

## Executive conclusion

Soma should use a **layered safety authority model** driven by risk assessment and explicit safety functions.

> **Safety is not one module. It is a chain of independent and partially redundant mechanisms with different trust levels, reaction times, and failure assumptions.**

A practical baseline is:

```text
Application / AI
      |
Public command + lease admission
      |
robot-runtime policy / mode authority
      |
robot-rt Safety Supervisor (non-safety-rated protective software by default)
      |
Drive / actuator protection
      |
Independent Safety MCU / safety controller
      |
STO / brake / power isolation / physical E-stop
```

Higher layers may request motion; lower-trust layers must not be able to bypass the safety functions beneath them.

The safety-authority names in this document use the `SA-*` prefix so they do not collide with Soma's platform/software layers (`L-2` through `L5`). `SA-*` describes authority and independence, not a process placement or a certification claim.

## Start from hazard analysis, not interface design

ISO 12100 defines a general machinery methodology around hazard identification, risk estimation/evaluation, risk reduction, documentation, and verification. Soma should use this mindset even before formal certification is in scope.

The architecture should begin with:

```text
hazard
 -> hazardous situation
 -> required risk reduction
 -> safety function
 -> implementation allocation
 -> diagnostic coverage / fault assumptions
 -> verification method
```

Examples vary sharply by embodiment:

### Wheeled AMR

- collision with people;
- runaway motion;
- brake failure;
- cliff/drop risk;
- unexpected restart;
- unsafe maintenance/manual mode.

### Legged robot

- fall/collapse;
- high-energy limb impact;
- loss of balance after communications failure;
- thermal/current runaway;
- unsafe recovery/get-up behavior;
- pinching/contact hazards.

### Mobile manipulator

Combines mobile-platform hazards with manipulator reach, collision, grasped-object, and tool hazards.

There should therefore be a common Soma safety framework plus **product-specific safety case and safety functions**.

## Standards landscape

Soma should treat standards as design inputs, not claim generic compliance.

- **ISO 12100** — general machinery risk assessment/risk reduction methodology.
- **ISO 13849-1:2023** — methodology for safety-related parts of control systems and performance levels.
- **ISO 10218-1/2:2025** — industrial robot and application/cell safety; importantly, the scope excludes many public/service/mobile cases.
- **ISO 3691-4:2023** — driverless industrial trucks / AGVs / AMRs and their systems.
- **ISO 13482:2014** — personal-care robots; a newer service-robot edition is under development in 2026.

The product context determines which Type-C standards and regulatory requirements matter. A generic reference platform should remain standard-aware but product-neutral.

## Safety authority layers

### SA-0: independent stop and energy-control path

Examples:

- physical emergency stop;
- Safe Torque Off or equivalent drive safety input;
- brake control designed for a safe failure mode;
- contactor / power-path isolation where needed.

This path must not depend solely on Linux, Zenoh/DDS, Python, or an application process.

Physical E-stop, STO or equivalent torque inhibition, required energy isolation, and safe brake behavior must remain effective when the main compute is hung, rebooting, compromised, or unpowered. These are distinct functions and final elements: E-stop initiates a stop function, STO prevents drive-generated torque, a brake controls motion, and a contactor may isolate a power path. One does not imply the others or a complete safe state. A Linux GPIO handler is not an independent E-stop chain.

### SA-1: independent safety controller

A separate Safety MCU / safety PLC / certified subsystem may own functions such as:

- E-stop chain monitoring;
- watchdog of main compute;
- enabling power only in valid state;
- brake/STO sequencing;
- physical safety inputs;
- redundant speed/position limits where product risk requires it;
- independent reset/restart rules.

The exact implementation depends on the required safety integrity/performance level; Soma only defines the architectural interface.

The watchdog that removes drive enable or initiates a required safe action after loss of main-compute supervision belongs on this independent path. Its final action must not require `robot-rt`, `robot-runtime`, a network service, or Linux scheduling to run.

### SA-2: drive/device protection

Servo drives and device MCUs should own the fastest local protections:

- over-current;
- over-voltage/under-voltage;
- over-temperature;
- encoder plausibility;
- hardware position/speed limits where available;
- communication watchdog;
- motor/drive fault state.

Host software should consume these faults but should not reimplement every electrical protection at 1 kHz.

### SA-3: `robot-rt` Safety Supervisor

This is the most trusted general-purpose host-software protection layer, but it is **non-safety-rated by default**. Unless a product-specific safety case qualifies the complete hardware/software/toolchain, `robot-rt` must not be treated as the sole implementation of a required safety function.

Responsibilities can include:

- command deadline/Plant-timeline validation;
- enforcement of active `SafetyProfile` joint position/velocity/torque limits;
- whole-body power/thermal budget;
- self-collision envelope;
- dynamic stability constraints;
- base speed/acceleration envelope;
- fall/contact detection;
- sensor validity and stale-state handling;
- RT deadline/watchdog handling;
- transition to predefined safe/degraded behaviors.

This layer must execute locally and remain effective if `robot-runtime`, the SDK client, ROS 2, or the network disappears.

Conversely, loss, deadlock, or corruption of `robot-rt` must be detected below Linux by the SA-1/SA-2 watchdog and enable chain. SA-3 improves protection, control quality, diagnostics, and graceful degradation; it does not replace independently wired E-stop/STO/energy-isolation mechanisms.

### SA-4: runtime authority and mode policy

`robot-runtime` handles lower-integrity operational rules:

- leases / command-source arbitration;
- mode permissions;
- user/developer authorization;
- application capability limits;
- action lifecycle;
- maintenance/developer mode gates.

A runtime policy can make the system safer, but failure of runtime must be safely handled below it.

### SA-5: applications

Navigation, grasping, AI policies, Python code, ROS applications, and cloud systems are **not safety authorities**. They operate inside constraints provided by lower layers.

## SafetyProfile governance

Safety-authoritative limits and safe-behavior selection must not be ordinary fields in a shared robot model or a freely replaceable controller/model bundle. Soma should represent them in a separately governed `SafetyProfile` that includes, at minimum:

```text
safety_profile_id / schema_version
compatible product, hardware, and safety-controller identities
authoritative position / velocity / torque / power limits
mode-specific envelopes and derating rules
fault-to-safe-behavior mappings
required safety inputs / devices
activation and rollback policy
content checksums
signatures and signer role
```

`SafetyProfile` has an independent release, review, signature, authorization, and activation lifecycle. Switching a `ProductModelManifest`, simulator asset, policy, or ordinary `ControlProfile` must never silently change the active safety authority. Simulation may add a separately identified and hashed `TestConstraintSet`; it cannot mutate, relax, or reuse the identity of a deployable `SafetyProfile`.

`robot-rt` consumes a validated profile and reports its ID/hash in decisions and recordings. The effective safety envelope is the intersection of that profile and all applicable independent SA-0/SA-1/SA-2 bounds; host activation can never widen a lower-authority constraint. Accepting a profile in host software is not proof that those mechanisms were updated or that a product safety function is satisfied.

## Requested, admitted, safety-output, and applied commands

Safety intervention must be observable:

```text
Requested Command
       |
SA-4 authority/mode/timing validation
       |
Admitted Command
       |
SA-3 safety envelope
       |
Safety Output Command
       |
controller/drive constraints
       |
Applied Command
```

Admission rejection, safety modification, and lower-authority constraint should each produce a structured reason such as:

```text
JOINT_VELOCITY_LIMIT
POWER_BUDGET_DERATE
SELF_COLLISION_ENVELOPE
STALE_STATE
BASE_SPEED_SAFETY_ZONE
THERMAL_DERATE
```

This is critical for debugging learned policies and external SDK behavior.

## Safe states are product-specific

Do not hard-code `STOP = zero torque` as a universal safe state.

Examples:

### Wheeled robot

Possible safe state:

- command zero velocity;
- controlled deceleration;
- brake engage;
- drive disable if required.

### Biped

Immediate torque-off can itself cause a dangerous collapse. Depending on the fault and mechanical design, a safer response could be:

- freeze/hold briefly;
- damping mode;
- controlled crouch/kneel;
- fall-management behavior;
- then disable torque.

### Manipulator

Holding a payload, gravity, brakes, and tool hazards affect the safe response.

Therefore Soma needs a **SafeBehavior policy interface**, implemented per embodiment/product and approved independently of arbitrary application code.

Deployable SafeBehavior selection and its authoritative bounds belong to the governed `SafetyProfile`; the implementation may reside across SA-0 through SA-3 according to the product safety case.

## Fault taxonomy

A single `error_code` integer is inadequate.

Recommended axes:

### Origin

```text
hardware
drive
bus
sensor
rt/controller
safety
runtime
application
security/update
```

### Severity

Example product-level scale:

```text
INFO
WARNING
DEGRADED
STOP_REQUIRED
EMERGENCY
```

### Persistence

```text
transient
persistent
latched
```

### Recoverability

```text
auto-recoverable
operator-reset
service-required
power-cycle-required
non-recoverable
```

### Scope

```text
component
control domain
whole robot
```

These dimensions should be structured fields, not encoded into free-form names.

## Fault lifecycle

```text
DETECTED
  |
  +--> ACTIVE -----> CLEARED
  |       |
  |       +--------> LATCHED -> ACK/RESET -> CLEARED
  |
  +--> INVALIDATED / DUPLICATE (diagnostic processing)
```

A Fault object should retain:

```text
fault_id
fault_code
component_id
first_seen / last_seen
count
severity
latching
recoverability
active
associated safety action
related release/update/action/lease/boot IDs
```

## Degraded operation

Production robots should not treat all faults as binary RUN/DEAD.

Potential states:

```text
NORMAL
DEGRADED
CONTROLLED_STOP
SAFE_DISABLED
EMERGENCY_STOPPED
SERVICE_MODE
```

Examples of degradation:

- one nonessential camera unavailable;
- reduced actuator thermal envelope;
- localization degraded but manual/local motion still allowed;
- one arm disabled while mobile base remains available;
- network unavailable while local autonomous task continues.

The Capability service should reflect runtime degradation so clients do not keep requesting unavailable functions.

## Mode and safety state must be different

Do not overload one enum for everything.

Example:

```text
OperationalMode:
  IDLE / MANUAL / AUTONOMOUS / RESEARCH / MAINTENANCE

SafetyState:
  SAFE / DEGRADED / STOPPING / STO_ACTIVE / ESTOP_ACTIVE / FAULTED
```

An application mode change can be allowed or denied based on SafetyState.

## Watchdog hierarchy

A robust robot uses multiple watchdogs:

```text
application heartbeat        low trust
runtime heartbeat            -> robot-rt
robot-rt heartbeat           -> safety MCU / drive enable chain
drive bus watchdog           -> local drive safe action
safety controller self-test  -> physical safe output
```

Timeouts must be chosen according to reaction-time requirements. They should not all be the same generic “1 second heartbeat.”

The final watchdog and disable path must be independent of Linux. A host watchdog process that shares the same kernel, scheduler, power rail, or failure domain as `robot-rt` is useful diagnostics, but it does not satisfy this independence requirement.

## Low-level SDK / research mode

If Soma exposes joint impedance/torque commands externally, research mode should require explicit gating:

- supported hardware SKU/configuration;
- physical E-stop;
- safe workspace / support rig where applicable;
- authenticated developer role;
- explicit activation;
- restricted network/topology if required;
- command TTL at RT boundary;
- governed `SafetyProfile` hard position/velocity/torque/power limits;
- self-collision/stability constraints where feasible;
- immutable safety authority below external controller;
- complete audit/flight recording.

External access to host real-time control must never imply bypassing SA-0 through SA-3 protection.

## Security-safety interaction

ISO 13849 notes that cybersecurity issues can affect safety functions even though security is outside its direct scope.

Soma should assume that:

- stolen control credentials can become a safety hazard;
- unauthorized developer mode is a safety hazard;
- OTA compromise can alter safety code;
- denial of service can trigger unsafe loss-of-command behavior if watchdog policy is poorly designed.

Safety architecture therefore depends on secure identity, signed updates, least privilege, and fail-safe network-loss semantics.

## Simulation and HIL

Safety behavior must be testable before physical incidents.

### SIL fault injection

- stale command;
- old Plant timeline;
- controller deadline miss;
- sensor freeze/bias;
- actuator saturation;
- thermal/power derating;
- simulated fall/contact;
- runtime disconnect.

### HIL

- EtherCAT slave loss;
- CAN bus-off;
- drive watchdog;
- safety MCU timeout;
- E-stop/STO chain;
- brake sequencing;
- power-cycle behavior.

Simulation should use the same fault/event codes where semantics are equivalent.

## Safety evidence and observability

For every intervention capture:

```text
what was requested
what state was observed
which rule/function triggered
what was admitted, emitted by safety, and applied
which safe behavior ran
time/Plant-timeline/tick
hardware, product-model, calibration, safety-profile, and release identity
```

This should feed MCAP flight recording plus structured Fault/Event telemetry.

## Proposed Soma safety interfaces

```text
SafetyInput
  RobotState
  AdmittedCommand
  OperationalMode
  Lease/authority summary
  PlantHealth

SafetyDecision
  disposition: ACCEPT / MODIFY / REJECT / STOP
  safety_output_command
  safety_actions[]
  reasons[]
  resulting_safety_state
  safety_profile_id / hash
```

The detailed implementation remains embodiment-specific, but the decision/audit semantics can be common.

## ADR implications

This research supports decisions roughly equivalent to:

1. Physical E-stop/STO/critical energy isolation must not depend only on Linux.
2. Device/drive protection, independent safety controller, and RT software safety are distinct authority layers.
3. `robot-rt` is non-safety-rated protective software by default and cannot be the sole implementation of a required safety function.
4. Safety-authoritative limits and SafeBehavior mappings live in an independently governed `SafetyProfile`, not an ordinary model/control bundle.
5. Public low-level control cannot bypass safety authority.
6. Faults and safety interventions use structured lifecycles/codes.
7. Safe behavior is embodiment/product-specific rather than universally “zero torque.”
8. Safety requirements are derived from risk assessment and target product standards.

## Experiments / work products required

1. Create a Hazard Analysis template for Soma reference embodiments.
2. Define first 20–30 structured fault/safety codes.
3. Implement stale-command + velocity/torque envelope in MuJoCo Plant.
4. Kill `robot-runtime` and prove `robot-rt` remains safe.
5. Kill `robot-rt` and verify independent watchdog/Plant mock transitions correctly.
6. HIL prototype for E-stop/STO/watchdog behavior on first real platform.
7. Define research-mode activation checklist.

## Primary references

- ISO 12100:2010 — risk assessment and risk reduction: https://www.iso.org/standard/51528.html
- ISO 13849-1:2023 — safety-related control systems: https://www.iso.org/standard/73481.html
- ISO 10218-1:2025 — industrial robot safety: https://www.iso.org/standard/73933.html
- ISO 10218-2:2025 — industrial robot applications/cells: https://www.iso.org/standard/73934.html
- ISO 3691-4:2023 — driverless industrial trucks/AMRs: https://www.iso.org/standard/83545.html
- ISO 13482:2014 — personal care robots: https://www.iso.org/standard/53820.html
- ISO/FDIS 13482 — next service-robot edition under development: https://www.iso.org/standard/83498.html
- ISO/TR 23482-1:2020 — safety-related test methods: https://www.iso.org/standard/71564.html
