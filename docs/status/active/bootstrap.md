status: ACTIVE
source_plan: docs/plans/bootstrap-plan.md
control_plane: /root
latest_intent: execute approved bootstrap plan via intuitive-flow
current_slice: direct MuJoCo Plant and Soma process path
blocker: hardware N0/N1 require connected Reachy Mini Lite and human physical-actuation approval
last_proven: cargo test --workspace (6 passed); cargo check --workspace; git diff --check
completed: approved plan intake; fixed Reachy profile; Plant/ControlCore boundary; lifecycle and health state; observable admission, rejection, applied-target and measured-position-hold results
next_slice: pin the Reachy MuJoCo asset slice and implement ReachySimPlant, then add the Protobuf/Zenoh runtime boundary and Python scenario
next_proof: headless MuJoCo reset/movement/TTL integration test, followed by Python-to-Plant scenario test
stop_condition: stop before torque-enabled hardware work or any external/human gate
no_touch: generic robot manifests, BHL, official daemon integration, camera/audio, hardware writes
parked: N0 native probe; N1 physical actuation authorization; official four-way comparison
