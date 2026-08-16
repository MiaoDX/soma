# Robot Protocol and Data Model

> Status: Deep Research. Goal: define the semantic contract that can survive transport, language, ROS distro, and robot-embodiment changes.

## Question

What should Soma expose as its stable public robot contract, and how should identity, state, commands, actions, events, authority, capability discovery, errors, time, and schema evolution be modeled?

## Executive conclusion

Soma should not define its public API as “whatever topics the internal runtime happens to publish.” The stable boundary should be a **transport-neutral Robot Protocol** with a small number of explicit interaction semantics:

```text
Discovery / Identity / Capability
State Streams
Command Streams
RPC
Actions
Events / Faults
Lease / Authority
Bulk-data descriptors
```

The protocol should be generated from a language-neutral network schema, with **Protobuf currently the strongest default candidate** for public structured messages. DDS, Zenoh, gRPC, Python, C++, and ROS 2 should map to the same semantic model rather than becoming separate API definitions.

## Lessons from adjacent systems

### MAVLink: protocol semantics matter more than transport

MAVLink is useful as a design reference because it separates several concepts that robotics SDKs often blur together:

- heartbeat-based component discovery;
- system/component identity;
- capability discovery and metadata;
- data streams;
- acknowledged commands;
- long-running commands with progress and cancellation;
- structured events with sequence numbers;
- time synchronization;
- file/parameter/mission microservices.

Soma should not copy MAVLink's compact message format or seven-parameter commands, but it should adopt the principle that **different interaction semantics deserve different protocol behaviors**.

### DDS/XTypes: type evolution is a real architectural concern

DDS XTypes formalizes final/appendable/mutable type evolution, showing that compatibility semantics need to be explicit. The protocol should not assume that “new fields probably work.”

### Protobuf: conservative schema evolution

Protobuf provides strong cross-language tooling and clear binary compatibility rules. In particular:

- field numbers are wire identity and must never be reused;
- removed numbers/names should be reserved;
- unknown fields allow old readers to tolerate many additive changes;
- changing existing field numbers is unsafe.

Soma should add stricter project-level rules above Protobuf's wire compatibility.

## Identity hierarchy

Do not infer identity from topic names or network addresses.

Recommended hierarchy:

```text
organization_id / tenant_id   optional at robot-local layer
fleet_id                      optional locally
robot_id                      persistent product identity
session_id / boot_id          changes with boot/runtime session
component_id                  stable logical component identity
instance_id                   dynamic instance if multiple copies exist
```

For hardware inventory also distinguish:

```text
robot_model_id
hardware_revision
mainboard_serial
actuator/sensor device identities
model_bundle_id
calibration_id
release_id
```

`robot_id` must remain independent of IP address, hostname, DDS participant ID, or Zenoh key expression.

## Capability discovery

Clients should not infer features only from version numbers.

Conceptually:

```proto
message RobotCapabilities {
  RobotIdentity identity = 1;
  repeated Capability capabilities = 2;
  repeated ControlDomain control_domains = 3;
  ProtocolCompatibility protocol = 4;
  repeated InterfaceProfile profiles = 5;
}
```

A capability should identify:

- stable capability ID;
- version/revision;
- supported modes/actions;
- limits/rates;
- required authority;
- optional schema/metadata URI or hash.

This is similar in spirit to MAVLink component information/capability metadata: a generic client should learn what the connected machine supports without hard-coding every model.

## State streams

State is sampled information, not an acknowledgement channel.

Every important state envelope should carry enough provenance to reason about freshness and discontinuity:

```proto
message SampleHeader {
  uint64 epoch_id = 1;
  uint64 sequence = 2;
  TimePoint capture_time = 3;
  TimePoint publish_time = 4;
  string source_component = 5;
  uint64 schema_revision = 6;
}
```

High-frequency RT-internal state may use fixed layouts instead of Protobuf, but public semantics must remain equivalent.

State-stream API needs an explicit delivery policy such as:

