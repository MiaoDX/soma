from __future__ import annotations

import argparse
import queue
import time
from pathlib import Path

import zenoh

from soma_client import soma_pb2

from .common import (
    ACTUATORS,
    DWELL_S,
    SAMPLE_PERIOD_S,
    TRACE_DELTAS_RAD,
    WARMUP_S,
    manifest,
    monotonic_ns,
    sample,
    write_jsonl,
)

COMMAND_KEY = "soma/reachy/command"
STATE_KEY = "soma/reachy/state"


def receive(states: queue.Queue, timeout: float = 5.0):
    item = states.get(timeout=timeout)
    return soma_pb2.ActuatorState.FromString(bytes(item.payload))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    config = zenoh.Config.from_json5(
        '{mode:"client",connect:{endpoints:["tcp/127.0.0.1:7447"]},'
        'scouting:{multicast:{enabled:false}}}'
    )
    with zenoh.open(config) as session:
        states: queue.Queue = queue.Queue()
        with session.declare_subscriber(STATE_KEY, states.put):
            initial_state = receive(states)
            if len(initial_state.positions_rad) != len(ACTUATORS):
                raise RuntimeError("Soma did not publish the fixed nine-actuator profile")
            time.sleep(WARMUP_S)
            while not states.empty():
                initial_state = receive(states, 0.1)
            initial = list(initial_state.positions_rad)
            target = [value + delta for value, delta in zip(initial, TRACE_DELTAS_RAD)]
            sent_ns = monotonic_ns()
            request = soma_pb2.RtRequest(
                target=soma_pb2.ActuatorTarget(
                    positions_rad=target,
                    sequence=initial_state.sequence + 100,
                    timeline=initial_state.timeline,
                    ttl_ns=int((DWELL_S + 1.0) * 1_000_000_000),
                )
            )
            session.put(COMMAND_KEY, request.SerializeToString())
            records = [
                manifest(
                    "soma",
                    {
                        "source": "crates/soma-sim/assets/reachy-mini",
                        "upstream_commit": "20bc9eedc81ddc552235d222ca7e39205b2c2481",
                        "mujoco": "3.9.0",
                    },
                    initial,
                )
            ]
            deadline = time.monotonic() + DWELL_S
            while time.monotonic() < deadline:
                state = receive(states)
                records.append(
                    sample(
                        "soma",
                        monotonic_ns(),
                        sent_ns,
                        target,
                        list(state.positions_rad),
                        sequence=state.sequence,
                        timeline=state.timeline,
                        capture_monotonic_ns=state.capture_monotonic_ns,
                        command_disposition=state.command_disposition,
                    )
                )
            write_jsonl(args.output, records)


if __name__ == "__main__":
    main()

