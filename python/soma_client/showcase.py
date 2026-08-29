from __future__ import annotations

import json
import math
from dataclasses import dataclass
from pathlib import Path

import numpy as np
from reachy_mini_rust_kinematics import ReachyMiniRustKinematics

ACTUATOR_COUNT = 9
SAMPLE_HZ = 25
DURATION_SECONDS = 12.0
TTL_NS = 200_000_000
BODY_YAW_LIMITS = (-2.792526803190975, 2.792526803190879)
STEWART_LIMITS = (
    (-0.8377580409572196, 1.3962634015955222),
    (-1.396263401595614, 1.2217304763958803),
    (-0.8377580409572173, 1.3962634015955244),
    (-1.3962634015953894, 0.8377580409573525),
    (-1.2217304763962082, 1.396263401595286),
    (-1.3962634015954123, 0.8377580409573296),
)
ANTENNA_LIMITS = (-math.pi, math.pi)


@dataclass(frozen=True)
class Keyframe:
    name: str
    time_seconds: float
    positions: tuple[float, ...]


@dataclass(frozen=True)
class Sample:
    time_seconds: float
    positions: tuple[float, ...]


def minimum_jerk(progress: float) -> float:
    if not 0.0 <= progress <= 1.0:
        raise ValueError("progress must be in [0, 1]")
    return progress**3 * (10.0 + progress * (-15.0 + 6.0 * progress))


def _rotation_pose(roll: float = 0.0, pitch: float = 0.0) -> np.ndarray:
    cx, sx = math.cos(roll), math.sin(roll)
    cy, sy = math.cos(pitch), math.sin(pitch)
    rotation_x = np.array(((1, 0, 0), (0, cx, -sx), (0, sx, cx)))
    rotation_y = np.array(((cy, 0, sy), (0, 1, 0), (-sy, 0, cy)))
    pose = np.eye(4)
    pose[:3, :3] = rotation_x @ rotation_y
    return pose


def _kinematics(data_path: Path) -> tuple[ReachyMiniRustKinematics, float]:
    data = json.loads(data_path.read_text())
    engine = ReachyMiniRustKinematics(data["motor_arm_length"], data["rod_length"])
    for motor in data["motors"]:
        engine.add_branch(
            motor["branch_position"],
            np.linalg.inv(motor["T_motor_world"]).tolist(),
            1 if motor["solution"] else -1,
        )
    sleep_pose = np.array(
        (
            (0.911, 0.004, 0.413, -0.021),
            (-0.004, 1.0, -0.001, 0.001),
            (-0.413, -0.001, 0.911, -0.044),
            (0.0, 0.0, 0.0, 1.0),
        )
    )
    sleep_pose[2, 3] += data["head_z_offset"]
    engine.reset_forward_kinematics(sleep_pose.tolist())
    return engine, data["head_z_offset"]


def _head_target(
    engine: ReachyMiniRustKinematics,
    head_z_offset: float,
    *,
    yaw: float = 0.0,
    roll: float = 0.0,
    pitch: float = 0.0,
    right_antenna: float = 0.0,
    left_antenna: float = 0.0,
) -> tuple[float, ...]:
    pose = _rotation_pose(roll, pitch)
    pose[2, 3] += head_z_offset
    head = engine.inverse_kinematics_safe(
        pose.tolist(),
        body_yaw=yaw,
        max_relative_yaw=math.radians(65),
        max_body_yaw=math.radians(160),
    )
    return tuple(head + [right_antenna, left_antenna])


def keyframes(data_path: Path) -> tuple[Keyframe, ...]:
    engine, offset = _kinematics(data_path)
    pose = lambda **kwargs: _head_target(engine, offset, **kwargs)
    return (
        Keyframe("neutral", 0.0, pose()),
        Keyframe("body-yaw-left", 1.0, pose(yaw=-0.45)),
        Keyframe("body-yaw-right", 2.2, pose(yaw=0.45)),
        Keyframe("body-yaw-neutral", 3.2, pose()),
        Keyframe("antennae-synchronized", 4.2, pose(right_antenna=0.65, left_antenna=0.65)),
        Keyframe("antennae-alternating-left", 5.2, pose(right_antenna=-0.65, left_antenna=0.65)),
        Keyframe("antennae-alternating-right", 6.2, pose(right_antenna=0.65, left_antenna=-0.65)),
        Keyframe("coordinated-yaw-antennae", 7.52, pose(yaw=-0.35, right_antenna=0.55, left_antenna=0.25)),
        Keyframe("head-nod", 8.8, pose(pitch=math.radians(8))),
        Keyframe("head-tilt-left", 10.0, pose(roll=math.radians(8))),
        Keyframe("head-tilt-right", 10.8, pose(roll=math.radians(-8))),
        Keyframe("return-neutral", 11.2, pose()),
        Keyframe("neutral-hold", DURATION_SECONDS, pose()),
    )


def samples(data_path: Path) -> tuple[Sample, ...]:
    poses = keyframes(data_path)
    result = [Sample(0.0, poses[0].positions)]
    for start, end in zip(poses, poses[1:]):
        steps = round((end.time_seconds - start.time_seconds) * SAMPLE_HZ)
        for step in range(1, steps + 1):
            progress = step / steps
            weight = minimum_jerk(progress)
            positions = tuple(
                source + (target - source) * weight
                for source, target in zip(start.positions, end.positions)
            )
            result.append(Sample(start.time_seconds + step / SAMPLE_HZ, positions))
    return tuple(result)


def validate_target(positions: tuple[float, ...]) -> None:
    if len(positions) != ACTUATOR_COUNT or not all(math.isfinite(value) for value in positions):
        raise ValueError("showcase target must contain nine finite positions")
    limits = (BODY_YAW_LIMITS, *STEWART_LIMITS, ANTENNA_LIMITS, ANTENNA_LIMITS)
    if any(not low <= value <= high for value, (low, high) in zip(positions, limits)):
        raise ValueError("showcase target exceeds the pinned model limits")
