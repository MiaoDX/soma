from __future__ import annotations

import argparse
import json
import math
import queue
import struct
import time
from pathlib import Path

import numpy as np
import onnxruntime as ort
import zenoh

STATE_KEY = "soma/open-duck-v2/state"
TARGET_KEY = "soma/open-duck-v2/target"
STATE = struct.Struct("<23QII39f")
TARGET = struct.Struct("<4Q14f")
REJECTION_NAMES = ("decode", "timeline", "sequence", "expired", "invalid", "runtime_generation")
GAIT_PHASE_PERIOD = 27
OBSERVATION_SHAPE = (1, 101)
ACTION_SHAPE = (1, 14)
ACTION_SCALE = np.float32(0.25)
SLEW_RATE_RAD_S = np.float32(5.24)
POLICY_PERIOD_S = np.float32(0.02)
TARGET_TTL_NS = 40_000_000
TARGET_ABS_LIMIT_RAD = np.float32(10.0)
CHECKPOINT_SHA256 = "cb61453a8bcb547ccfdeb4f03ba0fa67ebcf767dcf4aa6e5c9a0d92b302f9b23"
DEFAULT_POSE = np.array([
    0.002, 0.053, -0.63, 1.368, -0.784, 0.0, 0.0, 0.0, 0.0,
    -0.003, -0.065, 0.635, 1.379, -0.796,
], dtype=np.float32)


def decode_state(payload: bytes) -> dict[str, object]:
    values = STATE.unpack(payload)
    facts = values[25:]
    reason = ("none", *REJECTION_NAMES)[values[23]] if values[23] <= 6 else "unknown"
    return {
        "sequence": values[0], "timeline": values[1], "capture_ns": values[2],
        "requested": values[3], "admitted": values[4], "applied": values[5],
        "message_age_ns": values[6], "runtime_dropped_targets": values[7],
        "rejection_counts": dict(zip(REJECTION_NAMES, values[8:14])),
        "max_rejection_age_ns": dict(zip(REJECTION_NAMES, values[14:20])),
        "last_rejection": {
            "reason": reason, "sequence": values[20], "age_ns": values[21],
            "ttl_ns": values[22],
        },
        "flags": values[24],
        "positions": np.asarray(facts[:14], dtype=np.float32),
        "velocities": np.asarray(facts[14:28], dtype=np.float32),
        "gyro": np.asarray(facts[28:31], dtype=np.float32),
        "acceleration": np.asarray(facts[31:34], dtype=np.float32),
        "contacts": np.asarray(facts[34:36], dtype=np.float32),
        "root_height": facts[36], "root_roll": facts[37], "root_pitch": facts[38],
    }


class LatestStateBuffer:
    def __init__(self) -> None:
        self.states: queue.Queue[bytes] = queue.Queue(maxsize=1)
        self.dropped = 0

    def receive(self, sample: object) -> None:
        try:
            self.states.get_nowait()
            self.dropped += 1
        except queue.Empty:
            pass
        try:
            self.states.put_nowait(bytes(sample.payload))
        except queue.Full:
            self.dropped += 1

    def get(self, timeout: float) -> bytes:
        return self.states.get(timeout=timeout)


