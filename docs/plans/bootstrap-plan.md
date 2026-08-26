# Soma Bootstrap Plan

> Status: Approved execution plan, Reachy Mini-only, 2026-08-23.
> The first release has exactly two Plant profiles: Reachy simulation and
> Reachy Mini Lite hardware. Other robot platforms are deliberately out of
> scope. The official Reachy daemon remains a comparison target, not a runtime
> dependency for the native hardware path. The 9-actuator public command, N1
> physical-actuation gate, and four-way official comparison are approved.

## Execution Status

Status: ACTIVE. The fixed profile, `ControlCore`, pinned `ReachySimPlant`, two
Rust process boundary, Protobuf/loopback Zenoh transport, and Python acceptance
scenario are implemented. `scripts/run-sim-scenario` proves the complete
simulation acceptance flow. Remaining required work is the hardware N0/N1
gates, native bus worker and staged motion, followed by the four-way official
comparison.

## Goal

Build one small end-to-end system that can command the same Reachy Mini profile
in MuJoCo and on a connected Reachy Mini Lite:

```text
Python client
    |
    | minimal Protobuf over loopback Zenoh
    v
robot-runtime (Rust, non-RT)
    |
    | bounded command/state mailbox
    v
robot-rt (Rust)
    |
    | Reachy ControlCore + Plant contract
    +-----------------------------+
    |                             |
    v                             v
Reachy MuJoCo Plant        Reachy Hardware Plant
                                  |
                                  v
                         Rust Dynamixel bus adapter
                                  |
                                  v
                         USB-serial / TTL / motors
```

The first physical milestone must prove that the lower path works without
starting the official daemon or importing the official
`reachy_mini_motor_controller` wheel. The official stack is then run as a
separate comparison path over the same trajectories.

## Native Feasibility Gates

Simulation and the read-only hardware probe may proceed independently. No
torque-enabled hardware motion may start until the physical-actuation gate is
approved and the purchased unit passes the read-only gate.

### Gate N0: read-only native proof

A standalone Rust probe must:

1. open the Lite USB serial device (the documented CH343 interface, normally
   discovered as VID `1a86`, PID `55d3`, with an explicit device-path override)
   and record the resolved USB identity/path;
2. configure 1,000,000 baud, 8N1, with an explicit short serial timeout;
3. send Dynamixel Protocol 2.0 ping requests to IDs 10 through 18;
4. read model/firmware identity, position, torque state, voltage and hardware
   errors;
5. audit baud, homing offsets, min/max limits, operating modes, shutdown bits,
   motor assignment and the purchased unit's configuration against the pinned
   Reachy profile;
6. demonstrate exclusive serial ownership during the test through process
   inspection and a failed second open/lock attempt;
7. record raw packets, latency, retries, timeouts, response errors and the
   detected ID set without enabling torque or changing persistent registers.

The probe may use the Apache-2.0 `rustypot` crate as a generic protocol
implementation, but must not depend on the Reachy-specific motor-controller
crate. Reimplementing the packet codec is unnecessary. Using pinned official
configuration and kinematics as attributed reference inputs is allowed and does
not constitute clean-room independence.

N0 stops native implementation and reopens D-22 if the expected ID/model set is
wrong, required configuration cannot be read, serial ownership is unstable, or
the torque-disable state cannot be established. A recoverable permissions or
cabling fault may be fixed and N0 rerun; unexpected hardware/protocol behavior
is not worked around silently.

### Gate N1: physical actuation authorization

Before any torque enable, approve and rehearse a local physical-actuation
profile containing:

- a hazard review for the actual test pose and support fixture;
- operator procedure and workspace exclusion;
- independent power/torque removal reachable by the operator;
- abort behavior for stale state, port loss, timeout, dropped cycles, model or
  configuration mismatch, voltage/error status, and operator stop;
- stage-specific relative-delta bounds derived from the actual actuator and
  mechanism, not a generic amplitude.

This gate authorizes a controlled experiment; it is not physical-safety
qualification.

### Current feasibility conclusion

The evidence currently supports native control:

- the Lite hardware documentation says the controller board exposes a
  Dynamixel Motor TTL connection;
- the official scan tool directly uses `rustypot::Xl330PyController` to ping the
  bus;
- the official motor controller opens the serial port at 1 Mbps and performs
  standard Dynamixel Protocol 2 reads/writes;
