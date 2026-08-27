# Open Duck Mini v2 Frozen Case (Stage 0 Draft)

This file remains incomplete until Stage 0 freezes the acceptance case. The
earlier eight-second rollout is exploratory, not an acceptance sample. Absolute
gait floors must now be declared from robot scale and documented examples
before collecting the accepted reference runs.

| Field | Frozen value |
|---|---|
| policy checkpoint | `BEST_WALK_ONNX_2.onnx` SHA-256 `3c606f9381a1710cc8fecdb7442787dcbfce3ee9bc02a6f1224774ab2b3a1067` |
| cadence | 500 Hz RT/physics; 50 Hz policy frame/target; decimation 10 |
| inference provider | ONNX Runtime CPU execution provider |
| observation width | 101 float32 values |
| action width | 14 float32 values |
| normal schedule | observe, infer, send immediately, apply on the first available RT tick, then zero-order hold until replaced/expired |
| injected delay | explicit 20 ms fault case; consumes the original deadline and is not subject to the normal gait floor |
| seed, duration, command, reset pose, floors | PENDING Stage 0 reference contract |

The reference command must fail closed while any PENDING field remains.
