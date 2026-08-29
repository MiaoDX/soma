status: ACTIVE
source_gate: user-approved Reachy Mini showcase trial via intuitive-flow
control_plane: /root
latest_intent: render the Reachy Mini MJCF natively in the existing Rerun showcase and review the effect locally
current_slice: promote the proven robot view to a full-height showcase pane and complete time-sync browser review
blocker: none
blocker_fingerprint: none
last_proven: the robot-only MJCF removes floor-dominated framing and browser QA visibly renders Reachy; the initial grid makes the 3D pane too short
completed: research, exact-model compatibility probe, implementation, structural RRD verification, floor correction, and two browser passes
next_slice: use a full-height 3D pane, then verify two time positions produce different robot pixels alongside matching plots
next_proof: scripts/build-sim-showcase followed by RRD inspection and browser review
stop_condition: stop if correct visualization requires a control-path, snapshot-protocol, hardware, or second-model change
no_touch: real-time semantics, hardware, Open Duck, generic robot manifests, URDF conversion
parked: optional synchronized MuJoCo camera inside Rerun
