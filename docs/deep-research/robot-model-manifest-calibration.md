# Robot Model, Manifest, and Calibration

> Status: Deep Research. Goal: define the canonical robot identity/model layer that remains stable across hardware variants, simulators, SDKs, and policies.

## Question

Should Soma treat URDF, MJCF, USD, or another simulator format as the source of truth for a robot? How should hardware identity, joint topology, transmissions, sensors, frames, calibration, simulation tuning, and policy compatibility be represented across wheeled, legged, and mobile-manipulation embodiments?

## Executive conclusion

No existing simulation/robot description format should be Soma's complete source of truth.

> **Soma should define a canonical `ProductModelManifest` for shared engineering semantics, then compose it with separately governed instance, calibration, control, safety, and runtime-capability artifacts. URDF, MJCF, USD, runtime tables, and policy schemas are generated or validated views of that composition.**

The reason is semantic mismatch: URDF, MJCF, USD Physics, and SDFormat model different concerns and have different capabilities. A conversion pipeline can preserve a useful common subset, but cannot be assumed to be lossless.

The split is intentional:

```text
ProductModelManifest       shared product topology and nominal semantics
RobotInstanceManifest      one physical robot's provisioned identity/configuration
DeviceInventory            observed installed components and firmware revisions
CalibrationSet             serial-specific measured corrections
ControlProfile             operational controller tuning and non-authoritative limits
SafetyProfile              independently governed safety-authoritative limits/behavior
RuntimeCapabilitySnapshot  live, ephemeral availability and degradation state
```

These artifacts have different owners, lifecycles, trust levels, and hash domains. Combining them into one mutable `RobotManifest` would make a model update capable of silently changing per-robot state or safety authority.

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
ProductModelManifest
  + Hardware Variant / backend physics / ROS semantic overlays
                 |
         model compiler / validator
                 |
         +-------+------------------+
         |                          |
         v                          v
URDF/MJCF/USD target assets    topology/schema artifacts

ProductModelManifest
RobotInstanceManifest + DeviceInventory
CalibrationSet
ControlProfile
SafetyProfile (independent trust/release domain)
                 |
         runtime validator / composer
                 |
         +-------+------------------+
         |                          |
         v                          v
RT tables and controller input  public composed RobotManifest
                 |
       live health / faults / mode
                 |
       RuntimeCapabilitySnapshot
       (runtime output, not model input)
```

The composed public view may retain the compatibility name `RobotManifest`, but it is a runtime projection rather than a separately authored source file. It should include the IDs and hashes of every contributing artifact so consumers can tell which product model, instance, calibration, control, and safety state produced it.

## Product and instance identity

Separate product identity from model artifacts.

Recommended `ProductModelManifest` identity fields:

```text
robot_model_id
product_family
variant_id
hardware_revision
product_model_bundle_id
manifest_schema_version
```

`robot_model_id` is the stable product-model identity retained for protocol compatibility; it does not identify a serial-numbered physical robot.

Recommended `RobotInstanceManifest` fields:

```text
robot_serial
robot_model_id / compatible product-model range
manufacturing identity
installed option identifiers
authorized component-set / inventory-policy reference
provisioning / ownership identity references
instance_manifest_schema_version
```

`DeviceInventory` records actual installed boards, drives, actuators, sensors, safety controllers, bus addresses, serial numbers, and firmware/hardware revisions. Product-model actuator and device entries describe engineering requirements and logical roles; they must not be mistaken for the observed inventory of one robot.

`RobotInstanceManifest` is the provisioned, authorized identity/configuration for the physical robot and changes only through an explicit provisioning or service transaction, for example an installed-option or component-identity change. `DeviceInventory` is a revisioned L0 read-back/attestation of what is actually present; firmware OTA, device replacement, or bus-address change produces a new inventory revision. The Product Profile defines who may author/sign the instance manifest, who attests inventory, and which mismatches inhibit motion.

A runtime should expose both:

- what physical robot this is;
- what product model bundle is currently active;
- which inventory, calibration, control, and safety artifacts are active.

`robot_serial` and all other instance-specific fields are excluded from the shared `ProductModelManifest` canonical bytes and hash. Two conforming robots of the same product revision should resolve to the same product-model hash even though their serial numbers, installed-device identities, and calibrations differ.

## Topology model

The `ProductModelManifest` should distinguish:

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
- mechanical/kinematic design bounds (not deployed safety authority);
- home/reference position;
- control modes supported;
- logical control group.

### Actuators

- actuator ID;
- required device/drive class and logical role;
- motor parameters where needed;
- command/state capabilities;
- engineering thermal/electrical ratings;
- bus/device mapping kept in hardware variant and instance inventory rather than public semantics.

Actual device serial numbers, installed firmware, and observed bus addresses belong to the `DeviceInventory` referenced by `RobotInstanceManifest`, not to the shared actuator definition.

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
- mechanical design bounds and rating metadata that safety engineering may use as inputs.

Mechanical ratings are not automatically deployable safety limits. Safety-authoritative envelopes belong to a separately released `SafetyProfile`.

### CalibrationSet

Owned by the physical robot and tied to hardware identities. The validated set is applied as a serial-specific overlay during composition:

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
content checksum(s)
signature(s) and signer identity/role
```

