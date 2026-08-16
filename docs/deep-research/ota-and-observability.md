# OTA and Observability

> Status: Deep Research baseline. OTA and observability are treated together as Soma's lifecycle/operations plane.

## Question

How should a production robot update, prove health, diagnose failures, recover safely, and operate across a fleet when software spans Linux, RT processes, MCU/FPGA firmware, robot models, calibration, policies, and configuration?

## Working thesis

OTA and observability are one feedback loop:

> **OTA changes the system. Observability determines whether that change is healthy, explains failures, and provides evidence for rollback and release decisions.**

```text
Fleet / Release Service
  Release registry | rollout | artifact trust | fleet health
                         |
                  secure network
                         |
Robot Operations Plane
  update-agent | health-agent | observer | recorder
       |              |            |          |
 installers       health gate     OTLP       MCAP
       \______________|____________|__________/
                         |
               robot-runtime / robot-rt
                         |
                MCU / FPGA / devices
```

## Release, not file update

A production release should be a versioned compatibility set rather than independent file downloads.

Conceptually:

```yaml
release_id: soma-2026.08.16-rc3
generation: 184
channel: production

hardware_compatibility:
  robot_models: [reference_wheeled_v1]
  mainboard_revisions: [rev_c, rev_d]

artifacts:
  - id: main-os
    type: rootfs
    version: 4.8.1
    digest: sha256:...
    activation: reboot

  - id: robot-runtime
    type: application
    version: 2.7.0
    digest: sha256:...

  - id: power-mcu
    type: mcu-firmware
    version: 1.12.3
    digest: sha256:...

  - id: locomotion-policy
    type: policy-bundle
    version: walk-37
    digest: sha256:...

compatibility:
  protocol_min: 5
  protocol_max: 7
  model_bundle: model-42
  calibration_schema: 3
```

The ReleaseManifest should eventually cover dependency/compatibility relationships rather than merely list artifacts.

## Different artifacts require different lifecycle rules

| Artifact | Preferred strategy | Rollback/recovery |
|---|---|---|
| Linux kernel/rootfs/critical drivers | A/B immutable image | bootloader fallback |
| `robot-rt` | generally released with validated OS/driver baseline | slot rollback |
| `robot-runtime` | immutable app slot or image | atomic switch |
| MCU firmware | dual bank / test-confirm | bootloader revert |
| FPGA | candidate + golden image | golden fallback |
| Policy/model | content-addressed signed bundle | atomic manifest switch |
| Configuration | versioned/schema-validated | restore previous |
| Calibration | device-owned, separately backed up/migrated | explicit recovery; never silently overwrite |

Model/policy/calibration should not be treated as generic firmware: their compatibility is often semantic and tied to sensor/joint schemas.

## Linux A/B baseline

A practical first implementation can use **RAUC + U-Boot**:

```text
boot_a / rootfs_a
boot_b / rootfs_b
data
recovery
```

Update the inactive slot, verify, mark it pending, reboot, then only mark it good after **robot-level** health checks.

`Linux booted` is not sufficient health.

## MCU baseline

Use a dual-bank bootloader pattern such as **MCUboot**:

```text
write candidate -> verify -> test boot -> application self-test
                                  | pass       | fail/reset
                                  v            v
                               confirm       revert
```

Each secondary controller should verify its own image and report firmware identity/reset reason to the robot coordinator.

## Robot update coordinator

The main compute acts as the coordinator for secondary ECUs/devices:

```text
OTA service
    |
main robot computer
    |
    +-- safety/power MCU
    +-- joint controllers
    +-- hand controller
    +-- sensor firmware where updateable
    +-- FPGA
```

This resembles the Primary/Secondary model used by Uptane. The coordinator maintains a complete hardware/software inventory and records partial-update state durably.

A distributed robot update cannot usually be perfectly atomic across all devices. The design target should be a **recoverable transaction**:

- every step is journaled;
- operations are idempotent where possible;
- power loss has a known recovery path;
- incompatible partial states do not enable motion;
- the coordinator can resume, rollback, or enter service mode.

## Update state machine

```text
IDLE
 -> CANDIDATE_FOUND
 -> PRECHECK
 -> DOWNLOADING
 -> VERIFIED
 -> STAGED
 -> WAITING_FOR_SAFE_ACTIVATION
 -> ACTIVATING
 -> BOOT_PENDING
 -> HEALTH_OBSERVATION
      | pass          | fail
      v               v
   COMMITTED       ROLLBACK / RECOVERY
```

Every transition is a structured persistent event with `update_id`, `release_id`, reason, timestamps, and artifact state.

## Safe activation gate

A robot should not activate updates merely because a maintenance timer fired.

Possible gates:

```text
robot is stationary / docked / supported as required
no active motion lease
safety state is safe
actuators are disabled or controlled-safe
battery SOC/temperature are acceptable
external power available when required
sufficient disk space
no current rollback/recovery operation
hardware inventory matches release
```

Download may occur during operation with strict resource limits. Installation/activation of critical components normally should not.

## Robot-level mark-good

A candidate boot should be observed before permanent commit.

