# Official Simulation D0 Blocker

Status: `BLOCKED_NEEDS_LOCAL_VALIDATION`

The official Reachy Mini simulation comparison stopped at the D0 environment
gate on 2026-08-25. No official daemon was launched, no actuator command was
sent, and no Soma runtime or public protocol was changed.

## Pinned Official Environment

- repository: `https://github.com/pollen-robotics/reachy_mini.git`
- release: `v1.9.0`
- commit: `b7e686d994a178353ebf81ea935de82ce65af733`
- Python: `3.12.12`
- official MuJoCo dependency: `mujoco==3.3.0`
- documented launch: `reachy-mini-daemon --sim`

The release documentation states that the simulator presents the same local
daemon interface as a Reachy Mini Lite. That command surface was not treated as
proof of a nine-actuator shaft mapping because the daemon could not be launched
on this host.

## Reproduction

```bash
git clone --depth 1 --branch v1.9.0 \
  https://github.com/pollen-robotics/reachy_mini.git /tmp/reachy_mini
git -C /tmp/reachy_mini rev-parse HEAD
uv venv --python 3.12 /tmp/soma-official-v1.9.0
uv pip install --python /tmp/soma-official-v1.9.0/bin/python \
  '/tmp/reachy_mini[mujoco]'
```

The install fails while building the release's unconditional
`pygobject==3.46.0` dependency:

```text
Package gobject-introspection-1.0 was not found in the pkg-config search path.
No package 'gobject-introspection-1.0' found
help: pygobject (v3.46.0) was included because reachy-mini (v1.9.0)
      depends on pygobject
```

The host therefore cannot run the pinned official product path without an OS
dependency change. Installing system packages or bypassing the official
dependency graph is outside this D0 run and would weaken reproducibility.

## Capability Audit At Stop

| Field | Capability | Evidence |
| --- | --- | --- |
| Official release/commit | `COMMON` | Release and commit resolved from upstream Git. |
| Python version | `COMMON` | Isolated Python 3.12.12 environment created. |
| MuJoCo version | `COMMON` | Official `mujoco` extra pins 3.3.0. |
| Official launch command | `COMMON` | Upstream simulation guide specifies `reachy-mini-daemon --sim`. |
| Nine actuator order | `UNAVAILABLE` | Live daemon and SDK round trip did not run. |
| Shaft units and signs | `UNAVAILABLE` | Live mapping was not observed; no values were guessed. |
| Limits and initial pose | `UNAVAILABLE` | No live readiness state was captured. |
| Physics timestep/decimation | `UNAVAILABLE` | No running official backend was observed. |
| Warmup/readiness | `UNAVAILABLE` | Official launch did not reach readiness. |
| TTL/sequence/timeline/disposition | `SOMA_ONLY` | Not synthesized on the official side. |

## Resume Gate

Resume D0 only in an isolated host/container with the pinned release's native
GObject introspection prerequisites available. The next required proof remains
a live official daemon launch followed by a public SDK nine-actuator round trip
covering order, units, signs, initial state, cadence, and readiness. Do not
start D1 recorder or analyzer work before that proof passes.
