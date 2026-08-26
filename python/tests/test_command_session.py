import math
import unittest

from soma_client import soma_pb2
from soma_client.command_session import CommandSession


def state(*, positions=None, timeline=4, sequence=10, **changes):
    values = [0.0] * 9 if positions is None else positions
    fields = dict(
        positions_rad=values,
        timeline=timeline,
        sequence=sequence,
        health=soma_pb2.PLANT_HEALTH_HEALTHY,
        command_disposition=soma_pb2.COMMAND_DISPOSITION_NO_COMMAND,
        applied_source=soma_pb2.APPLIED_SOURCE_MEASURED_POSITION_HOLD,
    )
    fields.update(changes)
    return soma_pb2.ActuatorState(**fields)


class CommandSessionTest(unittest.TestCase):
    def test_bootstrap_requires_healthy_nine_finite_positions(self):
        machine = CommandSession()
        for positions in ([0.0] * 8, [0.0] * 8 + [math.inf]):
            with self.assertRaises(ValueError):
                machine.observe(state(positions=positions))
        unhealthy = state(health=soma_pb2.PLANT_HEALTH_STALE_STATE)
        with self.assertRaises(ValueError):
            machine.observe(unhealthy)
        measured = [index / 10 for index in range(9)]
        machine.observe(state(positions=measured))
        for actual, expected in zip(machine.target, measured):
            self.assertAlmostEqual(actual, expected, places=6)
        self.assertIsNot(machine.target, measured)

    def test_each_key_changes_only_its_coordinates(self):
        expected = {
            "a": {0: -0.05}, "d": {0: 0.05},
            "q": {7: -0.10, 8: 0.10}, "e": {7: 0.10, 8: -0.10},
        }
        for key, changed in expected.items():
            machine = CommandSession()
            machine.observe(state())
            target = machine.command_for_key(key).target
            for index, value in enumerate(target.positions_rad):
                self.assertAlmostEqual(value, changed.get(index, 0.0), places=6)

    def test_targets_clamp_and_sequences_increase(self):
        machine = CommandSession()
        positions = [0.0] * 9
        positions[0] = 2.7925268 - 0.01
        positions[7] = math.pi - 0.01
        positions[8] = -math.pi + 0.01
        machine.observe(state(positions=positions))
        first = machine.command_for_key("d").target
        second = machine.command_for_key("e").target
        self.assertAlmostEqual(first.positions_rad[0], 2.7925268, places=6)
        self.assertAlmostEqual(second.positions_rad[7], math.pi, places=6)
        self.assertAlmostEqual(second.positions_rad[8], -math.pi, places=6)
        self.assertGreater(second.sequence, first.sequence)

    def test_timeline_change_discards_pending_and_rebases(self):
        machine = CommandSession()
        machine.observe(state())
        old = machine.command_for_key("d").target
        new_positions = [0.4] * 9
        machine.observe(state(positions=new_positions, timeline=5, sequence=2))
        self.assertIsNone(machine.pending_sequence)
        for actual, expected in zip(machine.target, new_positions):
            self.assertAlmostEqual(actual, expected, places=6)
        new = machine.command_for_key("a").target
        self.assertEqual(new.timeline, 5)
        self.assertGreater(new.sequence, 2)
        self.assertNotEqual(old.timeline, new.timeline)

    def test_evidence_and_event_cardinality_are_honest(self):
        machine = CommandSession()
        machine.observe(state())
        self.assertIsNone(machine.command_for_key("x"))
        command = machine.command_for_key("A")
        self.assertIsNotNone(command)
        accepted = machine.observe(state(
            sequence=11,
            command_disposition=soma_pb2.COMMAND_DISPOSITION_ACCEPTED,
            applied_source=soma_pb2.APPLIED_SOURCE_TARGET,
            applied_sequence=command.target.sequence,
        ))
        self.assertEqual(accepted.disposition, "COMMAND_DISPOSITION_ACCEPTED")
        self.assertEqual(accepted.applied, "APPLIED_SOURCE_TARGET")
        self.assertIsNone(machine.pending_sequence)
        held = machine.observe(state(
            sequence=12,
            applied_source=soma_pb2.APPLIED_SOURCE_MEASURED_POSITION_HOLD,
            expiry_transition=True,
        ))
        self.assertEqual(held.applied, "APPLIED_SOURCE_MEASURED_POSITION_HOLD")
        rejected = machine.observe(state(
            sequence=13,
            command_disposition=soma_pb2.COMMAND_DISPOSITION_REJECTED,
            rejection_reason=soma_pb2.REJECTION_REASON_SEQUENCE,
        ))
        self.assertEqual(rejected.disposition, "COMMAND_DISPOSITION_REJECTED")


if __name__ == "__main__":
    unittest.main()