- `LATEST` — overwrite/drop old samples; ideal for joint state/UI;
- `BOUNDED_RELIABLE` — preserve order up to capacity;
- `SAMPLED` — rate-limit from a higher-rate source;
- bulk stream with external/shared payload descriptor.

Avoid an implicit unbounded queue anywhere in the SDK.

## Commands

A command is **intent** and must be distinguishable from what the robot actually applied.

```proto
message CommandEnvelope {
  uint64 epoch_id = 1;
  uint64 sequence = 2;
  LeaseToken lease = 3;
  TimePoint created_at = 4;
  optional TimePoint target_apply_time = 5;
  TimePoint valid_until = 6;
  string source_id = 7;
  CommandPayload payload = 8;
}
```

The runtime validates:

- epoch;
- lease generation and domain;
- mode;
- command age/deadline;
- capability;
- shape/units/limits;
- safety policy.

### Requested / Accepted / Applied

Soma should explicitly model three values:

```text
RequestedCommand  -> what client asked for
AcceptedCommand   -> after protocol/mode/safety admission
AppliedCommand    -> what reached the Plant / controller
```

When safety modifies output, tooling should be able to explain the difference.

## Lease and authority

Command-source arbitration must be a first-class protocol concept.

```text
Control domains:
  locomotion
  whole_body
  left_arm
  right_arm
  left_hand
  right_hand
  head
  navigation
  payload:<id>
```

A lease token should contain at least:

```text
lease_id
generation
robot/session identity
domain
holder/source
issued_at
expires_at
priority/policy if applicable
```

`generation` prevents a stale client from reusing an older logical lease after authority has been revoked and reacquired.

Safety and local emergency control always outrank public leases.

## RPC

Use RPC for short, bounded transactions:

```text
GetIdentity
GetCapabilities
GetHealth
GetConfiguration
SetConfiguration
StartRecording
GetModelManifest
GetReleaseInfo
```

RPC responses should return structured status codes; no user should need to parse log strings to understand an API failure.

## Actions

Actions represent long-running state machines:

```text
Stand
Sit
Calibrate
Home
FollowTrajectory
NavigateTo
ExecuteSkill
Dock
```

An Action instance needs:

```text
action_id
request/result type
state
progress
cancel/preempt semantics
timeout/deadline
lease relationship
terminal error/status
```

Suggested lifecycle:

```text
SUBMITTED -> ACCEPTED -> RUNNING -> SUCCEEDED
                         |  |  |
                         |  |  +-> FAILED
                         |  +----> CANCELLED
                         +-------> PREEMPTED
```

MAVLink's command acknowledgement demonstrates the usefulness of explicit `IN_PROGRESS`, result, progress, and cancellation semantics; Soma should use richer typed payloads.

## Events and faults

Events are ordered asynchronous facts, not log lines.

```proto
message EventEnvelope {
  uint64 epoch_id = 1;
  uint64 sequence = 2;
  TimePoint occurrence_time = 3;
  Severity severity = 4;
  string event_code = 5;
  string component_id = 6;
  map<string, TypedValue> arguments = 7;
  Correlation correlation = 8;
}
```

A receiving client can detect missed event sequence ranges and request/recover history if the profile supports it, similar to event-sequence concepts used in MAVLink.

A `Fault` is a domain object with lifecycle, not merely severity=error. Recommended state:

```text
fault_id
fault_code
active/latching
first_seen / last_seen
count
recoverability
component
probable cause
recommended action
related event/update/mission/action IDs
```

## Error model

Separate:

1. **Transport errors** — disconnected, timeout, authentication failure.
2. **Protocol errors** — incompatible schema/profile, malformed request.
3. **Admission errors** — stale epoch, no lease, wrong mode, deadline expired.
4. **Robot/domain errors** — actuator fault, path blocked, calibration failed.
5. **Safety outcomes** — command rejected/modified/stopped.

Use stable machine-readable codes plus human-readable detail.

Never make client logic depend on exact English error strings.

## Units and frames

The protocol must standardize:

