status: ACTIVE
source_plan: docs/plans/open-duck-mini-walk-policy.md
control_plane: /root
latest_intent: train and evaluate a new Open Duck policy using the pinned Playground path because the published checkpoint is implausibly sensitive to one 2 ms application delay
current_slice: upstream 300M baseline trained and exported; candidate fails Soma zero-delay qualification
blocker: both the published checkpoint and a newly trained 300M upstream baseline fall in Soma's exact harness, including zero-delay application; the training environment and qualification model/observation path are not yet equivalent
blocker_fingerprint: open-duck-same-tick-vs-async-contract-conflict-v1
last_proven: the 300M upstream baseline completed at 300,482,560 steps in about 78 minutes, exported to an 883,948-byte 101->14 ONNX model, and passed ONNX checker/runtime; best-eval and final candidates both fail Soma zero-delay with approximately -0.189 m root height and 3.14 rad roll
timing_contract: 500 Hz RT/physics, 50 Hz policy frame; send immediately from captured state; apply latest admitted target on first available 2 ms RT tick; bounded zero-order hold; fixed 20 ms delay is fault injection only
completed: source checkout, compiled model inspection, provenance audit, ONNX metadata audit, official headless rollout, policy timing comparison, minimal vendored bundle, frozen Stage 0 case, user-approved baseline selection, two passing baseline reference runs, const-generic core seam with Reachy aliases, fixed OpenDuckSimPlant, isolated Duck transport contract, synthetic 50/500 Hz bounded-path tests, combined state/evidence transport, thin Python CPU ONNX policy client, exact golden/fault/process evidence tests, same-tick versus asynchronous isolation, a restorable 10k-step Playground GPU smoke checkpoint, and the 8192-environment production-shaped GPU capacity/throughput calibration
next_slice: reconcile Playground `flat_terrain_backlash` training assets/reward/observation semantics with Soma's frozen `flat_terrain` qualification before any latency-aware retraining
next_proof: a model-equivalence experiment must show the trained candidate is evaluated on the exact training model and observation contract; only then run the full zero-delay/2 ms/20 ms/process matrix
stop_condition: stop if provenance/license, exact model mapping, policy semantics, or same-tick reference gait floors cannot be established without unapproved sources or tuning
no_touch: Reachy hardware probe and N0/N1; Open Duck hardware; generic robot manifests; ROS 2; interactive control; camera/audio/antennas/expressions; silent replacement of the frozen checkpoint or vendoring training artifacts into Soma
parked: interactive velocity input and simultaneous profiles remain deferred; MuJoCo/Rerun visualization is in first accepted rollout scope; post-Stage-4 D-04 review compares process isolation with Rust-hosted and single-process async/periodic alternatives
