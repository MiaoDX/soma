//! Experimental Open Duck transport contract; intentionally separate from soma.v1.

pub const OPEN_DUCK_RUNTIME_SOCKET: &str = "/tmp/soma-open-duck-runtime.sock";
pub const OPEN_DUCK_RT_SOCKET: &str = "/tmp/soma-open-duck-rt.sock";
pub const OPEN_DUCK_STATE_KEY: &str = "soma/open-duck-v2/state";
pub const OPEN_DUCK_TARGET_KEY: &str = "soma/open-duck-v2/target";
pub const OPEN_DUCK_ACTUATORS: usize = 14;

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
}
