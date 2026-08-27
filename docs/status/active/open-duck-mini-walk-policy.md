status: ACTIVE
source_plan: docs/plans/open-duck-mini-walk-policy.md
control_plane: /root
latest_intent: execute the approved Open Duck Mini walk-policy plan with intuitive-flow
current_slice: Stage 2 fixed Plant, isolated transport, and synthetic RT contract implemented; process entry points next
blocker: none after user-approved baseline selection
blocker_fingerprint: none
last_proven: two deterministic baseline reference runs with BEST_WALK_ONNX at vx=0.30 m/s, each 0.18623 m displacement; const-generic core preserves workspace tests; OpenDuckSimPlant validates real model dimensions/order, 2 ms steps, 10-step policy decimation, reset timeline; isolated Duck transport and bounded latest-value synthetic tests pass
timing_contract: 500 Hz RT/physics, 50 Hz policy frame; send immediately from captured state; apply latest admitted target on first available 2 ms RT tick; bounded zero-order hold; fixed 20 ms delay is fault injection only
completed: source checkout, compiled model inspection, provenance audit, ONNX metadata audit, official headless rollout, policy timing comparison, minimal vendored bundle, frozen Stage 0 case, user-approved baseline selection, two passing baseline reference runs, const-generic core seam with Reachy aliases, fixed OpenDuckSimPlant, isolated Duck transport contract, and synthetic 50/500 Hz bounded-path tests
next_slice: Stage 2 fixed Open Duck runtime and robot-rt entry points
next_proof: process startup/clean shutdown, target/state transport, and complete synthetic path through the fixed Plant
stop_condition: stop if provenance/license, exact model mapping, policy semantics, or same-tick reference gait floors cannot be established without unapproved sources or tuning
no_touch: Reachy hardware probe and N0/N1; Open Duck hardware; generic robot manifests; policy training; ROS 2; interactive control; camera/audio/antennas/expressions
parked: interactive velocity input and simultaneous profiles remain deferred; MuJoCo/Rerun visualization is in first accepted rollout scope; post-Stage-4 D-04 review compares process isolation with Rust-hosted and single-process async/periodic alternatives
