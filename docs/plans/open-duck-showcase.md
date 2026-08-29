# Open Duck Mini Simulation Showcase

> Plan status: LOCALLY VERIFIED - AWAITING USER REVIEW
> Control plane: `/root`

## Goal

Publish one deterministic 12-second Open Duck Mini composite rollout with the
MuJoCo video, complete animated MJCF robot, and policy evidence synchronized in
one Rerun recording.

## Scope

- Replace the user-facing repeated fixed walk runs with one composite policy
  rollout using the pinned policy's `vx`, `vy`, and yaw command inputs.
- Keep the frozen fixed walk acceptance as an internal correctness gate; do not
  render it as a second demo or metrics table.
- Record full generalized positions and velocities for read-only MJCF replay in
  Rerun alongside commands, root stability, contacts, joint targets, measured
  positions, and policy actions.
- Keep the existing Pages route and artifact names.

## Non-goals

- No checkpoint, public protocol, runtime/RT, Plant, Zenoh, hardware, or
  viewer-originated control changes.
- No generic choreography or robot visualization framework.

## Acceptance

1. The builder produces one 12-second video and one Rerun archive from one
   rollout; metadata has `run`, not repeated `runs`.
2. Rerun contains Open Duck Mesh3D geometry and base transforms through the end
   of the rollout, plus all three command streams and existing policy evidence.
3. The default Rerun Blueprint uses a fixed path overview that keeps the robot
   visible for the complete rollout.
4. The existing frozen reference acceptance and workspace checks remain green.
5. A browser review on the local network confirms video and Rerun readability.

## Stop Gates

Stop before changing the pinned policy contract, increasing commands beyond the
upstream demonstrated ranges, or changing authoritative runtime behavior.

## Local Review

- Showcase: `http://10.169.12.60:8004/`
- Rerun: `http://10.169.12.60:9101/?url=rerun%2Bhttp%3A%2F%2F10.169.12.60%3A9901%2Fproxy`
- Verified at the final `11.98s` sample with the complete robot visible and all
  telemetry panels synchronized on `sim_time`.
