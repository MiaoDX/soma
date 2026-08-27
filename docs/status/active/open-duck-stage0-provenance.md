# Open Duck Mini v2 Stage 0 Provenance

This capsule records verified source and compatibility facts. It is not the
Stage 0 acceptance result: the vendored manifest, golden fixtures, frozen gait
floors, and repeated reference runs remain pending.

## Source pins

| Item | Value |
|---|---|
| Robot/checkpoint repository | `https://github.com/apirrone/Open_Duck_Mini.git` |
| Branch / commit | `v2` / `b23317a485b3cec7d8417f352478778b3475173c` |
| Reference model/runner repository | `https://github.com/apirrone/Open_Duck_Playground.git` |
| Reference commit | `b9be205ac64488c23504ca42e5ec790337adeec3` |
| License posture | Both repositories are treated as Apache-2.0 under the user's explicit direction |
| Checkpoint | `BEST_WALK_ONNX_2.onnx`, 884177 bytes, SHA-256 `3c606f9381a1710cc8fecdb7442787dcbfce3ee9bc02a6f1224774ab2b3a1067` |
| Model XML | `playground/open_duck_mini_v2/xmls/open_duck_mini_v2.xml`, SHA-256 `968b18de4e3f55b31252155f52779fa490989f5da92bc9b308e0bb4e81d6bb5c` |
| Flat scene XML | `playground/open_duck_mini_v2/xmls/scene_flat_terrain.xml`, SHA-256 `f087afa8bbe13f92e934d4a03571f2134abe7d5bb963fce22eec04822e9ae6d0` |

The vendored bundle contains only the flat-scene XML include closure,
referenced runtime meshes, checkpoint, licenses, and upstream manifest. It
excludes training code, the full Playground application, print/CAD sources,
rough/backlash scenes, and unused assets. Normal build, CI, and runtime do not
download upstream content.

## Verified model and policy contract

Loaded from the pinned flat scene with MuJoCo:

* `nq=21`, `nv=20`, `nu=14`, timestep `0.002` seconds;
* 14 policy actuators in the named order recorded by the plan;
* 15 model sensors and a `home` keyframe;
* physics cadence 500 Hz and policy/target cadence 50 Hz, with ten physics
  steps per policy period;
* ONNX input `obs`, float tensor `[1, 101]`;
* ONNX output `continuous_actions`, float tensor `[1, 14]`.

The 101 values are gyro 3, accelerometer 3 with `+1.3` on x, velocity/head
command 7, relative joint position 14, joint velocity scaled by `0.05` 14,
three action-history frames 42, previous motor target 14, foot contacts 2, and
gait phase cosine/sine 2.

Action application is `default_actuator + action * 0.25`. The motor target is
slew-limited to `5.24 rad/s` per 20 ms policy update; the present runner's
low-pass action filter is disabled. The official ordering is observe, infer,
update target, then continue 2 ms physics steps. It does not insert a fixed
20 ms policy-frame delay.

## Exploratory reference evidence

An eight-second headless reproduction with command `[0.1, 0, 0]` completed
400 policy ticks and 4000 physics steps, stayed upright, and moved about
`0.271 m` forward. This run established feasibility and the contract above.
It is explicitly exploratory and is not an acceptance sample. Stage 0 freezes
absolute gait floors before collecting the two accepted reference runs.

## Superseded main-repository mismatch

The main repository's older v2 runner uses a 16-actuator model, constructs a
different observation with zero padding, and cannot coherently apply the
14-output checkpoint. That finding remains useful provenance, but it no longer
blocks Stage 0: the pinned Playground 14-actuator model and runner are the
canonical compatibility reference.