- motor setup tools directly change ID, baud rate, offsets, limits, operating
  mode and shutdown configuration.

No private motion daemon or proprietary motor protocol is indicated. This
supports the narrower claim that Soma can own the host process, serial port,
Dynamixel packets, Reachy mapping, lifecycle and timeout behavior. It does not
claim ownership of smart-servo firmware, embedded position/current loops,
controller-board behavior or independent safety authority. The
remaining uncertainty is physical: exact purchased-board wiring, permissions,
power state, calibration and the behavior of the custom plastic-geared
XC330-M288-PG. N0/N1 resolve these uncertainties progressively.

## Reachy Profile Contract

Keep one fixed profile. Do not create a generic robot manifest or multi-robot
adapter framework.

### Actuation

- 9 fixed actuator IDs: body yaw 10, Stewart motors 11--16, antennas 17--18;
- the recommended M1 public command is one fixed 9-element actuator-position
  target in radians, ordered as body yaw, Stewart 1--6, right antenna, left
  antenna;
- position mode is the first supported command mode;
- torque enable/disable is an explicit lifecycle operation, not a generic
  command union;
- current mode and arbitrary register writes stay behind a diagnostic/native
  interface until a concrete use requires them;
- these are actuator-shaft coordinates, not six independent Cartesian or
  human-like head joints; public head-pose commands and IK are deferred until
  the native actuator path is proven.

### State

The first fixed-layout state contains:

- 9 present actuator positions;
- optional present current and voltage fields for hardware diagnostics;
- optional head pose derived by the pinned Reachy kinematics layer for
  comparison, not required for M1 command execution;
- sequence, source timestamp, Plant timeline and state age;
- health and bus status sufficient to report missing motors, stale reads and
  communication errors.

Camera, microphone, speaker and IMU are not part of the cyclic control message.
Camera/audio are deferred bulk/media streams. IMU is optional because Lite and
MuJoCo do not provide the Wireless BMI088 path.

### Commands and safety

- one immediate actuator-position target with sequence and local TTL;
- reset for simulation only;
- explicit enable/disable and stop behavior for hardware bring-up;
- command expiry freezes the active target at the latest validated measured
  actuator positions and reports the transition; it does not automatically
  disable torque and allow the Stewart mechanism to fall;
- all native writes go through one bus owner; the official daemon must be
  stopped before native mode starts.

## Implementation Order

### 0A. Read-only native probe

- implement and run N0 when Lite hardware is available;
- this may proceed alongside simulation work but gates all torque-enabled work;
- do not add a generic public `ActuatorBus`/`ActuatorDevice` hierarchy: keep one
  private Reachy bus worker behind `ReachyNativePlant`.

### 1. Workspace and profile types

- replace the BHL-specific state and command types with the fixed Reachy
  profile;
- keep the existing Rust workspace, Protobuf, Zenoh and bounded SPSC shape;
- define `ReachyActuatorState`, `ReachyActuatorTarget`, `PlantHealth`, lifecycle,
  timeout and applied-command results;
- remove BHL policy/checkpoint requirements from the bootstrap acceptance path.

### 2. Direct MuJoCo Plant

- pin the Reachy Mini SDK commit and exact `scene.xml`/MJCF asset slice;
- load the 9-actuator model in Rust MuJoCo;
- implement the Reachy actuator order, limits and position semantics; defer IK,
  keeping forward kinematics only if needed for official comparison;
- run headlessly first, with an optional viewer outside the periodic path;
- prove reset, movement, state publication, command TTL and measured-position
  hold.

### 3. Soma process path

- implement `robot-rt` ControlCore against `ReachySimPlant`;
- implement `robot-runtime` and the thin Python client;
- run one scenario: connect, observe state, move selected actuators, expire a
  command into measured-position hold, reset, and reject an old-timeline
  command;
- keep camera/audio out of this milestone.

### 4. Native bus worker and hardware bring-up

- after N0 and N1 pass, implement one private Soma-owned Rust Reachy bus worker
  using `rustypot`;
- make the worker own discovery, model checks, position reads, synchronized
  goal writes, torque state, voltage/error reads and bounded retries;
- initially run it as a dedicated L0 bus/I/O worker with bounded mailboxes;
  do not put blocking serial calls directly in the periodic `robot-rt` loop;
- promote through read-only audit, torque-off same-position write/readback,
  one low-risk antenna movement, coupled Stewart movement, then a short complete
  profile trajectory;
