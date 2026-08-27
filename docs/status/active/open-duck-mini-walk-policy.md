status: ACTIVE
source_plan: docs/plans/open-duck-mini-walk-policy.md
control_plane: /root
latest_intent: execute the approved Open Duck Mini walk-policy plan with intuitive-flow
current_slice: Stage 4 full-duration process-path gait gate evaluated
blocker: full 8 s Soma process rollout fails frozen root-height and roll/pitch floors despite valid applied lineage and no rejection
blocker_fingerprint: soma-process-gait-floor-failure-v1
last_proven: 2 s supervised clean and stall control evidence passes, but the required 8 s process rollout falls: minimum root height -0.189 m, maximum absolute roll 3.141 rad, maximum absolute pitch 1.430 rad; 393 policy states/targets observed with no rejection
timing_contract: 500 Hz RT/physics, 50 Hz policy frame; send immediately from captured state; apply latest admitted target on first available 2 ms RT tick; bounded zero-order hold; fixed 20 ms delay is fault injection only
completed: source checkout, compiled model inspection, provenance audit, ONNX metadata audit, official headless rollout, policy timing comparison, minimal vendored bundle, frozen Stage 0 case, user-approved baseline selection, two passing baseline reference runs, const-generic core seam with Reachy aliases, fixed OpenDuckSimPlant, isolated Duck transport contract, synthetic 50/500 Hz bounded-path tests, combined state/evidence transport, and thin Python CPU ONNX policy client
next_slice: finish Stage 3 exact golden observation/action and deadline/fault integration
next_proof: determine whether the failed full-duration gait is caused by policy-frame loss/jitter without tuning the frozen case; stop for reshaping if the pinned asynchronous path cannot meet the frozen floors
stop_condition: stop if provenance/license, exact model mapping, policy semantics, or same-tick reference gait floors cannot be established without unapproved sources or tuning
no_touch: Reachy hardware probe and N0/N1; Open Duck hardware; generic robot manifests; policy training; ROS 2; interactive control; camera/audio/antennas/expressions
parked: interactive velocity input and simultaneous profiles remain deferred; MuJoCo/Rerun visualization is in first accepted rollout scope; post-Stage-4 D-04 review compares process isolation with Rust-hosted and single-process async/periodic alternatives
