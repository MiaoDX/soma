//! Direct headless MuJoCo Plant for the pinned Reachy Mini profile.

use std::path::Path;
use std::sync::Arc;

use mujoco_rs::prelude::{MjData, MjModel};
use soma_core::{
    ActuatorPositions, Lifecycle, Plant, PlantHealth, ReachyActuatorState, ACTUATOR_COUNT,
};

pub const ACTUATOR_NAMES: [&str; ACTUATOR_COUNT] = [
    "yaw_body",
    "stewart_1",
    "stewart_2",
    "stewart_3",
    "stewart_4",
    "stewart_5",
    "stewart_6",
    "right_antenna",
    "left_antenna",
];
pub const REACHY_SCENE_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/reachy-mini/scene.xml");

#[derive(Debug)]
pub enum SimError {
    Load(String),
    WrongModelSize {
        actuators: usize,
    },
    MissingActuator(&'static str),
    WrongActuatorOrder {
        name: &'static str,
        expected: usize,
        actual: usize,
    },
    TargetOutOfRange {
        name: &'static str,
        value: f32,
        min: f32,
        max: f32,
    },
}

pub struct ReachySimPlant {
    data: MjData<Arc<MjModel>>,
    timeline: u64,
    sequence: u64,
    limits: [[f32; 2]; ACTUATOR_COUNT],
    limited: [bool; ACTUATOR_COUNT],
}

impl ReachySimPlant {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, SimError> {
        let model =
            Arc::new(MjModel::from_xml(path).map_err(|error| SimError::Load(error.to_string()))?);
        if model.nu() as usize != ACTUATOR_COUNT {
            return Err(SimError::WrongModelSize {
                actuators: model.nu() as usize,
            });
        }
        for (expected, name) in ACTUATOR_NAMES.into_iter().enumerate() {
            let actuator = model
                .actuator(name)
                .ok_or(SimError::MissingActuator(name))?;
            if actuator.id != expected {
                return Err(SimError::WrongActuatorOrder {
                    name,
                    expected,
                    actual: actuator.id,
                });
            }
        }
        let joint_ids = ACTUATOR_NAMES.map(|name| {
            model
                .joint(name)
                .expect("actuator and joint names are identical in the pinned profile")
                .id
        });
        let limits = std::array::from_fn(|index| {
            let range = model.jnt_range()[joint_ids[index]];
            [range[0] as f32, range[1] as f32]
        });
        let limited = std::array::from_fn(|index| model.jnt_limited()[joint_ids[index]]);
        Ok(Self {
            data: MjData::new(model),
            timeline: 1,
            sequence: 0,
            limits,
            limited,
        })
    }

    pub fn reset(&mut self) {
        self.data.reset();
        self.timeline = self.timeline.wrapping_add(1);
        self.sequence = 0;
    }

    pub fn step(&mut self) {
        self.data.step();
    }

    pub fn model_dimensions(&self) -> (usize, usize, usize) {
        let model = self.data.model();
        (
            model.nq() as usize,
            model.nv() as usize,
            model.nu() as usize,
        )
    }

    fn positions(&self) -> ActuatorPositions {
        std::array::from_fn(|index| {
            let joint = self
                .data
                .joint(ACTUATOR_NAMES[index])
                .expect("actuator joint was validated when the model loaded");
            joint.view(&self.data).qpos[0] as f32
        })
    }

    pub fn validate_positions(&self, positions_rad: ActuatorPositions) -> Result<(), SimError> {
        for (index, value) in positions_rad.into_iter().enumerate() {
            let [min, max] = self.limits[index];
            if !value.is_finite() || (self.limited[index] && (value < min || value > max)) {
                return Err(SimError::TargetOutOfRange {
                    name: ACTUATOR_NAMES[index],
                    value,
                    min,
                    max,
                });
            }
        }
        Ok(())
    }
}

impl Plant for ReachySimPlant {
    type Error = SimError;

    fn read_state(&mut self) -> Result<ReachyActuatorState, Self::Error> {
        self.sequence = self.sequence.wrapping_add(1);
        Ok(ReachyActuatorState {
            positions_rad: self.positions(),
            sequence: self.sequence,
            timeline: self.timeline,
            timestamp_ns: (self.data.time() * 1_000_000_000.0) as u64,
            lifecycle: Lifecycle::Enabled,
            health: PlantHealth::Healthy,
        })
    }

    fn apply_positions(&mut self, positions_rad: ActuatorPositions) -> Result<(), Self::Error> {
        self.validate_positions(positions_rad)?;
        self.data
            .ctrl_mut()
            .copy_from_slice(&positions_rad.map(f64::from));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soma_core::{AppliedCommand, ControlCore, ReachyActuatorTarget};
    use std::path::PathBuf;

    fn scene() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/reachy-mini/scene.xml")
    }

    #[test]
    fn pinned_reachy_model_loads_with_the_expected_dimensions() {
        let plant = ReachySimPlant::load(scene()).unwrap();
        assert_eq!(plant.model_dimensions(), (37, 30, 9));
    }

    #[test]
    fn target_moves_the_model_and_reset_changes_the_timeline() {
        let mut plant = ReachySimPlant::load(scene()).unwrap();
        let before = plant.read_state().unwrap();
        let mut target = before.positions_rad;
        target[0] += 0.2;
        plant.apply_positions(target).unwrap();
        for _ in 0..100 {
            plant.step();
        }
        let moved = plant.read_state().unwrap();
        assert!((moved.positions_rad[0] - before.positions_rad[0]).abs() > 0.01);

        plant.reset();
        let reset = plant.read_state().unwrap();
        assert_ne!(reset.timeline, moved.timeline);
        assert_eq!(reset.sequence, 1);
    }

    #[test]
    fn control_core_expires_to_measured_position_hold_on_the_real_model() {
        let mut plant = ReachySimPlant::load(scene()).unwrap();
        let initial = plant.read_state().unwrap();
        let mut positions = initial.positions_rad;
        positions[0] += 0.1;
        let command = ReachyActuatorTarget {
            positions_rad: positions,
            sequence: initial.sequence + 2,
            timeline: initial.timeline,
            issued_at_ns: 0,
            ttl_ns: 10,
        };
        let mut core = ControlCore::new();
        core.tick(&mut plant, Some(command), 9).unwrap();
        plant.step();
        let expired = core.tick(&mut plant, None, 10).unwrap();
        assert!(matches!(
            expired.applied.command,
            AppliedCommand::MeasuredPositionHold { .. }
        ));
        assert!(expired.applied.expiry_transition);
    }

    #[test]
    fn plant_rejects_targets_outside_the_pinned_model_limits() {
        let mut plant = ReachySimPlant::load(scene()).unwrap();
        let mut target = plant.read_state().unwrap().positions_rad;
        target[0] = f32::INFINITY;
        assert!(matches!(
            plant.apply_positions(target),
            Err(SimError::TargetOutOfRange {
                name: "yaw_body",
                ..
            })
        ));
    }
}
