# Open Duck Mini Community Repository Research

Research date: 2026-08-26

## Research Brief

- Question: Are there community repositories besides the three upstream
  `apirrone` repositories that can provide a legally usable, reproducible
  Open Duck Mini v2 policy contract or MuJoCo walk reference?
- Decision: Determine whether Stage 0 can use a new independent reference
  source, and identify candidates worth legal/technical review.
- Included: GitHub repositories matching Open Duck Mini, BDX/mini BDX,
  `BEST_WALK_ONNX_2.onnx`, `open_duck_mini_v2`, MuJoCo, ONNX, Isaac Lab, or
  runtime terms; direct source inspection of the strongest candidates.
- Excluded: generic biped projects without Open Duck assets or the pinned
  policy; hardware-only build logs unless they add a policy contract.
- Freshness: GitHub API/search state and repository HEADs observed 2026-08-26.
- Completion test: each material candidate has a source URL, commit, license
  status, execution surface, and relevance to the Soma Stage 0 gate.

## Executive Summary

The community has more relevant projects than the three original upstream
repositories, but they split into different categories:

1. `JAEWOOK6488/open-duck-web` is the strongest new technical reference. Its
   source is MIT, it includes the pinned checkpoint with the exact SHA-256
   `3c606f...a1067`, publishes an explicit 101-field/14-action contract, and
   its Node headless verification passed locally: 12 seconds of simulation,
   0.948 m forward displacement, minimum body height 0.157 m, alternating foot
   contacts, and all checks green. It uses a 14-actuator trimmed model
   (`nq=21`, `nv=20`, `nu=14`), not the full upstream 16-actuator model.
2. `TimeTreker/soridormi` is MIT and contains an extensive independent runtime,
   observation builder, policy profiles, parity tooling, and the same upstream
   submodules. Its own docs describe the exact 101 layout, but it explicitly
   depends on the unlicensed upstream Runtime/Playground submodules and its
   docs describe first-walk attempts rather than a frozen acceptance result.
3. `RobVanProd/open-duck-mini-rdkx5-native-runtime` documents and tests a
   101/14 contract and has offline mock timing evidence, but it is a hardware
   runtime for D-Robotics RDK-X5. The checked-in phase-5 policy gate is
   `NOT_RUN`; it does not provide a MuJoCo reference rollout or the pinned
   checkpoint.
4. `SteveNguyen/openduckminiv2_playground`, `GiulioRomualdi/isaaclab.open_duck_mini`,
   `zhangzijie-pro/open-duck-isaac-lab`, and `benoit-robotics/bdx_walk_rl` are
   training/simulation environments. They train or play new policies and do
   not ship the pinned `BEST_WALK_ONNX_2.onnx` contract needed by this plan.
5. Forks and deployment wrappers are numerous, but GitHub metadata reports no
   declared license for most Runtime/Playground forks. Forking an unlicensed
   source does not resolve the provenance issue.

The new browser project is therefore a promising independent compatibility
source, not yet a complete Stage 0 unblock. It can support the 101-field and
14-output semantics under a plan amendment and legal review, but it cannot by
itself prove the required full-model 16-actuator mapping and excluded antenna
values.

## Findings

### `JAEWOOK6488/open-duck-web`

Source: https://github.com/JAEWOOK6488/open-duck-web (HEAD
`b3ff1285575a21c10f4c1e8953f9a1a48f3f53f6`). Source code is MIT; `NOTICE`
attributes the Open Duck model/mesh/policy to Apache-2.0 upstream and records
the asset sources. The included `public/policy/walk.onnx` hash is exactly
`3c606f9381a1710cc8fecdb7442787dcbfce3ee9bc02a6f1224774ab2b3a1067`.

The repository states and implements this layout:

```text
gyro 3 + accelerometer 3 + command 7 + joint offset 14
+ scaled velocity 14 + action history 42 + motor targets 14
+ contacts 2 + imitation phase 2 = 101
```

It records `action_scale=0.25`, velocity scale `0.05`, 50 Hz policy cadence,
500 Hz simulation, decimation 10, phase period 27, accelerometer x bias 1.3,
and the exact 14 controlled joint names. Its `scripts/verify.mjs` is an
independent Node/MuJoCo-WASM/ONNX Runtime runner. After `npm install` and
`npm run assets`, I ran `npm run verify` successfully in the current
environment. The runner reported `nq=21 nv=20 nu=14`, finite 101-value
observations, no fall, 0.948 m final x, 0.095 m/s average speed, alternating
contacts, head command movement, and exit status 0.

Important limitation: its XML is a trimmed 14-actuator model copied from the
Playground XML, not the full pinned upstream `robot.xml` (`nq=23`, `nv=22`,
`nu=16`). It proves a policy-compatible 14-actuator simulation, but not the
plan's required validation of two excluded antenna actuators and their fixed
values in the full model. Its `NOTICE` says the interface followed the
upstream inference scripts; this is an independent implementation, but the
legal status of relying on undocumented facts from an unlicensed repository
still deserves explicit review.

### `TimeTreker/soridormi`

Source: https://github.com/TimeTreker/soridormi (MIT; HEAD observed
`ea40c45c0286809038387a06878b249292eb8873`). It has a detailed independent
101/14 observation builder, policy profile, action mapper, ONNX inspection,
trace parity tools, and MuJoCo runtime. Its documented layout matches the
required segments and includes the 1.3 accelerometer bias, 0.05 velocity scale,
three action-history frames, motor targets, contacts, and phase.

