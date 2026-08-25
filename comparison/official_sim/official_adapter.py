from __future__ import annotations

import argparse
import time
from pathlib import Path

from reachy_mini.io.protocol import SetAntennasCmd, SetHeadJointsCmd
from reachy_mini.io.ws_client import WSClient

from .common import (
    DWELL_S,
    SAMPLE_PERIOD_S,
    TRACE_DELTAS_RAD,
    WARMUP_S,
    manifest,
    monotonic_ns,
    sample,
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
    args = parser.parse_args()
    client = WSClient(args.host, args.port)
    try:
        client.wait_for_connection(timeout=10.0)
        time.sleep(WARMUP_S)
        initial = positions(client)
        target = [value + delta for value, delta in zip(initial, TRACE_DELTAS_RAD)]
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
                initial,
            )
        ]
        deadline = time.monotonic() + DWELL_S
        while time.monotonic() < deadline:
            records.append(
                sample(
                    "official",
                    monotonic_ns(),
                    sent_ns,
                    target,
                    positions(client),
                    vendor_timestamp="UNAVAILABLE",
                )
            )
            time.sleep(SAMPLE_PERIOD_S)
        moved = [
            abs(actual - start)
            for actual, start in zip(records[-1]["measured_positions_rad"], initial)
        ]
        if max(moved[index] for index in (0, 7, 8)) < 0.05:
            raise RuntimeError("official public SDK commands produced no observed motion")
        write_jsonl(args.output, records)
    finally:
        client.disconnect()


if __name__ == "__main__":
    main()
