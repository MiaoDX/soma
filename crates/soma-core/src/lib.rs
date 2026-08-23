//! Minimal Reachy Mini profile contract shared by simulation and native plants.

pub const ACTUATOR_COUNT: usize = 9;
pub const ACTUATOR_IDS: [u8; ACTUATOR_COUNT] = [10, 11, 12, 13, 14, 15, 16, 17, 18];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReachyActuatorState {
    pub positions_rad: [f32; ACTUATOR_COUNT],
    pub sequence: u64,
    pub timeline: u64,
    pub timestamp_ns: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReachyActuatorTarget {
    pub positions_rad: [f32; ACTUATOR_COUNT],
    pub sequence: u64,
    pub timeline: u64,
    pub issued_at_ns: u64,
    pub ttl_ns: u64,
}

impl ReachyActuatorTarget {
    pub fn is_expired(self, now_ns: u64) -> bool {
        now_ns.saturating_sub(self.issued_at_ns) >= self.ttl_ns
    }

    pub fn applies_to(self, timeline: u64, last_sequence: u64) -> bool {
        self.timeline == timeline && self.sequence > last_sequence
    }
}

/// Expiry holds the last validated measured position to avoid dropping a
/// coupled Stewart mechanism when torque remains enabled.
pub fn target_or_hold(
    target: Option<ReachyActuatorTarget>,
    measured: ReachyActuatorState,
    now_ns: u64,
) -> [f32; ACTUATOR_COUNT] {
    match target {
        Some(target) if !target.is_expired(now_ns) => target.positions_rad,
        _ => measured.positions_rad,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> ReachyActuatorState {
        ReachyActuatorState {
            positions_rad: [0.1; ACTUATOR_COUNT],
            sequence: 3,
            timeline: 7,
            timestamp_ns: 100,
        }
    }

    #[test]
    fn ids_are_fixed_to_reachy_profile() {
        assert_eq!(ACTUATOR_IDS, [10, 11, 12, 13, 14, 15, 16, 17, 18]);
    }

    #[test]
    fn expired_target_holds_measured_position() {
        let measured = state();
        let target = ReachyActuatorTarget {
            positions_rad: [1.0; ACTUATOR_COUNT],
            sequence: 4,
            timeline: 7,
            issued_at_ns: 100,
            ttl_ns: 10,
        };
        assert_eq!(
            target_or_hold(Some(target), measured, 110),
            [0.1; ACTUATOR_COUNT]
        );
        assert_eq!(
            target_or_hold(Some(target), measured, 109),
            [1.0; ACTUATOR_COUNT]
        );
    }

    #[test]
    fn stale_timeline_and_sequence_are_rejected() {
        let target = ReachyActuatorTarget {
            positions_rad: [0.0; ACTUATOR_COUNT],
            sequence: 3,
            timeline: 6,
            issued_at_ns: 0,
            ttl_ns: 100,
        };
        assert!(!target.applies_to(7, 2));
        assert!(!ReachyActuatorTarget {
            timeline: 7,
            ..target
        }
        .applies_to(7, 3));
        assert!(ReachyActuatorTarget {
            timeline: 7,
            ..target
        }
        .applies_to(7, 2));
    }
}
