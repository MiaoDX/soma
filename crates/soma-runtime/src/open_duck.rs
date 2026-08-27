//! Experimental Open Duck transport contract; intentionally separate from soma.v1.

pub const OPEN_DUCK_RUNTIME_SOCKET: &str = "/tmp/soma-open-duck-runtime.sock";
pub const OPEN_DUCK_RT_SOCKET: &str = "/tmp/soma-open-duck-rt.sock";
pub const OPEN_DUCK_STATE_KEY: &str = "soma/open-duck-v2/state";
pub const OPEN_DUCK_TARGET_KEY: &str = "soma/open-duck-v2/target";
pub const OPEN_DUCK_ACTUATORS: usize = 14;
pub const OPEN_DUCK_PHYSICS_HZ: u32 = 500;
pub const OPEN_DUCK_POLICY_HZ: u32 = 50;
pub const OPEN_DUCK_TARGET_BYTES: usize = 8 * 4 + 14 * 4;
pub const OPEN_DUCK_STATE_FLOATS: usize = 14 + 14 + 3 + 3;
pub const OPEN_DUCK_STATE_BYTES: usize = 8 * 7 + 4 + OPEN_DUCK_STATE_FLOATS * 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpenDuckState {
    pub positions_rad: [f32; OPEN_DUCK_ACTUATORS],
    pub velocities_rad_s: [f32; OPEN_DUCK_ACTUATORS],
    pub gyro_rad_s: [f32; 3],
    pub acceleration_m_s2: [f32; 3],
    pub sequence: u64,
    pub timeline: u64,
    pub capture_monotonic_ns: u64,
    pub requested_sequence: u64,
    pub admitted_sequence: u64,
    pub applied_sequence: u64,
    pub message_age_ns: u64,
    pub flags: u32,
}

pub fn encode_state(state: &OpenDuckState, out: &mut [u8; OPEN_DUCK_STATE_BYTES]) {
    for (i, value) in [
        state.sequence,
        state.timeline,
        state.capture_monotonic_ns,
        state.requested_sequence,
        state.admitted_sequence,
        state.applied_sequence,
        state.message_age_ns,
    ]
    .into_iter()
    .enumerate()
    {
        out[i * 8..i * 8 + 8].copy_from_slice(&value.to_le_bytes());
    }
    out[56..60].copy_from_slice(&state.flags.to_le_bytes());
    let values = state
        .positions_rad
        .iter()
        .chain(state.velocities_rad_s.iter())
        .chain(state.gyro_rad_s.iter())
        .chain(state.acceleration_m_s2.iter());
    for (i, value) in values.enumerate() {
        out[60 + i * 4..64 + i * 4].copy_from_slice(&value.to_le_bytes());
    }
}

pub fn decode_state(bytes: &[u8]) -> Option<OpenDuckState> {
    if bytes.len() != OPEN_DUCK_STATE_BYTES {
        return None;
    }
    let u64_at = |offset| {
        Some(u64::from_le_bytes(
            bytes[offset..offset + 8].try_into().ok()?,
        ))
    };
    let f32_at = |index| {
        Some(f32::from_le_bytes(
            bytes[60 + index * 4..64 + index * 4].try_into().ok()?,
        ))
    };
    let positions_rad = std::array::from_fn(|i| f32_at(i).unwrap());
    let velocities_rad_s = std::array::from_fn(|i| f32_at(14 + i).unwrap());
    let gyro_rad_s = std::array::from_fn(|i| f32_at(28 + i).unwrap());
    let acceleration_m_s2 = std::array::from_fn(|i| f32_at(31 + i).unwrap());
    let state = OpenDuckState {
        positions_rad,
        velocities_rad_s,
        gyro_rad_s,
        acceleration_m_s2,
        sequence: u64_at(0)?,
        timeline: u64_at(8)?,
        capture_monotonic_ns: u64_at(16)?,
        requested_sequence: u64_at(24)?,
        admitted_sequence: u64_at(32)?,
        applied_sequence: u64_at(40)?,
        message_age_ns: u64_at(48)?,
        flags: u32::from_le_bytes(bytes[56..60].try_into().ok()?),
    };
    state
        .positions_rad
        .iter()
        .chain(state.velocities_rad_s.iter())
        .chain(state.gyro_rad_s.iter())
        .chain(state.acceleration_m_s2.iter())
        .all(|v| v.is_finite())
        .then_some(state)
}

pub fn encode_target(target: &OpenDuckTarget, out: &mut [u8; OPEN_DUCK_TARGET_BYTES]) {
    out[..8].copy_from_slice(&target.sequence.to_le_bytes());
    out[8..16].copy_from_slice(&target.timeline.to_le_bytes());
    out[16..24].copy_from_slice(&target.capture_monotonic_ns.to_le_bytes());
    out[24..32].copy_from_slice(&target.ttl_ns.to_le_bytes());
    for (i, value) in target.positions_rad.iter().enumerate() {
        out[32 + i * 4..36 + i * 4].copy_from_slice(&value.to_le_bytes());
    }
}

