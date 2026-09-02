status: ACTIVE
source_plan: docs/plans/soma-rust-policy-and-runtime-hardening-plan.md
owner: root
latest_intent: execute approved plan with intuitive-flow
current_slice: phases A-D implemented; documentation closure complete; live ONNX provisioning remains pending
last_proven: Rust workspace tests/clippy/fmt and focused Python ABI tests pass
next_action: commit coherent hardening slice; run simulation/product gates where environment permits
next_proof: scripts/run-sim-scenario; scripts/run-open-duck-walk policy --case <frozen-case> --repeat 2
stop_condition: backend/provisioning, ABI, topology, or hardware gate requires user decision
no_touch: hardware actuation, Micro Duck repo, generic frameworks, RT inference/async
parked: live hardware proof remains explicitly out of scope
