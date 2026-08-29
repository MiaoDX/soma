import math
from pathlib import Path

import pytest

from soma_client.showcase import (
    DURATION_SECONDS,
    SAMPLE_HZ,
    keyframes,
    minimum_jerk,
    samples,
    validate_target,
)

DATA = Path(__file__).parents[2] / "crates/soma-sim/assets/reachy-mini/kinematics_data.json"


def test_minimum_jerk_endpoints_and_monotonicity():
    values = [minimum_jerk(step / 100) for step in range(101)]
    assert values[0] == 0.0
    assert values[-1] == 1.0
    assert values == sorted(values)
    assert minimum_jerk(0.5) == pytest.approx(0.5)


def test_pose_order_and_limits():
    poses = keyframes(DATA)
    assert [pose.name for pose in poses] == [
        "neutral", "body-yaw-left", "body-yaw-right", "body-yaw-neutral",
        "antennae-synchronized", "antennae-alternating-left",
        "antennae-alternating-right", "coordinated-yaw-antennae", "head-nod",
        "head-tilt-left", "head-tilt-right", "return-neutral", "neutral-hold",
    ]
    for pose in poses:
        validate_target(pose.positions)
    assert poses[0].positions == pytest.approx(poses[-2].positions)
    assert poses[-2].positions == pytest.approx(poses[-1].positions)
    assert poses[1].positions[0] < 0 < poses[2].positions[0]
    assert poses[4].positions[7] == pytest.approx(poses[4].positions[8])
    assert poses[5].positions[7] == pytest.approx(-poses[5].positions[8])
    assert any(abs(a - b) > 0.02 for a, b in zip(poses[8].positions[1:7], poses[0].positions[1:7]))


def test_samples_have_deterministic_duration_rate_and_complete_targets():
    motion = samples(DATA)
    assert SAMPLE_HZ == 25
    assert DURATION_SECONDS == 12.0
    assert len(motion) == int(DURATION_SECONDS * SAMPLE_HZ) + 1
    assert motion[-1].time_seconds == pytest.approx(DURATION_SECONDS)
    assert all(math.isclose(b.time_seconds - a.time_seconds, 1 / SAMPLE_HZ) for a, b in zip(motion, motion[1:]))
    assert all(len(sample.positions) == 9 for sample in motion)
    assert all(all(math.isfinite(value) for value in sample.positions) for sample in motion)
    for sample in motion:
        validate_target(sample.positions)
