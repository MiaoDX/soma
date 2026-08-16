# Robot Model, Manifest, and Calibration

> Status: Deep Research. Goal: define the canonical robot identity/model layer that remains stable across hardware variants, simulators, SDKs, and policies.

## Question

Should Soma treat URDF, MJCF, USD, or another simulator format as the source of truth for a robot? How should hardware identity, joint topology, transmissions, sensors, frames, calibration, simulation tuning, and policy compatibility be represented across wheeled, legged, and mobile-manipulation embodiments?

## Executive conclusion

No existing simulation/robot description format should be Soma's complete source of truth.

> **Soma should define a canonical `RobotManifest` / model bundle for engineering identity and compatibility, then generate or validate URDF, MJCF, USD, runtime tables, and policy schemas from it.**

The reason is semantic mismatch: URDF, MJCF, USD Physics, and SDFormat model different concerns and have different capabilities. A conversion pipeline can preserve a useful common subset, but cannot be assumed to be lossless.

## Why the current industry pattern is insufficient

A common robotics repository contains:

```text
robot.urdf
robot.xml       # MJCF
robot.usd
joint_map.yaml
limits.yaml
calibration.yaml
policy_config.yaml
```

with duplicated values maintained manually.

This creates predictable failure modes:

- joint order differs between policy and hardware;
- sign conventions differ between simulator and robot;
- modified inertial parameters exist only in MJCF;
- camera calibration is updated but USD is stale;
- a coupled wrist is simplified differently in MoveIt and physics;
- a policy is loaded onto a robot revision with a changed gearbox or sensor layout.

Soma should make those mismatches detectable by construction.

## What the major formats actually optimize for

### URDF

URDF is deeply useful as an ecosystem interchange format for ROS tooling, kinematic trees, meshes, joints, and common sensor/control integrations. Its broad ecosystem support makes Soma compatibility essential.

But Soma should not assume URDF can encode every actuator/transmission, simulator parameter, calibration lifecycle, hardware revision, closed-loop/coupled mechanism, or multi-backend physics detail required by a production platform.

### MJCF

MuJoCo's MJCF exposes rich simulator-specific semantics. Its actuator model separates transmission, activation dynamics, and force generation. It supports tendons, equality constraints, rich contact/solver properties, and simulator sensors. MuJoCo explicitly notes that one actuator can affect multiple joints and multiple actuators can contribute force at a joint.

This reinforces the Soma rule:

> **Actuator, transmission, and joint are distinct engineering entities.**

MJCF is therefore an important generated/curated simulation artifact, but not a suitable full product/hardware inventory format.

### USD / USD Physics

USD is a composition and scene-description system with increasingly rich physics schemas. USD Physics models rigid bodies, masses, collision, joint reference frames, drives, articulations, and filtering, and is especially important for Isaac Sim and asset composition.

USD also has semantics that differ from a simple robot kinematic tree. For example, a physics joint has two local reference frames associated with its constrained bodies; articulation topology is inferred from bodies/joints and can represent fixed or floating roots.

USD is excellent for simulation/digital-world composition but should not own production device identity, firmware compatibility, calibration state, or runtime control authority.

### SDFormat

SDFormat's evolution illustrates how subtle frame semantics become. Modern SDF supports named frames and explicit `relative_to` semantics, while older versions had different/legacy interpretation rules.

This is a warning for Soma: coordinate-frame semantics must be explicit and versioned independently of any one serialization syntax.

## Industry implementation lessons

### Unitree

`unitree_mujoco` ships robot MJCF assets and intentionally keeps motor numbering consistent with real robot hardware while reusing SDK2 low-level messages. The important lesson is not “MJCF is canonical”; it is that **hardware joint/motor identity and public control semantics must match simulation**.

### AgiBot Genie Sim

Genie Sim's newer asset organization absorbs URDF-to-USD importer/version differences behind staged per-robot assets and backend-specific physics payloads. It also explicitly handles coupled-joint constraints and different physics backends.

