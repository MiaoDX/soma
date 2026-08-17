# Middleware and IPC

> Status: Deep Research baseline. Soma has not yet selected its production distributed middleware.

## Question

How should Soma move data between the RT core, local services, robot-local distributed components, external SDK clients, ROS 2, and fleet/cloud systems without turning one middleware into a universal dependency?

## Working thesis

There is no single transport that is optimal for all robot data paths. Soma should define **communication semantics first** and use multiple bounded data planes:

```text
Hard RT calls            -> direct memory / fixed Rust structs
RT <-> runtime           -> fixed SPSC shared memory
Local large payload      -> shared-memory blob pool / iceoryx2 candidate
Robot distributed plane  -> Zenoh candidate
Enterprise/control plane -> gRPC/HTTP where appropriate
DDS/ROS ecosystem        -> optional Cyclone DDS / ROS 2 gateway
```

The stable boundary is the **Robot Protocol**, not the transport implementation.

## Why not put DDS/Zenoh in the servo loop?

A 500 Hz–2 kHz controller needs bounded execution and explicit overload behavior. General-purpose discovery, serialization, allocation, retries, sockets, async runtimes, and subscriber queues create timing behavior that is useful above RT but unnecessary inside it.

Therefore:

```text
EtherCAT -> Plant -> Controller -> Safety -> EtherCAT
                         |
                  bounded RT IPC
                         |
                    Robot Runtime
                         |
                Zenoh/DDS/gRPC/etc.
```

A network or middleware failure must not stop the local servo/safety loop.

## RT/runtime shared memory

This IPC contract is small enough to define explicitly rather than beginning with a general pub/sub API. The implementation remains an experiment: compare a minimal SPSC mailbox with a mature community shared-memory primitive, then choose the smallest option that satisfies the same behavior and measurements.

Recommended logical channels:

- command mailbox/ring: runtime -> RT;
- state ring: RT -> runtime;
- event ring: RT -> runtime;
- low-rate configuration handshake outside the cyclic path.

Required properties:

- fixed capacity;
- no heap allocation in the cyclic path;
- SPSC where possible;
- sequence, Plant-timeline, and producer/consumer generation identifiers;
- cache-line aware layout;
- explicit overwrite/drop policy;
- command TTL/deadline;
- ABI/version handshake;
- observable overflow and age.

The IPC is an **internal implementation ABI**, not a public SDK contract. This allows it to evolve more aggressively than the Robot Protocol.

Each shared region begins with a non-payload handshake header containing at least `magic`, layout/ABI hash, release ID, producer generation, consumer generation, Plant timeline ID, capacity, and monotonic overflow/drop counters. Both sides validate the header before motion enable, publish their generation with defined acquire/release memory ordering, and discard mailboxes from a previous generation. A restart never interprets residual bytes as a fresh command.

The baseline OTA unit upgrades `robot-rt`, `robot-runtime`, and their IPC ABI atomically; rolling interoperability across different internal ABI hashes is not promised unless a separately tested compatibility profile says otherwise. Generation mismatch, corrupt header, or unsupported ABI leaves the system motion-inhibited and requires the explicit lifecycle/re-arm path.

## Large local payloads

Images, depth maps, point clouds, audio, and large tensors should not be repeatedly serialized through the same path as small control/state messages.

A useful model is:

```text
Public message
  BlobDescriptor {
    pool/region
    offset
    length
    format
    shape
    strides
    capture_time
    lifetime token
  }

          |
          +---- shared memory blob pool ----> payload bytes
```

**iceoryx2** is a strong candidate to evaluate because it is Rust-oriented and targets zero-copy inter-process communication. Soma should still benchmark it against a smaller internal blob pool before making it foundational.

## Zenoh

Zenoh is currently the preferred distributed-plane candidate because it aligns with several Soma goals:

- strong Rust API;
- pub/sub plus query/reply patterns;
- peer/client/router topologies;
- local network and routed/WAN use;
- better fit for robot-to-laptop/fleet scenarios than assuming multicast discovery everywhere;
- ROS 2/DDS bridging is available when needed.

Potential roles:

```text
robot-runtime <-> local non-RT services
robot <-> developer laptop
robot <-> site gateway
site gateway <-> fleet
```

Questions that require benchmarks rather than assumption:

- latency distribution for small state messages;
- behavior under Wi-Fi loss and reconnection;
- memory/CPU footprint on target ARM compute;
- discovery/router behavior across customer networks;
- QoS semantics needed by the Robot Protocol;
- security/key lifecycle;
- large subscriber counts;
- backpressure and slow-consumer behavior.

## Cyclone DDS

Cyclone DDS remains strategically important even if it is not Soma's default runtime transport.

Reasons:

- mature DDS semantics and QoS;
- direct compatibility with DDS-centric robotics ecosystems;
- natural ROS 2 interoperability;
- useful reference for Unitree-style SDK behavior;
- standardized concepts such as reliability, history, deadline, liveliness, and type evolution.

Risks for Soma:

- the core implementation is C, so a Rust-first product needs a controlled FFI/gateway boundary;
- DDS discovery/topology can be awkward in Wi-Fi, routed, or customer networks;
- making DDS IDL the canonical schema could accidentally couple the entire architecture to DDS/ROS assumptions.

