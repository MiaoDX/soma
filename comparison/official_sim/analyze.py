from __future__ import annotations

import argparse
import json
import math
import statistics
from pathlib import Path

from .common import ACTUATORS, load_suite, select_case, validate_movement

SETTLING_TOLERANCE_RAD = 0.03
STEADY_FRACTION = 0.2


def load(path: Path) -> tuple[dict, list[dict]]:
    records = [json.loads(line) for line in path.read_text().splitlines() if line]
    if not records or records[0].get("record_type") != "manifest":
        raise ValueError(f"{path} has no leading manifest")
    manifest = records[0]
    if manifest.get("actuator_order") != ACTUATORS or manifest.get("units") != "radians":
        raise ValueError(f"{path} does not use the fixed comparison profile")
    samples = records[1:]
    if not samples or any(record.get("record_type") != "sample" for record in samples):
        raise ValueError(f"{path} has no valid samples")
    return manifest, samples


def load_case(path: Path, suite: dict, case: dict, implementation: str) -> tuple[dict, list[dict]]:
    manifest_record, samples = load(path)
    expected = {
        "implementation": implementation,
        "suite_sha256": suite["suite_sha256"],
        "case_id": case["id"],
        "case_order": case["order"],
        "representative_case": suite["representative_case"],
    }
    for key, value in expected.items():
        if manifest_record.get(key) != value:
            raise ValueError(f"{path} has invalid {key}")
    if len(samples) != suite["sample_count"]:
        raise ValueError(f"{path} must contain exactly {suite['sample_count']} samples")
    if [row.get("planned_sample_index") for row in samples] != list(range(suite["sample_count"])):
        raise ValueError(f"{path} has an invalid planned sample schedule")
    validate_movement(manifest_record["initial_positions_rad"], samples, case["commanded_indexes"])
    return manifest_record, samples


def actuator_metrics(samples: list[dict], index: int) -> dict:
    errors = [
        row["measured_positions_rad"][index] - row["target_positions_rad"][index]
        for row in samples
    ]
    initial = samples[0]["measured_positions_rad"][index]
    target = samples[0]["target_positions_rad"][index]
    direction = 1.0 if target >= initial else -1.0
    directed = [(row["measured_positions_rad"][index] - target) * direction for row in samples]
    steady_count = max(1, math.ceil(len(errors) * STEADY_FRACTION))
    settled_index = None
    for offset in range(len(errors)):
        if all(abs(error) <= SETTLING_TOLERANCE_RAD for error in errors[offset:]):
            settled_index = offset
            break
    times = [row["host_observed_monotonic_ns"] for row in samples]
    return {
        "rms_error_rad": math.sqrt(sum(error * error for error in errors) / len(errors)),
        "max_error_rad": max(abs(error) for error in errors),
        "overshoot_rad": max(0.0, max(directed)),
        "steady_state_error_rad": statistics.mean(errors[-steady_count:]),
        "settling_time_ms": (
            (times[settled_index] - samples[0]["host_command_sent_monotonic_ns"]) / 1e6
            if settled_index is not None
            else None
        ),
    }


def metrics(manifest: dict, samples: list[dict]) -> dict:
    intervals = [
        (right["host_observed_monotonic_ns"] - left["host_observed_monotonic_ns"]) / 1e6
        for left, right in zip(samples, samples[1:])
    ]
    return {
        "implementation": manifest["implementation"],
        "model": manifest["model"],
        "capabilities": manifest["capabilities"],
        "sample_count": len(samples),
        "first_observation_latency_ms": (
            samples[0]["host_observed_monotonic_ns"]
            - samples[0]["host_command_sent_monotonic_ns"]
        )
        / 1e6,
        "update_interval_ms": {
            "median": statistics.median(intervals) if intervals else None,
            "max": max(intervals) if intervals else None,
        },
        "actuators": {
            name: actuator_metrics(samples, index) for index, name in enumerate(ACTUATORS)
        },
    }


def suite_results(suite: dict, run_dir: Path) -> dict:
    cases = []
    for case_id in [item["id"] for item in suite["cases"]]:
        case = select_case(suite, case_id)
        case_dir = run_dir / f"{case['order']:02d}-{case_id}"
        status_path = case_dir / "case-status.json"
        status = json.loads(status_path.read_text()) if status_path.exists() else {
            "case_id": case_id, "case_order": case["order"], "status": "failed",
            "failure_reason": "case-status.json missing",
        }
        result = {"id": case_id, "order": case["order"], "status": status}
        if status.get("status") == "success":
            implementations = []
            for implementation in ("soma", "official"):
                path = case_dir / f"{implementation}.jsonl"
                implementations.append(metrics(*load_case(path, suite, case, implementation)))
            result["implementations"] = implementations
        cases.append(result)
    return {
        "schema_version": 2,
        "suite_sha256": suite["suite_sha256"],
        "case_order": [case["id"] for case in suite["cases"]],
        "representative_case": suite["representative_case"],
        "cases": cases,
    }


