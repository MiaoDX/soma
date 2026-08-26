status: ACTIVE
source_plan: docs/plans/simulation-showcase.md
control_plane: /root
project_status_writer: /root
latest_intent: execute the approved Reachy Mini simulation showcase through intuitive-flow
current_slice: implement deterministic local media and static report generation
blocker: none
blocker_fingerprint: none
last_proven: existing visual scenario passes under Xvfb, but host screenshot capture is not valid artifact evidence
completed: execution contract approved and bounded
next_slice: isolated implementation of poster, WebM, Rerun recording, report, and verifier
next_proof: scripts/build-sim-showcase output/simulation-showcase
stop_condition: stop before public protocol, Plant/RT semantics, robot-profile, hardware, or repository-setting changes
no_touch: hardware writes, N1 motion, generic robot manifests, additional simulator backends, camera/audio product paths
parked: live GitHub Pages deployment until repository permissions and default-branch run are available
