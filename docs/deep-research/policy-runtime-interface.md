# Policy/Inference to Real-Time Interface

> Status: Deep Research. Seeded from architecture review; needs a bounded
> experiment before any part of it becomes an ADR.

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

This is not a hypothetical concern for a later milestone. `SafetyProfile`
already claims ownership of "safe behavior" under fault conditions
(`safety-and-fault-architecture.md`). A missed or late policy inference is a
fault condition. Until this document has an answer, that fault path is
undefined, and no `SA-3` implementation can claim to handle it.

## Findings

### The four unresolved questions

1. **Timeout policy.** When no new command arrives before an RT tick needs
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

The four questions above are pure control-and-timing semantics; none of them
require a trained policy, a GPU, or physical hardware to answer. They can be
exercised with a synthetic command source that produces a value at a fixed
low rate — a lookup table or a sine wave is sufficient to force every
timeout, interpolation, and staleness code path.

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

- Add a `command_staleness_policy` (or equivalent) field to `SafetyProfile`,
  scoped by embodiment class, with hold/decay/fallback/inhibit as the
  initial enum.
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

## Trade-offs

Answering these questions now, before a real model exists, costs design time
against artifacts nobody can yet evaluate empirically. Not answering them
means `SafetyProfile`'s "safe behavior under fault" claim has an unaddressed
fault class through V0 and M2, and the mailbox/interpolator boundary risks
being fixed by accident rather than by decision once real integration work
starts.

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
