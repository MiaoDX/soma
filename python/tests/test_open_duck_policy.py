import struct
import numpy as np

from soma_client.open_duck_policy import STATE, decode_state


def test_combined_state_decode_preserves_lineage_and_facts():
    payload = STATE.pack(11, 2, 90, 8, 7, 6, 5, 3, *range(34))
    state = decode_state(payload)
    assert (state["sequence"], state["timeline"], state["applied"]) == (11, 2, 6)
    np.testing.assert_array_equal(state["positions"], np.arange(14, dtype=np.float32))
    assert struct.calcsize("<7QI34f") == len(payload)
