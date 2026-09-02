status: PARKED
source_plan: docs/plans/soma-rust-policy-and-runtime-hardening-plan.md
owner: root
latest_intent: execute approved plan with intuitive-flow
current_slice: phases A-D and F complete; native ONNX provisioning/manual live proof remains pending
last_proven: Rust fmt/tests/clippy, focused Python ABI tests, simulation scenario, two frozen Rust-default policy runs, and two Python reference runs pass
next_action: obtain reproducible native Rust ONNX Runtime provisioning and run full Python suite/manual live validation
next_proof: cd python && uv run pytest; Rust worker build/run with pinned ONNX backend
stop_condition: backend/provisioning, ABI, topology, or hardware gate requires user decision
no_touch: hardware actuation, Micro Duck repo, generic frameworks, RT inference/async
parked: live hardware proof remains explicitly out of scope
