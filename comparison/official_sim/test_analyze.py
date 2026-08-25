from __future__ import annotations

import unittest
import json
import math
import tempfile
from pathlib import Path

from .analyze import actuator_metrics, render_suite, suite_results
from .common import ACTUATORS, load_suite, select_case, validate_movement


class AnalyzerTest(unittest.TestCase):
    def test_metrics_use_target_minus_measured_trace(self) -> None:
        rows = []
        for index, measured in enumerate([0.0, 0.05, 0.1, 0.1]):
            rows.append(
                {
                    "host_observed_monotonic_ns": (index + 1) * 20_000_000,
                    "host_command_sent_monotonic_ns": 0,
                    "target_positions_rad": [0.1] * len(ACTUATORS),
                    "measured_positions_rad": [measured] * len(ACTUATORS),
                }
            )
        result = actuator_metrics(rows, 0)
        self.assertAlmostEqual(result["max_error_rad"], 0.1)
        self.assertAlmostEqual(result["steady_state_error_rad"], 0.0)
        self.assertEqual(result["settling_time_ms"], 60.0)
        self.assertEqual(result["overshoot_rad"], 0.0)


class SuiteTest(unittest.TestCase):
    def setUp(self) -> None:
        self.suite = {
            "schema_version": 1,
            "actuator_order": ACTUATORS,
            "units": "radians",
            "warmup_s": 1.0,
            "sample_period_s": 0.02,
            "dwell_s": 2.0,
            "representative_case": "case-a",
            "cases": [{"id": "case-a", "deltas_rad": [0.15] + [0.0] * 8}],
        }

    def load(self, suite: dict) -> dict:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "suite.json"
            path.write_text(json.dumps(suite))
            return load_suite(path)

    def test_suite_hash_is_over_exact_file_bytes(self) -> None:
        first = self.load(self.suite)
        second = self.load({**self.suite, "warmup_s": 1.5})
        self.assertNotEqual(first["suite_sha256"], second["suite_sha256"])
        self.assertEqual(select_case(first, "case-a")["commanded_indexes"], [0])

    def test_rejects_unknown_key(self) -> None:
        with self.assertRaisesRegex(ValueError, "top-level keys"):
            self.load({**self.suite, "extra": True})

    def test_rejects_duplicate_ids(self) -> None:
        duplicate = {**self.suite, "cases": self.suite["cases"] * 2}
        with self.assertRaisesRegex(ValueError, "unique"):
            self.load(duplicate)

    def test_rejects_wrong_actuator_count(self) -> None:
        invalid = {**self.suite, "cases": [{"id": "case-a", "deltas_rad": [0.15]}]}
        with self.assertRaisesRegex(ValueError, "nine finite"):
            self.load(invalid)

    def test_rejects_nonfinite_delta(self) -> None:
        invalid = {**self.suite, "cases": [{"id": "case-a", "deltas_rad": [math.nan] + [0.0] * 8}]}
        with self.assertRaisesRegex(ValueError, "nine finite"):
            self.load(invalid)

    def test_rejects_missing_representative(self) -> None:
        with self.assertRaisesRegex(ValueError, "representative_case"):
            self.load({**self.suite, "representative_case": "missing"})

    def test_movement_must_hold_for_every_commanded_actuator(self) -> None:
        samples = [{"measured_positions_rad": [0.1] + [0.01] + [0.0] * 7}]
        with self.assertRaisesRegex(RuntimeError, "stewart_1"):
            validate_movement([0.0] * 9, samples, [0, 1])

    def test_failed_case_remains_in_aggregate_report(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            suite_path = root / "suite.json"
            suite_path.write_text(json.dumps(self.suite))
            suite = load_suite(suite_path)
            case_dir = root / "00-case-a"
            case_dir.mkdir()
            (case_dir / "case-status.json").write_text(json.dumps({
                "schema_version": 1, "case_id": "case-a", "case_order": 0,
                "status": "failed", "failure_reason": "injected adapter failure",
            }))
            report = suite_results(suite, root)
            rendered = render_suite(report, suite)
            self.assertEqual(report["cases"][0]["status"]["status"], "failed")
            self.assertIn("injected adapter failure", rendered)


if __name__ == "__main__":
    unittest.main()
