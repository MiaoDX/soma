status: BLOCKED
source_plan: docs/plans/official-simulation-comparison-plan.md
control_plane: /root
latest_intent: execute the preflighted official simulation comparison in a new context via intuitive-flow
current_slice: D0 official simulator environment and nine-actuator comparability gate
blocker: pinned Reachy Mini v1.9.0 installation cannot build pygobject because this host lacks gobject-introspection-1.0 development metadata
blocker_fingerprint: official-v1.9.0:missing-gobject-introspection-1.0
last_proven: simulation acceptance passed; N0 rerun failed because no CH343 VID 1a86 PID 55d3 is enumerated and /dev/serial/by-id has no matching device
completed: fixed Reachy profile and ControlCore; pinned real MuJoCo Plant; typed Protobuf; loopback Zenoh runtime; bounded exclusive Unix datagram mailbox; Python simulation acceptance scenario; standalone read-only N0 probe
next_slice: resume D0 on an isolated host/container with the pinned release's native prerequisites; do not start D1
next_proof: live official daemon launch and public SDK nine-actuator round trip covering order/unit/sign/initial-state/cadence/readiness
stop_condition: stop at D0 if comparison requires a vendor patch, direct official MjData access, public Soma contract change, or cannot be reproduced within the half-day gate
no_touch: generic robot manifests, BHL, hardware writes, N1 motion, production official-daemon dependency, camera/audio, App framework, Isaac/Genie backend
parked: hardware N0 until the Lite arrives; N1 physical actuation authorization; hardware legs of the official four-way comparison; App and additional simulator shaping