- SI units by default;
- angle unit radians unless a domain explicitly says otherwise;
- quaternion order;
- axis handedness;
- joint positive direction;
- force/torque frame;
- covariance semantics;
- invalid/unknown value representation;
- frame identity and frame graph rules.

Frame names should come from the RobotManifest, not be invented by individual applications.

## Versioning model

Avoid a single global `sdk_version == robot_version` requirement.

Track independently:

```text
protocol major/minor
capability versions
RobotManifest/model bundle
runtime release
firmware inventory
client SDK version
```

### Proposed compatibility policy

- **Protocol major**: intentional breaking semantic change; explicit negotiation failure across unsupported majors.
- **Protocol minor**: additive/backward-compatible semantics under project rules.
- Capabilities declare their own version/profile.
- Clients must ignore unknown additive fields/capabilities where safe.
- Removed Protobuf fields are reserved permanently.
- Enums need an `UNSPECIFIED/UNKNOWN` path and clients must tolerate unknown numeric values.

## Schema source of truth

Recommended:

```text
proto/                  public network schema
rt-types/               fixed RT data contract
adapters/dds/           generated/mapped DDS types where needed
adapters/ros2/          ROS messages/actions mapping
bindings/python/        generated + ergonomic API
```

Do **not** make ROS messages or DDS IDL the universal source of truth.

## Transport mapping

The Robot Protocol should define logical names independently of transport.

Example:

```text
logical: robot.state.joints
Zenoh:   soma/<robot-id>/state/joints
DDS:     mapped Topic + type
ROS2:    /soma/joint_states or standard adapter mapping
gRPC:    StateService.StreamJointState
```

This avoids leaking Zenoh key-expression syntax into the public API.

## Bulk data

Images, point clouds, tensors, large maps, and artifacts should use descriptors plus a data plane appropriate to topology:

```text
BlobDescriptor
  content_type
  encoding
  size
  digest
  timestamp
  local_shm_handle OR stream/object URI
```

Do not serialize multi-megabyte payloads into every control envelope by default.

## Proposed V0 protocol surface

Freeze only a narrow first version:

```text
Identity / Capability
Health / Fault events
JointState / ImuState
ModeState
Acquire/Renew/Release Lease
Base velocity or generic locomotion command
Joint command profile for research mode
Stand/Stop Action
Recording control
RobotManifest metadata query
```

Navigation/manipulation/skill APIs can evolve after V0 semantics prove stable.

## ADR implications

This research supports decisions along these lines:

1. Public protocol is ROS-independent and transport-neutral.
2. Protobuf is the default public structured schema candidate.
3. State, command, RPC, action, event, and lease are distinct semantics.
4. Command validity includes epoch/time/authority.
5. Capability discovery is preferred over model-specific API assumptions.
6. Requested/accepted/applied commands are observable separately.

## Experiments required

1. Define V0 `.proto` and generate Rust/Python/C++ clients.
2. Map the same contract to Zenoh and gRPC.
3. Build ROS 2 adapter for JointState + one Action without changing core schema.
4. Compatibility CI: old client/new robot and new client/old robot.
5. Fuzz decode/unknown-field/unknown-enum cases.
6. Stale epoch, expired command, lost lease, duplicate sequence tests.

## Primary references

- Protocol Buffers language/evolution guide: https://protobuf.dev/programming-guides/editions/
- Protocol Buffers proto3 guide: https://protobuf.dev/programming-guides/proto3/
- OMG DDS XTypes: https://www.omg.org/spec/DDS-XTypes/
- MAVLink Guide: https://mavlink.io/en/
- MAVLink microservices: https://mavlink.io/en/services/
- MAVLink Command Protocol: https://mavlink.io/en/services/command.html
- MAVLink Component Metadata: https://mavlink.io/en/services/component_metadata.html
- MAVLink Time Synchronization: https://mavlink.io/en/services/timesync.html
- Unitree SDK2: https://github.com/unitreerobotics/unitree_sdk2