pub fn decode_target(bytes: &[u8]) -> Option<OpenDuckTarget> {
    if bytes.len() != OPEN_DUCK_TARGET_BYTES {
        return None;
    }
    let u64_at = |offset| {
        Some(u64::from_le_bytes(
            bytes[offset..offset + 8].try_into().ok()?,
        ))
    };
    let mut positions_rad = [0.0; OPEN_DUCK_ACTUATORS];
    for (i, value) in positions_rad.iter_mut().enumerate() {
        *value = f32::from_le_bytes(bytes[32 + i * 4..36 + i * 4].try_into().ok()?);
    }
    if positions_rad.iter().any(|value| !value.is_finite()) {
        return None;
    }
    Some(OpenDuckTarget {
        positions_rad,
        sequence: u64_at(0)?,
        timeline: u64_at(8)?,
        capture_monotonic_ns: u64_at(16)?,
        ttl_ns: u64_at(24)?,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpenDuckTarget {
    pub positions_rad: [f32; OPEN_DUCK_ACTUATORS],
    pub sequence: u64,
    pub timeline: u64,
    pub capture_monotonic_ns: u64,
    pub ttl_ns: u64,
}

impl OpenDuckTarget {
    pub fn valid_for(&self, now_ns: u64, timeline: u64, last_sequence: Option<u64>) -> bool {
        self.timeline == timeline
            && last_sequence.is_none_or(|last| self.sequence > last)
            && now_ns.saturating_sub(self.capture_monotonic_ns) < self.ttl_ns
            && self.positions_rad.iter().all(|value| value.is_finite())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profile_endpoints_are_isolated() {
        assert_ne!(OPEN_DUCK_RUNTIME_SOCKET, super::super::RUNTIME_SOCKET);
        assert_ne!(OPEN_DUCK_RT_SOCKET, super::super::RT_SOCKET);
        assert_ne!(OPEN_DUCK_STATE_KEY, super::super::STATE_KEY);
    }
    #[test]
    fn target_rejects_stale_or_invalid_lineage() {
        let target = OpenDuckTarget {
            positions_rad: [0.0; OPEN_DUCK_ACTUATORS],
            sequence: 2,
            timeline: 7,
            capture_monotonic_ns: 100,
            ttl_ns: 20,
        };
        assert!(target.valid_for(119, 7, Some(1)));
        assert!(!target.valid_for(120, 7, Some(1)));
        assert!(!target.valid_for(101, 6, Some(1)));
        assert!(!target.valid_for(101, 7, Some(2)));
    }

    #[test]
    fn target_codec_drops_malformed_and_non_finite_payloads() {
        let target = OpenDuckTarget {
            positions_rad: [0.0; OPEN_DUCK_ACTUATORS],
            sequence: 1,
            timeline: 1,
            capture_monotonic_ns: 10,
            ttl_ns: 20,
        };
        let mut bytes = [0; OPEN_DUCK_TARGET_BYTES];
        encode_target(&target, &mut bytes);
        assert_eq!(decode_target(&bytes), Some(target));
        assert!(decode_target(&bytes[..bytes.len() - 1]).is_none());
        bytes[32..36].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(decode_target(&bytes).is_none());
    }

    #[test]
    fn target_lineage_fault_matrix_is_fail_closed() {
        let target = OpenDuckTarget {
            positions_rad: [0.0; OPEN_DUCK_ACTUATORS],
            sequence: 10,
            timeline: 4,
            capture_monotonic_ns: 100,
            ttl_ns: 20,
        };
        assert!(target.valid_for(119, 4, Some(9)));
        assert!(!target.valid_for(119, 4, Some(10))); // duplicate
        assert!(!target.valid_for(119, 3, Some(9))); // wrong timeline
        assert!(!target.valid_for(120, 4, Some(9))); // expired original lease
        let mut non_finite = target;
        non_finite.positions_rad[3] = f32::INFINITY;
        assert!(!non_finite.valid_for(119, 4, Some(9)));
    }

    #[test]
    fn synthetic_latest_value_contract_is_bounded() {
        let mut latest = None;
        for sequence in 1..=100_u64 {
            latest = Some(OpenDuckTarget {
                positions_rad: [sequence as f32; OPEN_DUCK_ACTUATORS],
                sequence,
                timeline: 4,
                capture_monotonic_ns: sequence * 2_000_000,
                ttl_ns: 20_000_000,
            });
        }
        let target = latest.unwrap();
        assert_eq!(target.sequence, 100);
        assert!(target.valid_for(200_000_001, 4, Some(99)));
        assert!(!target.valid_for(220_000_000, 4, Some(99)));
    }

    #[test]
    fn combined_state_codec_preserves_lineage_and_rejects_non_finite_facts() {
        let mut state = OpenDuckState {
            positions_rad: [1.0; 14],
            velocities_rad_s: [2.0; 14],
            gyro_rad_s: [3.0; 3],
            acceleration_m_s2: [4.0; 3],
            sequence: 11,
            timeline: 2,
            capture_monotonic_ns: 90,
            requested_sequence: 8,
            admitted_sequence: 7,
            applied_sequence: 6,
            message_age_ns: 5,
            flags: 3,
        };
        let mut bytes = [0; OPEN_DUCK_STATE_BYTES];
        encode_state(&state, &mut bytes);
        assert_eq!(decode_state(&bytes), Some(state));
        state.gyro_rad_s[0] = f32::NAN;
        encode_state(&state, &mut bytes);
        assert!(decode_state(&bytes).is_none());
    }
}
