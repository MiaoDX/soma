status: ACTIVE
source_plan: docs/plans/open-duck-mini-walk-policy.md
control_plane: /root
latest_intent: execute the approved Open Duck Mini walk-policy plan with intuitive-flow
current_slice: Stage 4 full-duration process-path gait gate evaluated
blocker: approved timing and gait contracts conflict: the pinned policy meets frozen floors only with official same-tick application, while even one 2 ms delayed direct-reference application falls
blocker_fingerprint: open-duck-same-tick-vs-async-contract-conflict-v1
last_proven: direct diagnostic reference meets the frozen gait only at zero delay; one 2 ms delayed application falls to -0.189 m root height with 3.142 rad roll, reproducing the asynchronous Soma failure; a single dropped policy tick also reproduces the fall
timing_contract: 500 Hz RT/physics, 50 Hz policy frame; send immediately from captured state; apply latest admitted target on first available 2 ms RT tick; bounded zero-order hold; fixed 20 ms delay is fault injection only
completed: source checkout, compiled model inspection, provenance audit, ONNX metadata audit, official headless rollout, policy timing comparison, minimal vendored bundle, frozen Stage 0 case, user-approved baseline selection, two passing baseline reference runs, const-generic core seam with Reachy aliases, fixed OpenDuckSimPlant, isolated Duck transport contract, synthetic 50/500 Hz bounded-path tests, combined state/evidence transport, and thin Python CPU ONNX policy client
next_slice: finish Stage 3 exact golden observation/action and deadline/fault integration
next_proof: user reshaping decision is required by the plan hard stop: change the pinned policy/model, change the asynchronous timing outcome, or separate same-tick compatibility from asynchronous gait acceptance; do not tune or weaken floors silently
stop_condition: stop if provenance/license, exact model mapping, policy semantics, or same-tick reference gait floors cannot be established without unapproved sources or tuning
no_touch: Reachy hardware probe and N0/N1; Open Duck hardware; generic robot manifests; policy training; ROS 2; interactive control; camera/audio/antennas/expressions
parked: interactive velocity input and simultaneous profiles remain deferred; MuJoCo/Rerun visualization is in first accepted rollout scope; post-Stage-4 D-04 review compares process isolation with Rust-hosted and single-process async/periodic alternatives
