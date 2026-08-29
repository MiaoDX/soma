# Open Duck Mini v2 Frozen Case

Status: FROZEN Stage 0 reference contract.

| Field | Frozen value |
|---|---|
| checkpoint | `BEST_WALK_ONNX_2.onnx`, SHA-256 `3c606f9381a1710cc8fecdb7442787dcbfce3ee9bc02a6f1224774ab2b3a1067` |
| seed | `7` |
| duration | `8 s` |
| command | `[0.1, 0, 0]` m/s, yaw `0` |
| cadence | 500 Hz physics/RT, 50 Hz policy, decimation 10 |
| reset | model `home` keyframe |
| minimum duration | `7.5 s` |
| minimum forward displacement | `0.10 m` |
| root height floor | `0.08 m` |
| roll/pitch envelope | `|roll| <= 0.8 rad`, `|pitch| <= 0.8 rad` |
| non-foot collision count | `0` |
| injected delay | `20 ms`, original deadline retained; gait floors not required |

The reference command must fail closed if this file or the provenance manifest
is incomplete. Numeric tolerances for named golden observation/action ticks are
`1e-5` absolute for float32 values.
