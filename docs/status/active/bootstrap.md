status: READY
source_plan: docs/plans/official-simulation-case-suite-plan.md
control_plane: /root
latest_intent: execute the preflighted fixed official simulation case suite in one new context via intuitive-flow
current_slice: planning loop and execution preflight complete; implementation has not started
blocker: none; Docker image supplies the official native prerequisites and Git LFS assets
blocker_fingerprint: none
last_proven: simulation acceptance passed; N0 rerun failed because no CH343 VID 1a86 PID 55d3 is enumerated and /dev/serial/by-id has no matching device
completed: fixed Reachy profile and ControlCore; pinned real MuJoCo Plant; typed Protobuf; loopback Zenoh runtime; bounded exclusive Unix datagram mailbox; Python simulation acceptance scenario; standalone read-only N0 probe
next_slice: implement S0-S3 of the fixed three-case official simulation suite
next_proof: two clean suite runs with stable structural evidence, valid commanded movement, and one committed aggregate report
stop_condition: stop if a case requires a vendor patch, direct official MjData access, public Soma contract change, unsafe target, new dependency, generic abstraction, or revised selection policy
no_touch: generic robot manifests, BHL, hardware writes, N1 motion, production official-daemon dependency, camera/audio, App framework, Isaac/Genie backend
parked: hardware N0 until the Lite arrives; N1 physical actuation authorization; hardware legs of the official four-way comparison; App and additional simulator shaping
