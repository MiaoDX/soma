from __future__ import annotations

import json
import time
from pathlib import Path

ACTUATORS = [
    "yaw_body",
    "stewart_1",
    "stewart_2",
    "stewart_3",
    "stewart_4",
    "stewart_5",
    "stewart_6",
    "right_antenna",
    "left_antenna",
]
TRACE_DELTAS_RAD = [0.15, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.15, -0.15]
SAMPLE_PERIOD_S = 0.02
WARMUP_S = 1.0
DWELL_S = 2.0


def monotonic_ns() -> int:
    return time.monotonic_ns()


def write_jsonl(path: Path, records: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as stream:
        for record in records:
            stream.write(json.dumps(record, sort_keys=True, separators=(",", ":")))
            stream.write("\n")


def manifest(implementation: str, model: dict, initial: list[float]) -> dict:
    return {
        "record_type": "manifest",
        "schema_version": 1,
        "implementation": implementation,
        "model": model,
        "actuator_order": ACTUATORS,
        "units": "radians",
        "command_sign": "positive follows named MJCF joint / public SDK sign",
        "initial_positions_rad": initial,
        "trace": {
            "kind": "relative_step",
            "deltas_rad": TRACE_DELTAS_RAD,
            "warmup_s": WARMUP_S,
            "sample_period_s": SAMPLE_PERIOD_S,
            "dwell_s": DWELL_S,
        },
        "capabilities": {
            "target": "COMMON",
            "measured": "COMMON",
            "host_timestamps": "COMMON",
            "startup_stop": "COMMON",
            "sequence": "SOMA_ONLY" if implementation == "soma" else "UNAVAILABLE",
            "timeline": "SOMA_ONLY" if implementation == "soma" else "UNAVAILABLE",
            "ttl": "SOMA_ONLY" if implementation == "soma" else "UNAVAILABLE",
            "disposition": "SOMA_ONLY" if implementation == "soma" else "UNAVAILABLE",
        },
    }


def sample(
    implementation: str,
    observed_ns: int,
    sent_ns: int,
    target: list[float],
    measured: list[float],
    **provenance: object,
) -> dict:
    if len(target) != len(ACTUATORS) or len(measured) != len(ACTUATORS):
        raise ValueError("comparison records require exactly nine actuator values")
    return {
        "record_type": "sample",
        "implementation": implementation,
        "phase": "motion",
        "host_observed_monotonic_ns": observed_ns,
        "host_command_sent_monotonic_ns": sent_ns,
        "target_positions_rad": target,
        "measured_positions_rad": measured,
        "provenance": provenance,
    }