- before torque enable: read present position, write that same position, verify
  limits/mode/errors, enable torque, and verify torque readback; only then issue
  a bounded present-position-relative delta;
- block promotion on unresolved motor errors, stale state, port loss, model or
  configuration mismatch, timeout, or failed independent stop rehearsal;
- record command latency, state age, dropped cycles and serial timeout behavior.

### 5. Official comparison

Run the same fixed actuator trajectories in four configurations:

1. `Soma + ReachySimPlant`;
2. official Reachy daemon + official MuJoCo backend;
3. `Soma + ReachyNativePlant` on Lite;
4. official Reachy daemon + Lite hardware.

Compare, at minimum:

- actuator ordering and radians conversion;
- actuator and optional forward-kinematics agreement;
- target-to-state latency and jitter;
- limit and torque-mode behavior;
- stale-command and disconnect behavior;
- motor discovery and error reporting;
- whether native mode can complete all required motion with the official daemon
  fully stopped.

The comparison is an acceptance artifact, not a requirement to preserve API
compatibility with the official SDK.

## Acceptance

The Reachy simulation milestone is complete when:

```text
start robot-rt + robot-runtime
 -> Python connects
 -> typed 9-actuator state arrives
 -> target moves the MuJoCo Reachy model
 -> TTL expiry selects measured-position hold
 -> reset changes the Plant timeline
 -> an old-timeline command is rejected
```

The native hardware milestone is complete only when N0/N1 have passed and the
official daemon is stopped:

```text
Soma probe audits IDs 10..18 and the complete configuration read-only
 -> writes the measured target back while torque is off
 -> verifies independent stop and enables torque explicitly
 -> verifies torque state and moves one antenna by a bounded relative delta
 -> moves the complete profile through a short trajectory
 -> expires a command while enabled and enters the declared measured-position hold
 -> disables torque with readback and completes independent power removal
 -> dependency/process/module audit finds no Reachy daemon or Reachy-specific
    motor-controller package installed, loaded, linked, or spawned
```

Required evidence:

| Claim | Proof |
| --- | --- |
| Reachy model is a usable Plant | Rust headless MuJoCo test with pinned assets |
| Soma process boundary is real | Python -> runtime -> mailbox -> RT -> Plant integration test |
| Native hardware does not require the Reachy-specific driver | dependency/process/module audit plus direct ping/read/write and motion logs |
| Exclusive bus ownership is demonstrated | process inspection plus failed second-open/lock test |
| Commands expire as declared | enabled hardware expiry test, measured-position hold, state age and applied-command record |
| Physical stop works for the experiment | torque-disable readback, bounded post-disable observation and rehearsed independent power removal |
| Official behavior is understood | simulation and Lite trajectory comparison with packet/timing/state differences |

The first real-hardware run is characterization, not a hard-real-time or
physical-safety qualification.

## Explicitly Deferred

| Work | Reopen when |
| --- | --- |
| Other robot platforms, BHL, Atom S, Open Duck, SO-101 | Only after the Reachy two-profile path is complete and a concrete need exists |
| ROS 2, fleet, cloud, OTA and security qualification | A non-local or distributable product workflow exists |
| Generic model manifests and multi-robot conformance | A second active embodiment exists |
| Generic torque/velocity/impedance command unions | Reachy hardware evidence requires one of them |
| Camera bulk transport and audio streaming | The motion path and state timing are stable |
| Wireless CM4 deployment and BMI088 | Lite native path is working and Wireless hardware is available |
| Policy training and sim-to-real learning | A deterministic Reachy control profile exists and a task needs learning |
| Supervisor/recovery, replay and release packaging | The two-process happy path is repeatable |

Deferred items are not backlog. They must not add schemas, fixtures or
extension points before their trigger occurs.

## Non-Goals

- supporting any robot other than Reachy Mini;
- using the official daemon as an invisible dependency in native mode;
- claiming the native serial worker is hard real-time before measurement;
- reproducing every official media, REST, WebSocket or app feature;
- creating a universal Dynamixel API;
- qualifying physical safety, production security or long-duration reliability.

## Definition Of Done

This plan is complete when both `ReachySimPlant` and `ReachyNativePlant` pass
their focused acceptance flows, the native path works with the official daemon
and Reachy-specific motor-controller wheel absent, and the four-way comparison
report explains which host-side behavior is Soma-owned versus retained as
hardware, smart-servo firmware or vendor-device behavior.
