# Periodic Bottom-Up Robot Stack Research

Purpose: regularly compare Soma's real implementation with strong community
practice while building a bottom-up robot software stack. This is an evidence
and learning loop, not a backlog generator and not permission to widen the
current product scope.

## Research Directions

Each cycle should cover a small number of these directions, chosen by current
Soma risk or an upcoming decision:

1. **Hardware and L0/L1:** actuator buses, device drivers, sensor timing,
   calibration, control rates, watchdogs and independent safety paths.
2. **Plant and simulation:** model identity, MuJoCo/Isaac/MJX topology,
   sim-to-real assumptions, HIL/SIL, replay and conformance evidence.
3. **Runtime and IPC:** process ownership, scheduling, bounded mailboxes,
   lifecycle, leases, command arbitration, health and failure containment.
4. **Policy and data:** observation/action contracts, model metadata, policy
   hosting, teleoperation recording, datasets, training/export and deployment.
5. **Product operations:** provisioning, update/rollback, recovery, logging,
   fleet management, remote access and degraded/offline behavior.
6. **Application and community layer:** SDKs, modules, missions, visualization,
   media, documentation, BOM, assembly and contribution workflows.

## Cadence

Run a lightweight scan monthly or before a material architecture decision. Do
one deeper case study each quarter. Revisit existing references when their
default branch, release, hardware or documented architecture changes materially.

Prefer a balanced sample: one hardware-first project, one control/runtime
project, one learning/data project and one product/application stack when the
cycle permits. Keep Open Duck Mini/Playground and MicroDuck explicitly separate.

## Method

1. Start from the current Soma status, active plan and relevant architecture
   seam; state the decision the research should inform.
2. Search for first-party repositories and design documents. Pin repository,
   branch/release and commit for claims that affect compatibility.
3. Inspect implementation, not only README marketing: process graph, control
   loop, I/O ownership, schemas, rates, failure behavior, tests and licenses.
4. Compare the reference with Soma's actual code and proof, marking each claim
   as `verified`, `documented`, `tentative` or `unknown`.
5. Record both positive lessons and counterexamples. Ask what the reference
   makes simpler, what it leaves unsafe, and what assumptions do not transfer.
6. End with explicit Soma implications, non-implications, gaps and a stop
   decision. A reference does not become scope merely because it is interesting.

## Required Output

For each promoted project, keep a short case study in this directory with:

- source and commit/release pins;
- hardware and software shape;
- relevant interfaces, rates and ownership;
- strengths, failure modes and license/artifact caveats;
- comparison with Soma's current implementation;
- lessons to borrow, things not to copy, and open questions.

Update the [research index](README.md). Update architecture, decisions or plans
only when a conclusion is accepted through the normal project workflow.

## Guardrails

- Do not turn a repository list into a generic robot framework.
- Do not infer safety, real-time guarantees or physical transfer from a
  simulator demo, README claim or ONNX checkpoint.
- Do not place Python, middleware, network I/O, blocking I/O or unbounded work
  in Soma's periodic `robot-rt` path based on a community pattern.
- Preserve source lineage, model identity, calibration, license and evidence
  boundaries; call out contradictions instead of silently normalizing them.
- Stop when another search pass is unlikely to change the decision. Record
  excluded candidates and missing evidence briefly rather than expanding scope.

## Current Starting Set

The existing catalog covers Reachy Mini, Open Duck Mini/Playground, MicroDuck,
Viam, RoboParty, TARS-AI, Pupper, Mini Pupper, Poppy, BotBrain, LeRobot,
Copper, Eclipse S-CORE, fieldbus/HAL projects, middleware and OTA tooling.
Future cycles should deepen or challenge this set before adding many more names.
