import json
import runpy
from pathlib import Path

import pytest


ROOT = Path(__file__).resolve().parents[2]
METRICS = runpy.run_path(ROOT / "scripts/compare-open-duck-policy-metrics")


def evidence(**changes):
    value = {
        "emitted": 384,
        "states": 385,
        "applied": True,
        "expiry": False,
        "rejected": False,
        "max_inference_ns": 400_000,
        "max_message_age_ns": 20_000_000,
        "dropped_states": 2,
        "runtime_dropped_targets": 1,
        "min_root_height_m": 0.15,
        "max_abs_roll_rad": 0.1,
        "max_abs_pitch_rad": 0.12,
    }
    value.update(changes)
    return value


def runner_output(backend="rust", **changes):
    return json.dumps({
        "mode": "policy",
        "backend": backend,
        "process_path": True,
        "evidence": evidence(**changes),
    })


def test_parser_accepts_one_clean_process_result():
    parsed = METRICS["parse_run_output"]("diagnostic\n" + runner_output(), "rust")
    assert parsed["max_inference_ns"] == 400_000
    assert parsed["applied"] is True


@pytest.mark.parametrize(
    ("payload", "message"),
    [
        ("", "no output"),
        ("not-json", "final line is not JSON"),
        (runner_output(backend="python"), "does not match"),
        (runner_output(applied=False), "never reported applied control"),
        (runner_output(max_inference_ns=float("nan")), "must be finite"),
    ],
)
def test_parser_rejects_untrustworthy_results(payload, message):
    with pytest.raises(ValueError, match=message):
        METRICS["parse_run_output"](payload, "rust")


def test_aggregation_preserves_raw_runs_and_summarizes_run_peaks(tmp_path):
    case = tmp_path / "case.md"
    case.write_text("frozen case\n")
    rust = [evidence(max_inference_ns=200_000), evidence(max_inference_ns=400_000)]
    python = [evidence(max_inference_ns=500_000), evidence(max_inference_ns=700_000)]

    report = METRICS["build_report"](
        case, ["rust", "python"], {"rust": rust, "python": python}
    )

    assert report["schema_version"] == 1
    assert report["results"]["rust"]["runs"] == rust
    assert report["results"]["rust"]["summary"]["max_inference_ns"] == {
        "min": 200_000.0,
        "mean": 300_000.0,
        "max": 400_000.0,
    }
    assert report["results"]["python"]["summary"]["max_inference_ns"]["mean"] == 600_000

    markdown = METRICS["render_markdown"](report)
    assert "Each cell is `mean [min, max]`" in markdown
    assert "| Run max inference | ms | 0.300 [0.200, 0.400] | 0.600 [0.500, 0.700] |" in markdown
    assert "`-50.0%` versus Python" in markdown
    assert "does not measure CPU, RSS, startup time" in markdown


def test_aggregation_surfaces_rejected_runs(tmp_path):
    case = tmp_path / "case.md"
    case.write_text("frozen case\n")
    runs = [evidence(), evidence(rejected=True)]

    report = METRICS["build_report"](case, ["rust"], {"rust": runs})

    assert report["results"]["rust"]["rejected_run_count"] == 1
    assert "rejected runs = `1`" in METRICS["render_markdown"](report)


def test_single_backend_report_omits_relative_claim(tmp_path):
    case = tmp_path / "case.md"
    case.write_text("frozen case\n")
    report = METRICS["build_report"](case, ["rust"], {"rust": [evidence()]})

    markdown = METRICS["render_markdown"](report)
    assert "Direct Comparison" not in markdown
    assert "`rust`: 1 runs, all applied = `true`, rejected runs = `0`" in markdown
