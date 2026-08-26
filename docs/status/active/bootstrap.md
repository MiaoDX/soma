status: BLOCKED
source_plan: docs/plans/bootstrap-plan.md
control_plane: /root
latest_intent: execute approved bootstrap plan via intuitive-flow
current_slice: hardware N0 read-only native proof
blocker: N0 probe is implemented but no CH343 VID 1a86 PID 55d3 is connected; N1 later requires human physical-actuation approval
blocker_fingerprint: external_hardware:no-reachy-mini-lite-ch343
last_proven: simulation acceptance passed; N0 rerun failed because no CH343 VID 1a86 PID 55d3 is enumerated and /dev/serial/by-id has no matching device
completed: fixed Reachy profile and ControlCore; pinned real MuJoCo Plant; typed Protobuf; loopback Zenoh runtime; bounded exclusive Unix datagram mailbox; Python simulation acceptance scenario; standalone read-only N0 probe
next_slice: connect a Reachy Mini Lite and run cargo run --bin soma-reachy-probe -- to execute the N0 identity/configuration/exclusive-ownership audit
next_proof: N0 raw packet, identity, configuration, latency, retry, timeout and exclusive-open evidence with torque disabled
stop_condition: stop before torque-enabled hardware work or any external/human gate
no_touch: generic robot manifests, BHL, official daemon integration, camera/audio, hardware writes
parked: N0 native probe; N1 physical actuation authorization; official four-way comparison
