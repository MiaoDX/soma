# Rust and Real-Time Runtime

> Status: Deep Research. Goal: turn the current “Rust-first” preference into an explicit language/runtime boundary ADR.

## Question

Can Soma use Rust as the primary language from HAL through real-time control, robot runtime, SDK client, and selected MCU firmware without compromising deterministic timing, vendor integration, or long-term maintainability?

## Executive conclusion

Yes — with an important qualification:

> **Rust can be Soma's primary systems language, but real-time behavior comes from the operating-system and execution model, not from Rust itself.**

The recommended production boundary is:

- **Rust strongly preferred** for RT data types, HAL abstractions, Plant contract, safety logic, control orchestration, runtime services, protocol/client logic, tooling, and Python bindings.
- **C/C++ accepted behind narrow boundaries** for mature vendor SDKs, EtherCAT masters, GPU stacks, simulator APIs, and ROS 2 adapters.
- **`unsafe` isolated** in `*-sys`, FFI, memory-mapping, and carefully reviewed lock-free/RT primitives.
- **No stable Rust ABI is assumed** across dynamically loaded components; use static linking, C ABI, or process boundaries.

## Why Rust is attractive for Soma

### Memory and concurrency safety

Safe Rust prevents data races through ownership, borrowing, and `Send`/`Sync` rules. This removes a large class of failures that are particularly expensive in long-running robot processes with shared state, FFI, device drivers, and concurrent services.

Rust does **not** prevent all race conditions or scheduling bugs. Real-time ordering and deadline correctness still require explicit architecture and testing.

### Strong domain types

Robotics is full of accidental semantic collisions:

```text
MotorId vs JointId
Robot monotonic time vs UTC
position vs velocity vs torque
sensor frame vs body frame
requested vs accepted vs applied command
```

Rust makes it practical to encode these distinctions into types instead of relying on naming conventions around raw integers and floats.

### Explicit ownership at subsystem boundaries

Ownership is useful for expressing:

- exclusive access to a hardware device;
- transfer of command buffers;
- no-allocation fixed-capacity structures;
- lifecycle states;
- safe wrappers around FFI handles.

## What Rust does not solve

PREEMPT_RT reduces scheduling latency by making most kernel execution preemptible, using threaded interrupts and priority-inheritance-aware locking. A real-time user process must still control scheduling policy, CPU/IRQ placement, memory faults, blocking calls, and worst-case workload.

Soma's RT discipline should include:

```text
PREEMPT_RT kernel
SCHED_FIFO/SCHED_RR policy as validated
CPU affinity / isolation
IRQ affinity
mlockall + prefaulting
absolute-time scheduling
no allocation in cyclic path
no filesystem/network I/O in cyclic path
no blocking mutex in cyclic path
bounded algorithms
telemetry/logging outside RT thread
```

