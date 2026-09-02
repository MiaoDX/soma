# Open Duck Policy Runtime Comparison

Measured on 2026-09-02 using the frozen Open Duck case
`59a29cb8ba9acc460267be7cd4929fcf7344cfe031cd19490247c14bf3c6a184`.
Both backends ran the complete isolated runtime/RT/policy process topology five
times. Each cell is `mean [min, max]` across rollouts.

| Metric | Unit | Rust | Python |
| --- | --- | ---: | ---: |
| Emitted targets | count | 385.600 [385.000, 386.000] | 384.800 [383.000, 386.000] |
| Run max inference | ms | 0.289 [0.229, 0.314] | 0.402 [0.389, 0.411] |
| Mean inference | ms | 0.147 [0.123, 0.168] | 0.179 [0.138, 0.206] |
| Run max message age | ms | 20.483 [20.048, 22.198] | 20.089 [20.071, 20.112] |
| Dropped states | count | 0.000 [0.000, 0.000] | 1.000 [0.000, 3.000] |
| Runtime dropped targets | count | 0.000 [0.000, 0.000] | 0.000 [0.000, 0.000] |
| Publisher matching wait | ms | 439.094 [423.152, 467.565] | 0.000 [0.000, 0.000] |
| Run min root height | m | 0.150 [0.150, 0.150] | 0.153 [0.150, 0.157] |
| Run max absolute roll | rad | 0.084 [0.084, 0.084] | 0.090 [0.080, 0.105] |
| Run max absolute pitch | rad | 0.117 [0.117, 0.117] | 0.118 [0.110, 0.127] |

Rust mean inference latency was 17.9% lower and its mean run-maximum inference
latency was 28.1% lower than Python in this sample. End-to-end message age and
motion envelopes were comparable. All runs applied targets and completed.

All six attributable rejection counts were zero for both backends: decode,
timeline, sequence, expired, invalid, and runtime generation. The Rust worker
waited 423-468 ms for Zenoh to confirm a matching target subscriber before it
reported ready. Before that readiness gate, 4 of 5 Rust runs produced 6 expired
targets at ages 62-126 ms against the fixed 40 ms TTL, concentrated at early
state sequences 11 and 31. Ten Rust qualification runs after the gate and this
final 5-run Rust sample produced no rejection. This isolates and closes the
startup publisher-matching race without changing TTL or admission semantics.

This is a same-host qualification baseline, not a general language benchmark.
It does not measure CPU, RSS, startup time, p50/p95/p99 latency, hardware
behavior, or statistical significance. Raw run evidence is generated under
ignored `output/` by:

```bash
scripts/compare-open-duck-policy-metrics --repeat 5
```
