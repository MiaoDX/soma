status: READY
source_plan: docs/plans/official-simulation-comparison-plan.md
control_plane: /root
latest_intent: execute the preflighted official simulation comparison in a new context via intuitive-flow
current_slice: D3 complete; repeatable simulation-only official comparison
blocker: none; Docker image supplies the official native prerequisites and Git LFS assets
blocker_fingerprint: none
last_proven: simulation acceptance passed; N0 rerun failed because no CH343 VID 1a86 PID 55d3 is enumerated and /dev/serial/by-id has no matching device
completed: fixed Reachy profile and ControlCore; pinned real MuJoCo Plant; typed Protobuf; loopback Zenoh runtime; bounded exclusive Unix datagram mailbox; Python simulation acceptance scenario; standalone read-only N0 probe
next_slice: hardware N0 remains the next separate bootstrap gate
next_proof: rerun `scripts/run-official-sim-comparison` for fresh ignored evidence; hardware work still requires the Reachy Lite and N1 authorization
stop_condition: stop at D0 if comparison requires a vendor patch, direct official MjData access, public Soma contract change, or cannot be reproduced within the half-day gate
no_touch: generic robot manifests, BHL, hardware writes, N1 motion, production official-daemon dependency, camera/audio, App framework, Isaac/Genie backend
parked: hardware N0 until the Lite arrives; N1 physical actuation authorization; hardware legs of the official four-way comparison; App and additional simulator shaping
