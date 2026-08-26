status: BLOCKED
source_plan: docs/plans/reachy-command-session.md
control_plane: /root
latest_intent: execute approved simulation command-session plan via intuitive-flow
current_slice: desktop-backed manual interaction acceptance
blocker: no human desktop review has confirmed physical key input, visible MuJoCo motion, and Rerun evidence semantics
blocker_fingerprint: external_human:desktop-teleop-visual-review
last_proven: unit/workspace gates, headless scenarios, bounded ADQE command path, sequential Xvfb visual startup, and owned-process/socket teardown pass
completed: terminal state machine; launcher and supervision; bounded integration route; focused tests; human docs
next_slice: on a desktop run scripts/run-sim-teleop --visualize and perform the plan's final A/D/Q/E review
next_proof: desktop keyboard, visible motion, Rerun requested/admitted/applied/expiry evidence, post-TTL operation, and Ctrl-C teardown
stop_condition: stop at any protocol/generic-abstraction expansion or the final desktop-backed manual gate
no_touch: hardware bootstrap capsule, hardware writes, official daemon, generic robot/target/session abstractions, viewer write-back
parked: head-pose IK; continuous input; hardware actuation
