# Policy/Inference to Real-Time Interface

> Status: D-19 and D-04 are resolved for the approved Open Duck workload.
> Latest-value timing, matched-subscriber readiness, bounded zero-order hold,
> command TTL, `control_mode -> SafeBehavior`, lineage, and the Rust-default /
> Python-oracle split are measured. Action chunking remains deferred.

## Question

How does a low-rate, latency-variable command source (a VLA/WBC policy, a
teleoperation stream, or in V0 a synthetic cadence source) drive the 1 kHz
deterministic control loop safely, and what happens when it is late, absent,
or produces an implausible command?

## Context

Soma's reference architecture and Decision Register are precise about the
boundary between `robot-runtime` (SA-4) and `robot-rt` (SA-3): lease,
timeline, deadline, and envelope semantics are all specified. They are silent
about the boundary one level higher — between whatever is producing the
`JointCommand` stream at 5–50 Hz and the fixed-layout mailbox that
`robot-rt` consumes at 1 kHz. `PolicyBundle` names an artifact (model,
schema, normalization, history length, rate) but does not define its runtime
behavior.

This was not a hypothetical concern for a later milestone. `SafetyProfile`
already claims ownership of "safe behavior" under fault conditions
(`safety-and-fault-architecture.md`). A missed or late policy inference is a
fault condition. D-19 therefore freezes the M1a subset below before an `SA-3`
implementation claims to handle it.

## Findings

### The M1a contract and deferred questions

1. **Command TTL and safe behavior.** When no new command arrives before an RT tick needs
   one, what does `robot-rt` do — hold the last accepted command, decay it
   toward a safe target, extrapolate, or switch to a named fallback
   controller? This must be a `SafetyProfile`-governed decision, not an
   application-code default, because it directly determines what the robot
   does when its brain stops talking.
2. **Decimation and interpolation ownership.** A 20 Hz command stream driving
   a 1 kHz loop needs an explicit interpolation strategy (zero-order hold,
   linear, or spline) and an explicit owner (`robot-runtime` before the
   mailbox, or `robot-rt` after it). If the interpolator sits in `robot-rt`,
   it is inside the `SA-3` envelope and must be validated as such; if it sits
   in `robot-runtime`, `robot-rt` never sees the low-rate signal and cannot
   reason about its staleness directly.

These two decisions are frozen before the mailbox layout. M1a records the
chosen interpolation owner, a bounded TTL, an explicit mapping from each
supported `control_mode` to a named `SafeBehavior`, and the resulting
requested/admitted/safety-output/applied lineage.

The approved Open Duck workload now supplies a representative observation and
deployment pipeline. It freezes capture-time lineage and history ordering for
that profile without creating a generic observation API:

3. **Observation time alignment.** History-buffer assembly for the policy
   needs a defined alignment (capture-time vs. tick-time) and a defined
   behavior for a missing frame. Training-time and deployment-time alignment
   must match bit-for-bit, or the hardest class of sim-to-real bug is
   reintroduced silently.
4. **Action chunking.** Contemporary VLA/WBC policies commonly emit an action
   *chunk* (a short sequence) rather than a single command. Chunk execution,
   preemption mid-chunk, and chunk-boundary blending all interact with lease
   and `plant_timeline_id` semantics and are currently unmodeled.

A fifth, lower-priority question — GPU/CPU resource contention between
inference and perception, and its effect on runtime scheduling — remains out of
scope until both workloads share representative target compute.

### Why this cannot wait for real hardware

Interpolation, TTL, safe-behavior selection and lineage are pure
control-and-timing semantics; none requires a trained policy, a GPU or
physical hardware. A lookup table or sine wave at a fixed low rate is enough
to force those paths. Open Duck now provides the representative model needed
to verify one concrete observation-alignment contract; chunking remains
deferred.

### Reference scheduling choices

"Synchronous" is not a sufficient classification for a robot control stack.
The useful questions are whether sensor delivery, inference, and low-level
actuation share a thread; which rates they use; and how the consumer handles a
target between policy updates.

| Reference | State and inference | Low-level application | Choice and consequence |
| --- | --- | --- | --- |
| Open Duck Playground | One MuJoCo runner samples observation and runs ONNX every ten 2 ms physics steps | The new target is written immediately, then held for subsequent physics steps | Same-thread 50 Hz reference with no fixed 20 ms frame delay; simplest compatibility oracle |
| Berkeley Humanoid Lite | The loop computes an action from the returned observation | `step(action)` applies it over the next policy interval's physics substeps | Synchronous application-level loop; communication details remain below that interface |
| Unitree SDK2 G1 example | DDS `LowState` arrives through a subscriber callback | A separate recurrent 2 ms thread publishes `LowCmd` from the latest state/command | Asynchronous delivery plus periodic actuation; callback and command thread are not lockstep |
| Unitree RL Lab | A dedicated policy thread updates processed actions at `step_dt` | A separate 1 kHz FSM thread reads the latest processed action and publishes `LowCmd` | Explicit latest-value split; closest public reference to Soma's policy/RT timing semantics |
| AgiBot X1 Infer | ROS2/AimRT callbacks update IMU, joint, and velocity-command buffers; ONNX runs every tenth 1 kHz control tick | The 1 kHz control loop publishes the current action every tick | Asynchronous inputs with inference coupled to the cyclic thread; simpler phase relation but inference jitter enters that thread |

Soma follows the Unitree RL Lab timing shape while retaining a stronger process
boundary: Duck inference completes at 50 Hz independently, the runtime
coalesces to the latest target, and the 500 Hz periodic owner applies it at the
first available 2 ms RT tick. The resulting latency is variable and measured.
A fixed 20 ms delay is a fault-injection case, not the nominal pipeline.

