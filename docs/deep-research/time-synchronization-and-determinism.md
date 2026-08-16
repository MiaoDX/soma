# Time, Synchronization, and Determinism

> Status: Deep Research. Goal: define Soma's time model before RT control, multi-sensor fusion, simulation, replay, and fleet diagnostics independently invent incompatible timestamp semantics.

## Question

How should Soma represent time across a real-time control loop, EtherCAT Distributed Clocks, Linux system clocks, PTP hardware clocks, external sensors, simulation, record/replay, and multi-computer robots?

## Executive conclusion

Soma needs an explicit **multi-domain time model**.

> **A timestamp is not meaningful unless the receiver knows which clock produced it, what event it timestamps, how synchronized that clock is, and whether the timeline has been reset.**

The recommended model distinguishes at least:

```text
ROBOT_MONOTONIC   local control deadlines and durations
SIMULATION        lockstep/paused/resettable simulated time
PTP/TAI           cross-computer/sensor correlation when synchronized
UTC               human/fleet/calendar correlation, not RT deadlines
DEVICE/DC         hardware-local clocks such as EtherCAT DC or sensor PHC
```

Soma should avoid treating Unix/UTC time as the universal control clock.

## Linux clock semantics

Linux exposes clocks with materially different properties.

### `CLOCK_MONOTONIC`

- monotonically non-decreasing;
- not affected by discontinuous wall-clock changes;
- may be frequency-adjusted by time synchronization;
- suitable for local timeouts/deadlines/durations.

This should be the default basis for physical robot process-local control timing.

### `CLOCK_MONOTONIC_RAW`

- exposes raw hardware-based monotonic time without NTP/`adjtime` frequency adjustments;
- useful for measurement/diagnostics in some timing experiments;
- not automatically the best API for normal control scheduling because it is intentionally unsynchronized.

### `CLOCK_REALTIME`

- wall-clock/UTC-oriented;
- can jump when system time is changed;
- appropriate for human-visible timestamps, certificates, logs, fleet correlation when disciplined;
- inappropriate as the sole source for RT command deadlines.

### `CLOCK_TAI`

Linux provides TAI-like system time that avoids UTC leap-second discontinuities. This can be attractive for globally synchronized machine timestamps when the platform's PTP/UTC/TAI configuration is correctly maintained.

## Proposed Soma time structure

Avoid a naked `uint64 timestamp_ns` in public protocol.

Conceptually:

```text
TimePoint {
  domain
  nanoseconds
  epoch_id
  clock_id / source where relevant
  uncertainty_ns optional
}
```

For high-rate internal structures, encode the domain by the interface contract rather than repeating a verbose header every cycle, but preserve equivalent semantics.

## Four distinct timestamp meanings

Every sampled value should distinguish, when relevant:

```text
capture_time       when the physical/simulated phenomenon was sampled
receive_time       when Soma received it
publish_time       when a state was published externally
target_apply_time  when a command should become effective
valid_until        command expiration/deadline
```

Do not replace all of these with one generic `timestamp`.

For control/state estimation, **capture time** is usually more important than network receive time.

## Epoch

A monotonic numeric timestamp alone does not protect against timeline discontinuities such as:

- simulator reset;
- runtime restart;
- robot reboot;
- replay seek;
- restore from simulation snapshot.

Soma should attach an `epoch_id` to control/session timelines.

```text
(epoch=12, tick=980)
```

is not comparable to:

```text
(epoch=13, tick=3)
```

A command from an old epoch is rejected even if its numeric timestamp would otherwise appear fresh.

## Tick vs time

Hard/soft RT loops benefit from both:

```text
tick         discrete cycle identity
sample_time  clock-domain timestamp
```

`tick` makes duplicate/missed cycle reasoning explicit and is useful for deterministic simulation/replay. Time handles physical durations and cross-rate sensors.

## EtherCAT Distributed Clocks

EtherCAT Distributed Clocks synchronize hardware clocks in DC-capable slaves and can trigger synchronous input/output events. Beckhoff documentation describes local hardware clocks, typically nanosecond-resolution, continuously synchronized to a reference device, enabling sub-microsecond-class device synchronization in suitable systems.

