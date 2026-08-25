from __future__ import annotations

import argparse
import math
import queue
import select
import sys
import termios
import time
import tty
from dataclasses import dataclass
from typing import Iterable

import zenoh

from soma_client import soma_pb2
from soma_client.scenario import COMMAND_KEY, STATE_KEY

ACTUATOR_COUNT = 9
BODY_YAW = 0
RIGHT_ANTENNA = 7
LEFT_ANTENNA = 8
YAW_STEP = 0.05
ANTENNA_STEP = 0.10
BODY_YAW_LIMITS = (-2.792526803190975, 2.792526803190879)
ANTENNA_LIMITS = (-math.pi, math.pi)
TTL_NS = 250_000_000


@dataclass(frozen=True)
class Evidence:
    timeline: int
    sequence: int
    measured: tuple[float, ...]
    requested: tuple[float, ...] | None
    disposition: str
    applied: str


class CommandSession:
    def __init__(self) -> None:
        self.timeline: int | None = None
        self.next_sequence = 0
        self.measured: list[float] | None = None
        self.target: list[float] | None = None
        self.pending_sequence: int | None = None
        self.evidence: Evidence | None = None

    @property
    def ready(self) -> bool:
        return self.target is not None

    def observe(self, state: soma_pb2.ActuatorState) -> Evidence:
        positions = list(state.positions_rad)
        valid = (
            state.health == soma_pb2.PLANT_HEALTH_HEALTHY
            and len(positions) == ACTUATOR_COUNT
            and all(math.isfinite(value) for value in positions)
        )
        if not valid:
            self.measured = None
            self.target = None
            self.pending_sequence = None
            raise ValueError("state is unhealthy or does not contain nine finite positions")

        timeline_changed = self.timeline != state.timeline
        self.measured = positions
        if timeline_changed or self.target is None:
            self.timeline = state.timeline
            self.target = positions.copy()
            self.pending_sequence = None
            self.next_sequence = state.sequence + 1
        else:
            self.next_sequence = max(self.next_sequence, state.sequence + 1)

        disposition = soma_pb2.CommandDisposition.Name(state.command_disposition)
        applied = soma_pb2.AppliedSource.Name(state.applied_source)
        requested = tuple(self.target) if self.pending_sequence is not None else None
        self.evidence = Evidence(
            timeline=state.timeline,
            sequence=state.sequence,
            measured=tuple(positions),
            requested=requested,
            disposition=disposition,
            applied=applied,
        )
        if (
            state.command_disposition == soma_pb2.COMMAND_DISPOSITION_REJECTED
            or (
                state.command_disposition == soma_pb2.COMMAND_DISPOSITION_ACCEPTED
                and state.applied_sequence == self.pending_sequence
            )
        ):
            self.pending_sequence = None
        return self.evidence

    def command_for_key(self, key: str) -> soma_pb2.RtRequest | None:
        if not self.ready or self.timeline is None:
            return None
        key = key.lower()
        deltas = {
            "a": ((BODY_YAW, -YAW_STEP, BODY_YAW_LIMITS),),
            "d": ((BODY_YAW, YAW_STEP, BODY_YAW_LIMITS),),
            "q": (
                (RIGHT_ANTENNA, -ANTENNA_STEP, ANTENNA_LIMITS),
                (LEFT_ANTENNA, ANTENNA_STEP, ANTENNA_LIMITS),
            ),
            "e": (
                (RIGHT_ANTENNA, ANTENNA_STEP, ANTENNA_LIMITS),
                (LEFT_ANTENNA, -ANTENNA_STEP, ANTENNA_LIMITS),
            ),
        }
        if key not in deltas:
            return None
        for index, delta, bounds in deltas[key]:
            self.target[index] = min(bounds[1], max(bounds[0], self.target[index] + delta))
        sequence = self.next_sequence
        self.next_sequence += 1
        self.pending_sequence = sequence
        return soma_pb2.RtRequest(
            target=soma_pb2.ActuatorTarget(
                positions_rad=self.target,
                sequence=sequence,
                timeline=self.timeline,
                issued_at_ns=time.monotonic_ns(),
                ttl_ns=TTL_NS,
            )
        )


def _config() -> zenoh.Config:
    return zenoh.Config.from_json5(
        '{mode:"client",connect:{endpoints:["tcp/127.0.0.1:7447"]},'
        'scouting:{multicast:{enabled:false}}}'
    )


def _render(machine: CommandSession, message: str = "") -> None:
    if machine.evidence is None:
        line = "waiting for healthy nine-position state"
    else:
        evidence = machine.evidence
        requested = "none" if evidence.requested is None else (
            f"{evidence.requested[0]:+.2f}/{evidence.requested[7]:+.2f}/"
            f"{evidence.requested[8]:+.2f}"
        )
        line = (
            f"timeline={evidence.timeline} state-seq={evidence.sequence} "
            f"measured yaw/R/L={evidence.measured[0]:+.2f}/"
            f"{evidence.measured[7]:+.2f}/{evidence.measured[8]:+.2f} "
            f"requested={requested} disposition={evidence.disposition} "
            f"applied={evidence.applied}"
        )
    print(f"\r{line} {message}".rstrip(), end="\n" if message else "", flush=True)


def _run(keys: Iterable[str] | None) -> None:
    machine = CommandSession()
    states: queue.Queue = queue.Queue(maxsize=64)
    bounded = iter(keys) if keys is not None else None
    terminal = None
    with zenoh.open(_config()) as session:
        with session.declare_subscriber(STATE_KEY, lambda sample: states.put_nowait(sample)):
            old_terminal = None
            if bounded is None:
                try:
                    # The launcher supervises this process in the background,
                    # where stdin is /dev/null. Reopen the controlling TTY so
                    # interactive input remains available.
                    terminal = open("/dev/tty", "r")
                    old_terminal = termios.tcgetattr(terminal)
                    tty.setcbreak(terminal.fileno())
                except (OSError, termios.error) as error:
                    if terminal is not None:
                        terminal.close()
                    raise RuntimeError(
                        "interactive mode requires a controlling terminal; "
                        "use --keys KEYS for a non-interactive session"
                    ) from error
            try:
                while True:
                    try:
                        sample = states.get(timeout=0.05)
                        state = soma_pb2.ActuatorState.FromString(bytes(sample.payload))
                        try:
                            machine.observe(state)
                            _render(machine)
                        except ValueError as error:
                            _render(machine, str(error))
                    except queue.Empty:
                        pass

                    key = None
                    if bounded is not None and machine.ready and machine.pending_sequence is None:
                        try:
                            key = next(bounded)
                        except StopIteration:
                            return
                    elif bounded is None and select.select([terminal], [], [], 0)[0]:
                        key = terminal.read(1)
                    if key is not None:
                        request = machine.command_for_key(key)
                        if request is not None:
                            session.put(COMMAND_KEY, request.SerializeToString())
            finally:
                if old_terminal is not None:
                    termios.tcsetattr(terminal, termios.TCSADRAIN, old_terminal)
                if terminal is not None:
                    terminal.close()


def main() -> None:
    parser = argparse.ArgumentParser(description="Reachy Mini simulation command session")
    parser.add_argument(
        "--keys",
        metavar="KEYS",
        help="bounded non-interactive key sequence for integration testing",
    )
    args = parser.parse_args()
    try:
        _run(args.keys)
    except RuntimeError as error:
        parser.error(str(error))


if __name__ == "__main__":
    main()