class Policy:
    def __init__(self, checkpoint: Path, velocity_x: float = 0.3) -> None:
        options = ort.SessionOptions()
        options.intra_op_num_threads = 1
        options.inter_op_num_threads = 1
        options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
        self.session = ort.InferenceSession(str(checkpoint), sess_options=options,
                                            providers=["CPUExecutionProvider"])
        if tuple(self.session.get_inputs()[0].shape) != OBSERVATION_SHAPE:
            raise ValueError(f"Open Duck observation ABI mismatch: {self.session.get_inputs()[0].shape}")
        if tuple(self.session.get_outputs()[0].shape) != ACTION_SHAPE:
            raise ValueError(f"Open Duck action ABI mismatch: {self.session.get_outputs()[0].shape}")
        self.velocity_x = velocity_x
        self.default = DEFAULT_POSE.copy()
        self.previous = DEFAULT_POSE.copy()
        self.history = [np.zeros(14, np.float32) for _ in range(3)]
        self.phase_tick = 0
        self.phase = np.zeros(2, np.float32)
        self.last_observation: np.ndarray | None = None
        self.last_action: np.ndarray | None = None

    def infer(self, state: dict[str, object]) -> np.ndarray:
        required = ("positions", "velocities", "gyro", "acceleration", "contacts")
        if any(key not in state for key in required):
            raise ValueError("Open Duck state is missing required facts")
        positions = np.asarray(state["positions"], np.float32)
        velocities = np.asarray(state["velocities"], np.float32)
        gyro = np.asarray(state["gyro"], np.float32)
        contacts = np.asarray(state["contacts"], np.float32)
        if (positions.shape, velocities.shape, gyro.shape, contacts.shape) != ((14,), (14,), (3,), (2,)):
            raise ValueError("Open Duck state fact shape mismatch")
        if not all(np.isfinite(values).all() for values in (positions, velocities, gyro, contacts)):
            raise ValueError("Open Duck state contains non-finite values")
        acceleration = np.asarray(state["acceleration"], np.float32).copy()
        if acceleration.shape != (3,) or not np.isfinite(acceleration).all():
            raise ValueError("Open Duck acceleration shape or finite-value mismatch")
        acceleration[0] += 1.3
        command = np.array([self.velocity_x, 0, 0, 0, 0, 0, 0], np.float32)
        observation = np.concatenate((gyro, acceleration, command,
            positions - self.default, velocities * 0.05,
            *self.history, self.previous, np.asarray(state["contacts"], np.float32),
            self.phase)).astype(np.float32)
        if observation.shape != (101,) or not np.isfinite(observation).all():
            raise RuntimeError("invalid Open Duck policy observation or action")
        raw = np.asarray(self.session.run(None, {"obs": observation[None, :]})[0])
        if raw.shape != ACTION_SHAPE or not np.isfinite(raw).all():
            raise RuntimeError("invalid Open Duck policy action")
        action = raw[0].astype(np.float32)
        self.history = [action, *self.history[:2]]
        self.last_observation = observation.copy()
        self.last_action = action.copy()
        proposed = self.default + action * ACTION_SCALE
        self.previous = np.clip(proposed, self.previous - SLEW_RATE_RAD_S * POLICY_PERIOD_S,
                                self.previous + SLEW_RATE_RAD_S * POLICY_PERIOD_S)
        self.phase_tick = (self.phase_tick + 1) % GAIT_PHASE_PERIOD
        self.phase = np.array([
            math.cos(self.phase_tick / GAIT_PHASE_PERIOD * 2 * math.pi),
            math.sin(self.phase_tick / GAIT_PHASE_PERIOD * 2 * math.pi),
        ], np.float32)
        return self.previous

    @staticmethod
    def target_payload(sequence: int, state: dict[str, object], positions: np.ndarray) -> bytes:
        positions = np.asarray(positions, np.float32)
        if positions.shape != (14,) or not np.isfinite(positions).all():
            raise ValueError("Open Duck target shape or finite-value mismatch")
        if np.any(np.abs(positions) > TARGET_ABS_LIMIT_RAD):
            raise ValueError("Open Duck target is out of range")
        if "timeline" not in state or "capture_ns" not in state:
            raise ValueError("Open Duck target is missing lineage")
        return TARGET.pack(sequence, int(state["timeline"]), int(state["capture_ns"]),
                           TARGET_TTL_NS, *positions)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--stall-after", type=int)
    parser.add_argument("--duration", type=float)
    parser.add_argument("--ready-file", type=Path)
    parser.add_argument("--vx", type=float, default=0.3)
    args = parser.parse_args()
    config = zenoh.Config.from_json5('{mode:"client",connect:{endpoints:["tcp/127.0.0.1:7448"]},scouting:{multicast:{enabled:false}}}')
    policy = Policy(args.checkpoint, velocity_x=args.vx)
    latest = LatestStateBuffer()
    with zenoh.open(config) as session, session.declare_subscriber(STATE_KEY, latest.receive):
        if args.ready_file is not None:
            args.ready_file.touch()
        emitted = 0
        started = time.monotonic()
        evidence = {"states": 0, "applied": False, "expiry": False, "rejected": False,
                    "max_message_age_ns": 0, "last_requested": 0,
                    "last_admitted": 0, "last_applied": 0,
                    "min_root_height_m": float("inf"), "max_abs_roll_rad": 0.0,
                    "max_abs_pitch_rad": 0.0, "first_state_sequence": 0,
                    "last_state_sequence": 0, "max_state_sequence_gap": 0,
                    "max_inference_ns": 0, "dropped_states": 0,
                    "total_inference_ns": 0, "mean_inference_ns": 0,
                    "runtime_dropped_targets": 0,
                    "rejection_counts": {name: 0 for name in REJECTION_NAMES},
                    "max_rejection_age_ns": {name: 0 for name in REJECTION_NAMES},
                    "last_rejection": {"reason": "none", "sequence": 0,
                                       "age_ns": 0, "ttl_ns": 0}}
        while True:
            state = decode_state(latest.get(timeout=5))
            evidence["states"] += 1
            evidence["dropped_states"] = latest.dropped
            evidence["runtime_dropped_targets"] = int(state["runtime_dropped_targets"])
            sequence = int(state["sequence"])
            if evidence["first_state_sequence"] == 0:
                evidence["first_state_sequence"] = sequence
            if evidence["last_state_sequence"]:
                evidence["max_state_sequence_gap"] = max(
                    evidence["max_state_sequence_gap"],
                    sequence - evidence["last_state_sequence"])
            evidence["last_state_sequence"] = sequence
            evidence["applied"] |= bool(int(state["flags"]) & 1)
            evidence["expiry"] |= bool(int(state["flags"]) & 4)
            evidence["rejected"] |= bool(int(state["flags"]) & 8) or any(
                state["rejection_counts"].values())
            evidence["rejection_counts"] = {
                name: max(evidence["rejection_counts"][name], state["rejection_counts"][name])
                for name in REJECTION_NAMES
            }
            evidence["max_rejection_age_ns"] = {
                name: max(evidence["max_rejection_age_ns"][name], state["max_rejection_age_ns"][name])
                for name in REJECTION_NAMES
            }
            if state["last_rejection"]["reason"] != "none":
                evidence["last_rejection"] = state["last_rejection"]
            evidence["max_message_age_ns"] = max(evidence["max_message_age_ns"], int(state["message_age_ns"]))
            evidence["last_requested"] = int(state["requested"])
            evidence["last_admitted"] = int(state["admitted"])
            evidence["last_applied"] = int(state["applied"])
            evidence["min_root_height_m"] = min(evidence["min_root_height_m"], float(state["root_height"]))
            evidence["max_abs_roll_rad"] = max(evidence["max_abs_roll_rad"], abs(float(state["root_roll"])))
            evidence["max_abs_pitch_rad"] = max(evidence["max_abs_pitch_rad"], abs(float(state["root_pitch"])))
            if args.stall_after is not None and emitted >= args.stall_after:
                if evidence["applied"]:
                    if (args.duration is not None and time.monotonic() - started >= args.duration
                            and evidence["expiry"]):
                        print(json.dumps({"status": "stall-complete", "emitted": emitted, **evidence}))
                        return
                    continue
            inference_started = time.monotonic_ns()
            target = policy.infer(state)
            inference_ns = time.monotonic_ns() - inference_started
            evidence["max_inference_ns"] = max(evidence["max_inference_ns"], inference_ns)
            evidence["total_inference_ns"] += inference_ns
            emitted += 1
            evidence["mean_inference_ns"] = evidence["total_inference_ns"] // emitted
            payload = policy.target_payload(int(state["sequence"]), state, target)
            session.put(TARGET_KEY, payload)
            if (args.duration is not None and time.monotonic() - started >= args.duration
                    and evidence["applied"]):
                print(json.dumps({"status": "complete", "emitted": emitted, **evidence}))
                return


if __name__ == "__main__":
    main()