This supports Soma's proposed model:

```text
canonical engineering model
        +
backend-specific overlays
```

rather than hoping one generated file is sufficient everywhere.

## Proposed Soma model stack

```text
RobotManifest
  identity
  topology
  actuator/device inventory
  frames
  nominal physical parameters
  control/safety limits
  capability declarations
       |
       +---- Calibration Overlay (per serial number)
       |
       +---- Hardware Variant Overlay
       |
       +---- MuJoCo Physics Overlay
       +---- Isaac/USD Physics Overlay
       +---- Genesis Overlay
       +---- ROS/MoveIt semantic overlay
       |
   model compiler / validator
       |
       +--> RT joint/actuator tables
       +--> public RobotManifest metadata
       +--> URDF/Xacro
       +--> MJCF
       +--> USD assets/manifests
       +--> policy observation/action maps
       +--> conformance hashes
```

## Robot identity

Separate product identity from model artifacts.

Recommended fields:

```text
robot_model_id
product_family
variant_id
hardware_revision
robot_serial
model_bundle_id
manifest_schema_version
```

A runtime should expose both:

- what physical robot this is;
- what model bundle is currently active.

## Topology model

The manifest should distinguish:

### Bodies / links

- stable body ID and name;
- parent/topological relation;
- nominal transform;
- mass/inertia reference;
- collision/visual asset references.

### Joints / generalized coordinates

- stable joint ID/name;
- type and axis;
- positive direction;
- limits;
- home/reference position;
- control modes supported;
- logical control group.

### Actuators

- actuator ID;
- physical device/drive identity;
- motor parameters where needed;
- command/state capabilities;
- thermal/electrical limits;
- bus/device mapping kept in hardware overlay rather than public semantics.

### Transmissions

Must support more than 1:1 gearing:

- scalar gearbox;
- differential;
- tendon/cable;
- coupled wrist;
- series elasticity;
- multiple actuators to multiple coordinates;
- mechanically constrained coordinates.

A simple `joint_to_motor_index[]` is insufficient as the long-term model.

## Frames

Soma needs a canonical frame graph independent of ROS TF naming conventions.

Each frame needs:

```text
frame_id
stable name
parent/reference relationship
nominal transform
semantic role
whether transform is fixed / calibrated / runtime-estimated
```

Examples:

```text
base
body
imu
camera_left_optical
lidar
left_foot
left_gripper_tcp
map/odom-like runtime frames declared separately
```

Frame conventions must define handedness, quaternion ordering, units, and optical sensor conventions.

ROS 2 Adapter maps these frames into TF/REP conventions as needed; ROS is not the source of truth.

## Nominal parameters vs calibration

Do not merge design nominal values and serial-number-specific calibration into one file.

### Nominal model

Versioned with product/model bundle:

- nominal link geometry;
- designed gear ratios;
- nominal inertias;
- joint axes;
- sensor mounting design;
- safety envelope defaults.

### Calibration overlay

Owned by the physical robot and tied to hardware identities:

- encoder zero offsets;
- actuator torque/current calibration;
- IMU alignment/bias parameters;
- camera intrinsics;
- camera/LiDAR extrinsics;
- force/torque sensor offsets/scales;
- wheel radius/base calibration;
- serial-specific joint alignment.

Calibration should include:

```text
calibration_id
schema_version
robot_serial
component serial(s)
created_at
method/tool version
operator/station if relevant
input artifact hashes
quality metrics
signature/checksum
```

## Calibration lifecycle

Calibration is not ordinary configuration and should not be overwritten silently by OTA.

Recommended lifecycle:

```text
UNCALIBRATED
 -> CANDIDATE
 -> VALIDATED
 -> ACTIVE
 -> SUPERSEDED
```

Rules:

- activation requires hardware identity match;
- schema migration is explicit;
- previous calibration remains recoverable;
- replacing a calibrated component invalidates dependent calibration scopes;
- health diagnostics can mark a calibration suspect without automatically rewriting it.