The RT benchmark must report p50/p99/p99.9/**max**, not only average latency.

## Recommended process model

```text
robot-rt                            robot-runtime
--------                            -------------
PREEMPT_RT                          normal Linux scheduling
fixed-capacity data                 Tokio allowed
HAL / Plant                         protocol/network
estimator/controller                lease/action engine
software safety                     recording/OTA/diagnostics
watchdog
      \________ bounded SHM ________/
```

### `robot-rt`

Recommended restrictions:

- no Tokio/async executor;
- no DDS/Zenoh/gRPC in the cyclic path;
- no Python;
- no ROS;
- fixed-size/fixed-capacity messages;
- preallocated buffers;
- no unbounded channels;
- explicit panic/fatal-error behavior;
- narrow FFI.

### `robot-runtime`

Rust async is a strong fit here:

- Zenoh / network communication;
- RPC/action lifecycle;
- authentication and leases;
- model/policy management;
- diagnostics and recording;
- OTA coordination;
- Python SDK client implementation.

## Panic policy

Rust supports unwind and abort panic strategies. Unwinding across the wrong FFI ABI can produce undefined behavior, and non-unwinding `extern "C"` boundaries should never allow foreign exceptions to cross into Rust.

Recommended policy:

### `robot-rt`

Prefer `panic = "abort"` for production builds.

Rationale:

- a panic indicates violation of an invariant that should not be recovered from inside the cyclic controller;
- deterministic process termination is easier to reason about than unwinding through RT/FFI resources;
- the supervising safety architecture must remain safe when `robot-rt` dies.

This implies the system must not rely on destructor execution for safety-critical shutdown. Drive watchdogs / safety controller / independent supervision own that responsibility.

### `robot-runtime`

Unwinding may remain acceptable where useful, but panics should be contained at task/process boundaries and converted into structured faults. Critical services should still avoid using panic as a normal error mechanism.

## Allocation policy

Rust's normal heap allocator is not inherently bounded for RT use.

### RT initialization phase

Allocation is acceptable before activation:

- discover/configure devices;
- load RobotManifest;
- construct controller objects;
- allocate telemetry rings;
- allocate working memory;
- prefault and lock memory.

### Active cyclic phase

Avoid allocation entirely. Prefer:

- arrays with compile-time or boot-time bounded capacities;
- `ArrayVec`/equivalent fixed-capacity containers after review;
- arenas allocated before activation;
- indexed tables instead of strings/maps;
- fixed event structures.

CI should include an allocator-instrumented RT test that fails if allocation occurs after `activate()`.

## `unsafe` policy

The goal is not “zero unsafe”; it is **small, reviewable unsafe islands**.

Suggested crates:

```text
vendor-foo-sys        raw bindings only
ethercat-sys          raw mature C master wrapper, if used
mujoco-sys            native ABI
robot-shm-sys         mmap/atomic layout primitives

robot-hal             safe API
robot-plant           safe API
robot-rt              safe by default
```

Rules:

1. `unsafe` must have a written safety contract.
2. Raw vendor types cannot escape `*-sys`/wrapper crates.
3. `#[repr(C)]` is mandatory for C ABI/shared-layout structures.
4. No Rust trait object crosses a C ABI or shared-memory ABI.
5. No panic/foreign exception crosses a non-unwinding FFI boundary.
6. Manual `Send`/`Sync` implementations require dedicated review/tests.

## ABI and plugin strategy

Rust does not provide a stable general-purpose ABI for trait objects across independently compiled versions.

Do not define a public plugin system based on:

```text
dlopen(plugin.so) -> Rust trait object
```

Preferred choices:

- compile-time/static Rust components for tightly controlled core code;
- **C ABI** for in-process third-party native plugins;
- **Robot Protocol / process isolation** for applications and large extension points.

This is also why ROS 2 and simulator adapters should usually be separate processes/libraries rather than injected into the RT core.

## Vendor and ecosystem reality

A production robot will encounter C/C++ ecosystems:

- EtherCAT masters and drive SDKs;
- CUDA/TensorRT/vendor inference engines;
- cameras/LiDARs;
- MuJoCo C API;
- ROS 2 `rclcpp` and plugin ecosystems.

Soma should optimize for **safe integration**, not ideological language purity.

## MCU Rust

Rust `no_std` is viable for selected MCUs and can be evaluated for:

- non-safety-critical peripheral controllers;
- board management;
- simple sensors/actuators;
- new hardware where the team controls BSP/toolchain.

Do not require Rust for all MCU firmware initially. Existing certified/vendor SDK ecosystems and team expertise may justify C/C++.

Safety-qualified Rust toolchains such as Ferrocene demonstrate that Rust can participate in ISO 26262 / IEC 61508 workflows, but tool qualification is not equivalent to certifying the complete robot software or runtime.

## Python boundary

Recommended:

```text
Python API
   |
PyO3 binding
   |
robot-client (Rust)
   |
Robot Protocol
```

This keeps reconnect, lease, timeout, protocol evolution, and transport behavior in one Rust implementation. Python remains an application/control client and never owns the hard real-time loop.

`abi3` should be evaluated for Python wheel compatibility across Python minor releases.

## Production coding profile proposal

A future Soma coding profile should define:

### Core/RT

- stable Rust toolchain pinned per release;
- deny warnings in CI;
- `unsafe_code` denied by default except approved crates/modules;
- clippy profile plus custom lints;
- panic abort in RT production profile;
- dependency allowlist for RT crates;
- no implicit runtime initialization;
- bounded collections only in cyclic state;
- reproducible Cargo.lock/toolchain.

### Verification

- Miri where applicable for unsafe abstractions;
- sanitizers on native/FFI test builds;
- loom or equivalent concurrency model tests for small primitives;
- fuzz protocol parsers and FFI translation;
- RT allocation detector;
- cyclictest + end-to-end servo-loop benchmark on target hardware;
- long-duration soak tests.

## ADR implications

The research supports an ADR roughly equivalent to:

> **Soma is Rust-first, not Rust-only.** The public/core architectural APIs are defined in Rust and language-neutral schemas. Unsafe/vendor code is isolated. Hard RT has a restricted execution profile. C ABI/process boundaries are used when ecosystem maturity outweighs language uniformity.

## Experiments required before freezing

1. `robot-rt` 1 kHz benchmark on target ARM/x86 with PREEMPT_RT.
2. Allocation detector after activation.
3. Rust SPSC SHM benchmark and crash/restart behavior.
4. EtherCrab vs selected C EtherCAT master on actual slave topology.
5. PyO3 client overhead and packaging matrix.
6. FFI error/panic/fault-injection tests.

## Primary references

- Linux PREEMPT_RT documentation: https://docs.kernel.org/core-api/real-time/
- Linux PREEMPT_RT theory: https://docs.kernel.org/core-api/real-time/theory.html
- Rust Reference — panic: https://doc.rust-lang.org/stable/reference/panic.html
- Rustonomicon — FFI: https://doc.rust-lang.org/nomicon/ffi.html
- Rustonomicon — data races: https://doc.rust-lang.org/nomicon/races.html
- Rustonomicon — Send/Sync: https://doc.rust-lang.org/nomicon/send-and-sync.html
- Ferrocene qualification plan: https://public-docs.ferrocene.dev/main/qualification/plan/index.html
- EtherCrab: https://github.com/ethercrab-rs/ethercrab