def metric_for(report: dict, case_id: str, implementation: str, actuator: str) -> dict | None:
    case = next((item for item in report["cases"] if item["id"] == case_id), None)
    if case is None:
        return None
    if case["status"].get("status") != "success":
        return None
    result = next(item for item in case["implementations"] if item["implementation"] == implementation)
    return result["actuators"][actuator]


def render_suite(report: dict, suite: dict) -> str:
    lines = [
        "# Official Simulation Fixed Case Suite",
        "",
        f"Suite SHA-256: `{report['suite_sha256']}`",
        f"Declared order: {', '.join(report['case_order'])}",
        f"Representative: `{report['representative_case']}` because it covers both commanded actuator groups, not because of measured results.",
        "",
        "## Case Matrix",
        "",
        "| Case | Status | Soma samples | Official samples | Commanded actuators |",
        "| --- | --- | ---: | ---: | --- |",
    ]
    for case_result, definition in zip(report["cases"], suite["cases"]):
        commanded = ", ".join(ACTUATORS[index] for index, value in enumerate(definition["deltas_rad"]) if value)
        if case_result["status"].get("status") == "success":
            counts = {item["implementation"]: item["sample_count"] for item in case_result["implementations"]}
            lines.append(f"| {case_result['id']} | success | {counts['soma']} | {counts['official']} | {commanded} |")
        else:
            reason = case_result["status"].get("failure_reason", "unknown failure").replace("|", "\\|")
            lines.append(f"| {case_result['id']} | {case_result['status'].get('status', 'failed')}: {reason} | - | - | {commanded} |")
    lines.extend(["", "## Per-Case Metrics", ""])
    for case in report["cases"]:
        lines.extend([f"### {case['id']}", ""])
        if case["status"].get("status") != "success":
            lines.extend([f"No metrics: {case['status'].get('failure_reason', 'case failed')}", ""])
            continue
        lines.extend([
            "| Implementation | Samples | First observation ms | Median interval ms | Max interval ms | Model |",
            "| --- | ---: | ---: | ---: | ---: | --- |",
        ])
        for result in case["implementations"]:
            lines.append(
                f"| {result['implementation']} | {result['sample_count']} | {result['first_observation_latency_ms']:.3f} | "
                f"{result['update_interval_ms']['median']:.3f} | {result['update_interval_ms']['max']:.3f} | "
                f"`{json.dumps(result['model'], sort_keys=True)}` |"
            )
        lines.extend(["", "| Implementation | Actuator | RMS | Max | Overshoot | Steady | Settling ms |", "| --- | --- | ---: | ---: | ---: | ---: | ---: |"])
        definition = select_case(suite, case["id"])
        for result in case["implementations"]:
            for index in definition["commanded_indexes"]:
                name = ACTUATORS[index]
                value = result["actuators"][name]
                settling = "unsettled" if value["settling_time_ms"] is None else f"{value['settling_time_ms']:.3f}"
                lines.append(f"| {result['implementation']} | {name} | {value['rms_error_rad']:.6f} | {value['max_error_rad']:.6f} | {value['overshoot_rad']:.6f} | {value['steady_state_error_rad']:.6f} | {settling} |")
        lines.append("")
    lines.extend(["## Descriptive Interaction Deltas", "", "Combined minus isolated RMS error (rad); descriptive only, never a selection or pass/fail score.", "", "| Implementation | Actuator | Delta |", "| --- | --- | ---: |"])
    for implementation in ("soma", "official"):
        for isolated, actuator in (("yaw-step", "yaw_body"), ("mirrored-antennas", "right_antenna"), ("mirrored-antennas", "left_antenna")):
            base = metric_for(report, isolated, implementation, actuator)
            combined = metric_for(report, "combined-yaw-antennas", implementation, actuator)
            value = "unavailable" if base is None or combined is None else f"{combined['rms_error_rad'] - base['rms_error_rad']:.6f}"
            lines.append(f"| {implementation} | {actuator} | {value} |")
    lines.extend(["", "## Evidence Boundary", "", "Every declared case remains in this report. Semantic, tracking, and host-runtime measures remain separate; no metric ranks or selects cases.", ""])
    return "\n".join(lines)


def stable_summary(suite: dict, run_dir: Path) -> dict:
    report = suite_results(suite, run_dir)
    return {
        "suite_sha256": report["suite_sha256"],
        "case_order": report["case_order"],
        "representative_case": report["representative_case"],
        "cases": [
            {
                "id": case["id"],
                "order": case["order"],
                "status_schema_version": case["status"].get("schema_version"),
                "status": case["status"].get("status"),
                "implementations": [
                    {
                        "implementation": result["implementation"],
                        "sample_count": result["sample_count"],
                        "capabilities": result["capabilities"],
                    }
                    for result in case.get("implementations", [])
                ],
            }
            for case in report["cases"]
        ],
    }


