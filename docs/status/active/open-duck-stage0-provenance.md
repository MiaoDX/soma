# Open Duck Mini v2 Stage 0 Provenance

This manifest records the currently verified Stage 0 source and compatibility
facts. It is not an acceptance result: gait floors and golden observations are
still pending the independent reference runner.

## Source pins

| Item | Value |
|---|---|
| Repository | `https://github.com/apirrone/Open_Duck_Mini.git` |
| Branch / commit | `v2` / `b23317a485b3cec7d8417f352478778b3475173c` |
| License | Apache-2.0 (`LICENSE`, SHA-256 `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4`) |
| Checkpoint | `BEST_WALK_ONNX_2.onnx`, 884177 bytes, SHA-256 `3c606f9381a1710cc8fecdb7442787dcbfce3ee9bc02a6f1224774ab2b3a1067` |
| Model XML | `mini_bdx/robots/open_duck_mini_v2/robot.xml`, SHA-256 `a134222e3b789f491b5a4a4781c6d19c91c86ad4d8481795b6b03f5eab6e0c7a` |
| Scene XML | `mini_bdx/robots/open_duck_mini_v2/scene.xml`, SHA-256 `52efe3c153ce1ea6e7b9c415f9b2ad0e7cb33f75b508fff565f980711176a725` |

The complete mesh dependency set is the 45 `.stl` files referenced by the
pinned `robot.xml`; their per-file checksums were captured during inspection
and must be emitted into the committed compatibility manifest when assets are
vendored. No unlicensed Runtime or Playground source was used.

## Compiled model facts

Loaded with Python `mujoco==3.6.0` from the pinned `scene.xml`:

* `nq=23`, `nv=22`, `nu=16`, timestep `0.002` seconds.
* 14 policy actuators are named, in order: `left_hip_yaw`, `left_hip_roll`,
  `left_hip_pitch`, `left_knee`, `left_ankle`, `neck_pitch`, `head_pitch`,
  `head_yaw`, `head_roll`, `right_hip_yaw`, `right_hip_roll`,
  `right_hip_pitch`, `right_knee`, `right_ankle`.
* Excluded actuators are `left_antenna` and `right_antenna`; their fixed value
  is not yet accepted and must be declared before rollout.

## ONNX facts

Loaded with `onnxruntime==1.24.4` using `CPUExecutionProvider` and `numpy==2.4.0`:

* input: `obs`, float tensor `[1, 101]`;
* output: `continuous_actions`, float tensor `[1, 14]`.

Observation segment meanings, action scaling, reset fill, phase convention,
and event ordering remain hypotheses until the golden tick is independently
reconstructed from licensed/public facts.

## Hard-stop finding

The Apache-2.0 repository's `experiments/v2/onnx_AWD_mujoco.py` and
`onnx_AWD_mujoco_motor_control.py` do not establish the contract required by
the approved plan. They build observations from projected gravity, all 16
joint positions, all 16 joint velocities, two contacts, 16 previous actions,
and three commands. One path pads that 83-value vector with 18 zeros to reach
101. Both initialize 16-action state, while the pinned checkpoint inspected
above emits 14 actions. The source therefore neither defines the plan's named
101-field ledger nor a coherent 14-action-to-16-actuator application rule.

The plan forbids copying or translating the unlicensed Runtime/Playground
implementation and requires stopping when the licensed source set cannot
establish exact semantics. Stage 0 is consequently blocked before rollout and
before any shared or Reachy code change.

## Exhausted licensed-source checks

The full `v2` history was fetched and inspected after the initial finding.
Commit `981750bd0d160bb7677d162a46dff1b0871c7135`, which introduced
`BEST_WALK_ONNX_2.onnx`, contains the same 16-joint observation construction,
18-zero padding, and 14-output incompatibility. Later history contains no
corrected runner or alternate feature ledger for this checkpoint. The ONNX
model metadata is empty apart from `tf2onnx` producer/graph identification, so
it provides no feature order, normalization, action mapping, or reset/event
semantics.

The Runtime repository still resolves to the plan's inspected commit
`32037347dc43186a017f2116bcfde7c461b81f54` and still exposes no `LICENSE` at
its default branch. These checks leave no license-compatible authoritative
source from which to prove the exact Stage 0 contract.
