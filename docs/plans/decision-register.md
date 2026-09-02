# Current Decision Register

> Status: Thin index for decisions that constrain the current implementation.
> Research documents preserve the wider design space; this register does not
> track decisions for work that has no active trigger.

## Rule

A decision belongs here only when changing it would alter the current happy
path or the next implementation step. Reversible implementation details remain
in code and tests. Future product, hardware, security, release, replay, and
fleet choices return only when their trigger exists.

## Current Decisions

| ID | Decision | Current posture | Proof or reversal trigger |
| --- | --- | --- | --- |
| D-02 | Control/Plant boundary | One `ControlCore` drives a bounded Plant contract; MuJoCo is the first Plant | The fixed-step movement, timeout, and reset tests must use this boundary |
| D-04 | Runtime boundary | Keep separate `robot-runtime` and `robot-rt` roles. The fixed Duck qualification uses a separate Rust non-RT policy worker by default, a Python oracle on explicit request, a profile-specific non-RT runtime adapter, and a periodic RT owner. The measured Rust/Python comparison does not justify merging inference into the runtime or RT process; process count is not itself a safety claim. | Reopen when target deployment, recovery, privilege, or measured process overhead makes the current isolation materially unsuitable |
| D-05 | RT/runtime IPC | Start with a minimal bounded SPSC command mailbox and state mailbox | Compare or replace only after measured load, recovery needs, or maintenance cost justify it |
| D-06 | Client transport | Use loopback Zenoh for the first end-to-end path | Compare only for a demonstrated envelope, operations, or interoperability problem |
| D-07 | Time and reset | Immediate commands carry TTL; reset creates an opaque Plant timeline | Add tick-target or synchronized time only for a concrete controller requirement |
| D-14 | Local actuation and security scope | Local physical experiments require the approved N1 actuation gate; no cryptographic mechanism or production-security claim | Reopen security before non-local control, external distribution, or OTA; N1 governs local physical motion |
| D-19 | Policy-to-control timing and fallback | A low-rate policy sends immediately from captured state; readiness includes a matched downstream subscriber. The faster periodic consumer applies the latest admitted target on its first available tick, holds it with bounded zero-order hold, and owns TTL expiry and `SafeBehavior`; Duck uses 500 Hz RT/physics and 50 Hz policy frames, while Reachy's cadence is unchanged. | Revisit interpolation beyond hold or action chunks only for a concrete policy; revisit readiness when transport discovery semantics change |
| D-21 | Public wire schema | Use a minimal Protobuf command/state/reset schema without compatibility guarantees | Freeze compatibility only when an artifact is released or an external consumer exists |
| D-22 | Robot and hardware path | Reachy Mini remains the supported profile with MuJoCo and gated native work; one approved fixed Open Duck Mini MuJoCo profile qualifies the same control architecture without adding generic robot discovery or Duck hardware | Reopen if Reachy N0 finds an unexpected dependency, Duck cannot preserve the fixed-profile boundary, or a third embodiment is proposed |
| D-23 | Open Duck policy ABI/runtime | Freeze one fixed `101 -> 14` ABI, 27-tick phase, three-action history, default pose, 0.25 scale, 5.24 rad/s slew, 40 ms TTL, and pinned checkpoint checksum. Rust owns the non-RT adapter/default route; Python remains the explicit oracle. ONNX native runtime provisioning is required before deployment packaging. | Reopen only if the pinned checkpoint or ABI changes, or supported runtime provisioning cannot be made reproducible |

## Deliberately Unregistered

The following subjects have research coverage but no current implementation
decision: universal model metadata, vendor adapters, replay frameworks, durable
recording, release identity, packaging matrices, target qualification,
cryptographic trust and OTA, ROS integration, additional simulators, fleet
operations, and owner-controlled hardware layers.

Their evidence remains under [`docs/deep-research/`](../deep-research/README.md).
Their re-entry conditions are listed in the
[`Explicitly Deferred`](bootstrap-plan.md#explicitly-deferred) table.

## Discipline

- Do not add a row merely because a topic matters eventually.
- Do not write an ADR for a reversible default with no external consumer.
- Close a current experiment by updating the row or recording a durable ADR.
- Remove a row when its implementation scope is deferred; Git history retains
  the previous analysis.
