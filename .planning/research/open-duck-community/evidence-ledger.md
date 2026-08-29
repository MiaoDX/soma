# Evidence Ledger: Open Duck Community Research

| claim_id | claim | source | source_class | evidence | confidence |
|---|---|---|---|---|---|
| C1 | `open-duck-web` source is MIT and separates upstream Apache assets | https://github.com/JAEWOOK6488/open-duck-web/blob/main/LICENSE; https://github.com/JAEWOOK6488/open-duck-web/blob/main/NOTICE | Primary | LICENSE and NOTICE explicitly split source vs model/mesh/policy | Verified |
| C2 | `open-duck-web` embeds the pinned checkpoint | https://github.com/JAEWOOK6488/open-duck-web/blob/main/NOTICE; local SHA-256 | Primary | `walk.onnx` hash `3c606f...a1067`, exact plan hash | Verified |
| C3 | `open-duck-web` defines the 101 layout and 14 actions | https://github.com/JAEWOOK6488/open-duck-web/blob/main/src/constants.js; `src/policy.js` | Primary | explicit segment table and implementation | Verified |
| C4 | `open-duck-web` headless walk verifier passes locally | local checkout HEAD `b3ff128`, `scripts/verify.mjs`, command `npm run assets && npm run verify` | Primary | exit 0; 0.948 m x, min z 0.157 m, contacts and all checks green | Verified |
| C5 | Its model is trimmed to 14 actuators | local `verify` output and `public/model` XML | Primary | `nq=21 nv=20 nu=14`; XML hashes match Playground trimmed XML | Verified |
| C6 | Soridormi is MIT but retains unlicensed upstream submodules | https://github.com/TimeTreker/soridormi; README; git submodule status | Primary | MIT LICENSE; README says upstream repos retain own licenses; pinned submodules | Verified |
| C7 | Soridormi implements a 101/14 builder | https://github.com/TimeTreker/soridormi/blob/main/src/soridormi_runtime/observation_builder.py | Primary | segment comments total 101; action size 14 | Verified |
| C8 | RDK-X5 runtime documents 101/14 but has no MuJoCo acceptance | https://github.com/RobVanProd/open-duck-mini-rdkx5-native-runtime; `docs/OBSERVATION_ACTION_CONTRACT.md`; `artifacts/gates/phase_5_policy/NOT_RUN.md` | Primary | contract and offline mock described; policy gate NOT_RUN | Verified |
| C9 | Isaac Lab candidates train different policies | https://github.com/GiulioRomualdi/isaaclab.open_duck_mini; https://github.com/zhangzijie-pro/open-duck-isaac-lab; https://github.com/benoit-robotics/bdx_walk_rl | Primary | READMEs describe Isaac Lab tasks/checkpoints, not pinned ONNX | Supported |
| C10 | Runtime/Playground forks generally do not declare a license | GitHub API repository search for `Open_Duck_Mini_Runtime` and `Open_Duck_Playground` | Primary metadata | `license` is null for upstream and sampled forks | Supported |
