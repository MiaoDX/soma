from __future__ import annotations

import argparse
import json
import math
import statistics
from pathlib import Path

from .common import ACTUATORS

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
    parser.add_argument("--soma", type=Path, required=True)
    parser.add_argument("--official", type=Path, required=True)
    parser.add_argument("--json", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
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
