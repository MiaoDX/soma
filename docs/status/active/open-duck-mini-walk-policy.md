status: ACTIVE
source_plan: docs/plans/open-duck-mini-walk-policy.md
control_plane: /root
latest_intent: fix the proven Soma observation-adapter bugs and re-evaluate both the trained 300M policy and the published Open Duck checkpoints
current_slice: repaired adapter and successful direct/process qualification; retain the frozen published checkpoint
blocker: none; the prior zero-delay and 2 ms failures were invalid because the adapter supplied a 100-tick phase and zero contacts instead of the official 27-tick phase and measured foot contacts
blocker_fingerprint: none
last_proven: the repaired Soma direct harness runs the trained best-eval checkpoint and both published checkpoints for eight seconds with at least 1.37 m forward displacement, root height at least 0.150 m, and roll/pitch below 0.12 rad. Frozen BEST_WALK_ONNX remains stable at 0, 2, and 20 ms injected delay with 1.410, 1.420, and 1.339 m displacement; its full process path also completes cleanly with no drop, reject, or expiry
timing_contract: 500 Hz RT/physics, 50 Hz policy frame; send immediately from captured state; apply latest admitted target on first available 2 ms RT tick; bounded zero-order hold; fixed 20 ms delay is fault injection only
completed: source checkout, compiled model inspection, provenance audit, ONNX metadata audit, official headless rollout, policy timing comparison, minimal vendored bundle, frozen Stage 0 case, user-approved baseline selection, two passing baseline reference runs, const-generic core seam with Reachy aliases, fixed OpenDuckSimPlant, isolated Duck transport contract, synthetic 50/500 Hz bounded-path tests, combined state/evidence transport, thin Python CPU ONNX policy client, exact golden/fault/process evidence tests, same-tick versus asynchronous isolation, a restorable 10k-step Playground GPU smoke checkpoint, the 8192-environment production-shaped GPU capacity/throughput calibration, and the measured-contact/27-tick observation-adapter repair
next_slice: rerun the complete frozen acceptance command after landing, then continue the approved visualization/acceptance path using the existing published checkpoint
next_proof: one post-commit frozen acceptance run must preserve the repaired direct and process gait envelopes; no latency-aware retraining or checkpoint replacement is justified
stop_condition: stop if provenance/license, exact model mapping, policy semantics, or same-tick reference gait floors cannot be established without unapproved sources or tuning
no_touch: Reachy hardware probe and N0/N1; Open Duck hardware; generic robot manifests; ROS 2; interactive control; camera/audio/antennas/expressions; silent replacement of the frozen checkpoint or vendoring training artifacts into Soma
parked: interactive velocity input and simultaneous profiles remain deferred; MuJoCo/Rerun visualization is in first accepted rollout scope; post-Stage-4 D-04 review compares process isolation with Rust-hosted and single-process async/periodic alternatives
