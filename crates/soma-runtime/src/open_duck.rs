//! Experimental Open Duck transport contract; intentionally separate from soma.v1.

pub const OPEN_DUCK_RUNTIME_SOCKET: &str = "/tmp/soma-open-duck-runtime.sock";
pub const OPEN_DUCK_RT_SOCKET: &str = "/tmp/soma-open-duck-rt.sock";
pub const OPEN_DUCK_STATE_KEY: &str = "soma/open-duck-v2/state";
pub const OPEN_DUCK_TARGET_KEY: &str = "soma/open-duck-v2/target";
pub const OPEN_DUCK_ACTUATORS: usize = 14;
pub const OPEN_DUCK_PHYSICS_HZ: u32 = 500;
pub const OPEN_DUCK_POLICY_HZ: u32 = 50;
pub const OPEN_DUCK_TARGET_BYTES: usize = 8 * 4 + 14 * 4;

pub fn encode_target(target: &OpenDuckTarget, out: &mut [u8; OPEN_DUCK_TARGET_BYTES]) {
    out[..8].copy_from_slice(&target.sequence.to_le_bytes());
    out[8..16].copy_from_slice(&target.timeline.to_le_bytes());
    out[16..24].copy_from_slice(&target.capture_monotonic_ns.to_le_bytes());
    out[24..32].copy_from_slice(&target.ttl_ns.to_le_bytes());
    for (i, value) in target.positions_rad.iter().enumerate() { out[32 + i * 4..36 + i * 4].copy_from_slice(&value.to_le_bytes()); }
}

pub fn decode_target(bytes: &[u8]) -> Option<OpenDuckTarget> {
    if bytes.len() != OPEN_DUCK_TARGET_BYTES { return None; }
    let u64_at = |offset| Some(u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?));
    let mut positions_rad = [0.0; OPEN_DUCK_ACTUATORS];
    for (i, value) in positions_rad.iter_mut().enumerate() { *value = f32::from_le_bytes(bytes[32 + i * 4..36 + i * 4].try_into().ok()?); }
    Some(OpenDuckTarget { positions_rad, sequence: u64_at(0)?, timeline: u64_at(8)?, capture_monotonic_ns: u64_at(16)?, ttl_ns: u64_at(24)? })
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
        let target = OpenDuckTarget { positions_rad: [0.0; OPEN_DUCK_ACTUATORS], sequence: 2, timeline: 7, capture_monotonic_ns: 100, ttl_ns: 20 };
        assert!(target.valid_for(119, 7, Some(1)));
        assert!(!target.valid_for(120, 7, Some(1)));
        assert!(!target.valid_for(101, 6, Some(1)));
        assert!(!target.valid_for(101, 7, Some(2)));
    }

    #[test]
    fn synthetic_latest_value_contract_is_bounded() {
        let mut latest = None;
        for sequence in 1..=100_u64 {
            latest = Some(OpenDuckTarget { positions_rad: [sequence as f32; OPEN_DUCK_ACTUATORS], sequence, timeline: 4, capture_monotonic_ns: sequence * 2_000_000, ttl_ns: 20_000_000 });
        }
        let target = latest.unwrap();
        assert_eq!(target.sequence, 100);
        assert!(target.valid_for(200_000_001, 4, Some(99)));
        assert!(!target.valid_for(220_000_000, 4, Some(99)));
    }
}
