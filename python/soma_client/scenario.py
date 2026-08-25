from __future__ import annotations

import queue
import argparse
import time

import zenoh

from soma_client import soma_pb2

COMMAND_KEY = "soma/reachy/command"
STATE_KEY = "soma/reachy/state"


def wait_for(states: queue.Queue, predicate, timeout: float = 5.0):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            sample = states.get(timeout=min(0.2, deadline - time.monotonic()))
        except queue.Empty:
            continue
        state = soma_pb2.ActuatorState.FromString(bytes(sample.payload))
        if predicate(state):
            return state
    raise TimeoutError("expected state transition was not observed")


def pace(enabled: bool, seconds: float) -> None:
    if enabled:
        time.sleep(seconds)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--visualize", action="store_true")
    args = parser.parse_args()
    config = zenoh.Config.from_json5(
        """{
          mode: "client",
          connect: { endpoints: ["tcp/127.0.0.1:7447"] },
          scouting: { multicast: { enabled: false } }
        }"""
    )
    with zenoh.open(config) as session:
        states: queue.Queue = queue.Queue()
        with session.declare_subscriber(STATE_KEY, states.put):
            initial = wait_for(states, lambda state: len(state.positions_rad) == 9)
            old_timeline = initial.timeline
            start_yaw = initial.positions_rad[0]
            pace(args.visualize, 1.0)

            positions = list(initial.positions_rad)
            positions[0] += 0.2
            command = soma_pb2.RtRequest(
                target=soma_pb2.ActuatorTarget(
                    positions_rad=positions,
                    sequence=initial.sequence + 100,
                    timeline=initial.timeline,
                    ttl_ns=200_000_000,
                )
            )
            session.put(COMMAND_KEY, command.SerializeToString())
            accepted = wait_for(
                states,
                lambda state: state.command_disposition
                == soma_pb2.COMMAND_DISPOSITION_ACCEPTED,
            )
            assert accepted.health == soma_pb2.PLANT_HEALTH_HEALTHY
            assert accepted.state_age_ns > 0
            wait_for(states, lambda state: abs(state.positions_rad[0] - start_yaw) > 0.01)
            pace(args.visualize, 2.0)
            held = wait_for(states, lambda state: state.expiry_transition)
            assert held.applied_source == soma_pb2.APPLIED_SOURCE_MEASURED_POSITION_HOLD
            pace(args.visualize, 1.0)

            session.put(
                COMMAND_KEY,
                soma_pb2.RtRequest(reset=True).SerializeToString(),
            )
            reset = wait_for(states, lambda state: state.timeline != old_timeline)
            pace(args.visualize, 2.0)

            old_command = soma_pb2.RtRequest(
                target=soma_pb2.ActuatorTarget(
                    positions_rad=list(reset.positions_rad),
                    sequence=reset.sequence + 100,
                    timeline=old_timeline,
                    ttl_ns=200_000_000,
                )
            )
            session.put(COMMAND_KEY, old_command.SerializeToString())
            wait_for(
                states,
                lambda state: state.command_disposition
                == soma_pb2.COMMAND_DISPOSITION_REJECTED
                and state.rejection_reason == soma_pb2.REJECTION_REASON_TIMELINE,
            )

            print(
                f"scenario passed: timeline {old_timeline} -> {reset.timeline}, "
                f"yaw delta > 0.01 rad, TTL -> measured hold, old timeline rejected"
            )


if __name__ == "__main__":
    main()
