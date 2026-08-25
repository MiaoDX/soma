from __future__ import annotations

import hashlib
import json
import math
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
SUITE_PATH = Path(__file__).with_name("suite.json")
SUITE_KEYS = {
    "schema_version",
    "actuator_order",
    "units",
    "warmup_s",
    "sample_period_s",
    "dwell_s",
    "representative_case",
    "cases",
}
CASE_KEYS = {"id", "deltas_rad"}
MOVEMENT_THRESHOLD_RAD = 0.05


def load_suite(path: Path = SUITE_PATH) -> dict:
    raw = path.read_bytes()
    try:
        suite = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError(f"invalid suite JSON: {error}") from error
    if not isinstance(suite, dict) or set(suite) != SUITE_KEYS:
        raise ValueError("suite must contain exactly the documented top-level keys")
    if suite["schema_version"] != 1:
        raise ValueError("unsupported suite schema_version")
    if suite["actuator_order"] != ACTUATORS or suite["units"] != "radians":
        raise ValueError("suite must use the fixed nine-actuator radians profile")
    for key in ("warmup_s", "sample_period_s", "dwell_s"):
        value = suite[key]
        if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or value <= 0:
            raise ValueError(f"{key} must be a positive finite number")
    sample_count = suite["dwell_s"] / suite["sample_period_s"]
    if not sample_count.is_integer() or int(sample_count) != 100:
        raise ValueError("suite timing must define exactly 100 planned samples")
    cases = suite["cases"]
    if not isinstance(cases, list) or not cases:
        raise ValueError("suite cases must be a non-empty ordered array")
    ids = []
    for case in cases:
        if not isinstance(case, dict) or set(case) != CASE_KEYS:
            raise ValueError("each case must contain exactly id and deltas_rad")
        case_id = case["id"]
        if not isinstance(case_id, str) or not case_id:
            raise ValueError("case id must be a non-empty string")
        deltas = case["deltas_rad"]
        if not isinstance(deltas, list) or len(deltas) != len(ACTUATORS) or any(
            isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value)
            for value in deltas
        ):
            raise ValueError("case deltas_rad must contain exactly nine finite numbers")
        if not any(deltas):
            raise ValueError("each case must command at least one actuator")
        ids.append(case_id)
    if len(ids) != len(set(ids)):
        raise ValueError("suite case ids must be unique")
    if suite["representative_case"] not in ids:
        raise ValueError("representative_case must name a declared case")
    suite["suite_sha256"] = hashlib.sha256(raw).hexdigest()
    suite["sample_count"] = int(sample_count)
    return suite


def select_case(suite: dict, case_id: str) -> dict:
    for order, case in enumerate(suite["cases"]):
        if case["id"] == case_id:
            return {**case, "order": order, "commanded_indexes": [i for i, value in enumerate(case["deltas_rad"]) if value != 0]}
    raise ValueError(f"unknown suite case: {case_id}")


def monotonic_ns() -> int:
    return time.monotonic_ns()


def write_jsonl(path: Path, records: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as stream:
        for record in records:
            stream.write(json.dumps(record, sort_keys=True, separators=(",", ":")))
            stream.write("\n")


def manifest(implementation: str, model: dict, initial: list[float], suite: dict, case: dict, warmup_actual_s: float, command_sent_ns: int) -> dict:
    return {
        "record_type": "manifest",
        "schema_version": 2,
        "implementation": implementation,
        "model": model,
        "suite_sha256": suite["suite_sha256"],
        "case_id": case["id"],
        "case_order": case["order"],
        "representative_case": suite["representative_case"],
        "actuator_order": suite["actuator_order"],
        "units": suite["units"],
        "command_sign": "positive follows named MJCF joint / public SDK sign",
        "initial_positions_rad": initial,
        "warmup_actual_s": warmup_actual_s,
        "command_sent_monotonic_ns": command_sent_ns,
        "trace": {
            "kind": "relative_step",
            "deltas_rad": case["deltas_rad"],
            "commanded_indexes": case["commanded_indexes"],
            "warmup_s": suite["warmup_s"],
            "sample_period_s": suite["sample_period_s"],
            "dwell_s": suite["dwell_s"],
            "planned_sample_count": suite["sample_count"],
        },
        "capabilities": {
            "target": "COMMON", "measured": "COMMON", "host_timestamps": "COMMON", "startup_stop": "COMMON",
            "sequence": "SOMA_ONLY" if implementation == "soma" else "UNAVAILABLE",
            "timeline": "SOMA_ONLY" if implementation == "soma" else "UNAVAILABLE",
            "ttl": "SOMA_ONLY" if implementation == "soma" else "UNAVAILABLE",
            "disposition": "SOMA_ONLY" if implementation == "soma" else "UNAVAILABLE",
        },
    }


def sample(implementation: str, observed_ns: int, sent_ns: int, target: list[float], measured: list[float], planned_index: int, planned_deadline_ns: int, **provenance: object) -> dict:
    if len(target) != len(ACTUATORS) or len(measured) != len(ACTUATORS):
        raise ValueError("comparison records require exactly nine actuator values")
    return {
        "record_type": "sample", "implementation": implementation, "phase": "motion",
        "planned_sample_index": planned_index, "planned_deadline_monotonic_ns": planned_deadline_ns,
        "lateness_ns": observed_ns - planned_deadline_ns,
        "host_observed_monotonic_ns": observed_ns, "host_command_sent_monotonic_ns": sent_ns,
        "target_positions_rad": target, "measured_positions_rad": measured, "provenance": provenance,
    }


def validate_movement(initial: list[float], samples: list[dict], commanded_indexes: list[int]) -> None:
    for index in commanded_indexes:
        movement = max(abs(row["measured_positions_rad"][index] - initial[index]) for row in samples)
        if movement < MOVEMENT_THRESHOLD_RAD:
            raise RuntimeError(f"commanded actuator {ACTUATORS[index]} moved only {movement:.6f} rad")