Soma should distinguish:

```text
EtherCAT bus/DC time
Linux RT scheduling time
PTP/global time
```

These clocks may be related, but they are not automatically the same clock.

### RT integration question

A future EtherCAT ADR/implementation must choose how the host loop aligns with DC:

- host controls cycle and slaves synchronize to it;
- host scheduler is phase-aligned to DC reference;
- external/global clock disciplines one or both.

The selected master/slave topology must expose measured DC offset/jitter as observability metrics.

## PTP / hardware clocks

For multi-computer robots and hardware-timestamped Ethernet sensors, Linux PTP is the default technology to evaluate.

`ptp4l` synchronizes PTP clocks over the network, commonly using NIC hardware timestamping / PTP Hardware Clocks (PHCs). `phc2sys` can synchronize the Linux system clock to/from PHCs according to the deployed topology.

Important consequences:

- hardware timestamp support is NIC/driver/topology dependent;
- PHC and system clock are distinct until explicitly synchronized;
- PTP time scale and UTC differ because of leap-second handling;
- switch/PHY timestamping location affects path-delay jitter;
- time health must be observable, not assumed.

## Multi-computer robot recommendation

A production profile could use:

```text
PTP Grandmaster / selected reference
        |
   Ethernet network
        |
PHC on compute A ---- ptp4l/phc2sys ---- system time A
PHC on compute B ---- ptp4l/phc2sys ---- system time B
sensor PHC/hardware timestamps where available
```

But local control deadlines still use local monotonic time. Global synchronized time is used for cross-machine sample correlation and evidence.

## Sensor timestamp contract

Every driver should document the timestamp source:

```text
HOST_RECEIVE
HOST_CAPTURE
DEVICE_CLOCK
PHC_HARDWARE
PTP_SYNCHRONIZED_DEVICE
SIMULATION
```

and expose clock synchronization status where necessary.

For sensors with an independent device clock, the driver/runtime needs a clock model:

```text
host_time ~= scale * device_time + offset
```

with estimated offset/drift/uncertainty.

Do not silently rewrite a device timestamp into host time without retaining provenance/quality.

## Time synchronization health

Expose metrics/events such as:

```text
clock_sync_state
ptp_offset_ns
ptp_path_delay_ns
clock_frequency_adjustment
last_sync_age
sensor_clock_offset_ns
sensor_clock_drift_ppm
ethercat_dc_offset_ns
clock_step_event_total
timesync_outlier_total
```

State estimation can degrade/reject sensors if synchronization uncertainty exceeds product-specific thresholds.

## Simulation time

Simulation time is intentionally unlike wall clock:

- can pause;
- can run slower/faster than real time;
- can step one tick;
- can reset;
- can restore snapshots;
- can run many parallel environments.

Therefore simulation uses its own time domain and epoch.

### Lockstep contract

Recommended controller-in-the-loop semantic:

```text
Simulator emits State(epoch=5, tick=100, sim_time=T)
Controller computes Command(target_tick=100)
Simulator applies command
Simulator advances physics
Simulator emits tick=101
```

The RT controller should be able to run from an abstract scheduler/clock interface during SIL rather than calling wall-clock APIs everywhere.

## Real-time scheduler abstraction

Do not make every control module call `clock_gettime` independently.

Provide `CycleContext`:

```text
CycleContext {
  epoch_id
  tick
  cycle_start
  nominal_period
  deadline
  clock_domain
}
```

The scheduler owns waiting/wakeup; controller logic consumes the context.

Production backend:

```text
Linux monotonic absolute-time scheduler
```

Simulation backend:

```text
lockstep simulation scheduler
```

This materially improves simulator/physical reuse.

## Network time sync protocols

MAVLink TIMESYNC is a useful example of a lightweight application-level offset estimator using request/response timestamps and repeated filtering. Soma may need a similar fallback for external SDK clients that do not participate in PTP.

Important distinction:

- PTP/hardware synchronization can establish high-quality infrastructure time;
- application ping/timesync can estimate offset/RTT across ordinary clients;
- the protocol must expose the resulting quality rather than equate the two.

## Command timing

Command envelopes should support different timing modes explicitly.

### Immediate/latest command

```text
apply as soon as accepted
valid_until required
```

Useful for joystick/base velocity streams.

### Scheduled command

```text
target_apply_time + clock domain + valid window
```

Useful for synchronized multi-component execution when supported.

### Tick-targeted command

```text
epoch + target_tick
```

Useful inside deterministic controller/simulation paths.

A recipient must reject scheduled commands if it cannot interpret or meet the requested clock semantics.

## Deterministic replay

“Deterministic” needs multiple levels.

### Protocol deterministic

Given the same recorded messages/times, state machines produce the same logical transitions.

### Controller deterministic

Given identical input buffers and cycle context on a supported build/platform, controller outputs match within defined tolerance or exactly where feasible.

### Simulator deterministic

Given simulator version, model, seed, initial state, and commands, one backend reproduces behavior within its supported determinism guarantees.

### Cross-backend deterministic

Generally unrealistic. MuJoCo, Isaac/PhysX, and Genesis should be compared via behavioral metrics/tolerances, not bitwise trajectories.

## Replay evidence

MCAP/incident recording should capture:

```text
epoch/tick
capture and receive times
time domains
clock health/offset
release/model/policy identity
requested/accepted/applied commands
simulator/version/seed where applicable
```

Without timing provenance, replay can reproduce message order but not the original control problem.

## Time-related failure modes

Soma tests should inject:

- UTC/system clock step;
- PTP loss of lock;
- PHC offset jump;
- sensor timestamp freeze;
- device clock drift;
- out-of-order samples;
- old epoch commands;
- scheduled command after deadline;
- simulator reset with in-flight commands;
- network client with badly skewed clock.

Safety logic should use local age/deadline calculations robustly even when wall-clock synchronization degrades.

## Proposed V0 time rules

1. `robot-rt` physical cycle uses local monotonic time plus epoch/tick.
2. Public samples identify their time domain and capture semantics.
3. Public commands always have sequence + epoch and use a local-validity TTL/deadline model.
4. Simulation has a distinct resettable domain and explicit lockstep tick.
5. UTC is metadata/human/fleet time, not the hard control deadline source.
6. PTP is an optional production profile for multi-compute/high-quality sensor correlation, with health telemetry.

## ADR implications

This research supports decisions roughly equivalent to:

- Soma defines explicit time domains and epoch/tick semantics.
- Control deadlines use monotonic local time.
- Simulation time is first-class and never masquerades as physical wall clock.
- PTP/PHC is the primary candidate for synchronized multi-computer/sensor time.
- EtherCAT DC is a bus/device synchronization domain that must be integrated explicitly.
- timestamp source/quality is part of sensor metadata.

## Experiments required

1. Implement `Clock`/`Scheduler` abstraction and physical + simulation backends.
2. Measure `clock_nanosleep`/RT loop jitter on target hardware.
3. Bring up linuxptp between two reference computers with hardware timestamping.
4. Measure camera/IMU correlation under software vs hardware/PTP timestamps.
5. Measure and record EtherCAT DC offset/jitter.
6. Reset MuJoCo mid-stream and verify old-epoch command rejection.
7. MCAP replay preserving epoch/tick and time-domain metadata.

## Primary references

- Linux `clock_gettime`: https://man7.org/linux/man-pages/man3/clock_gettime.3.html
- Linux network timestamping: https://docs.kernel.org/networking/timestamping.html
- linuxptp `ptp4l`: https://www.linuxptp.org/documentation/ptp4l/
- linuxptp `phc2sys`: https://www.linuxptp.org/documentation/phc2sys/
- EtherCAT Distributed Clocks: https://infosys.beckhoff.com/content/1033/ethercatsystem/2469118347.html
- EtherCAT DC defaults/synchronization: https://infosys.beckhoff.com/content/1033/ethercatsystem/2469102219.html
- MAVLink Time Synchronization v2: https://mavlink.io/en/services/timesync.html
