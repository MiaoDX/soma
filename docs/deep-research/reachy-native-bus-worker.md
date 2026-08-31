# Reachy Mini Native Bus Worker

Research date: 2026-08-31
Scope: One hardware-first case study for Soma's next native milestone: the
Reachy Mini Lite nine-servo Dynamixel path, the first-party Reachy motor
controller, `rustypot`, and ROBOTIS Protocol 2.0. This does not qualify a
physical robot, replace the N0/N1 gates, or broaden Soma beyond the fixed
Reachy profile.

## Research Brief

- **Question:** What should Soma require from a Reachy native L0 bus worker
  before torque-enabled motion, and which parts of the existing reference
  implementation are safe to borrow?
- **Decision or audience:** Bootstrap implementation and N0/N1 reviewers.
- **Included:** nine fixed IDs, model/configuration identity, Protocol 2.0
  framing and error semantics, batched reads/writes, retries, freshness,
  servo watchdogs, and comparison with Soma's probe.
- **Excluded:** physical hazard approval, motor firmware ownership, gait,
  hard-real-time claims, and copying the Reachy-specific controller source.
- **Freshness/date boundary:** sources and repository heads checked on
  2026-08-31; compatibility claims pin releases/commits below.
- **Completion test:** one Reachy-specific case study, claim-level ledger,
  adversarial protocol check, explicit Soma implications and stop decision.
- **Constraints:** public first-party sources and local source inspection;
  no connected Lite was available, so no physical timing or fault injection.

## Executive Summary

Soma should keep a single Reachy-specific bus owner in an L0 worker, using a
bounded command mailbox and batched Protocol 2.0 transactions. The official
controller demonstrates this shape: one serial owner at 1 Mbps, sync reads for
positions/telemetry, sync writes for goal positions and torque, and a separate
thread/timer that publishes the latest state. Its 10 ms serial timeout,
20 ms transient retry delay, and 50 Hz default loop are useful starting
parameters, not guarantees for Soma's periodic path.

Two compatibility hazards must be fixed before N0 can be treated as a model
gate. The Lite datasheet documents the custom base as XC330-M288-PG with model
number 1240, while `rustypot` 1.6.0's XL330 registry only names 1190 and 1200;
the current probe reads a model but never checks it in `configuration_matches`.
Also, Protocol 2.0 defines one 7-bit error number plus an Alert bit (0x80),
whereas `rustypot` decodes the byte as several independent bits and omits the
Alert bit. A successful CRC/ID parse therefore is not sufficient evidence of
device acceptance or absence of a hardware alert.

## Findings

### Fixed hardware and software shape

