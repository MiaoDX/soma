status: ACTIVE
source_plan: docs/plans/simulation-cadence-refactor.md
control_plane: /root
latest_intent: refactor simulation cadence through intuitive-flow, retest, document, and pre-check future robot adapters
current_slice: cadence validation, ten-substep control-period advancement, and release launch implemented; live comparison reruns pending
blocker: none; Docker image supplies the official native prerequisites and Git LFS assets
blocker_fingerprint: none
last_proven: focused schedule/runtime tests, Clippy, and the release headless scenario pass; one advance now proves exactly 20 ms of simulation time
completed: fixed Reachy profile and ControlCore; pinned real MuJoCo Plant; typed Protobuf; loopback Zenoh runtime; bounded exclusive Unix datagram mailbox; Python simulation acceptance scenario; standalone read-only N0 probe; fixed three-case official simulation suite and committed report
next_slice: run two clean fixed-suite comparisons, analyze repeatability, and refresh quantitative docs
next_proof: both suites pass with stable structure and materially corrected tracking metrics
stop_condition: stop if cadence correctness requires a public protocol, ControlCore semantic change, generic robot manifest, or new simulator backend
no_touch: generic robot manifests, BHL, hardware writes, N1 motion, production official-daemon dependency, camera/audio, App framework, Isaac/Genie backend
parked: hardware N0 until the Lite arrives; N1 physical actuation authorization; hardware legs of the official four-way comparison; App and additional simulator shaping
