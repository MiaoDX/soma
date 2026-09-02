import json
import os
import struct
import subprocess
from types import SimpleNamespace
from pathlib import Path

import mujoco
import numpy as np

from soma_client.open_duck_policy import (
    GAIT_PHASE_PERIOD,
    LatestStateBuffer,
    Policy,
    STATE,
    TARGET,
    decode_state,
)


class _Tensor:
    def __init__(self, shape):
        self.shape = shape


class _FakeSession:
    def __init__(self, output, input_shape=(1, 101), output_shape=(1, 14)):
        self.output = output
        self.input_shape = input_shape
        self.output_shape = output_shape

    def get_inputs(self):
        return [_Tensor(self.input_shape)]

    def get_outputs(self):
        return [_Tensor(self.output_shape)]

    def run(self, _outputs, _feeds):
        return [self.output]


def _state():
    return {"positions": np.zeros(14, np.float32), "velocities": np.zeros(14, np.float32),
            "gyro": np.zeros(3, np.float32), "acceleration": np.zeros(3, np.float32),
            "contacts": np.zeros(2, np.float32)}


def test_policy_rejects_model_shape_and_nonfinite_action(monkeypatch, tmp_path):
    import soma_client.open_duck_policy as module
    monkeypatch.setattr(module.ort, "InferenceSession", lambda *args, **kwargs:
                        _FakeSession(np.zeros((1, 14), np.float32), output_shape=(1, 13)))
    with np.testing.assert_raises(ValueError):
        Policy(tmp_path / "fake.onnx")
    monkeypatch.setattr(module.ort, "InferenceSession", lambda *args, **kwargs:
                        _FakeSession(np.full((1, 14), np.nan, np.float32)))
    policy = Policy(tmp_path / "fake.onnx")
    with np.testing.assert_raises(RuntimeError):
        policy.infer(_state())


def test_policy_rejects_nonfinite_state(monkeypatch, tmp_path):
    import soma_client.open_duck_policy as module
    monkeypatch.setattr(module.ort, "InferenceSession", lambda *args, **kwargs:
                        _FakeSession(np.zeros((1, 14), np.float32)))
    policy = Policy(tmp_path / "fake.onnx")
    state = _state()
    state["positions"][0] = np.nan
    with np.testing.assert_raises(ValueError):
        policy.infer(state)


def test_target_payload_rejects_nonfinite_and_out_of_range_targets():
    state = {"timeline": 1, "capture_ns": 2}
    with np.testing.assert_raises(ValueError):
        Policy.target_payload(1, state, np.full(14, np.nan, np.float32))
    with np.testing.assert_raises(ValueError):
        Policy.target_payload(1, state, np.full(14, 11.0, np.float32))


def test_combined_state_decode_preserves_lineage_and_facts():
    payload = STATE.pack(11, 2, 90, 8, 7, 6, 5, 4, 3, *range(39))
    state = decode_state(payload)
    assert (state["sequence"], state["timeline"], state["applied"]) == (11, 2, 6)
    np.testing.assert_array_equal(state["positions"], np.arange(14, dtype=np.float32))
    np.testing.assert_array_equal(state["contacts"], np.array([34, 35], np.float32))
    assert state["runtime_dropped_targets"] == 4
    assert struct.calcsize("<8QI39f") == len(payload)


def test_target_keeps_original_capture_deadline_lineage():
    state = {"timeline": 9, "capture_ns": 123456789}
    payload = Policy.target_payload(4, state, np.zeros(14, np.float32))
    sequence, timeline, capture_ns, ttl_ns, *_ = TARGET.unpack(payload)
    assert (sequence, timeline, capture_ns, ttl_ns) == (4, 9, 123456789, 40_000_000)


def test_latest_state_buffer_counts_overwritten_states():
    latest = LatestStateBuffer()
    latest.receive(SimpleNamespace(payload=b"first"))
    latest.receive(SimpleNamespace(payload=b"second"))
    assert latest.get(timeout=0) == b"second"
    assert latest.dropped == 1


