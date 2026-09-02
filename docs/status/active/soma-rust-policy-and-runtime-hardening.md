status: BLOCKED
source_plan: docs/plans/soma-rust-policy-and-runtime-hardening-plan.md
owner: root
latest_intent: execute approved plan with intuitive-flow
current_slice: phases A-D and F spike complete; Rust ONNX backend provisioning is the blocker
blocker_fingerprint: backend-provisioning
last_proven: Rust fmt/tests/clippy, full Python suite (21 passed with plugin autoload disabled), simulation scenario, and both policy/reference product runs pass
next_action: user decision or supported toolchain/runtime provisioning for the approved `ort` backend
next_proof: Rust policy worker build/run with pinned ONNX backend
stop_condition: backend/provisioning, ABI, topology, or hardware gate requires user decision
no_touch: hardware actuation, Micro Duck repo, generic frameworks, RT inference/async
parked: live hardware proof remains explicitly out of scope
