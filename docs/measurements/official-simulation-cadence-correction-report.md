# Official Simulation Cadence Correction

## Finding

The original comparison advanced Soma's MuJoCo model once per 20 ms
`robot-rt` control period. The pinned model timestep is 2 ms, so two seconds of
wall time advanced only about 0.2 seconds of simulation time. The official
backend advances physics at 500 Hz and updates controls every ten steps at
50 Hz.

This was not an actuator-tuning difference. Soma's imported
`reachy_mini.xml` and the official v1.9.0 image copy have the same SHA-256,
`efd7e49d4288e5ef53945771a1f116584aa2c8b89721b725d5d77da9f0fcbf46`.
Their position gains, damping, friction, armature, force ranges, masses, and
inertias are therefore identical for this model.

## Correction

The simulation adapter now receives the 20 ms control period when loading the
model, reads its 2 ms physics timestep, validates an exact integral schedule,
and owns the resulting ten-substep advance. Zero, non-finite, shorter-than-one-
step, and non-integral schedules fail before execution. The generic hardware
`Plant` trait remains unchanged. Future MuJoCo robot adapters must reuse this
precheck and prove their expected decimation in a model-specific test.

Product simulation launchers now build and run Rust release binaries. Release
mode reduces avoidable execution overhead, but it did not correct the time
scale; the ten physics substeps did.

## Before And After

The table compares the historical fixed-suite Soma result with corrected clean
run 1. Settling is the first sample after which absolute error remains within
0.03 rad.

| Case | Actuator | Old Soma RMS | Corrected Soma RMS | Old settling | Corrected settling |
| --- | --- | ---: | ---: | ---: | ---: |
| yaw-step | yaw_body | 0.050281 | 0.018935 | 480.148 ms | 80.062 ms |
| mirrored-antennas | right_antenna | 0.043155 | 0.017362 | 460.053 ms | 80.089 ms |
| mirrored-antennas | left_antenna | 0.043155 | 0.017362 | 460.053 ms | 80.089 ms |
| combined-yaw-antennas | yaw_body | 0.050290 | 0.018937 | 480.098 ms | 80.098 ms |
| combined-yaw-antennas | right_antenna | 0.043188 | 0.017368 | 460.079 ms | 80.098 ms |
| combined-yaw-antennas | left_antenna | 0.043118 | 0.017355 | 460.079 ms | 80.098 ms |

## Corrected Soma Versus Official

| Case | Actuator | Soma RMS | Official RMS | Soma settling | Official settling |
| --- | --- | ---: | ---: | ---: | ---: |
| yaw-step | yaw_body | 0.018935 | 0.018948 | 80.062 ms | 80.067 ms |
| mirrored-antennas | right_antenna | 0.017362 | 0.017366 | 80.089 ms | 80.065 ms |
| mirrored-antennas | left_antenna | 0.017362 | 0.017367 | 80.089 ms | 80.065 ms |
| combined-yaw-antennas | yaw_body | 0.018937 | 0.018950 | 80.098 ms | 80.082 ms |
| combined-yaw-antennas | right_antenna | 0.017368 | 0.017376 | 80.098 ms | 80.082 ms |
| combined-yaw-antennas | left_antenna | 0.017355 | 0.017356 | 80.098 ms | 80.082 ms |

Both corrected clean suites passed with the same suite hash, case order,
representative label, status schema, capabilities, and exactly 100 planned
samples for each implementation and case. Every commanded actuator exceeded
the 0.05 rad movement gate. Soma's worst planned-sample lateness across the six
corrected streams was 2.88 ms against a 20 ms period, and no owned process or
container remained after either run. Raw evidence is retained under ignored
`output/official-sim-cadence-run1` and `output/official-sim-cadence-run2`.

These results establish parity for the declared public simulation cases, not
hardware parity or a general controller ranking. Soma uses MuJoCo 3.9.0 and the
official pinned image uses MuJoCo 3.3.0; host timing and numerical trajectories
remain descriptive rather than post-hoc selection gates.
