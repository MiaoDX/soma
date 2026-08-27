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

## Trained 300M baseline in the official demo

The locally trained best-eval checkpoint at step `257,556,480` remains upright
for 20 seconds in the official `MjInfer` loop on both the README default flat
scene and the `flat_terrain_backlash` training scene. On the backlash scene it
travels about `2.52 m` at `vx=0.15 m/s` and `1.75 m` at `vy=0.2 m/s`, while
keeping root height above `0.152 m` and absolute roll/pitch below `0.12 rad`.
The final 300M checkpoint behaves similarly.

The failed Soma rollout is therefore not evidence of a bad trained policy. The
pinned reference motion has a 27-policy-tick period, while Soma advances the
observation phase on a hard-coded 100-tick period and always supplies zero foot
contacts. Applying those two mismatches plus Soma's early first inference to
the official flat-scene loop reproduces the prior failure signature: root
height `-0.156 m`, roll `3.141 rad`, and pitch `1.555 rad` after eight seconds.

## Repaired Soma qualification

After carrying measured foot contacts through the fixed state codec and using
the pinned 27-tick phase, the same eight-second Soma direct harness produces:

| checkpoint | forward displacement | minimum root height | max roll/pitch |
|---|---:|---:|---:|
| trained best-eval `257,556,480` | `1.374 m` | `0.150 m` | `0.108 rad` |
| published `BEST_WALK_ONNX.onnx` | `1.410 m` | `0.150 m` | `0.117 rad` |
| published `BEST_WALK_ONNX_2.onnx` | `1.423 m` | `0.150 m` | `0.115 rad` |

The frozen published checkpoint also travels `1.410`, `1.420`, and `1.339 m`
with injected target delays of 0, 2, and 20 ms respectively, without violating
the posture envelope. Its supervised Rust/Zenoh/Python process rollout emits
393 policy targets over eight seconds with no dropped states, rejection, or
expiry and keeps root height above `0.155 m`. This clears the published
checkpoint and shows that the earlier latency conclusion was an adapter
artifact, not policy sensitivity.

## Velocity command sweep

The command is consumed as a velocity input, not a post-run speed setting. With
the same eight-second probe, `BEST_WALK_ONNX_2` displaced `-0.0056 m`, `0.0031
m`, `0.0186 m`, and `0.0276 m` at `vx=0.05`, `0.10`, `0.15`, and `0.30 m/s`.
`BEST_WALK_ONNX` displaced `0.0138 m`, `0.0350 m`, `0.0455 m`, and `0.1862 m`
at those commands. This confirms command sensitivity. The `0.30 m/s` probe is
diagnostic only because it exceeds the Playground runner's documented
`[-0.15, 0.15] m/s` command range.