The Lite hardware documentation identifies one custom XC330-M288-PG base,
two XL330-M077-T antenna servos, six XL330-M288-T Stewart servos, and a
Dynamixel TTL connection. The pinned Reachy configuration sets 1,000,000 baud,
IDs 10--18, position mode 3, shutdown mask 52, and measured per-Stewart raw
limits. These are **verified** by the [Reachy hardware datasheet](https://github.com/pollen-robotics/reachy_mini/blob/da0097361c1567f0daf61310e940616171028fd2/docs/source/platforms/reachy_mini_lite/hardware.md),
[pinned hardware config](https://github.com/pollen-robotics/reachy_mini/blob/20bc9eedc81ddc552235d222ca7e39205b2c2481/src/reachy_mini/assets/config/hardware_config.yaml),
and Soma's [provenance note](../../crates/soma-probe/UPSTREAM.md).

The first-party [Reachy motor controller](https://github.com/pollen-robotics/reachy-mini-motor-controller/tree/dff1c536a75735a950564e18240d6a67b056819c)
(release 1.5.6) owns the port, performs sync position/voltage/current reads,
sync goal/torque writes, checks missing IDs, waits for voltage, reboots motors
with non-zero hardware status, and publishes a timestamped position cache.
Its queue capacity is 100 and its default SDK-facing update loop is 50 Hz.
These implementation facts are **verified** from `controller.rs` and
`control_loop.rs`; they are not hard-real-time evidence.

### Transaction and timing implications

At 1 Mbps, use one sync write for the nine goal positions and one sync read for
the present-position block (optionally a second bounded read for current,
voltage, temperature, torque and hardware status). `rustypot` sends one
broadcast sync-read instruction and then waits for one status packet per ID;
sync-write intentionally waits for no status packets. ROBOTIS confirms that
broadcast sync write has no response, while sync read returns a response from
each device. Therefore:

1. acknowledge admission after local validation and successful packet write;
2. publish `applied` only after a later state sample proves the target was
   observed within the configured tolerance; and
3. classify missing, malformed, CRC, wrong-ID, timeout, and device-error
   responses separately.

The official 10 ms port timeout, 20 ms transient retry delay, and 50 Hz loop
are **supported starting points** from source, not a budget. Soma must measure
wire transaction duration, state age, retry count, dropped cycles, and worst
case mailbox delay on the actual Lite before choosing a cycle period.

### Protocol and safety boundary

ROBOTIS Protocol 2.0 specifies CRC over the stuffed packet, a status instruction
0x55, and an error byte with bit 7 = Alert and bits 0--6 = one error number.
The XL330 control table documents `Hardware Error Status(70)`, `Shutdown(63)`,
`Torque Enable(64)`, and `BUS Watchdog(98)`. BUS Watchdog values 1--127 mean
20 ms units; on expiry the servo stops, changes the register to -1, and makes
goal registers read-only until the watchdog is cleared. These are **verified**
by the [Protocol 2.0 specification](https://emanual.robotis.com/docs/en/dxl/protocol2/)
and [XL330-M288 control table](https://emanual.robotis.com/docs/en/dxl/x/xl330-m288/).

The pinned `rustypot` 1.6.0 source correctly validates header, length, CRC,
sender ID, and byte stuffing, but `DynamixelErrorV2::from_byte` iterates bits
1..6 and maps them as flags; it does not model bit 7 Alert or the mutually
exclusive 1..7 error number. This is **verified** from the [v1.6.0 source](https://github.com/pollen-robotics/rustypot/blob/02cb147c939c915fbe8e978b946469bd5200da66/src/dynamixel_protocol/v2.rs)
and contradicted by the [ROBOTIS SDK decoder](https://github.com/ROBOTIS-GIT/DynamixelSDK/blob/2ded684dff05a40ac78d6a16105c6ddc1b3b9930/c/src/dynamixel_sdk/protocol2_packet_handler.c).
Until corrected or wrapped, Soma must retain the raw status error byte and
read register 70 explicitly; it must not infer hardware safety from a generic
`Ok(())`.

### Model identity is a current N0 gap

The XC330-M288 page gives model number 1240; XL330-M288 gives 1200. The
`rustypot` 1.6.0 registry recognizes XL330-M077 (1190) and XL330-M288 (1200),
not XC330 1240. Soma's probe reads `model`, but its pass predicate checks only
IDs, torque-off, baud, offsets, limits, mode, and shutdown mask. This is a
**verified implementation gap**, not evidence that the purchased base is
wrong. N0 must add a fixed expected model per ID (including the custom base),
fail closed on mismatch, and preserve the raw model/firmware values in the
report.

### Comparison with Soma

Soma already has the correct ownership intent: the [N0 probe](../../crates/soma-probe/src/main.rs)
opens the CH343 port at 1 Mbps 8N1, checks official-daemon absence, sets serial
exclusivity, pings IDs 10--18, and records raw reads. It is **verified** as a
read-only gate, but it is deliberately a per-register audit, not a cyclic
worker. It currently performs many unicast reads after each ping, does not
measure a representative sync transaction, and does not check model identity.

The worker should therefore remain private to `ReachyNativePlant`, with a
bounded mailbox and explicit state-age/sequence evidence. It should not put
serial calls in periodic `robot-rt`, expose a generic actuator hierarchy, copy
the unlicensed Reachy controller repository, or treat the servo BUS Watchdog
as a substitute for Soma's independent N1 stop path.

## Contradictions And Uncertainty

### Error-byte interpretation

- **Position A:** `rustypot` maps several bits and returns a vector of errors
  ([source](https://github.com/pollen-robotics/rustypot/blob/02cb147c939c915fbe8e978b946469bd5200da66/src/dynamixel_protocol/v2.rs)).
- **Position B:** ROBOTIS defines one error number plus Alert bit
  ([specification](https://emanual.robotis.com/docs/en/dxl/protocol2/#error));
  its SDK masks Alert before switching on values 1--7.
- **Likely reason:** the crate's old decoder predates its 1.6.0 byte-stuffing
  update and was not aligned with the vendor definition.
- **Current judgment:** preserve raw bytes and treat Alert/non-zero error as a
  fault until a wrapper or upstream fix is validated. **Verified**, with a
  high-severity compatibility concern.
- **Resolution evidence:** fixture tests for error bytes 0x80, 0x81, 0x04 and
  0x07, plus a hardware status-packet capture.

### Model registry versus hardware

- **Position A:** the Lite datasheet says custom XC330-M288-PG, model 1240.
- **Position B:** generic XL330 support in `rustypot` recognizes 1190/1200.
- **Likely reason:** the base is a Reachy-custom geared variant, not the
  standard XL330 model family.
- **Current judgment:** do not force it through a generic model enum; compare
  the raw expected per-ID model in the Reachy profile. **Supported** pending
  N0 hardware readback.

## Gaps

- No connected Lite was available, so wire-time distributions, USB latency,
  dropped-cycle behavior, and physical fault responses remain unknown.
- No independent evidence establishes CH343 buffering, exact servo return
  delay on the purchased unit, or whether all nine devices share identical
  firmware revisions.
- The Reachy motor-controller repository has no detected license metadata;
  Soma should use the Apache-2.0 `rustypot` protocol crate as an adapter
  dependency and avoid source copying until terms are clarified.
- A Protocol 2.0 wrapper fix needs upstream coordination or a small local
  compatibility layer; this research does not modify dependencies.

## Soma Implications And Stop Decision

Before N1, update N0's expected-model table and raw error handling, then add a
bench-testable worker transaction fixture. After N0/N1, implement one
Reachy-specific L0 worker with bounded mailbox, one owner, sync goal write,
sync state read, explicit retry classification, monotonic state age, and
measured applied evidence. Configure the servo BUS Watchdog only through the
approved physical procedure; it is a device-side supplement, not authority
for torque enable or emergency stop.

Stop this research cycle here. Another repository scan is unlikely to change
the immediate decision; the next evidence should come from N0 hardware and
Protocol 2.0 fixture tests, not more catalog entries.

## Method

Subquestions covered hardware identity, first-party transaction shape, protocol
semantics, watchdog/fault behavior, and Soma gap comparison. Sources were
selected from the Reachy repository, its motor-controller source, the pinned
Apache-2.0 `rustypot` 1.6.0 crate/source, ROBOTIS Protocol 2.0 and XL330
manuals, and ROBOTIS SDK source. The adversarial pass compared the crate's
error decoder with the vendor decoder and compared documented model numbers
with the generic registry. No physical or timing claim was promoted beyond
`supported` without hardware evidence.
