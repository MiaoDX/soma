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
robot-rt Safety Supervisor
      |
Drive / actuator protection
      |
Independent Safety MCU / safety controller
      |
STO / brake / power isolation / physical E-stop
```

Higher layers may request motion; lower-trust layers must not be able to bypass the safety functions beneath them.

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

### Layer 0: physical energy isolation

Examples:

- physical emergency stop;
- Safe Torque Off or equivalent drive safety input;
- brake control designed for a safe failure mode;
- contactor / power-path isolation where needed.

This path must not depend solely on Linux, Zenoh/DDS, Python, or an application process.

### Layer 1: independent safety controller

A separate Safety MCU / safety PLC / certified subsystem may own functions such as:

- E-stop chain monitoring;
- watchdog of main compute;
- enabling power only in valid state;
- brake/STO sequencing;
- physical safety inputs;
- redundant speed/position limits where product risk requires it;
- independent reset/restart rules.

The exact implementation depends on the required safety integrity/performance level; Soma only defines the architectural interface.

### Layer 2: drive/device protection

Servo drives and device MCUs should own the fastest local protections:

- over-current;
- over-voltage/under-voltage;
- over-temperature;
- encoder plausibility;
- hardware position/speed limits where available;
- communication watchdog;
- motor/drive fault state.

Host software should consume these faults but should not reimplement every electrical protection at 1 kHz.

### Layer 3: `robot-rt` Safety Supervisor

This is the highest-trust general software safety layer.

Responsibilities can include:

- command deadline/epoch validation;
- joint position/velocity/torque limits;
- whole-body power/thermal budget;
- self-collision envelope;
- dynamic stability constraints;
- base speed/acceleration envelope;
- fall/contact detection;
- sensor validity and stale-state handling;
- RT deadline/watchdog handling;
- transition to predefined safe/degraded behaviors.

This layer must execute locally and remain effective if `robot-runtime`, the SDK client, ROS 2, or the network disappears.

### Layer 4: runtime authority and mode policy

`robot-runtime` handles lower-integrity operational rules:

- leases / command-source arbitration;
- mode permissions;
- user/developer authorization;
- application capability limits;
- action lifecycle;
- maintenance/developer mode gates.

A runtime policy can make the system safer, but failure of runtime must be safely handled below it.

### Layer 5: applications

Navigation, grasping, AI policies, Python code, ROS applications, and cloud systems are **not safety authorities**. They operate inside constraints provided by lower layers.

## Requested, accepted, and applied commands

Safety intervention must be observable:

```text
Requested Command
       |
Authority/mode validation
       |
Safety envelope
       |
Accepted Command
       |
controller/drive constraints
       |
Applied Command
```

A safety modification should produce a structured reason such as:

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

## Low-level SDK / research mode

If Soma exposes joint impedance/torque commands externally, research mode should require explicit gating:

- supported hardware SKU/configuration;
- physical E-stop;
- safe workspace / support rig where applicable;
- authenticated developer role;
- explicit activation;
- restricted network/topology if required;
- command TTL at RT boundary;
- hard position/velocity/torque/power limits;
- self-collision/stability constraints where feasible;
- immutable safety authority below external controller;
- complete audit/flight recording.

External access to L1 must never imply bypassing drive/safety-controller protection.

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
- old epoch;
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
what was accepted/applied
which safe behavior ran
time/epoch/tick
hardware/model/release identity
```

This should feed MCAP flight recording plus structured Fault/Event telemetry.

## Proposed Soma safety interfaces

```text
SafetyInput
  RobotState
  RequestedCommand
  OperationalMode
  Lease/authority summary
  PlantHealth

SafetyDecision
  disposition: ACCEPT / MODIFY / REJECT / STOP
  accepted_command
  safety_actions[]
  reasons[]
  resulting_safety_state
```

The detailed implementation remains embodiment-specific, but the decision/audit semantics can be common.

## ADR implications

This research supports decisions roughly equivalent to:

1. Physical E-stop/STO/critical energy isolation must not depend only on Linux.
2. Device/drive protection, independent safety controller, and RT software safety are distinct authority layers.
3. Public low-level control cannot bypass safety authority.
4. Faults and safety interventions use structured lifecycles/codes.
5. Safe behavior is embodiment/product-specific rather than universally “zero torque.”
6. Safety requirements are derived from risk assessment and target product standards.

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