Checksums and signatures are distinct fields with distinct semantics. A checksum identifies canonical content and detects accidental corruption; a signature authenticates an explicitly scoped checksum plus metadata under an approved key and signer role. A matching checksum alone conveys no release authority.

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

## Control, safety, and live capability artifacts

The product model describes what the design is. It must not become a catch-all for values with different change authority.

### ControlProfile

`ControlProfile` contains operational controller configuration such as:

```text
control_profile_id / schema_version
compatible product-model and hardware ranges
controller selection and gains
nominal rate / filter / estimator settings
ordinary command shaping and performance limits
required capabilities
content checksums
signatures where release policy requires them
```

Control profiles may be tuned and rolled out more frequently than the product model. Their limits can be more conservative than active safety limits, but they are not safety-authoritative and cannot relax a `SafetyProfile`.

### SafetyProfile

`SafetyProfile` contains safety-authoritative limits, required safety inputs/devices, derating rules, and fault-to-safe-behavior mappings. It has its own compatibility constraints and an independent review, signing, authorization, activation, audit, and rollback lifecycle.

It must not be embedded in a normal `ProductModelBundle`, controller update, policy bundle, or simulator overlay. Activating a new model or policy must leave the active `SafetyProfile` unchanged unless an explicitly authorized safety release is activated as a separate operation. The runtime and `robot-rt` must fail closed on a missing, incompatible, expired, or invalidly signed profile according to the product safety case.

Simulation can load a declared safety profile to exercise identical decision semantics. A simulation/scenario overlay may add stricter test constraints, but it cannot alter or relax the signed deployable profile while retaining the same profile identity/hash.

Those extra simulation restrictions form a separately identified `TestConstraintSet`, not a modified `SafetyProfile`. Logs and replay metadata record both hashes. At runtime, the effective envelope is the intersection of the active `SafetyProfile`, applicable `SA-0` through `SA-2` device/safety-controller constraints, and any more-conservative control or test constraint. Host software cannot use a profile activation to relax an independently configured lower-authority bound.

### RuntimeCapabilitySnapshot

`RuntimeCapabilitySnapshot` is an ephemeral, timestamped view of what is currently usable after composing product declarations, installed devices, calibration validity, active profiles, mode/authorization, faults, and degradation:

```text
snapshot_id / sequence / timestamp / boot_id
source artifact IDs and hashes
available / unavailable / degraded capabilities
supported command modes and current operational restrictions
component health and missing prerequisites
reason codes and validity horizon
```

It is output from the running system, not an input committed to the product model. A capability declared by the product is not proof that it is available on this robot at this moment.

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

## Artifact hashes, checksums, and signatures

Every product-model release should produce stable identifiers from canonicalized shared content:

```text
product_model_bundle_id
product_model_manifest_hash
joint_schema_hash
actuator_schema_hash
frame_schema_hash
sensor_schema_hash
```

The exact hash canonicalization must be defined by the model compiler, not by hashing arbitrary YAML formatting.

The shared product-model hash excludes `robot_serial`, installed device serials/firmware, calibration values, active control/safety profiles, and runtime health. Those artifacts carry their own IDs and hashes:

```text
robot_instance_manifest_hash
device_inventory_hash
calibration_set_hash
control_profile_hash
safety_profile_hash
runtime_capability_snapshot_id
```

The runtime can additionally compute a `composed_model_fingerprint` over an ordered, typed list of source artifact IDs/hashes. This fingerprint is useful for incident correlation and compatibility checks, but it does not merge their release or signing authority.

Keep integrity and authenticity metadata separate:

```text
content_checksums[]   # algorithm + canonicalization + digest
signatures[]          # signed scope/digest + key ID + signer role + signature
```

Checksums are deterministic content identities and corruption checks. Signatures prove authorization only under a configured trust policy. Repacking, transport checksums, and signatures over a release envelope must not be overloaded into a single ambiguous `checksum` or `signature/checksum` field.

These hashes are referenced by:

- runtime startup checks;
- PolicyBundle compatibility;
- MCAP/incident metadata;
- simulator conformance;
- SDK capability discovery;
- OTA ReleaseManifest.

Logs and OTA manifests should record the full source-artifact set, especially `safety_profile_hash`, rather than relying only on a combined model bundle ID.

## Policy compatibility

A policy cannot be considered compatible merely because tensor dimensions match.

PolicyBundle should identify:

```text
required robot/model family
required product-model hash/range
joint/action schema hash
observation schema hash
frame assumptions
required sensors/capabilities
policy/control rate
normalization/clipping
history layout
runtime/inference requirements
```