def verify_repeated_runs(suite: dict, first: Path, second: Path) -> dict:
    summaries = [stable_summary(suite, path) for path in (first, second)]
    if summaries[0] != summaries[1]:
        raise ValueError("clean suite runs disagree on stable structural evidence")
    if any(case["status"] != "success" for case in summaries[0]["cases"]):
        raise ValueError("repeated-run verification requires every case to succeed")
    return {"schema_version": 1, "runs": [str(first), str(second)], "stable": summaries[0]}


def render(report: dict, raw_paths: list[Path]) -> str:
    lines = [
        "# Soma vs Official Reachy Simulation",
        "",
        "This report compares behavior, not bitwise-identical trajectories.",
        "All actuator values are radians in the fixed nine-actuator order.",
        "Host monotonic clocks define each implementation's stimulus/observation window.",
        "",
        "## Inputs",
        "",
    ]
    lines.extend(f"- `{path.name}` (generated evidence)" for path in raw_paths)
    lines.extend(
        [
            "",
            "## Definitions",
            "",
            "- RMS/max error: measured minus target over the two-second dwell.",
            f"- Settling: first sample after which absolute error stays within {SETTLING_TOLERANCE_RAD:.2f} rad.",
            f"- Steady-state error: mean error over the final {int(STEADY_FRACTION * 100)}% of samples.",
            "- Overshoot: maximum target crossing in the commanded direction.",
            "- Timing: harness-local monotonic timestamps; no cross-process clock equivalence is claimed.",
            "- p99 is unavailable because this trace does not provide enough repeated runs.",
            "",
            "## Capability Labels",
            "",
            "Official sequence, TTL, timeline, disposition, and rejection semantics are `UNAVAILABLE`; they are not synthesized or scored. Soma retains them as `SOMA_ONLY` evidence.",
            "",
            "## Metrics",
            "",
        ]
    )
    for result in report["implementations"]:
        lines.extend(
            [
                f"### {result['implementation'].title()}",
                "",
                f"Samples: {result['sample_count']}",
                f"First observation latency: {result['first_observation_latency_ms']:.3f} ms",
                f"Median/max update interval: {result['update_interval_ms']['median']:.3f} / {result['update_interval_ms']['max']:.3f} ms",
                "",
                "| Actuator | RMS | Max | Overshoot | Steady | Settling ms |",
                "| --- | ---: | ---: | ---: | ---: | ---: |",
            ]
        )
        for name in ACTUATORS:
            value = result["actuators"][name]
            settling = "unsettled" if value["settling_time_ms"] is None else f"{value['settling_time_ms']:.3f}"
            lines.append(
                f"| {name} | {value['rms_error_rad']:.6f} | {value['max_error_rad']:.6f} | "
                f"{value['overshoot_rad']:.6f} | {value['steady_state_error_rad']:.6f} | {settling} |"
            )
        lines.append("")
    lines.extend(
        [
            "## Interpretation Boundary",
            "",
            "Semantic differences, motion tracking, and runtime timing are separate evidence. The report does not claim hardware parity, reliability, resource usage, or identical physics.",
            "",
        ]
    )
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--soma", type=Path)
    parser.add_argument("--official", type=Path)
    parser.add_argument("--suite", type=Path)
    parser.add_argument("--run-dir", type=Path)
    parser.add_argument("--json", type=Path, required=True)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--verify-runs", type=Path, nargs=2)
    args = parser.parse_args()
    if args.verify_runs:
        if not args.suite:
            parser.error("--suite is required with --verify-runs")
        result = verify_repeated_runs(load_suite(args.suite), *args.verify_runs)
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
        return
    if args.suite or args.run_dir:
        if not args.suite or not args.run_dir or not args.report:
            parser.error("--suite and --run-dir are required together")
        suite = load_suite(args.suite)
        report = suite_results(suite, args.run_dir)
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
        args.report.write_text(render_suite(report, suite))
        return
    if not args.soma or not args.official or not args.report:
        parser.error("--soma and --official are required for a single comparison")
    loaded = [load(args.soma), load(args.official)]
    report = {
        "schema_version": 1,
        "settling_tolerance_rad": SETTLING_TOLERANCE_RAD,
        "steady_fraction": STEADY_FRACTION,
        "implementations": [metrics(*item) for item in loaded],
    }
    args.json.parent.mkdir(parents=True, exist_ok=True)
    args.json.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    args.report.write_text(render(report, [args.soma, args.official]))


if __name__ == "__main__":
    main()