Health checks should include:

### System

- expected rootfs/slot;
- filesystem/data integrity;
- memory/disk headroom;
- time synchronization;
- no critical crash loop.

### RT

- `robot-rt` ready;
- scheduling/CPU affinity configured;
- memory locked where required;
- bounded cycle latency;
- zero unacceptable deadline misses during observation window;
- RT IPC healthy.

### Bus/device

- expected EtherCAT/CAN devices online;
- distributed clock/Working Counter acceptable;
- no bus-off;
- expected firmware inventory;
- IMU/encoder/power/safety controller valid.

### Robot

- RobotManifest/calibration compatibility;
- protocol self-test;
- safety self-test;
- optional product-specific controlled motion test.

The **local robot** decides whether to mark the slot good. Fleet/cloud health controls rollout expansion but should not be required for local boot recovery.

## Artifact trust

Separate three concerns:

1. **Transport security** — e.g. TLS/mTLS.
2. **Artifact signature** — binary/bundle authenticity.
3. **Repository/update metadata** — authorization, freshness, anti-rollback, compatible target selection.

TUF provides a strong software-update trust model with separated metadata roles. **Uptane** extends this thinking to vehicles/multiple ECUs and is a useful model for robots with many updateable controllers.

Do not implement custom signing/metadata cryptography when mature implementations exist.

## Fleet rollout

Production rollout should be cohort/ring based:

```text
simulation -> SIL -> HIL -> internal robots -> canary -> small cohort -> site/ring expansion -> fleet
```

A rollout controller should consider:

- hardware revision;
- robot model/embodiment;
- customer/site;
- maintenance window;
- offline status;
- release channel;
- minimum sample/run duration;
- customer approval where required.

Telemetry should automatically pause expansion when release health degrades.

Candidate release-health signals:

- install/boot failure rate;
- rollback rate;
- crash-free operating time;
- RT deadline misses;
- critical fault rate;
- safety interventions;
- task/mission success;
- power/thermal regressions.

Thresholds are product-specific and should be statistically meaningful rather than copied blindly.

# Observability

## Do not put every robot signal into OpenTelemetry

Soma needs multiple evidence systems:

| Data | System | Purpose |
|---|---|---|
| Metrics | OpenTelemetry/Prometheus-compatible backend | trends, SLOs, alerting |
| Structured events/logs | OTel logs / event store | lifecycle/fault reasoning |
| Traces | OpenTelemetry traces | SDK/action/service causality |
| High-rate robot data | MCAP | state/command/sensor incident evidence |
| Crash evidence | pstore/coredump/MCU retention | kernel/process/MCU root cause |

A 1 kHz joint stream, image, or point cloud should not become an OTLP log.

## RT telemetry budget

The RT process emits only bounded, low-disturbance primitives:

- atomic counters;
- fixed histogram buckets;
- fixed-size events;
- preallocated ring writes;
- explicit dropped-event counters.

No JSON, OTLP, network I/O, filesystem I/O, string formatting, dynamic allocation, or blocking logging in the cyclic path.

Example:

```rust
#[repr(C)]
pub struct RtEvent {
    pub timestamp_ns: u64,
    pub tick: u64,
    pub event_code: u32,
    pub severity: u8,
    pub component_id: u16,
    pub arg0: i64,
    pub arg1: i64,
    pub arg2: i64,
}
```

`robot-observer` converts these into structured logs/metrics outside RT.

## Important metrics

### RT

```text
cycle duration/jitter
execution duration
p99/p99.9/max latency
deadline miss count
command/state age
ring overflow
watchdog trips
```

### Bus

```text
EtherCAT Working Counter errors
DC offset
slave online state
CAN bus-off/error-passive
packet/frame errors
bus cycle duration
```

### Actuator/sensor

```text
temperature/current/voltage
torque and clipping
encoder/sensor faults
sample age/drop count
```

### Safety

```text
interventions
stops
command rejects
limit/torque clips
collision/fall events
```

### Runtime/SDK

```text
restarts
RPC/action latency/errors
queue depth
lease conflicts
reconnects
```

### OTA

```text
download/verify/stage duration
activation attempts
health-gate failures
boot attempts
rollback count
```

Avoid high-cardinality metric labels such as trace IDs, mission IDs, error strings, arbitrary file paths, or timestamps. Those belong in events/traces.

## Faults are domain objects, not strings

Define a structured lifecycle:

```text
FaultEvent
  fault_id
  event_code
  subsystem/component
  severity
  first_seen / occurrence time
  count
  latching
  recoverability
  active
  probable cause / recommended action
  release/update/boot/trace correlation
```

State transitions can include `RAISED`, `UPDATED`, `ACKNOWLEDGED`, and `CLEARED`.

Human-readable logs explain faults; they do not define them.

## Tracing

Tracing is useful for long causal paths:

```text
Python SDK request
 -> authentication/lease
 -> grasp action
 -> planning
 -> trajectory validation
 -> controller acceptance
 -> safety modification
 -> result
```

Do not create a span per EtherCAT cycle or per joint sample. Use sampling/tail-sampling to retain failed or anomalously slow operations.

