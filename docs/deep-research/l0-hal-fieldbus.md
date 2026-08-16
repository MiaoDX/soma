# Below the Plant: Firmware, HAL, and Fieldbus

> Status: Deep Research baseline. This document preserves the design space and evidence behind Soma's hardware boundary; it is not yet an ADR.

## Question

Where should Soma draw the boundary between physical hardware and the software robot, and how should board firmware, EtherCAT, CAN/CAN-FD, device drivers, transmissions, real-time scheduling, independent safety, and simulation fit beneath that boundary?

## Working thesis

Soma should distinguish **board/device firmware**, the host **HAL**, and the more general **Plant contract**. These responsibilities map to the canonical layers in [Layering and Trust Boundaries](../architecture/layering-and-trust-boundaries.md):

- **L-2** owns independent E-stop/STO/energy isolation and trust anchors that ordinary host software cannot bypass.
- **L-1** owns bootloaders, board/drive firmware, FOC, sensor acquisition, BMS behavior, local watchdogs, and device faults.
- The **L0 HAL** represents host-side integration with real hardware: fieldbus masters, device drivers, discovery, transmission mapping, calibration application, and hardware health.
- The **Plant contract** is the bounded control-facing boundary consumed at L1. A Plant may be backed by real hardware, MuJoCo, an external simulator through an RT-safe proxy, or HIL.
- Control algorithms should depend on the Plant contract, not on EtherCAT PDOs, CAN identifiers, ROS messages, or vendor SDK types.

```text
L1 Controller / Estimator / SA-3
             |
        Plant Contract
       /              \
Hardware Plant      Simulated Plant
      |             /     |      \
   L0 HAL       MuJoCo   RT-safe external proxy
      |
Bus + Device + Transmission
      |
EtherCAT / CAN-FD / SPI
      |
L-1 board / drive / sensor / BMS firmware

L-2 independent safety and trust constrains the physical path
```

## Below the Plant spans multiple layers

A useful decomposition is:

1. **Independent safety and trust (L-2)** — distinct stop, torque-inhibition, braking and energy-control final elements; independent enable/watchdog paths; hardware identity and recovery authority.
2. **Power electronics and motor control (L-1)** — PWM, current sensing, FOC, encoder sampling. Usually runs on an MCU or dedicated servo drive at tens of kHz.
3. **Servo/device firmware abstraction (L-1)** — turns electronics into a controlled device with commands, state, local faults, watchdogs, update/recovery, and thermal/electrical protection.
4. **Fieldbus transport and master (L0)** — moves bounded cyclic command/state data between many devices and the robot computer.
5. **Device drivers and inventory (L0)** — understand a specific drive, IMU, force sensor, hand, BMS, or power controller and report the actual hardware/firmware identities present.
6. **Transmission and calibration application (L0)** — maps motor-space to joint-space; gearbox, differential, tendon, SEA, coupled wrist, and serial-specific corrections.
7. **Robot HAL / HardwarePlant (L0)** — aggregates physical devices into a coherent hardware state, command, lifecycle, capability, and health surface.
8. **Plant contract (L0/L1 boundary)** — removes the distinction between real and simulated controlled systems for the controller without claiming that their lower implementations are equivalent.

This decomposition avoids both the common error `motor[i] == joint[i]` and the claim that a host joint-command API implies an open lower stack. A physical joint can contain multiple encoders, brakes, torque sensors, or coupled motors, while one motor can participate in multiple generalized coordinates. Access to `q/dq/tau/kp/kd` does not by itself establish access to drive firmware, boot keys, BMS, raw bus diagnostics, or the independent safety path.

## Canonical RT data

The cyclic data path should be fixed-size and allocation-free. Conceptually:

```rust
#[repr(C)]
pub struct RtJointState {
    pub position: f64,
    pub velocity: f64,
    pub estimated_torque: f64,
    pub status_bits: u32,
}

#[repr(C)]
pub struct RtJointCommand {
    pub position: f64,
    pub velocity: f64,
    pub feedforward_torque: f64,
    pub kp: f64,
    pub kd: f64,
    pub mode: u16,
}
```

The exact command model is embodiment/device dependent, but the system contract also needs fields that are often omitted by research SDKs:

- Plant timeline and sequence;
- target/apply tick;
- validity/deadline;
- control mode;
- command source/authority;
- validity masks;
- requested, admitted, safety-output, and applied command correlation.

## Device lifecycle and safe-output contract

Every physical HAL/device adapter must expose a uniform, testable lifecycle rather than only cyclic reads and writes:

```text
discover / inventory / configure
enable / disable / safe_output
fault_latch / fault_readback / authorized_fault_reset
firmware and bootloader identity
calibration identity and readback
device/bus time-sync health
watchdog configuration and observed state
```

`safe_output` is a product/device-specific inhibited command state, not a synonym for E-stop, STO, braking, or power isolation. After process restart, bus recovery, device reboot, ABI/generation mismatch, or fault reset, the adapter remains disabled/latched until the explicit lifecycle and re-arm conditions pass. HIL conformance injects Working Counter/DC loss, PDO/CAN loss, bus-off, slave-state fallback, drive reboot, and stale/corrupt inventory.

## EtherCAT

EtherCAT is a strong default candidate for dense, synchronized actuator networks such as humanoids because the important property is not raw Ethernet bandwidth but deterministic cyclic process data, topology diagnostics, and distributed clock synchronization.

Candidates to evaluate:

- **IgH/EtherLab EtherCAT Master** — mature Linux-oriented master and a useful reference for PREEMPT_RT deployments.
- **SOEM** — compact user-space EtherCAT master; easy to embed, but current licensing must be reviewed for commercial use.
- **EtherCrab** — Rust EtherCAT master worth benchmarking for a Rust-first Soma implementation.

Selection must be empirical. Required tests include:

- supported NICs and slaves;
- Distributed Clocks behavior;
- p50/p99/p99.9/max cycle latency;
- recovery after slave loss/rejoin;
- cable faults and Working Counter errors;
- startup/configuration time;
- CPU load under realistic topology;
- diagnostics quality;
- licensing and long-term maintenance.

A pure-Rust implementation is desirable but is not a sufficient reason to reject a more mature C implementation. Vendor APIs should be isolated behind narrow `*-sys`/FFI crates.

## CAN and CAN-FD

CAN-FD is attractive for hands, power electronics, peripheral controllers, low-DOF robots, and integrated servos where its bandwidth and arbitration model are sufficient.

On Linux, Soma should normally build on **SocketCAN** rather than inventing a proprietary host CAN stack. Device protocol parsing belongs above the transport.

```text
Actuator Driver
      |
   CanFdBus
      |
   SocketCAN
      |
 Linux driver
      |
CAN controller
```

Projects such as **mjbots/moteus** are valuable because they expose the entire actuator boundary: servo electronics, FOC firmware, CAN-FD protocol, host client, and robot-level use. This makes moteus a better L-1/L0 reference than SDKs that expose only host-side joint commands.

## Reference implementation: Open Dynamic Robot Initiative

The Open Dynamic Robot Initiative is one of the strongest public references for studying L-1/L0 end-to-end. Its ecosystem exposes motor driver electronics/firmware, master board, power board, host-side interfaces, and dynamic robot examples.

The important lesson is architectural rather than component-specific: high-frequency electrical control remains local to the actuator, while the host receives a bounded actuator abstraction at a lower cyclic rate.

## Real-time host

Soma's first Linux RT reference should assume PREEMPT_RT and explicitly configure:

- isolated/assigned RT CPU cores;
- real-time scheduling priority;
- IRQ affinity;
- `mlockall` / prefaulted memory;
- absolute-time cyclic scheduling;
- no allocation in the cyclic path;
- no blocking locks in the cyclic path;
- bounded computation;
- network/logging/update workloads outside the RT cores.

Rust does not make a system real-time automatically. These operating-system and workload constraints are the actual timing mechanism.

## Process boundary

The recommended production topology remains two major processes:

```text
robot-rt                           robot-runtime
---------                          -------------
Plant/HAL                          protocol
estimator                          lease/arbitration
controller                         actions
safety                             telemetry
watchdog                           SDK/network
    |                                  |
    +---- fixed SPSC shared memory ----+
```

Middleware should not appear in the 1 kHz servo path.

The RT/runtime IPC contract should define overload behavior explicitly:

- stale states may be dropped in favor of latest state;
- RT must never block because runtime is slow;
- commands must carry robot-local deadline and Plant-timeline identity;
- loss of runtime must trigger a locally defined safe behavior;
- ring overflow and command age are observable metrics.

## Safety boundary

Software safety belongs between requested commands and the plant, but hardware emergency protection cannot depend solely on Linux.

```text
Requested command
      |
SA-4 lease/mode/timing admission
      |
Admitted command
      |
SA-3 dynamic safety envelope
      |
Safety-output command
      |
Plant/HAL and lower-authority constraints
      |
Applied evidence
      |
Plant
```

A separate safety path should own, as appropriate to the product, E-stop, STO, brakes, power cut, drive-level limits, and independent watchdogs.

## Simulation implication

Simulation should implement the Plant contract, not fake EtherCAT unless the purpose is HIL.

- MuJoCo/Isaac/Genesis SIL: implement joint/sensor/plant semantics.
- HIL/bus testing: emulate drive state machines, PDOs/CAN frames, watchdogs, and bus faults.

This separation keeps controller validation fast while preserving a path to validate the real bus stack separately.

## Candidate Soma crates

```text
robot-types-rt
robot-plant
robot-hal
robot-transmission
robot-ethercat
robot-canfd
robot-safety
robot-rt-ipc
robot-rt
```

`robot-plant` should not depend on ROS, Zenoh, DDS, Tokio, or Python.

## Open questions / experiments

1. IgH vs SOEM vs EtherCrab on the target NIC/slave topology.
2. Required servo rate by embodiment: 500 Hz, 1 kHz, 2 kHz?
3. Which safety limits run in drive MCU, safety MCU, and host RT respectively?
4. Whether state estimation is inside `robot-rt` or a separately scheduled RT process.
5. Shared-memory ABI strategy across rolling upgrades.
6. How coupled transmissions are represented in the canonical model.
7. Whether external research mode can safely expose direct joint impedance/torque commands.

## Reference projects and sources

- Open Dynamic Robot Initiative — https://open-dynamic-robot-initiative.github.io/
- ODRI master-board — https://github.com/open-dynamic-robot-initiative/master-board
- ODRI power-board — https://github.com/open-dynamic-robot-initiative/power-board
- mjbots/moteus — https://github.com/mjbots/moteus
- Linux SocketCAN documentation — https://docs.kernel.org/networking/can.html
- IgH EtherCAT Master — https://etherlab.org/en_GB/ethercat
- SOEM — https://github.com/OpenEtherCATsociety/SOEM
- EtherCrab — https://github.com/ethercrab-rs/ethercrab
- Linux PREEMPT_RT documentation — https://docs.kernel.org/core-api/real-time/
