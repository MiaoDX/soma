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
STATE = struct.Struct("<7QI37f")
TARGET = struct.Struct("<4Q14f")
DEFAULT_POSE = np.array([
    0.002, 0.053, -0.63, 1.368, -0.784, 0.0, 0.0, 0.0, 0.0,
    -0.003, -0.065, 0.635, 1.379, -0.796,
], dtype=np.float32)


def decode_state(payload: bytes) -> dict[str, object]:
    values = STATE.unpack(payload)
    facts = values[8:]
    return {
        "sequence": values[0], "timeline": values[1], "capture_ns": values[2],
        "requested": values[3], "admitted": values[4], "applied": values[5],
        "message_age_ns": values[6], "flags": values[7],
        "positions": np.asarray(facts[:14], dtype=np.float32),
        "velocities": np.asarray(facts[14:28], dtype=np.float32),
        "gyro": np.asarray(facts[28:31], dtype=np.float32),
        "acceleration": np.asarray(facts[31:34], dtype=np.float32),
        "root_height": facts[34], "root_roll": facts[35], "root_pitch": facts[36],
    }


class Policy:
    def __init__(self, checkpoint: Path, velocity_x: float = 0.3) -> None:
        options = ort.SessionOptions()
        options.intra_op_num_threads = 1
        options.inter_op_num_threads = 1
        options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
        self.session = ort.InferenceSession(str(checkpoint), sess_options=options,
                                            providers=["CPUExecutionProvider"])
        assert self.session.get_inputs()[0].shape == [1, 101]
        assert self.session.get_outputs()[0].shape == [1, 14]
        self.velocity_x = velocity_x
        self.default = DEFAULT_POSE.copy()
        self.previous = DEFAULT_POSE.copy()
        self.history = [np.zeros(14, np.float32) for _ in range(3)]
        self.phase_tick = 0
        self.phase = np.zeros(2, np.float32)
        self.last_observation: np.ndarray | None = None
        self.last_action: np.ndarray | None = None

    def infer(self, state: dict[str, object]) -> np.ndarray:
        positions = state["positions"]
        assert isinstance(positions, np.ndarray)
        acceleration = np.asarray(state["acceleration"], np.float32).copy()
        acceleration[0] += 1.3
        command = np.array([self.velocity_x, 0, 0, 0, 0, 0, 0], np.float32)
        observation = np.concatenate((state["gyro"], acceleration, command,
            positions - self.default, np.asarray(state["velocities"]) * 0.05,
            *self.history, self.previous, np.zeros(2, np.float32), self.phase)).astype(np.float32)
        action = self.session.run(None, {"obs": observation[None, :]})[0][0].astype(np.float32)
        if observation.size != 101 or not np.isfinite(action).all():
            raise RuntimeError("invalid Open Duck policy observation or action")
        self.history = [action, *self.history[:2]]
        self.last_observation = observation.copy()
        self.last_action = action.copy()
        proposed = self.default + action * 0.25
        self.previous = np.clip(proposed, self.previous - 5.24 * 0.02, self.previous + 5.24 * 0.02)
        self.phase_tick = (self.phase_tick + 1) % 100
        self.phase = np.array([math.cos(self.phase_tick / 100 * 2 * math.pi),
                               math.sin(self.phase_tick / 100 * 2 * math.pi)], np.float32)
        return self.previous

    @staticmethod
    def target_payload(sequence: int, state: dict[str, object], positions: np.ndarray) -> bytes:
        return TARGET.pack(sequence, int(state["timeline"]), int(state["capture_ns"]),
                           40_000_000, *positions)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--stall-after", type=int)
    parser.add_argument("--duration", type=float)
    args = parser.parse_args()
    config = zenoh.Config.from_json5('{mode:"client",connect:{endpoints:["tcp/127.0.0.1:7448"]},scouting:{multicast:{enabled:false}}}')
    policy = Policy(args.checkpoint)
    states: queue.Queue = queue.Queue(maxsize=1)
    dropped_states = 0
    def latest(sample: object) -> None:
        nonlocal dropped_states
        try: states.get_nowait()
        except queue.Empty: pass
        try: states.put_nowait(bytes(sample.payload))
        except queue.Full: dropped_states += 1
    with zenoh.open(config) as session, session.declare_subscriber(STATE_KEY, latest):
        emitted = 0
        started = time.monotonic()
        evidence = {"states": 0, "applied": False, "expiry": False, "rejected": False,
                    "max_message_age_ns": 0, "last_requested": 0,
                    "last_admitted": 0, "last_applied": 0,
                    "min_root_height_m": float("inf"), "max_abs_roll_rad": 0.0,
                    "max_abs_pitch_rad": 0.0, "first_state_sequence": 0,
                    "last_state_sequence": 0, "max_state_sequence_gap": 0,
                    "max_inference_ns": 0, "dropped_states": 0}
        while True:
            state = decode_state(states.get(timeout=5))
            evidence["states"] += 1
            evidence["dropped_states"] = dropped_states
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
            evidence["rejected"] |= bool(int(state["flags"]) & 8)
            evidence["max_message_age_ns"] = max(evidence["max_message_age_ns"], int(state["message_age_ns"]))
            evidence["last_requested"] = int(state["requested"])
            evidence["last_admitted"] = int(state["admitted"])
            evidence["last_applied"] = int(state["applied"])
            evidence["min_root_height_m"] = min(evidence["min_root_height_m"], float(state["root_height"]))
            evidence["max_abs_roll_rad"] = max(evidence["max_abs_roll_rad"], abs(float(state["root_roll"])))
            evidence["max_abs_pitch_rad"] = max(evidence["max_abs_pitch_rad"], abs(float(state["root_pitch"])))
            if args.stall_after is not None and emitted >= args.stall_after:
                if args.duration is not None and time.monotonic() - started >= args.duration:
                    print(json.dumps({"status": "stall-complete", "emitted": emitted, **evidence}))
                    return
                continue
            inference_started = time.monotonic_ns()
            target = policy.infer(state)
            evidence["max_inference_ns"] = max(
                evidence["max_inference_ns"], time.monotonic_ns() - inference_started)
            emitted += 1
            payload = policy.target_payload(int(state["sequence"]), state, target)
            session.put(TARGET_KEY, payload)
            if args.duration is not None and time.monotonic() - started >= args.duration:
                print(json.dumps({"status": "complete", "emitted": emitted, **evidence}))
                return


if __name__ == "__main__":
    main()
