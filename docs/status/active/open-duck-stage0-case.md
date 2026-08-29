# Open Duck Mini v2 Frozen Case

Status: FROZEN Stage 0 reference contract (baseline revised by user approval,
2026-08-27).

| Field | Frozen value |
|---|---|
| checkpoint | `BEST_WALK_ONNX.onnx`, SHA-256 `cb61453a8bcb547ccfdeb4f03ba0fa67ebcf767dcf4aa6e5c9a0d92b302f9b23` |
| seed | `7` |
| duration | `8 s` |
| command | `[0.3, 0, 0]` m/s, yaw `0` (approved baseline; diagnostic range extension) |
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
