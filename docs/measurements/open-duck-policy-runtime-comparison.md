# Open Duck Policy Runtime Comparison

Measured on 2026-09-02 using the frozen Open Duck case
`59a29cb8ba9acc460267be7cd4929fcf7344cfe031cd19490247c14bf3c6a184`.
Both backends ran the complete isolated runtime/RT/policy process topology five
times. Each cell is `mean [min, max]` across rollouts.

| Metric | Unit | Rust | Python |
| --- | --- | ---: | ---: |
| Emitted targets | count | 381.800 [380.000, 383.000] | 384.600 [383.000, 386.000] |
| Run max inference | ms | 0.318 [0.276, 0.365] | 0.434 [0.391, 0.455] |
| Mean inference | ms | 0.157 [0.148, 0.166] | 0.201 [0.187, 0.210] |
| Run max message age | ms | 20.077 [20.029, 20.121] | 21.331 [20.041, 23.380] |
| Dropped states | count | 4.600 [3.000, 6.000] | 1.400 [0.000, 3.000] |
| Runtime dropped targets | count | 0.600 [0.000, 1.000] | 0.000 [0.000, 0.000] |
| Run min root height | m | 0.154 [0.150, 0.157] | 0.152 [0.150, 0.156] |
| Run max absolute roll | rad | 0.092 [0.085, 0.097] | 0.086 [0.080, 0.092] |
| Run max absolute pitch | rad | 0.115 [0.106, 0.120] | 0.117 [0.110, 0.125] |

Rust mean inference latency was 21.9% lower and its mean run-maximum inference
latency was 26.6% lower than Python in this sample. End-to-end message age and
motion envelopes were comparable. All runs applied targets and completed; two
Rust runs and one Python run latched at least one rejection, which remains
visible rather than being excluded from the comparison.

This is a same-host qualification baseline, not a general language benchmark.
It does not measure CPU, RSS, startup time, p50/p95/p99 latency, hardware
behavior, or statistical significance. Raw run evidence is generated under
ignored `output/` by:

```bash
scripts/compare-open-duck-policy-metrics --repeat 5
```
