status: PARKED
source_plan: docs/plans/soma-rust-policy-and-runtime-hardening-plan.md
owner: root
latest_intent: execute approved plan with intuitive-flow
current_slice: phases A-D and F complete; native ONNX provisioning/manual live proof remains pending
last_proven: Rust fmt/tests/clippy, focused Python ABI tests, simulation scenario, and two frozen Rust-default policy runs pass
next_action: resume only when native ONNX provisioning/manual live validation is available; then run explicit Python reference parity
next_proof: cd python && uv run pytest; scripts/run-open-duck-walk reference --case docs/status/active/open-duck-stage0-case.md --repeat 2
stop_condition: backend/provisioning, ABI, topology, or hardware gate requires user decision
no_touch: hardware actuation, Micro Duck repo, generic frameworks, RT inference/async
parked: live hardware proof remains explicitly out of scope
