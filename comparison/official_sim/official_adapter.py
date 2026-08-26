from __future__ import annotations

import argparse
import time
from pathlib import Path

from reachy_mini.io.protocol import SetAntennasCmd, SetHeadJointsCmd
from reachy_mini.io.ws_client import WSClient

from .common import (
    load_suite,
    manifest,
    monotonic_ns,
    sample,
    select_case,
    validate_movement,
    write_jsonl,
)


def positions(client: WSClient) -> list[float]:
    head, antennas = client.get_current_joints()
    return head + antennas


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="host.docker.internal")
    parser.add_argument("--port", type=int, default=18000)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--suite", type=Path, required=True)
    parser.add_argument("--case", required=True)
    args = parser.parse_args()
    suite = load_suite(args.suite)
    case = select_case(suite, args.case)
    client = WSClient(args.host, args.port)
    try:
        client.wait_for_connection(timeout=10.0)
        warmup_started = time.monotonic()
        time.sleep(suite["warmup_s"])
        warmup_actual_s = time.monotonic() - warmup_started
        initial = positions(client)
        target = [value + delta for value, delta in zip(initial, case["deltas_rad"])]
        sent_ns = monotonic_ns()
        client.send_command(SetHeadJointsCmd(joints=target[:7]))
        client.send_command(SetAntennasCmd(antennas=target[7:]))
        records = [
            manifest(
                "official",
                {
                    "repository": "pollen-robotics/reachy_mini",
                    "release": "v1.9.0",
                    "commit": "b7e686d994a178353ebf81ea935de82ce65af733",
                    "mujoco": "3.3.0",
                },
                initial, suite, case, warmup_actual_s, sent_ns,
            )
        ]
        period_ns = int(suite["sample_period_s"] * 1_000_000_000)
        for planned_index in range(suite["sample_count"]):
            deadline_ns = sent_ns + (planned_index + 1) * period_ns
            remaining = (deadline_ns - monotonic_ns()) / 1_000_000_000
            if remaining > 0:
                time.sleep(remaining)
            observed_ns = monotonic_ns()
            records.append(
                sample(
                    "official",
                    observed_ns,
                    sent_ns,
                    target,
                    positions(client),
                    planned_index,
                    deadline_ns,
                    vendor_timestamp="UNAVAILABLE",
                )
            )
        validate_movement(initial, records[1:], case["commanded_indexes"])
        write_jsonl(args.output, records)
    finally:
        client.disconnect()


if __name__ == "__main__":
    main()