## Simulation overlays

Physics engines need parameters that are not necessarily product truth:

```text
contact friction/compliance
solver parameters
joint damping/friction approximation
actuator latency/bandwidth
sensor noise/delay
mesh simplification
collision filtering
soft constraints
randomization ranges
```

These belong in backend/scenario overlays, not in the nominal hardware manifest.

We should explicitly distinguish:

```text
measured physical parameter
engineering nominal parameter
simulator-fitting parameter
training randomization distribution
```

They may have related values, but they are not semantically identical.

## Model bundle and hashes

Every model release should produce stable identifiers:

```text
model_bundle_id
manifest_hash
joint_schema_hash
actuator_schema_hash
frame_schema_hash
sensor_schema_hash
```

The exact hash canonicalization must be defined by the model compiler, not by hashing arbitrary YAML formatting.

These hashes are referenced by:

- runtime startup checks;
- PolicyBundle compatibility;
- MCAP/incident metadata;
- simulator conformance;
- SDK capability discovery;
- OTA ReleaseManifest.

## Policy compatibility

A policy cannot be considered compatible merely because tensor dimensions match.

PolicyBundle should identify:

```text
required robot/model family
joint/action schema hash
observation schema hash
frame assumptions
required sensors/capabilities
policy/control rate
normalization/clipping
history layout
runtime/inference requirements
```

The runtime refuses unsafe mismatches before enabling control.

## Asset packaging

Meshes, textures, calibration, and backend models need content-addressed/versioned packaging.

Suggested `ModelBundle` logical layout:

```text
manifest/
calibration-schema/
assets/visual/
assets/collision/
generated/urdf/
generated/mjcf/
generated/usd/
overlays/mujoco/
overlays/isaac/
overlays/genesis/
metadata/
```

Generated artifacts should record compiler version and source manifest hash.

## Conformance tests

A model compiler is only credible if it tests generated backends.

Required cross-backend checks:

- joint count/order/name/id;
- positive direction;
- limits;
- default/reference pose;
- frame transforms;
- mass/inertia sanity;
- actuator/transmission mapping;
- static gravity behavior;
- simple torque/position response;
- sensor frame/timestamp mapping;
- collision group expectations.

Do not require exact dynamics equality across engines; define tolerances and backend-specific expectations.

## Proposed V0 `RobotManifest`

Keep V0 small enough to implement:

```text
identity
links/bodies
joints
actuators
simple transmissions
sensors
frames
control/safety limits
capabilities
asset references
calibration schema references
```

Advanced tendon/closed-loop semantics can be modeled through extensible transmission/constraint records while V0 supports the first reference embodiment.

## ADR implications

This research supports decisions roughly equivalent to:

1. Soma owns a canonical RobotManifest/model bundle.
2. URDF, MJCF, and USD are generated/validated target artifacts, not the complete source of truth.
3. Calibration is serial-specific state with its own lifecycle.
4. Simulator tuning is an overlay, not hardware truth.
5. Policies and logs bind to model/schema identities.

## Experiments required

1. Define a minimal manifest for a differential-drive reference robot.
2. Extend it to a legged robot with >10 DOF.
3. Generate/validate URDF + MJCF from the same source.
4. Create USD/Isaac mapping prototype.
5. Prove calibration overlay changes sensor/joint transforms without modifying nominal source.
6. Validate PolicyBundle mismatch rejection.

## Primary references

- MuJoCo model/actuator overview: https://mujoco.readthedocs.io/en/stable/overview.html
- MuJoCo MJCF XML reference: https://mujoco.readthedocs.io/en/stable/XMLreference.html
- OpenUSD Physics schema: https://openusd.org/release/api/usd_physics_page_front.html
- SDFormat pose/frame semantics: https://sdformat.org/tutorials/specification/pose_frame_semantics/
- Unitree MuJoCo: https://github.com/unitreerobotics/unitree_mujoco
- AgiBot Genie Sim: https://github.com/AgibotTech/genie_sim