## Flight recorder

A production robot should maintain a local circular black-box buffer, for example tens of seconds before and after a trigger.

Triggers include:

- fall;
- E-stop/safety stop;
- RT deadline incident;
- bus/device loss;
- process/kernel/MCU crash;
- OTA failure;
- unexpected reboot;
- mission failure;
- explicit operator capture.

Capture:

```text
requested / accepted / applied commands
joint/IMU/base/contact/estimator state
bus health
safety events
mode/lease changes
runtime events
release/model/calibration/policy identity
selected sensors where privacy/storage allows
```

**MCAP** is a strong container candidate because it supports schemas/channels, indexed time-series data, metadata, compression, and append-oriented recording.

## Crash evidence

- Linux kernel: `pstore` / `ramoops` for panic/oops persistence across reboot.
- Userspace: `systemd-coredump` or equivalent, preserving executable/build identity.
- MCU: retention/backup registers or small durable crash journal with reset reason, firmware version, panic/fault code, PC/SP, watchdog state.

These sources should be correlated into one robot incident rather than uploaded independently with no context.

## Offline-first storage and degradation

Robots cannot assume cloud connectivity. Define priority classes:

- **P0** safety/update audit/critical fault — durable, highest upload priority;
- **P1** health metrics/important events — store and forward;
- **P2** logs/traces — sampled/compressed;
- **P3** raw high-rate data — circular retention, incident-triggered preservation.

When storage or network is constrained, degrade P3/P2 first. Telemetry must never block `robot-rt` or compromise safety.

## OTA-observability closed loop

An update should emit a durable sequence such as:

```text
update.discovered
update.precheck.started
update.download.completed
update.signature.verified
update.staged
update.activation.started
system.reboot
update.boot.pending
update.health_check.started
update.health_check.passed
update.committed
```

or:

```text
update.health_check.failed
update.rollback.started
system.reboot
update.rollback.completed
```

Correlation should use persistent identifiers such as `update_id`, `release_id`, and boot IDs; trace context alone is not sufficient across reboot.

## Simulation and release qualification

Every production candidate should progress through automated evidence stages:

```text
build/sign
 -> deterministic/unit replay
 -> MuJoCo regression
 -> Isaac/Genesis scenarios where relevant
 -> SIL
 -> HIL
 -> canary physical robot
 -> fleet
```

Simulation and physical telemetry should share metric/event semantics where meaningful so release comparisons are straightforward.

A future capability should replay physical incidents into simulation for counterfactual testing of controller/safety/update changes.

## Testing OTA/observability themselves

These systems require destructive/fault tests, not only unit tests:

- power cut during every update state;
- corrupt/truncated artifact;
- stale/replayed metadata;
- wrong hardware target;
- full disk;
- read-only/corrupt data partition;
- network partition/reconnect;
- clock skew;
- MCU fails halfway through fleet update;
- rollback loop;
- telemetry backend unavailable;
- telemetry overload/high event rate;
- crash during health observation;
- compromised/expired signing metadata scenarios.

## Supply chain

The release pipeline should eventually integrate:

- SBOM (e.g. SPDX);
- build provenance/SLSA-style evidence;
- artifact digest/signature;
- vulnerability scanning;
- license inventory;
- immutable release metadata;
- key rotation/revocation procedures.

Secure Boot, device identity, TPM/TEE/attestation, disk encryption, and debug-interface policy should be designed with OTA rather than added after fleet deployment.

## Candidate Soma components

```text
robot-update-agent
robot-health-agent
robot-observer
robot-recorder
robot-diagnostics
release-manifest
update-journal
artifact-installers/{rauc,mcuboot,runtime,policy}
```

## Open questions

1. RAUC vs OSTree/Mender/balena-style host update model for the first reference computer.
2. Exact trust service: TUF baseline vs fuller Uptane-style multi-ECU deployment from day one.
3. Secure Boot/TPM requirements for the reference hardware.
4. Health-observation duration and product-specific mark-good criteria.
5. Desired-state vs imperative deployment API for fleet operations.
6. MCAP retention size and privacy policy for camera/audio data.
7. OpenTelemetry Collector on robot vs lightweight OTLP exporter + site gateway.
8. How release health automatically pauses rollout without creating rollback oscillation.
9. Calibration migration/backup rules.
10. Recovery/factory image and physical service workflow.

## Reference projects and sources

- TUF — https://theupdateframework.io/
- Uptane — https://uptane.org/
- RAUC — https://rauc.io/
- MCUboot — https://docs.mcuboot.com/
- Mender — https://mender.io/
- OSTree — https://ostreedev.github.io/ostree/
- OpenTelemetry — https://opentelemetry.io/
- Prometheus instrumentation practices — https://prometheus.io/docs/practices/instrumentation/
- MCAP — https://mcap.dev/
- Linux pstore — https://docs.kernel.org/admin-guide/ramoops.html
- systemd-coredump — https://www.freedesktop.org/software/systemd/man/latest/systemd-coredump.html
- SPDX — https://spdx.dev/
- SLSA — https://slsa.dev/
