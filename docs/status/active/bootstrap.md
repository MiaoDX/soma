status: READY
source_plan: docs/plans/simulation-cadence-refactor.md
control_plane: /root
latest_intent: refactor simulation cadence through intuitive-flow, retest, document, and pre-check future robot adapters
current_slice: simulation cadence correction, cross-robot MuJoCo precheck, release launch, and corrected comparison evidence complete
blocker: none; Docker image supplies the official native prerequisites and Git LFS assets
blocker_fingerprint: none
last_proven: two corrected clean suites match Soma and official tracking near 80 ms, repeatability passes, and release headless acceptance remains green
completed: fixed Reachy profile and ControlCore; pinned real MuJoCo Plant; typed Protobuf; loopback Zenoh runtime; bounded exclusive Unix datagram mailbox; Python simulation acceptance scenario; standalone read-only N0 probe; fixed three-case official simulation suite and committed report
next_slice: none in this plan; await Reachy Mini Lite for hardware N0
next_proof: hardware N0 read-only probe, then explicit human N1 authorization gate
stop_condition: stop if cadence correctness requires a public protocol, ControlCore semantic change, generic robot manifest, or new simulator backend
no_touch: generic robot manifests, BHL, hardware writes, N1 motion, production official-daemon dependency, camera/audio, App framework, Isaac/Genie backend
parked: hardware N0 until the Lite arrives; N1 physical actuation authorization; hardware legs of the official four-way comparison; App and additional simulator shaping