def test_first_policy_tick_matches_frozen_reference_fixture():
    root = Path(__file__).resolve().parents[2]
    bundle = root / "crates/soma-sim/assets/open-duck-mini-v2"
    previous_cwd = Path.cwd()
    try:
        os.chdir(bundle / "xmls")
        model = mujoco.MjModel.from_xml_path("scene_flat_terrain.xml")
    finally:
        os.chdir(previous_cwd)
    data = mujoco.MjData(model)
    home = mujoco.mj_name2id(model, mujoco.mjtObj.mjOBJ_KEY, "home")
    mujoco.mj_resetDataKeyframe(model, data, home)
    mujoco.mj_step(model, data)
    names = ["left_hip_yaw", "left_hip_roll", "left_hip_pitch", "left_knee", "left_ankle",
        "neck_pitch", "head_pitch", "head_yaw", "head_roll", "right_hip_yaw",
        "right_hip_roll", "right_hip_pitch", "right_knee", "right_ankle"]
    positions = np.array([data.qpos[model.jnt_qposadr[mujoco.mj_name2id(
        model, mujoco.mjtObj.mjOBJ_JOINT, name)]] for name in names], np.float32)
    velocities = np.array([data.qvel[model.jnt_dofadr[mujoco.mj_name2id(
        model, mujoco.mjtObj.mjOBJ_JOINT, name)]] for name in names], np.float32)
    state = {"positions": positions, "velocities": velocities,
        "gyro": data.sensordata[:3].astype(np.float32),
        "acceleration": data.sensordata[6:9].astype(np.float32),
        "contacts": np.zeros(2, np.float32)}
    policy = Policy(bundle / "BEST_WALK_ONNX.onnx")
    policy.infer(state)
    fixture = json.loads((Path(__file__).parent / "fixtures/open_duck_first_tick.json").read_text())
    np.testing.assert_allclose(policy.last_observation, fixture["observation"], atol=1e-5, rtol=0)
    np.testing.assert_allclose(policy.last_action, fixture["action"], atol=1e-5, rtol=0)


def test_rust_onnx_action_matches_python_golden_fixture():
    root = Path(__file__).resolve().parents[2]
    runtime = subprocess.run(
        [str(root / "target/debug/open-duck-policy"),
         "--checkpoint", str(root / "crates/soma-sim/assets/open-duck-mini-v2/BEST_WALK_ONNX.onnx"),
         "--parity-fixture", str(Path(__file__).parent / "fixtures/open_duck_first_tick.json")],
        env={**os.environ, "ORT_DYLIB_PATH": os.environ["ORT_DYLIB_PATH"]},
        check=True, capture_output=True, text=True,
    )
    fixture = json.loads((Path(__file__).parent / "fixtures/open_duck_first_tick.json").read_text())
    np.testing.assert_allclose(json.loads(runtime.stdout), fixture["action"], atol=1e-5, rtol=0)


def test_policy_uses_contacts_and_wraps_the_official_gait_period():
    root = Path(__file__).resolve().parents[2]
    bundle = root / "crates/soma-sim/assets/open-duck-mini-v2"
    policy = Policy(bundle / "BEST_WALK_ONNX.onnx")
    state = {
        "positions": policy.default.copy(),
        "velocities": np.zeros(14, np.float32),
        "gyro": np.zeros(3, np.float32),
        "acceleration": np.zeros(3, np.float32),
        "contacts": np.array([1.0, 0.0], np.float32),
    }
    for _ in range(GAIT_PHASE_PERIOD):
        policy.infer(state)
    np.testing.assert_array_equal(policy.last_observation[-4:-2], state["contacts"])
    np.testing.assert_allclose(policy.phase, np.array([1.0, 0.0]), atol=1e-6, rtol=0)


def test_policy_places_requested_velocity_in_observation():
    root = Path(__file__).resolve().parents[2]
    checkpoint = root / "crates/soma-sim/assets/open-duck-mini-v2/BEST_WALK_ONNX.onnx"
    policy = Policy(checkpoint, velocity_x=0.17)
    state = {
        "positions": policy.default.copy(),
        "velocities": np.zeros(14, np.float32),
        "gyro": np.zeros(3, np.float32),
        "acceleration": np.zeros(3, np.float32),
        "contacts": np.zeros(2, np.float32),
    }
    policy.infer(state)
    assert policy.last_observation is not None
    assert policy.last_observation[6] == np.float32(0.17)
