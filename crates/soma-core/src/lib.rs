//! Executable Reachy Mini control-core contract shared by simulation and native plants.

pub const ACTUATOR_COUNT: usize = 9;
pub const ACTUATOR_IDS: [u8; ACTUATOR_COUNT] = [10, 11, 12, 13, 14, 15, 16, 17, 18];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    Disabled,
    Enabled,
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlantHealth {
    Healthy,
    StaleState,
    CommunicationFault,
    ConfigurationMismatch,
}

pub type ActuatorPositions = [f32; ACTUATOR_COUNT];
pub type ActuatorArray<const N: usize> = [f32; N];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActuatorState<const N: usize> {
    pub positions_rad: ActuatorArray<N>,
    pub sequence: u64,
    pub timeline: u64,
    pub timestamp_ns: u64,
    pub lifecycle: Lifecycle,
    pub health: PlantHealth,
}
pub type ReachyActuatorState = ActuatorState<ACTUATOR_COUNT>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActuatorTarget<const N: usize> {
    pub positions_rad: ActuatorArray<N>,
    pub sequence: u64,
    pub timeline: u64,
    pub issued_at_ns: u64,
    pub ttl_ns: u64,
}
pub type ReachyActuatorTarget = ActuatorTarget<ACTUATOR_COUNT>;

impl<const N: usize> ActuatorTarget<N> {
    pub fn is_expired(self, now_ns: u64) -> bool {
        now_ns.saturating_sub(self.issued_at_ns) >= self.ttl_ns
    }

