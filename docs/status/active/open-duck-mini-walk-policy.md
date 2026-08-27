status: ACTIVE
source_plan: docs/plans/open-duck-mini-walk-policy.md
control_plane: /root
latest_intent: train and evaluate a new Open Duck policy using the pinned Playground path because the published checkpoint is implausibly sensitive to one 2 ms application delay
current_slice: production-shaped 8192-environment GPU capacity gate complete; long training not started
blocker: the frozen published policy still cannot meet the asynchronous Soma gait gate; training is now an approved reshaping experiment, not evidence that the blocker is resolved
blocker_fingerprint: open-duck-same-tick-vs-async-contract-conflict-v1
last_proven: two 163,840-step calibrations completed with the official 8192-env, batch-256, unroll-20, 32-minibatch, 4-update PPO shape; 200 ms sampling measured 4,801 MiB peak GPU memory and 100% utilization; warm training throughput was 18,950 steps/s
timing_contract: 500 Hz RT/physics, 50 Hz policy frame; send immediately from captured state; apply latest admitted target on first available 2 ms RT tick; bounded zero-order hold; fixed 20 ms delay is fault injection only
completed: source checkout, compiled model inspection, provenance audit, ONNX metadata audit, official headless rollout, policy timing comparison, minimal vendored bundle, frozen Stage 0 case, user-approved baseline selection, two passing baseline reference runs, const-generic core seam with Reachy aliases, fixed OpenDuckSimPlant, isolated Duck transport contract, synthetic 50/500 Hz bounded-path tests, combined state/evidence transport, thin Python CPU ONNX policy client, exact golden/fault/process evidence tests, same-tick versus asynchronous isolation, a restorable 10k-step Playground GPU smoke checkpoint, and the 8192-environment production-shaped GPU capacity/throughput calibration
next_slice: launch the fixed-seed 150M upstream-baseline training run after user confirmation of the calibration result; retain all checkpoints and metrics outside Soma
next_proof: the baseline candidate must train and export successfully, then pass zero-delay, 2 ms, 20 ms, and full Soma process evaluations without changing frozen floors
stop_condition: stop if provenance/license, exact model mapping, policy semantics, or same-tick reference gait floors cannot be established without unapproved sources or tuning
no_touch: Reachy hardware probe and N0/N1; Open Duck hardware; generic robot manifests; ROS 2; interactive control; camera/audio/antennas/expressions; silent replacement of the frozen checkpoint or vendoring training artifacts into Soma
parked: interactive velocity input and simultaneous profiles remain deferred; MuJoCo/Rerun visualization is in first accepted rollout scope; post-Stage-4 D-04 review compares process isolation with Rust-hosted and single-process async/periodic alternatives
