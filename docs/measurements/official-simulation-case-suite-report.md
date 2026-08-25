# Official Simulation Fixed Case Suite

This committed report was generated from a clean suite run after implementing
the fixed three-case plan. It is structural representative evidence; raw logs
and JSONL traces remain in ignored `output/` directories.

Suite SHA-256: `e0b4ab12796573baa5346c51b331ffb31385a34f0017ec92e3a9b0663f240072`

Declared order: `yaw-step`, `mirrored-antennas`, `combined-yaw-antennas`.

Representative case: `combined-yaw-antennas`, selected by predeclared coverage
of both yaw and antenna actuator groups, never by measured outcome.

| Case | Status | Soma samples | Official samples | Commanded actuators |
| --- | --- | ---: | ---: | --- |
| yaw-step | success | 100 | 100 | yaw_body |
| mirrored-antennas | success | 100 | 100 | right_antenna, left_antenna |
| combined-yaw-antennas | success | 100 | 100 | yaw_body, right_antenna, left_antenna |

Both clean runs used independent Soma and official lifecycles per case. Every
commanded actuator moved at least 0.05 rad. The repeated-run verifier agreed on
suite identity, case order, representative label, status schema, capabilities,
and planned sample counts; timing and numerical metrics remain descriptive.

See the generated full matrix at `output/official-sim-suite-run1/report.md`
when reproducing the gate with `scripts/run-official-sim-comparison`.