The process boundary remains an ownership and failure-isolation choice, not a
safety result. D-04's completed same-host comparison selected a separate Rust
policy worker as the deployment default and retained isolated Python inference
as the oracle. Rust reduced mean inference latency in the representative sample,
while startup evidence showed that publisher declaration alone did not imply a
matched downstream subscriber. Readiness now waits for that match and fails
closed; inference remains outside both the profile runtime adapter and periodic
RT owner. A single-process async/periodic topology was not implemented because
the measured path provided no need to trade away the existing process seam.

## Alternatives

| Approach | Description | Trade-off |
| --- | --- | --- |
| Hold-last-command | `robot-rt` keeps applying the most recent admitted command past its nominal validity, up to a bounded TTL | Simple; can be actively unsafe for high-dynamics embodiments if held too long |
| Decay-to-safe | Command blends toward a declared safe target over a bounded window | Requires a per-embodiment "safe target" concept; more moving parts |
| Fallback controller | On timeout, `SA-3` switches to a separate low-authority controller (e.g., damping, stand) | Matches the safety architecture's existing "safe behavior" language; needs its own validated implementation |
| Reject and inhibit | Treat a stale command source like a stale lease: motion-inhibited until a fresh command arrives | Safest default; may be too conservative for legitimate short gaps (e.g., one dropped inference) |

None of these are mutually exclusive across embodiments; the point is that
V0 must pick one as the default and make it a `SafetyProfile` field rather
than leaving it as an application-code choice.

## Implications for Soma

- Add a bounded command TTL and `control_mode -> SafeBehavior` mapping to
  `SafetyProfile`; hold/decay/fallback/inhibit are candidate behaviors rather
  than a requirement to implement all four in M1a.
- Keep zero-order hold in the command consumer so it retains direct ownership
  of staleness, expiry, and applied evidence.
- Extend the requested/admitted/safety-output/applied command lineage
  (`docs/architecture/diagrams/soma-command-lineage.svg`) with a documented
  case for "no new requested command this tick" so a stale-source event is
  visible in the same evidence trail as a rejected command, not silently
  absent from it.
- M1a includes a synthetic 20 Hz cadence source specifically to force timeout,
  interpolation, and staleness paths independently of Duck inference. See the
  `cadence-source-decimation` row proposed in the bootstrap plan's
  verification matrix.
- Keep Duck observation alignment profile-specific and out of the generic
  Plant/mailbox API. Reopen action chunks only for a representative chunked
  policy.

## Trade-offs

Freezing zero-order hold, TTL, safe behavior, and lineage prevents the mailbox
boundary from being set accidentally and closes the stale-command fault path.
Duck supplies one concrete alignment contract without forcing it into a
generic API; chunking remains deferred.

## Sources

- `docs/architecture/reference-architecture.md` — `PolicyBundle`,
  requested/admitted/safety-output/applied lineage.
- `docs/deep-research/safety-and-fault-architecture.md` — `SafetyProfile`
  governance and "safe behavior" language.
- `docs/deep-research/robot-model-manifest-calibration.md` — `PolicyBundle`
  artifact fields (model, schema, normalization, history length, rate).
- `docs/deep-research/time-synchronization-and-determinism.md` — timeline and
  generation semantics that any staleness policy must compose with.
- [Open Duck Playground runner](https://github.com/apirrone/Open_Duck_Playground/blob/b9be205ac64488c23504ca42e5ec790337adeec3/playground/playground/open_duck_mini_v2/mujoco_infer.py)
- [Berkeley Humanoid Lite MuJoCo environment](https://github.com/HybridRobotics/berkeley-humanoid-lite/blob/984741a3623c93b0583ccfdc479f1f8b1c4d900e/source/berkeley_humanoid_lite/berkeley_humanoid_lite/environments/mujoco.py)
- [Unitree SDK2 G1 low-level example](https://github.com/unitreerobotics/unitree_sdk2_python/blob/65691c8a8bc53b98d3976dba4dbf9d5d20b2e7f5/example/g1/low_level/g1_low_level_example.py)
- [Unitree RL Lab policy thread](https://github.com/unitreerobotics/unitree_rl_lab/blob/4960b84732b0c2ec593dccbfe963fda1bcd7b1e3/deploy/include/FSM/State_RLBase.h) and [1 kHz FSM thread](https://github.com/unitreerobotics/unitree_rl_lab/blob/4960b84732b0c2ec593dccbfe963fda1bcd7b1e3/deploy/include/FSM/CtrlFSM.h)
- [AgiBot X1 control loop](https://github.com/AgibotTech/agibot_x1_infer/blob/9e0b818804d644fb9c9663e932dd33b03b24dfa4/src/module/control_module/src/control_module.cc) and [decimated RL inference](https://github.com/AgibotTech/agibot_x1_infer/blob/9e0b818804d644fb9c9663e932dd33b03b24dfa4/src/module/control_module/src/rl_controller.cc)

## Open questions

- Does the timeout/fallback policy belong entirely in `SA-3`, or does part of
  it require `SA-1`/`SA-0` involvement for high-dynamics embodiments (see the
  unresolved tension in `safety-and-fault-architecture.md` between
  `panic = "abort"` and controlled-crouch-style safe behavior)?
- Should chunk execution be modeled as a first-class RT-visible concept, or
  fully flattened into per-tick commands before the mailbox boundary?
- What common observation-alignment contract, if any, is justified after a
  future second policy workload rather than inferred from Duck alone?
