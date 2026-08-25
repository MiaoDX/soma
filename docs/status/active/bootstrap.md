status: READY
source_plan: docs/plans/official-simulation-case-suite-plan.md
control_plane: /root
latest_intent: execute the preflighted fixed official simulation case suite in one new context via intuitive-flow
current_slice: fixed three-case official simulation suite implemented and verified
blocker: none; Docker image supplies the official native prerequisites and Git LFS assets
blocker_fingerprint: none
last_proven: two clean suite runs passed with fixed suite hash/order/representative label, six 100-sample streams, movement validity, and clean teardown; N0 remains blocked because no CH343 VID 1a86 PID 55d3 is enumerated
completed: fixed Reachy profile and ControlCore; pinned real MuJoCo Plant; typed Protobuf; loopback Zenoh runtime; bounded exclusive Unix datagram mailbox; Python simulation acceptance scenario; standalone read-only N0 probe; fixed three-case official simulation suite and committed report
next_slice: none in this plan; await Reachy Mini Lite for N0
next_proof: hardware N0 read-only probe, then explicit human N1 authorization gate
stop_condition: stop if a case requires a vendor patch, direct official MjData access, public Soma contract change, unsafe target, new dependency, generic abstraction, or revised selection policy
no_touch: generic robot manifests, BHL, hardware writes, N1 motion, production official-daemon dependency, camera/audio, App framework, Isaac/Genie backend
parked: hardware N0 until the Lite arrives; N1 physical actuation authorization; hardware legs of the official four-way comparison; App and additional simulator shaping
