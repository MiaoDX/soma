status: ACTIVE
source_plan: docs/plans/bootstrap-plan.md
control_plane: /root
latest_intent: execute approved bootstrap plan via intuitive-flow
current_slice: hardware N0 read-only native proof
blocker: hardware N0/N1 require connected Reachy Mini Lite and human physical-actuation approval
last_proven: scripts/cargo-mujoco test --workspace (12 passed); scripts/cargo-mujoco check --workspace; scripts/run-sim-scenario; git diff --check
completed: fixed Reachy profile and ControlCore; pinned real MuJoCo Plant; typed Protobuf; loopback Zenoh runtime; bounded exclusive Unix datagram mailbox; Python simulation acceptance scenario
next_slice: connect a Reachy Mini Lite and execute the N0 read-only identity/configuration/exclusive-ownership audit
next_proof: N0 raw packet, identity, configuration, latency, retry, timeout and exclusive-open evidence with torque disabled
stop_condition: stop before torque-enabled hardware work or any external/human gate
no_touch: generic robot manifests, BHL, official daemon integration, camera/audio, hardware writes
parked: N0 native probe; N1 physical actuation authorization; official four-way comparison
