# Security Threat Model

> Status: Architecture constraint. This document defines threats, trust boundaries, and invariants. It does not choose authentication protocols, key stores, rate limits, certification targets, or a complete security test program.

## Purpose

Soma controls physical motion and installs artifacts that can change future behavior. Security therefore protects more than data confidentiality: it protects command authority, safety configuration, software identity, recovery, and the evidence needed to explain an incident.

The model is intentionally high level. Detailed mechanisms require ADRs and implementation evidence after the V0 runtime and target platform are known.

## Protected assets

- motion authority, leases, modes, and lifecycle state;
- `SafetyProfile` and lower-authority safety configuration;
- device identity, signing roles, update and recovery authority;
- product model, calibration, controller, policy, and release identity;
- requested-to-applied command evidence and incident recordings;
- developer, maintenance, provisioning, and recovery privileges;
- service availability needed for local control and safe degradation.

## Threat sources

- an unauthenticated, unauthorized, stale, or replaying client;
- a compromised L4 application, ROS bridge, developer laptop, or fleet service;
- malformed or excessive traffic that exhausts runtime resources;
- a compromised `robot-runtime` process attempting to bypass `robot-rt` or lower safety authority;
- an altered, incompatible, revoked, expired, or rollback-disallowed artifact;
- accidental operator or developer action under excessive privilege;
- physical access to debug, storage, buses, or recovery interfaces;
- corrupted, incomplete, or misleading evidence after a crash or update.

## Trust boundaries

The following diagram and invariants describe a qualified deployment profile.
M1's deliberately narrower exception is defined in "M1 development profile"
below and cannot be promoted beyond loopback.

```text
untrusted / partially trusted

L5 fleet, developer tools, L4 apps, ROS, Python
                       |
                       | authenticated Robot Protocol
                       v
                 robot-runtime (SA-4)
                       |
                       | bounded admitted commands
                       | generation / lease / deadline / timeline
                       v
                    robot-rt (SA-3)
                       |
                       | bounded Plant contract
                       v
                 L0 Plant / HAL / devices
                       |
                       v
            L-1 and independent SA-0..SA-2

separate trust path:
release authority -> verifier -> candidate activation -> known-good state
```

The boundary between `robot-runtime` and `robot-rt` is both a timing boundary and a trust boundary. Authentication at L2 cannot replace validation at L1, and compromise of either Linux process cannot be treated as defeating independent lower safety mechanisms by design.

## Security invariants

| Boundary | Invariant |
| --- | --- |
| Client to runtime | Outside `insecure-local-dev`, no client commands or acquires a protected resource without authenticated, authorized, current authority. |
| Lease and lifecycle | Old leases, generations, timelines, and expired commands never regain authority after restart, reset, revocation, or reassignment. |
| Runtime to RT | `robot-rt` accepts only bounded, version-compatible messages and independently validates freshness, mode, state, and `SA-3` constraints. |
| Resource exhaustion | Network, middleware, SDK, diagnostics, recording, and fleet load cannot block the periodic RT path or renew command validity through queueing. |
| Safety configuration | Product-model, controller, policy, simulator, or ordinary release activation cannot implicitly replace or relax `SafetyProfile` or lower-authority bounds. |
| Artifact activation | Identity-incompatible, altered, unauthorized, or rollback-disallowed artifacts do not replace the last known-good active state. |
| Developer mode | Elevated modes are explicit, scoped, observable, revocable, and do not silently survive a lifecycle transition that requires reauthorization. |
| Simulation separation | Simulation-only control and ground truth cannot be exercised through a physical endpoint or mistaken for deployable capability. |
| Evidence | Security, authority, safety, update, and command decisions remain attributable; missing or lossy evidence is reported rather than presented as complete. |
| Cloud independence | Fleet or cloud unavailability cannot disable required local safety behavior. |

## Artifact and role separation

A single release package may transport several artifacts, but transport does not merge authority:

```text
ReleaseManifest         integration identity and qualification state
SafetyProfile           independent safety authorization
ProductModelManifest    shared product semantics
CalibrationSet          serial-scoped measured state
PolicyBundle            application/policy compatibility
device firmware         device-specific update and recovery authority
```

The concrete key hierarchy, secure-boot chain, TPM/TEE use, developer credentials, rotation, revocation, and recovery ceremony are target-dependent ADRs.

## M1 development profile

M1 is explicitly `insecure-local-dev`: its public endpoint binds only to
loopback and implements no authentication, TLS, signing, trust store, secure
boot, anti-rollback, key rotation or revocation. The current schema carries no
`source_id`; if attribution is added before authentication, it must be presented
as evidence only and never as authenticated identity.

This profile cannot be used for non-local control, external distribution,
physical actuation or OTA. Any of those triggers reopens the relevant threat
analysis and requires concrete trust and cryptographic mechanisms before the
capability is enabled.

## Validation posture

M1 validates only that the loopback-only development profile cannot be
mistaken for a qualified deployment profile. Cryptographic verifier tests,
protocol security abuse cases, fuzzing, penetration testing, long-duration
denial-of-service testing, production key ceremonies, and physical
debug-interface validation are deferred until a non-local, distribution,
physical-actuation or OTA trigger exists.

Deferral does not weaken the long-term invariants above. It limits what M1 may
claim: architecture reviewed, local development only, no security mechanism
implemented or qualified.

## Out of scope

- claiming compliance with IEC 62443, ISO 21434, ISO 27001, or a functional-safety standard;
- selecting a universal identity provider or cryptographic suite;
- treating host authentication as a substitute for physical safety;
- publishing sensitive key custody, recovery material, or detailed production attack paths in ordinary SDK metadata.