    pub fn applies_to(self, timeline: u64, last_sequence: Option<u64>) -> bool {
        self.timeline == timeline && last_sequence.is_none_or(|last| self.sequence > last)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionApplication {
    Target,
    MeasuredPositionHold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyDisposition {
    /// The Plant accepted the value but a later state sample is needed to
    /// prove device-side observation.
    Submitted,
    /// The Plant can prove the value is already represented in its state.
    Confirmed,
}

/// The bounded cyclic boundary implemented by simulation and native Plant adapters.
pub trait Plant<const N: usize> {
    type Error;

    fn read_state(&mut self) -> Result<ActuatorState<N>, Self::Error>;
    fn apply_positions(
        &mut self,
        positions_rad: ActuatorArray<N>,
        application: PositionApplication,
    ) -> Result<ApplyDisposition, Self::Error>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectionReason {
    Timeline,
    Sequence,
    Expired,
    Invalid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandResult {
    NoCommand,
    Accepted {
        sequence: u64,
    },
    Rejected {
        sequence: u64,
        reason: RejectionReason,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CommandInput<const N: usize> {
    None,
    Target(ActuatorTarget<N>),
    Rejected {
        sequence: u64,
        reason: RejectionReason,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppliedCommand {
    Target { sequence: u64 },
    MeasuredPositionHold { sequence: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AppliedControl<const N: usize> {
    pub positions_rad: ActuatorArray<N>,
    pub command: AppliedCommand,
    /// True only on the tick that an active target expires.
    pub expiry_transition: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlTick<const N: usize> {
    pub measured: ActuatorState<N>,
    pub command_result: CommandResult,
    pub applied: AppliedControl<N>,
    pub apply_disposition: ApplyDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlError<E> {
    Read(E),
    Apply(E),
}

/// Minimal stateful command validator and actuator-position controller.
#[derive(Clone, Copy, Debug, Default)]
pub struct ControlCore<const N: usize> {
    timeline: Option<u64>,
    last_sequence: Option<u64>,
    active_target: Option<ActuatorTarget<N>>,
}

impl<const N: usize> ControlCore<N> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick<P: Plant<N>>(
        &mut self,
        plant: &mut P,
        command: Option<ActuatorTarget<N>>,
        now_ns: u64,
    ) -> Result<ControlTick<N>, ControlError<P::Error>> {
        self.tick_input(
            plant,
            command.map_or(CommandInput::None, CommandInput::Target),
            now_ns,
            true,
        )
    }

    /// Apply an asynchronously captured target. The target sequence belongs to
    /// the producer's source stream, so it is compared only with the last
    /// admitted target rather than the plant's faster measured-state sequence.
    pub fn tick_async<P: Plant<N>>(
        &mut self,
        plant: &mut P,
        command: Option<ActuatorTarget<N>>,
        now_ns: u64,
    ) -> Result<ControlTick<N>, ControlError<P::Error>> {
        self.tick_input(
            plant,
            command.map_or(CommandInput::None, CommandInput::Target),
            now_ns,
            false,
        )
    }

    pub fn tick_ingress<P: Plant<N>>(
        &mut self,
        plant: &mut P,
        command: CommandInput<N>,
        now_ns: u64,
    ) -> Result<ControlTick<N>, ControlError<P::Error>> {
        self.tick_input(plant, command, now_ns, true)
    }

    fn tick_input<P: Plant<N>>(
        &mut self,
        plant: &mut P,
        command: CommandInput<N>,
        now_ns: u64,
        guard_against_measured_sequence: bool,
    ) -> Result<ControlTick<N>, ControlError<P::Error>> {
        let measured = plant.read_state().map_err(ControlError::Read)?;
        self.observe_timeline(measured.timeline);
        if guard_against_measured_sequence && self.last_sequence.is_none() {
            self.last_sequence = Some(measured.sequence);
        }

        let command_result = match command {
            CommandInput::None => CommandResult::NoCommand,
            CommandInput::Rejected { sequence, reason } => {
                CommandResult::Rejected { sequence, reason }
            }
            CommandInput::Target(target) if target.timeline != measured.timeline => {
                CommandResult::Rejected {
                    sequence: target.sequence,
                    reason: RejectionReason::Timeline,
                }
            }
            CommandInput::Target(target)
                if self
                    .last_sequence
                    .is_some_and(|last| target.sequence <= last) =>
            {
                CommandResult::Rejected {
                    sequence: target.sequence,
                    reason: RejectionReason::Sequence,
                }
            }
            CommandInput::Target(target) if target.is_expired(now_ns) => CommandResult::Rejected {
                sequence: target.sequence,
                reason: RejectionReason::Expired,
            },
            CommandInput::Target(target) => {
                self.last_sequence = Some(target.sequence);
                self.active_target = Some(target);
                CommandResult::Accepted {
                    sequence: target.sequence,
                }
            }
        };

        let expired = self
            .active_target
            .is_some_and(|target| target.is_expired(now_ns));
        if expired {
            self.active_target = None;
        }

        let applied = match self.active_target {
            Some(target) => AppliedControl {
                positions_rad: target.positions_rad,
                command: AppliedCommand::Target {
                    sequence: target.sequence,
                },
                expiry_transition: false,
            },
            None => AppliedControl {
                positions_rad: measured.positions_rad,
                command: AppliedCommand::MeasuredPositionHold {
                    sequence: measured.sequence,
                },
                expiry_transition: expired,
            },
        };

        let application = match applied.command {
            AppliedCommand::Target { .. } => PositionApplication::Target,
            AppliedCommand::MeasuredPositionHold { .. } => {
                PositionApplication::MeasuredPositionHold
            }
        };
        let apply_disposition = plant
            .apply_positions(applied.positions_rad, application)
            .map_err(ControlError::Apply)?;

        Ok(ControlTick {
            measured,
            command_result,
            applied,
            apply_disposition,
        })
    }

    fn observe_timeline(&mut self, timeline: u64) {
        if self.timeline != Some(timeline) {
            self.timeline = Some(timeline);
            self.last_sequence = None;
            self.active_target = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlant {
        state: ReachyActuatorState,
        applied: ActuatorPositions,
        application: Option<PositionApplication>,
    }

    impl TestPlant {
        fn new() -> Self {
            Self {
                state: ReachyActuatorState {
                    positions_rad: [0.1; ACTUATOR_COUNT],
                    sequence: 3,
                    timeline: 7,
                    timestamp_ns: 100,
                    lifecycle: Lifecycle::Enabled,
                    health: PlantHealth::Healthy,
                },
                applied: [0.0; ACTUATOR_COUNT],
                application: None,
            }
        }
    }

    impl Plant<ACTUATOR_COUNT> for TestPlant {
        type Error = ();

        fn read_state(&mut self) -> Result<ReachyActuatorState, Self::Error> {
            Ok(self.state)
        }

        fn apply_positions(
            &mut self,
            positions_rad: ActuatorPositions,
            application: PositionApplication,
        ) -> Result<ApplyDisposition, Self::Error> {
            self.applied = positions_rad;
            self.application = Some(application);
            Ok(ApplyDisposition::Confirmed)
        }
    }

    fn target(sequence: u64, timeline: u64) -> ReachyActuatorTarget {
        ReachyActuatorTarget {
            positions_rad: [1.0; ACTUATOR_COUNT],
            sequence,
            timeline,
            issued_at_ns: 100,
            ttl_ns: 10,
        }
    }

    #[test]
    fn fixed_profile_has_exact_reachy_actuator_order() {
        assert_eq!(ACTUATOR_COUNT, 9);
        assert_eq!(ACTUATOR_IDS, [10, 11, 12, 13, 14, 15, 16, 17, 18]);
    }

    #[test]
    fn tick_applies_an_accepted_target_and_reports_plant_status() {
        let mut core = ControlCore::new();
        let mut plant = TestPlant::new();

        let tick = core.tick(&mut plant, Some(target(4, 7)), 105).unwrap();

        assert_eq!(tick.command_result, CommandResult::Accepted { sequence: 4 });
        assert_eq!(tick.applied.command, AppliedCommand::Target { sequence: 4 });
        assert_eq!(plant.applied, [1.0; ACTUATOR_COUNT]);
        assert_eq!(plant.application, Some(PositionApplication::Target));
        assert_eq!(tick.measured.lifecycle, Lifecycle::Enabled);
        assert_eq!(tick.measured.health, PlantHealth::Healthy);
    }

    #[test]
    fn timeline_and_sequence_rejections_do_not_replace_the_active_target() {
        let mut core = ControlCore::new();
        let mut plant = TestPlant::new();
        core.tick(&mut plant, Some(target(4, 7)), 105).unwrap();

        let old_timeline = core.tick(&mut plant, Some(target(5, 6)), 106).unwrap();
        assert_eq!(
            old_timeline.command_result,
            CommandResult::Rejected {
                sequence: 5,
                reason: RejectionReason::Timeline,
            }
        );
        assert_eq!(
            old_timeline.applied.command,
            AppliedCommand::Target { sequence: 4 }
        );

        let old_sequence = core.tick(&mut plant, Some(target(4, 7)), 107).unwrap();
        assert_eq!(
            old_sequence.command_result,
            CommandResult::Rejected {
                sequence: 4,
                reason: RejectionReason::Sequence,
            }
        );
        assert_eq!(
            old_sequence.applied.command,
            AppliedCommand::Target { sequence: 4 }
        );
    }

    #[test]
    fn expiry_transitions_once_to_latest_measured_position_hold() {
        let mut core = ControlCore::new();
        let mut plant = TestPlant::new();
        core.tick(&mut plant, Some(target(4, 7)), 109).unwrap();
        plant.state.positions_rad = [0.4; ACTUATOR_COUNT];

        let expired = core.tick(&mut plant, None, 110).unwrap();
        assert_eq!(expired.applied.positions_rad, [0.4; ACTUATOR_COUNT]);
        assert_eq!(
            expired.applied.command,
            AppliedCommand::MeasuredPositionHold { sequence: 3 }
        );
        assert!(expired.applied.expiry_transition);
        assert_eq!(
            plant.application,
            Some(PositionApplication::MeasuredPositionHold)
        );

        plant.state.positions_rad = [0.5; ACTUATOR_COUNT];
        let held = core.tick(&mut plant, None, 111).unwrap();
        assert_eq!(held.applied.positions_rad, [0.5; ACTUATOR_COUNT]);
        assert!(!held.applied.expiry_transition);
    }

    #[test]
    fn timeline_change_drops_the_old_target_and_restarts_sequence_admission() {
        let mut core = ControlCore::new();
        let mut plant = TestPlant::new();
        core.tick(&mut plant, Some(target(9, 7)), 105).unwrap();

        plant.state.timeline = 8;
        plant.state.sequence = 0;
        plant.state.positions_rad = [0.8; ACTUATOR_COUNT];
        let reset = core.tick(&mut plant, Some(target(1, 8)), 105).unwrap();

        assert_eq!(
            reset.command_result,
            CommandResult::Accepted { sequence: 1 }
        );
        assert_eq!(
            reset.applied.command,
            AppliedCommand::Target { sequence: 1 }
        );
    }

    #[test]
    fn already_expired_command_is_rejected_and_measured_position_is_applied() {
        let mut core = ControlCore::new();
        let mut plant = TestPlant::new();

        let tick = core.tick(&mut plant, Some(target(4, 7)), 110).unwrap();

        assert_eq!(
            tick.command_result,
            CommandResult::Rejected {
                sequence: 4,
                reason: RejectionReason::Expired,
            }
        );
        assert_eq!(plant.applied, [0.1; ACTUATOR_COUNT]);
        assert!(!tick.applied.expiry_transition);
    }

    #[test]
    fn ingress_rejection_is_reported_without_replacing_the_active_target() {
        let mut core = ControlCore::new();
        let mut plant = TestPlant::new();
        core.tick(&mut plant, Some(target(4, 7)), 105).unwrap();

        let tick = core
            .tick_ingress(
                &mut plant,
                CommandInput::Rejected {
                    sequence: 5,
                    reason: RejectionReason::Invalid,
                },
                106,
            )
            .unwrap();

        assert_eq!(
            tick.command_result,
            CommandResult::Rejected {
                sequence: 5,
                reason: RejectionReason::Invalid,
            }
        );
        assert_eq!(tick.applied.command, AppliedCommand::Target { sequence: 4 });
    }

    #[test]
    fn async_targets_use_producer_sequence_not_faster_measured_sequence() {
        let mut core = ControlCore::new();
        let mut plant = TestPlant::new();
        plant.state.sequence = 100;

        let first = core
            .tick_async(&mut plant, Some(target(1, 7)), 105)
            .unwrap();
        assert_eq!(
            first.command_result,
            CommandResult::Accepted { sequence: 1 }
        );

        plant.state.sequence = 110;
        let next = core
            .tick_async(&mut plant, Some(target(2, 7)), 106)
            .unwrap();
        assert_eq!(next.command_result, CommandResult::Accepted { sequence: 2 });
    }
}
