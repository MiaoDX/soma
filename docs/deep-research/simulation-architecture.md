# Simulation Architecture

> Status: Deep Research baseline. Simulation is a first-class Soma backend, not an afterthought or a ROS-only tool.

## Question

How can Soma support MuJoCo, Isaac Sim/Lab, Genesis, SIL, HIL, reinforcement learning, deterministic regression, and SDK-level simulation without coupling the production robot runtime to a simulator or forcing every simulation mode through the same transport path?

## Working thesis

Soma should standardize **control semantics, time, model identity, observations/actions, lifecycle, and safety behavior**, while allowing different simulators to use different execution topologies.

The key abstraction is the Plant contract:

```text
Controller / Estimator / Safety
             |
        Plant Contract
      /        |         \
 Hardware   MuJoCo    External Sim
   Plant     Plant     Isaac/Genesis
```

Simulation also needs a separate `SimulationControl` contract for capabilities that do not exist on a physical robot.

## Why simulation is not merely another HAL

HAL describes physical buses and devices. A simulator should not pretend it owns EtherCAT PDOs or CAN identifiers unless the explicit goal is HIL.

A normal physics simulator should implement generalized robot semantics:

- joint/sensor state;
- command application;
- timing;
- contacts;
- lifecycle;
- fault/safety semantics.

A separate HIL layer can emulate bus-level behavior when validating the actual driver stack.

## Two contracts

### Plant

Conceptually:

```rust
pub trait Plant {
    type Error;

    fn configure(&mut self) -> Result<PlantInfo, Self::Error>;
    fn activate(&mut self) -> Result<(), Self::Error>;
    fn read_cycle(&mut self, ctx: &CycleContext, state: &mut RtRobotState)
        -> Result<(), Self::Error>;
    fn write_cycle(&mut self, ctx: &CycleContext, command: &RtRobotCommand)
        -> Result<CommandResult, Self::Error>;
    fn health(&self) -> PlantHealth;
}
```

### SimulationControl

```rust
pub trait SimulationControl {
    type Snapshot;
    type Error;

    fn reset(&mut self, request: ResetRequest) -> Result<ResetResult, Self::Error>;
    fn step(&mut self, ticks: u32) -> Result<StepResult, Self::Error>;
    fn snapshot(&self) -> Result<Self::Snapshot, Self::Error>;
    fn restore(&mut self, snapshot: &Self::Snapshot) -> Result<(), Self::Error>;
    fn set_seed(&mut self, seed: u64) -> Result<(), Self::Error>;
    fn randomize(&mut self, spec: RandomizationSpec) -> Result<(), Self::Error>;
}
```

`reset_world`, ground truth, object spawning, and domain randomization must not leak into the normal real-robot API.

## Four simulation modes

### A. SDK/API-level simulator

Purpose: application and SDK development.

```text
Python / ROS2 / App
        |
  Robot Protocol
        |
  robot-runtime
        |
 Simulated Plant
        |
      MuJoCo
```

The application should be able to connect to a simulated endpoint with nearly the same public API as a real robot. This validates lease, actions, events, lifecycle, reconnect behavior, and SDK compatibility.

Unitree's simulation tools are a useful reference: simulation reuses important low-level SDK/DDS semantics so software can move between simulated and physical robots without replacing the application contract.

### B. Controller-in-the-loop / SIL

Purpose: validate production estimator, WBC, gait, joint controller, safety, and state machine.

```text
production robot-rt
       |
   Plant API
       |
MuJoCo / simulator
       |
Simulation Clock
```

The controller should run against simulation time rather than wall clock. This mode must support pause, step, reset, faster-than-real-time execution where possible, and deterministic scenario setup.

### C. Batch RL / synthetic data

Purpose: hundreds/thousands of parallel environments and GPU throughput.

```text
RL training process
       |
Native tensor API
       |
Isaac Lab / Genesis / GPU simulator
```

Do **not** force every environment through Zenoh, DDS, robot-runtime, or per-environment IPC. The shared contract here is semantic:

- observation schema;
- action schema;
- joint/frame ordering;
- normalization;
- clipping;
- control rate/decimation;
- policy runtime assumptions;
- model identity.

### D. HIL / bus-level simulation

Purpose: validate the real host, driver, bus, watchdog, and recovery behavior.

```text
real robot-rt host
       |
EtherCAT / CAN-FD
       |
virtual drive / bus emulator
       |
physics model
```

Only this mode needs to emulate PDOs, CAN frames, drive state machines, device dropouts, encoder faults, and bus timing.

## Time is a first-class simulation contract

Soma should define explicit time domains:

- `MONOTONIC_ROBOT` — physical control deadlines/watchdogs;
- `SIMULATION` — pause/step/reset/accelerated time;
- `UTC/PTP` — fleet correlation and external synchronized sensors.

Cyclic data should carry, where applicable:

```text
time_domain
epoch_id
tick
capture_time
publish_time
target_apply_time
valid_until
sequence_number
```

`epoch_id` changes when simulation resets or a runtime session is recreated. Commands from an old epoch are invalid even if they arrive late.

### Lockstep

A robust controller-in-the-loop semantic is:

```text
Simulator: State(epoch=3, tick=120)
       |
Controller: Command(target_tick=120)
       |
Simulator applies command and advances physics
       |
Simulator: State(epoch=3, tick=121)
```

This borrows an important lesson from PX4/ArduPilot SITL: simulation compatibility is fundamentally a **time/step contract**, not only a sensor/actuator API.

## Model identity

Every physical or simulated endpoint should expose stable identity:

