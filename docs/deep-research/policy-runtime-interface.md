# Policy/Inference to Real-Time Interface

> Status: M1a contract narrowed by D-19. Interpolation ownership, command TTL,
> `control_mode -> SafeBehavior`, and lineage are current scope. Observation
> alignment and action chunking are deferred until a real policy workload.

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

The following questions are real but deferred because M1 has no trained policy
or training/deployment pipeline to validate them against:

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
inference and perception, and its effect on `robot-runtime` scheduling — is
noted here but out of scope until a real inference workload exists.

### Why this cannot wait for real hardware or a real model

Interpolation, TTL, safe-behavior selection and lineage are pure
control-and-timing semantics; none requires a trained policy, a GPU or
physical hardware. A lookup table or sine wave at a fixed low rate is enough
to force those paths. Observation alignment and chunking, by contrast, are
not frozen without a representative model and pipeline.

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
- Decide interpolator ownership before the RT/runtime mailbox layout is
  finalized; changing it later changes the mailbox contract.
- Extend the requested/admitted/safety-output/applied command lineage
  (`docs/architecture/diagrams/soma-command-lineage.svg`) with a documented
  case for "no new requested command this tick" so a stale-source event is
  visible in the same evidence trail as a rejected command, not silently
  absent from it.
- M1a should include a synthetic 20 Hz cadence source (no trained model
  required) specifically to force the timeout, interpolation, and staleness
  paths to exist before hardware or a real policy arrive. See the
  `cadence-source-decimation` row proposed in the bootstrap plan's
  verification matrix.
- Do not put observation alignment or action chunks into the M1 mailbox or
  public schema. Reopen them when a representative policy workload exists.

## Trade-offs

Freezing interpolation, TTL, safe behavior and lineage now prevents the
mailbox boundary from being set accidentally and closes the stale-command
fault path. Deferring alignment and chunking avoids designing a policy
pipeline that M1 cannot evaluate empirically.

## Sources

- `docs/architecture/reference-architecture.md` — `PolicyBundle`,
  requested/admitted/safety-output/applied lineage.
- `docs/deep-research/safety-and-fault-architecture.md` — `SafetyProfile`
  governance and "safe behavior" language.
- `docs/deep-research/robot-model-manifest-calibration.md` — `PolicyBundle`
  artifact fields (model, schema, normalization, history length, rate).
- `docs/deep-research/time-synchronization-and-determinism.md` — timeline and
  generation semantics that any staleness policy must compose with.

## Open questions

- Does the timeout/fallback policy belong entirely in `SA-3`, or does part of
  it require `SA-1`/`SA-0` involvement for high-dynamics embodiments (see the
  unresolved tension in `safety-and-fault-architecture.md` between
  `panic = "abort"` and controlled-crouch-style safe behavior)?
- Should chunk execution be modeled as a first-class RT-visible concept, or
  fully flattened into per-tick commands before the mailbox boundary?
- What is the minimum viable observation-alignment contract that keeps
  batch/production paths bit-identical without over-specifying a training
  pipeline Soma does not own?
