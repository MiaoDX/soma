import json
import os
import struct
from pathlib import Path

import mujoco
import numpy as np

from soma_client.open_duck_policy import Policy, STATE, decode_state


def test_combined_state_decode_preserves_lineage_and_facts():
    payload = STATE.pack(11, 2, 90, 8, 7, 6, 5, 3, *range(34))
    state = decode_state(payload)
    assert (state["sequence"], state["timeline"], state["applied"]) == (11, 2, 6)
    np.testing.assert_array_equal(state["positions"], np.arange(14, dtype=np.float32))
    assert struct.calcsize("<7QI34f") == len(payload)


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
        "acceleration": data.sensordata[6:9].astype(np.float32)}
    policy = Policy(bundle / "BEST_WALK_ONNX.onnx")
    policy.infer(state)
    fixture = json.loads((Path(__file__).parent / "fixtures/open_duck_first_tick.json").read_text())
    np.testing.assert_allclose(policy.last_observation, fixture["observation"], atol=1e-5, rtol=0)
    np.testing.assert_allclose(policy.last_action, fixture["action"], atol=1e-5, rtol=0)
