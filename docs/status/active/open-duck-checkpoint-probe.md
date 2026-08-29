# Open Duck Checkpoint Compatibility Probe

Probe date: 2026-08-27. Scene: pinned
`Open_Duck_Playground` commit `b9be205ac64488c23504ca42e5ec790337adeec3`,
flat terrain, 8 seconds, command `[0.1, 0, 0]`, 500 Hz physics and 50 Hz
policy, same `101 -> 14` observation/action adapter.

| checkpoint | SHA-256 | ONNX result | displacement |
|---|---|---|---:|
| `BEST_WALK_ONNX.onnx` | `cb61453a8bcb547ccfdeb4f03ba0fa67ebcf767dcf4aa6e5c9a0d92b302f9b23` | finite `[1,14]` output | `0.0350 m` |
| `BEST_WALK_ONNX_2.onnx` | `3c606f9381a1710cc8fecdb7442787dcbfce3ee9bc02a6f1224774ab2b3a1067` | finite `[1,14]` output | `0.0031 m` |

Both files load successfully with ONNX Runtime CPU and satisfy the pinned
tensor contract. Neither clears the frozen `0.10 m` displacement floor. The
probe therefore rules out a corrupt ONNX file, but does not establish that
either published checkpoint is compatible with the pinned Playground model and
observation semantics.

The Playground repository contains the training and ONNX export path but no
committed pretrained checkpoint. Retraining and exporting a policy against the
pinned environment is the remaining technical fallback; it requires a separate
resource/time decision and must not silently replace the frozen checkpoint.

## Velocity command sweep

The command is consumed as a velocity input, not a post-run speed setting. With
the same eight-second probe, `BEST_WALK_ONNX_2` displaced `-0.0056 m`, `0.0031
m`, `0.0186 m`, and `0.0276 m` at `vx=0.05`, `0.10`, `0.15`, and `0.30 m/s`.
`BEST_WALK_ONNX` displaced `0.0138 m`, `0.0350 m`, `0.0455 m`, and `0.1862 m`
at those commands. This confirms command sensitivity. The `0.30 m/s` probe is
diagnostic only because it exceeds the Playground runner's documented
`[-0.15, 0.15] m/s` command range.
