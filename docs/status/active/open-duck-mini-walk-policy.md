status: READY_FOR_PREFLIGHT
source_plan: docs/plans/open-duck-mini-walk-policy.md
control_plane: /root
latest_intent: execute the approved Open Duck Mini walk-policy plan with intuitive-flow
current_slice: Stage 0 ready to execute after contract and provenance decisions
blocker: none after user-approved Apache-2.0 provenance assumption; implementation still must pass Stage 0 gates
blocker_fingerprint: none
last_proven: pinned Playground reference rollout, exact 101-field/14-action contract, and same-tick control ordering verified locally
timing_contract: 500 Hz RT/physics, 50 Hz policy frame; send immediately from captured state; apply latest admitted target on first available 2 ms RT tick; bounded zero-order hold; fixed 20 ms delay is fault injection only
completed: source checkout, compiled model inspection, provenance audit, ONNX metadata audit, official headless rollout, and policy timing comparison; no shared or Reachy code touched
next_slice: execute Stage 0 through intuitive-flow preflight
next_proof: vendored bundle manifest, same-tick golden fixture, injected-delay timing/lineage diagnostics
stop_condition: stop if provenance/license, exact model mapping, policy semantics, or same-tick reference gait floors cannot be established without unapproved sources or tuning
no_touch: Reachy hardware probe and N0/N1; Open Duck hardware; generic robot manifests; policy training; ROS 2; interactive control; camera/audio/antennas/expressions
parked: interactive velocity input and simultaneous profiles remain deferred; MuJoCo/Rerun visualization is in first accepted rollout scope; post-Stage-4 D-04 review compares process isolation with Rust-hosted and single-process async/periodic alternatives
