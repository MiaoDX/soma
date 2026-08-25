from __future__ import annotations

import unittest

from .analyze import actuator_metrics
from .common import ACTUATORS


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


if __name__ == "__main__":
    unittest.main()