```text
robot_model_id
hardware_revision
model_bundle_id
calibration_id
joint_schema_hash
frame_schema_hash
sensor_schema_hash
protocol_version
```

A policy or replay should never silently assume that `humanoid_v2` means the same joint order or inertial model as another build.

## Canonical model strategy

URDF, MJCF, and USD have different expressive capabilities. Soma should not assume lossless round-trip conversion.

Recommended model pipeline:

```text
RobotManifest / canonical engineering model
        |
        +-- common physical parameters
        +-- calibration overlay
        +-- MuJoCo overlay
        +-- Isaac/USD overlay
        +-- Genesis overlay
        |
        +--> generated/validated URDF
        +--> generated/validated MJCF
        +--> generated/validated USD
```

The model compiler should produce hashes used by runtime, logs, policies, and tests.

## MuJoCo role

MuJoCo should be Soma's first reference simulation backend because it is well suited to:

- low-latency control loops;
- headless CI;
- fast startup;
- deterministic scenario construction;
- direct native API integration;
- controller regression;
- SDK-level simulation.

A Rust adapter can wrap the C API behind a narrow FFI boundary. The production controller should be able to run against MuJoCo without importing ROS or Python.

## Isaac Sim / Isaac Lab role

Isaac should be treated as a heavy external simulation service/container rather than a dependency of the core Rust workspace.

Best fits:

- RTX sensors and perception;
- complex USD scenes;
- synthetic data;
- GPU reinforcement learning via Isaac Lab;
- manipulation/environment scenarios.

Its Python/CUDA/USD dependency matrix should be isolated from production runtime versions.

## Genesis role

Genesis is promising for GPU-parallel research and multi-physics experimentation. It should initially be integrated as an external Python adapter with a strict conformance layer, not as a production-core dependency.

The architecture should make it easy to add/remove a simulator without changing the Robot Protocol or controller semantics.

## Ground truth separation

Simulators expose information no real robot can observe perfectly: exact base pose, contact truth, object poses, noiseless velocity, external forces.

These must live in a `SimulationGroundTruth` capability, not in normal sensor messages.

This enables asymmetric training (e.g. privileged critic) while allowing tooling to detect accidental production dependencies on sim-only fields.

## PolicyBundle

A production policy is more than an ONNX file. Soma should define a bundle such as:

```text
PolicyBundle
  manifest
  model artifact
  observation schema
  action schema
  normalization
  clipping
  history length
  policy rate
  control decimation
  required model/schema hashes
  required capabilities
  runtime compatibility
  checksums/signature
```

This is critical for sim-to-real because joint order, observation order, normalization, and timing mismatches can be more dangerous than model-format mismatches.

## Fault injection

Simulation should make failure behavior testable:

- encoder freeze/dropout;
- IMU bias/noise/dropout;
- actuator stuck/weak/over-temperature;
- bus delay/drop;
- power sag;
- runtime disconnect;
- command delay/staleness;
- joint limit/safety intervention;
- time synchronization faults.

The same high-level fault/event codes should appear in simulation and physical logs where semantics match.

## Record and replay

Every significant run should capture enough metadata to reproduce the environment:

```text
release_id
runtime/controller build IDs
model_bundle_id
policy_bundle_id
simulator + version
scenario_id
seed
physics parameters
epoch/tick
external inputs
requested/accepted/applied commands
safety events
periodic snapshots where supported
```

MCAP is a good container for robot streams and metadata. Cross-simulator bitwise trajectory equality is not a realistic goal; define behavioral tolerances and task-level metrics instead.

## Conformance tests

Every simulator backend should pass a shared suite:

- identity/model manifest validation;
- joint direction/order/units;
- static pose and gravity checks;
- command semantics;
- reset/epoch behavior;
- action/state lifecycle;
- safety limits;
- sensor timestamps;
- record/replay metadata;
- SDK smoke tests.

Additional backend-specific tests are expected.

## Repository shape

Keep heavy simulators outside the core dependency graph:

```text
robot-core/
robot-sim-contract/
robot-sim-mujoco/
robot-sim-isaac/
robot-sim-genesis/
robot-sim-hil/
robot-sim-conformance/
```

These may begin as directories in one repository and split later if dependency/CI pressure warrants it.

## Open questions

1. Exact Plant trait shape and sync/async boundary.
2. Whether MuJoCo runs in-process with `robot-rt` for CI or as a separate process for stronger production parity; likely support both.
3. Canonical RobotManifest/model representation.
4. GPU tensor interchange for policy/runtime and batch simulation.
5. Required fidelity tiers: controller regression vs perception vs HIL.
6. Snapshot portability/versioning.
7. How real incident MCAP is transformed into replayable simulator scenarios.
8. How to quantify sim-to-real divergence continuously.

## Reference projects and sources

- Unitree MuJoCo — https://github.com/unitreerobotics/unitree_mujoco
- Unitree Isaac Lab — https://github.com/unitreerobotics/unitree_sim_isaaclab
- AgiBot Genie Sim — https://github.com/AgibotTech/genie_sim
- LimX Dynamics — https://github.com/limxdynamics
- Booster Robotics — https://github.com/BoosterRobotics
- MuJoCo — https://mujoco.org/
- NVIDIA Isaac Sim — https://developer.nvidia.com/isaac/sim
- NVIDIA Isaac Lab — https://isaac-sim.github.io/IsaacLab/
- Genesis — https://genesis-embodied-ai.github.io/
- PX4 simulation — https://docs.px4.io/main/en/simulation/
- ArduPilot SITL — https://ardupilot.org/dev/docs/sitl-simulator-software-in-the-loop.html
- MCAP — https://mcap.dev/
