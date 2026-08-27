status: ACTIVE
source_plan: docs/plans/open-duck-mini-walk-policy.md
control_plane: /root
latest_intent: train and evaluate a new Open Duck policy using the pinned Playground path because the published checkpoint is implausibly sensitive to one 2 ms application delay
current_slice: bounded Playground training experiment approved; 10k-step GPU smoke complete; long training not started
blocker: the frozen published policy still cannot meet the asynchronous Soma gait gate; training is now an approved reshaping experiment, not evidence that the blocker is resolved
blocker_fingerprint: open-duck-same-tick-vs-async-contract-conflict-v1
last_proven: pinned Playground commit b9be205 trains on the local RTX 3090 after reconstructing its 2025-compatible Python 3.12 environment; a 10,240-step smoke completed, wrote and restored a 23-leaf Orbax checkpoint, and exercised GPU PPO end to end
timing_contract: 500 Hz RT/physics, 50 Hz policy frame; send immediately from captured state; apply latest admitted target on first available 2 ms RT tick; bounded zero-order hold; fixed 20 ms delay is fault injection only
completed: source checkout, compiled model inspection, provenance audit, ONNX metadata audit, official headless rollout, policy timing comparison, minimal vendored bundle, frozen Stage 0 case, user-approved baseline selection, two passing baseline reference runs, const-generic core seam with Reachy aliases, fixed OpenDuckSimPlant, isolated Duck transport contract, synthetic 50/500 Hz bounded-path tests, combined state/evidence transport, thin Python CPU ONNX policy client, exact golden/fault/process evidence tests, same-tick versus asynchronous isolation, and a restorable 10k-step Playground GPU smoke checkpoint
next_slice: report smoke result and resource implications, then run a production-shaped throughput/memory calibration before any 150M-step baseline or latency-aware training run
next_proof: fixed-seed calibration must establish that the 24 GB RTX 3090 can sustain a production-shaped PPO configuration and produce a restorable checkpoint; trained candidates must then pass zero-delay, 2 ms, 20 ms, and full Soma process evaluations without changing frozen floors
stop_condition: stop if provenance/license, exact model mapping, policy semantics, or same-tick reference gait floors cannot be established without unapproved sources or tuning
no_touch: Reachy hardware probe and N0/N1; Open Duck hardware; generic robot manifests; ROS 2; interactive control; camera/audio/antennas/expressions; silent replacement of the frozen checkpoint or vendoring training artifacts into Soma
parked: interactive velocity input and simultaneous profiles remain deferred; MuJoCo/Rerun visualization is in first accepted rollout scope; post-Stage-4 D-04 review compares process isolation with Rust-hosted and single-process async/periodic alternatives
