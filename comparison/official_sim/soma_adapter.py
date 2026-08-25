from __future__ import annotations

import argparse
import queue
import time
from pathlib import Path

import zenoh

from soma_client import soma_pb2

from .common import (
    ACTUATORS,
    load_suite,
    manifest,
    monotonic_ns,
    sample,
    select_case,
    validate_movement,
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
    parser.add_argument("--suite", type=Path, required=True)
    parser.add_argument("--case", required=True)
    args = parser.parse_args()
    suite = load_suite(args.suite)
    case = select_case(suite, args.case)
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
            warmup_started = time.monotonic()
            time.sleep(suite["warmup_s"])
            warmup_actual_s = time.monotonic() - warmup_started
            while not states.empty():
                initial_state = receive(states, 0.1)
            initial = list(initial_state.positions_rad)
            target = [value + delta for value, delta in zip(initial, case["deltas_rad"])]
            sent_ns = monotonic_ns()
            request = soma_pb2.RtRequest(
                target=soma_pb2.ActuatorTarget(
                    positions_rad=target,
                    sequence=initial_state.sequence + 100,
                    timeline=initial_state.timeline,
                    ttl_ns=int((suite["dwell_s"] + 1.0) * 1_000_000_000),
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
                    initial, suite, case, warmup_actual_s, sent_ns,
                )
            ]
            period_ns = int(suite["sample_period_s"] * 1_000_000_000)
            for planned_index in range(suite["sample_count"]):
                deadline_ns = sent_ns + (planned_index + 1) * period_ns
                latest = None
                while monotonic_ns() < deadline_ns:
                    timeout = max(0.001, (deadline_ns - monotonic_ns()) / 1_000_000_000)
                    try:
                        latest = receive(states, timeout)
                    except queue.Empty:
                        break
                while not states.empty():
                    latest = receive(states, 0.1)
                if latest is not None:
                    state = latest
                observed_ns = monotonic_ns()
                records.append(
                    sample(
                        "soma",
                        observed_ns,
                        sent_ns,
                        target,
                        list(state.positions_rad),
                        planned_index,
                        deadline_ns,
                        sequence=state.sequence,
                        timeline=state.timeline,
                        capture_monotonic_ns=state.capture_monotonic_ns,
                        command_disposition=state.command_disposition,
                    )
                )
            validate_movement(initial, records[1:], case["commanded_indexes"])
            write_jsonl(args.output, records)


if __name__ == "__main__":
    main()