Recommended Cyclone DDS role while Zenoh is the provisional default:

```text
Robot Protocol
     |
Cyclone DDS Gateway
     |
DDS / Unitree-style / ROS-adjacent clients
```

## gRPC

gRPC is excellent for explicit service boundaries and should not be rejected simply because it is not the primary state stream.

Good fits:

- configuration;
- identity/capability queries;
- artifact/model transfer;
- management APIs;
- high-level task requests;
- enterprise integration;
- simulator management APIs.

Poor fits:

- hard real-time control;
- high-rate many-to-many state distribution;
- local zero-copy image/point-cloud delivery.

## Robot Protocol semantics

The public protocol should distinguish communication intent rather than exposing arbitrary topics.

### State / stream

Continuous information with explicit QoS/backpressure semantics.

Examples: joint state, IMU, odometry, health, sensor streams.

### Command stream

Continuous control intent. Commands require authority, Plant timeline, sequence, and robot-local validity/deadline.

### RPC

Short request/response operations such as identity, capabilities, configuration, or diagnostics.

### Action

Long-running, cancellable operations with progress and terminal result: stand, calibrate, navigate, follow trajectory, execute skill.

### Event

Asynchronous lifecycle/fault events: fault raised/cleared, lease lost, runtime restarted, safety intervention.

### Lease

Explicit control authority over domains such as locomotion, left arm, right arm, whole body, navigation, hand, or head.

## Schema strategy

Do not require one schema representation for every layer.

1. **RT schema** — fixed layout, fixed capacity, `#[repr(C)]`, internal.
2. **Public network schema** — evolvable, language-neutral; Protobuf is a strong current candidate.
3. **Bulk payload schema** — descriptors/handles plus out-of-band bytes.

The public schema must define compatibility rules from day one:

- never reuse removed field numbers;
- explicit major/minor protocol compatibility;
- capability discovery rather than version-only feature assumptions;
- unknown-field tolerance;
- stable enum/error semantics;
- model/schema hashes where ordering matters.

## Python SDK implication

The preferred architecture remains:

```text
Python API
   |
PyO3
   |
robot-client (Rust)
   |
Robot Protocol
   |
Zenoh / other transport
```

Connection, retry, lease, timeout, schema negotiation, and error mapping should be implemented once in Rust. Python should be async-first, with a sync facade as a convenience layer.

Python is never responsible for hard real-time hold behavior.

## ROS 2 implication

ROS 2 is an ecosystem adapter:

```text
Nav2 / MoveIt / ROS apps
          |
    robot-ros2-bridge
          |
      robot-client
          |
     Robot Protocol
```

This keeps Humble/Jazzy/Kilted and `rclcpp`/RMW dependencies outside the robot core. ROS 2 packages may be released per distro or containerized independently.

A `ros2_control` adapter should use the same authority/lease path rather than bypassing the runtime to talk directly to EtherCAT.

## Triggered benchmark plan

V0 adopts Zenoh provisionally rather than blocking implementation on a broad middleware bake-off. Run the following comparison only if Zenoh misses its declared Performance Envelope, creates an operational problem, or native DDS interoperability becomes a product requirement. When triggered, use a repeatable benchmark across:

### Topologies

- same process baseline;
- same host, separate processes;
- 1 GbE robot LAN;
- Wi-Fi;
- routed site network;
- optional WAN/site gateway.

### Payloads

- 64–256 B control/state messages;
- 1–8 KiB structured messages;
- image/point-cloud descriptors;
- representative large payload path.

### Load

- 1 publisher / 1 subscriber;
- many subscribers;
- multiple components;
- discovery storms/restarts;
- slow consumer;
- packet loss and reconnection.

### Metrics

- p50/p95/p99/p99.9/max latency;
- jitter;
- throughput;
- CPU and RSS;
- allocations where measurable;
- packet loss/recovery;
- startup/discovery time;
- reconnect time;
- queue/backpressure behavior.

The distributed comparison is limited to Zenoh and Cyclone DDS. gRPC is measured only for its intended service cases, and iceoryx2/local shared memory only for same-host payload paths; neither expands the distributed transport contest.

## Open questions

1. Should the public protocol bind directly to Zenoh key expressions or keep a transport-neutral logical namespace?
2. Protobuf vs another schema format for high-rate public structured state.
3. Whether the first product needs DDS compatibility at launch or can add it later.
4. Authentication model for local developers vs production fleet.
5. Whether local large tensors require GPU IPC in addition to CPU shared memory.
6. How to expose transport QoS without leaking middleware-specific concepts into the SDK.
7. How rolling runtime upgrades negotiate shared-memory ABI changes.

## Reference projects and sources

- Eclipse Zenoh — https://zenoh.io/
- zenoh-plugin-ros2dds — https://github.com/eclipse-zenoh/zenoh-plugin-ros2dds
- Eclipse Cyclone DDS — https://github.com/eclipse-cyclonedds/cyclonedds
- iceoryx2 — https://github.com/eclipse-iceoryx/iceoryx2
- gRPC — https://grpc.io/
- Protocol Buffers compatibility guidance — https://protobuf.dev/programming-guides/proto3/
- Unitree SDK2 — https://github.com/unitreerobotics/unitree_sdk2
