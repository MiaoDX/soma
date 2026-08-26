# Open Duck Mini v2 Frozen Case (Stage 0 Draft)

This file is intentionally incomplete until the independent reference runner
produces evidence. Values below are source/toolchain facts only; gait floors
must be predeclared from robot scale and documented examples before observing
the checkpoint output.

| Field | Frozen value |
|---|---|
| policy checkpoint | `BEST_WALK_ONNX_2.onnx` SHA-256 `3c606f9381a1710cc8fecdb7442787dcbfce3ee9bc02a6f1224774ab2b3a1067` |
| control cadence | 50 Hz policy/control; MuJoCo timestep 0.002 s |
| inference provider | ONNX Runtime CPU execution provider |
| observation width | 101 float32 values |
| action width | 14 float32 values |
| delayed schedule | state tick `k` may first apply at `k+1`, then held until replaced/expired |
| seed, duration, command, reset pose, floors | PENDING Stage 0 reference contract |

The reference command must fail closed while any PENDING field remains.