The runtime refuses incompatible product, schema, calibration, capability, and control-mode combinations before enabling control. Policy compatibility never authorizes a change to `SafetyProfile`, and tensor/schema compatibility is not a safety approval.

## Asset packaging and release boundaries

Meshes, textures, calibration schemas, and backend models need content-addressed/versioned packaging.

Suggested shared `ProductModelBundle` logical layout:

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

Per-robot calibration values are not stored in the shared product bundle; only calibration schemas and method compatibility may be included there. `RobotInstanceManifest`, `CalibrationSet`, `ControlProfile`, and `SafetyProfile` are separate signed or checksummed release/activation units with explicit compatibility references. `DeviceInventory` is a revisioned read-back/attestation produced by the running L0/device path, not an authored release unit; activation policy compares it with the provisioned instance and artifact requirements. A deployment package may transport several authored units together, but installation and activation remain separate state transitions, especially for `SafetyProfile`.

## Conformance tests

A model compiler is only credible if it tests generated backends.

Required cross-backend checks:

- joint count/order/name/id;
- positive direction;
- mechanical/kinematic design bounds;
- default/reference pose;
- frame transforms;
- mass/inertia sanity;
- actuator/transmission mapping;
- static gravity behavior;
- simple torque/position response;
- sensor frame/timestamp mapping;
- collision group expectations.

Do not require exact dynamics equality across engines; define tolerances and backend-specific expectations.

## Proposed V0 artifact set

Keep V0 small enough to implement:

### `ProductModelManifest`

```text
product identity
links/bodies
joints
actuators
simple transmissions
sensors
frames
mechanical ratings / nominal operational ranges
declared and required capabilities
asset references
calibration schema references
```

Advanced tendon/closed-loop semantics can be modeled through extensible transmission/constraint records while V0 supports the first reference embodiment.

### `RobotInstanceManifest`

```text
robot serial and product-model compatibility
installed options and authorized component-set / inventory policy
provisioning identity references
```

### `DeviceInventory`

```text
observed component IDs / serials / revisions
bus and logical-role mapping
firmware / bootloader inventory and attestation metadata
inventory sequence / timestamp / boot identity
```

### `CalibrationSet`

```text
robot and component identity binding
calibration scopes and values
method / evidence / quality metadata
lifecycle state
content checksums and signatures
```

### `ControlProfile`

```text
controller/rate/gain selection
ordinary control limits and shaping
compatibility and capability requirements
content checksums and release metadata
```

### `SafetyProfile`

```text
safety-authoritative limits
required safety inputs/devices
derating and safe-behavior mappings
independent compatibility, signing, activation, and rollback policy
```

### `RuntimeCapabilitySnapshot`

```text
source artifact IDs / hashes
current available/degraded/unavailable capabilities
current restrictions, health reasons, and validity horizon
```

The public composed `RobotManifest` view can be generated from these sources for SDK discovery; it is not an additional writable source of truth.

## ADR implications

This research supports decisions roughly equivalent to:

1. Soma owns a canonical shared `ProductModelManifest` / product model bundle.
2. URDF, MJCF, and USD are generated/validated target artifacts, not the complete source of truth.
3. Product model, robot instance/device inventory, calibration, control profile, `SafetyProfile`, and `RuntimeCapabilitySnapshot` are distinct artifacts with distinct lifecycles.
4. `robot_serial` and other instance state are excluded from the shared product-model hash.
5. `SafetyProfile` is independently governed and cannot be changed implicitly by a model, controller, policy, or simulator bundle.
6. Checksums/content hashes and authorization signatures are separate fields and validation steps.
7. Simulator tuning is an overlay, not hardware truth.
8. Policies and logs bind to the complete applicable artifact/schema identity set.

## Experiments required

1. Define a minimal manifest for a differential-drive reference robot.
2. Extend it to a legged robot with >10 DOF.
3. Generate/validate URDF + MJCF from the same source.
4. Create USD/Isaac mapping prototype.
5. Prove a `CalibrationSet` changes composed sensor/joint transforms without modifying nominal source.
6. Validate PolicyBundle mismatch rejection.
7. Prove two serial-numbered instances share one product-model hash while retaining different instance/calibration hashes.
8. Prove a product-model or control-profile rollout cannot change or relax the active `SafetyProfile`.
9. Verify checksum corruption detection and signature authorization as separate failure cases.

## Primary references

- MuJoCo model/actuator overview: https://mujoco.readthedocs.io/en/stable/overview.html
- MuJoCo MJCF XML reference: https://mujoco.readthedocs.io/en/stable/XMLreference.html
- OpenUSD Physics schema: https://openusd.org/release/api/usd_physics_page_front.html
- SDFormat pose/frame semantics: https://sdformat.org/tutorials/specification/pose_frame_semantics/
- Unitree MuJoCo: https://github.com/unitreerobotics/unitree_mujoco
- AgiBot Genie Sim: https://github.com/AgibotTech/genie_sim