However, its README explicitly adds `Open_Duck_Mini`,
`Open_Duck_Mini_Runtime`, and `Open_Duck_Playground` as submodules and says
those upstream repositories retain their own licenses. Its docs describe
first-walk and parity debugging and say stable walking is not guaranteed by
the first-walk patch. It is useful corroboration and engineering evidence,
but not an independently licensed replacement for the missing upstream
contract by itself.

### `RobVanProd/open-duck-mini-rdkx5-native-runtime`

Source: https://github.com/RobVanProd/open-duck-mini-rdkx5-native-runtime (HEAD
`b3b9b5ce2c528c3d88bb65258f9aa55c44a944e9`). The README documents a frozen
101/14 contract, 50 Hz timing, action scaling, 14 servo order, stale-sensor
rejection, and offline mock timing tests. The checked-in phase-5 policy gate
is `NOT_RUN`, and the project is aimed at direct hardware on an RDK-X5. No
MuJoCo headless walk reference or pinned checkpoint was found. Treat it as
contract corroboration only.

### Other simulation/training projects

- `SteveNguyen/openduckminiv2_playground` (HEAD
  `bd5a916bcb55f6cd93775e262b40574ad6b41af2`) has MuJoCo training and
  `mujoco_infer.py`, but no declared license and no pinned checkpoint in the
  repository.
- `GiulioRomualdi/isaaclab.open_duck_mini` (BSD-3-Clause, HEAD
  `dc63a9be7c30f33beb569ad9d8bf496b236e3fb0`) provides Isaac Lab AMP/RL tasks;
  policies are expected as generated checkpoints and have not been physically
  deployed. It is a different training stack, not a reference for the fixed
  ONNX policy.
- `zhangzijie-pro/open-duck-isaac-lab` (BSD-3-Clause, HEAD
  `2e85abe5f22b61e9e121f74f91823a814e02734b`) provides direct RL/imitation
  tasks and 14 controlled joints, but expects Isaac Lab checkpoints and does
  not ship the fixed ONNX policy.
- `benoit-robotics/bdx_walk_rl` (MIT metadata via GitHub, HEAD
  `3574c28365128036ac7abf725e7a999080aec7f8`) trains a related BDX robot in
  Isaac Lab using RSL-RL; it is not the Open Duck v2 policy/model contract.
- `dillondesilva/reconstructing-open-duck-mini-walk-engine` (HEAD
  `3de13de11731d549b67a0925871c589beb6dd141`) reconstructs reference-motion
  generation, not the ONNX walk runner, and has no declared license found.
- `rocketpowerllc/openduckrust` (HEAD
  `20560f280c7cc7ff611810a7c20b4679dcd2a88a`) is a Rust hardware runtime
  derivative; its README describes a 1:1 port of the unlicensed Python
  Runtime and no license file was found. It does not solve the simulation
  contract gate.

## Contradictions And Uncertainty

### Is `101 -> 14` now established?

- Position A: the original licensed Open Duck repository's checked-in MuJoCo
  scripts are internally inconsistent: they build 16-joint state/history and
  pad 18 values, while the ONNX emits 14 actions.
- Position B: independent community implementations (`open-duck-web`,
  Soridormi, and the RDK-X5 runtime) converge on the 101 layout with 14
  controlled joints.
- Current judgment: the layout is now strongly supported as a public
  compatibility fact, and `open-duck-web` provides executable evidence. The
  full-model excluded-actuator contract remains unresolved. Confidence:
  supported for policy layout; unresolved for full-model mapping.

### Does MIT source licensing license the upstream assets?

No. `open-duck-web`'s `NOTICE` separates MIT source from Apache-2.0 model,
meshes, and policy. That is a plausible and auditable arrangement, but Soma
must still record the exact asset provenance and confirm that the trimmed XML
and mesh modifications are covered by the upstream Apache terms. Confidence:
supported, not legal advice.

## Gaps

- No independent candidate found that runs the pinned ONNX against the full
  upstream 16-actuator model while declaring the two excluded actuator values.
- No public repository found with a complete, explicit license for the original
  Runtime or Playground source at their referenced commits.
- GitHub API/search and Sourcegraph searches are subject to rate limits and
  backend omissions; results are not an exhaustive census of every fork.
- The browser project's GitHub Actions status could not be queried because of
  transient GitHub API 403/rate limits; local verification succeeded after
  installing the locked dependencies and running its asset-generation step.

## Recommendation

Use `open-duck-web` as a candidate independent policy-contract/reference
source, subject to a plan/legal amendment. Before resuming Stage 0:

1. Obtain permission to use its MIT source and confirm the `NOTICE` treatment
   of the Apache model/policy assets.
2. Port or rerun its contract against the full upstream model and explicitly
   declare/check `left_antenna` and `right_antenna` fixed values.
3. Freeze its verified observation/action fixtures and compare them with the
   full-model fixture.
4. Only then amend the Stage 0 capsule/plan and resume implementation.

Do not use Runtime/Playground code or a fork merely because it has a different
owner; the GitHub search showed most such forks have no declared license.

## Method

I searched GitHub repository metadata for exact names, checkpoint name, and
Open Duck/MuJoCo/ONNX combinations; inspected primary repository READMEs,
licenses, dependency manifests, relevant source files, and submodule pins; and
ran the strongest candidate's headless verifier locally. Search snippets and
repository counts were used only for discovery. The source repositories and
local command outputs are the authoritative evidence for technical claims.
