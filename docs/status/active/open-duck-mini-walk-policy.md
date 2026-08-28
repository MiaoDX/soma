status: PARKED
source_plan: docs/plans/open-duck-mini-walk-policy.md
control_plane: /root
latest_intent: fix the proven Soma observation-adapter bugs and re-evaluate both the trained 300M policy and the published Open Duck checkpoints
current_slice: Stage 4 headless acceptance complete; retain the frozen published checkpoint
blocker: none; the prior zero-delay and 2 ms failures were invalid because the adapter supplied a 100-tick phase and zero contacts instead of the official 27-tick phase and measured foot contacts
blocker_fingerprint: none
last_proven: two frozen direct runs each traveled 1.40965 m with 62 contact transitions, 3.725 mm maximum stance slip, zero non-foot collisions, root height at least 0.15015 m, and roll/pitch below 0.12 rad. Two supervised process runs completed upright with no rejection, drop, or expiry; 2 ms and 20 ms delay cases traveled 1.41996 m and 1.33943 m. Repeated stalls observed applied target then expiry to measured hold. Rust/Python gates, Reachy scenario/teleop, and Reachy Xvfb visualization passed
timing_contract: 500 Hz RT/physics, 50 Hz policy frame; send immediately from captured state; apply latest admitted target on first available 2 ms RT tick; bounded zero-order hold; fixed 20 ms delay is fault injection only
completed: source checkout, compiled model inspection, provenance audit, ONNX metadata audit, official headless rollout, policy timing comparison, minimal vendored bundle, frozen Stage 0 case, user-approved baseline selection, two passing baseline reference runs, const-generic core seam with Reachy aliases, fixed OpenDuckSimPlant, isolated Duck transport contract, synthetic 50/500 Hz bounded-path tests, combined state/evidence transport, thin Python CPU ONNX policy client, exact golden/fault/process evidence tests, same-tick versus asynchronous isolation, a restorable 10k-step Playground GPU smoke checkpoint, the 8192-environment production-shaped GPU capacity/throughput calibration, and the measured-contact/27-tick observation-adapter repair
next_slice: optional D-04 process-topology review; Duck-specific visualization remains an observational enhancement rather than an acceptance gate
next_proof: none for the completed headless path; no latency-aware retraining or checkpoint replacement is justified
stop_condition: stop if provenance/license, exact model mapping, policy semantics, or same-tick reference gait floors cannot be established without unapproved sources or tuning
no_touch: Reachy hardware probe and N0/N1; Open Duck hardware; generic robot manifests; ROS 2; interactive control; camera/audio/antennas/expressions; silent replacement of the frozen checkpoint or vendoring training artifacts into Soma
parked: Duck-specific MuJoCo/Rerun visualization, interactive velocity input, and simultaneous profiles remain deferred; post-Stage-4 D-04 review compares process isolation with Rust-hosted and single-process async/periodic alternatives
