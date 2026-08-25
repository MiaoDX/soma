# Soma vs Official Reachy Simulation

This report compares behavior, not bitwise-identical trajectories.
All actuator values are radians in the fixed nine-actuator order.
Host monotonic clocks define each implementation's stimulus/observation window.

## Inputs

- `soma.jsonl` (generated evidence)
- `official.jsonl` (generated evidence)

## Definitions

- RMS/max error: measured minus target over the two-second dwell.
- Settling: first sample after which absolute error stays within 0.03 rad.
- Steady-state error: mean error over the final 20% of samples.
- Overshoot: maximum target crossing in the commanded direction.
- Timing: harness-local monotonic timestamps; no cross-process clock equivalence is claimed.
- p99 is unavailable because this trace does not provide enough repeated runs.

## Capability Labels

Official sequence, TTL, timeline, disposition, and rejection semantics are `UNAVAILABLE`; they are not synthesized or scored. Soma retains them as `SOMA_ONLY` evidence.

## Metrics

### Soma

Samples: 101
First observation latency: 19.327 ms
Median/max update interval: 20.011 / 22.977 ms

| Actuator | RMS | Max | Overshoot | Steady | Settling ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| yaw_body | 0.050043 | 0.150000 | 0.005832 | 0.004950 | 479.228 |
| stewart_1 | 0.000718 | 0.001115 | 0.000000 | -0.001034 | 19.327 |
| stewart_2 | 0.000792 | 0.001240 | 0.000000 | 0.001164 | 19.327 |
| stewart_3 | 0.000834 | 0.001243 | 0.000000 | -0.001157 | 19.327 |
| stewart_4 | 0.000828 | 0.001294 | 0.000007 | 0.001216 | 19.327 |
| stewart_5 | 0.000769 | 0.001178 | 0.000000 | -0.001096 | 19.327 |
| stewart_6 | 0.000610 | 0.000993 | 0.000000 | 0.000919 | 19.327 |
| right_antenna | 0.042975 | 0.150000 | 0.000000 | -0.005003 | 459.168 |
| left_antenna | 0.042907 | 0.150000 | 0.000000 | 0.005110 | 459.168 |

### Official

Samples: 100
First observation latency: 0.167 ms
Median/max update interval: 20.094 / 20.237 ms

| Actuator | RMS | Max | Overshoot | Steady | Settling ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| yaw_body | 0.024167 | 0.150000 | 0.005899 | 0.000105 | 80.629 |
| stewart_1 | 0.005942 | 0.007098 | 0.007098 | 0.005744 | 0.167 |
| stewart_2 | 0.023385 | 0.026087 | 0.000000 | -0.026059 | 0.167 |
| stewart_3 | 0.020185 | 0.022574 | 0.000000 | -0.022528 | 0.167 |
| stewart_4 | 0.008203 | 0.009551 | 0.009551 | 0.009523 | 0.167 |
| stewart_5 | 0.045455 | 0.050502 | 0.050502 | 0.050458 | unsettled |
| stewart_6 | 0.005207 | 0.008614 | 0.000009 | -0.002862 | 0.167 |
| right_antenna | 0.022954 | 0.150000 | 0.000009 | 0.000009 | 80.629 |
| left_antenna | 0.022939 | 0.150000 | 0.000010 | -0.000010 | 80.629 |

## Interpretation Boundary

Semantic differences, motion tracking, and runtime timing are separate evidence. The report does not claim hardware parity, reliability, resource usage, or identical physics.
