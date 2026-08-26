//! Direct headless MuJoCo Plant for the pinned Reachy Mini profile.

use std::path::Path;
use std::sync::Arc;

use mujoco_rs::prelude::{MjData, MjModel};
#[cfg(feature = "viewer")]
use mujoco_rs::viewer::MjViewer;
use soma_core::{
    ActuatorPositions, Lifecycle, Plant, PlantHealth, PositionApplication, ReachyActuatorState,
    ACTUATOR_COUNT,
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
pub const SNAPSHOT_MAGIC: [u8; 4] = *b"SMS1";
pub const SNAPSHOT_VERSION: u16 = 1;
pub const SNAPSHOT_NQ: usize = 37;
pub const SNAPSHOT_NV: usize = 30;
pub const SNAPSHOT_LEN: usize = 4 + 2 + 2 + 2 + 8 * 4 + 8 * (SNAPSHOT_NQ + SNAPSHOT_NV);

#[derive(Debug, Clone, PartialEq)]
pub struct ReachySimSnapshot {
    pub timeline: u64,
    pub sequence: u64,
    pub simulation_time_ns: u64,
    pub capture_monotonic_ns: u64,
    pub qpos: [f64; SNAPSHOT_NQ],
    pub qvel: [f64; SNAPSHOT_NV],
}

impl ReachySimSnapshot {
    pub fn encode(&self) -> [u8; SNAPSHOT_LEN] {
        let mut out = [0_u8; SNAPSHOT_LEN];
        let mut offset = 0;
        let put = |out: &mut [u8], offset: &mut usize, bytes: &[u8]| {
            out[*offset..*offset + bytes.len()].copy_from_slice(bytes);
            *offset += bytes.len();
        };
        put(&mut out, &mut offset, &SNAPSHOT_MAGIC);
        put(&mut out, &mut offset, &SNAPSHOT_VERSION.to_le_bytes());
        put(&mut out, &mut offset, &(SNAPSHOT_NQ as u16).to_le_bytes());
        put(&mut out, &mut offset, &(SNAPSHOT_NV as u16).to_le_bytes());
        for value in [
            self.timeline,
            self.sequence,
            self.simulation_time_ns,
            self.capture_monotonic_ns,
        ] {
            put(&mut out, &mut offset, &value.to_le_bytes());
        }
        for value in self.qpos.iter().chain(self.qvel.iter()) {
            put(&mut out, &mut offset, &value.to_le_bytes());
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != SNAPSHOT_LEN || bytes[..4] != SNAPSHOT_MAGIC {
            return Err("invalid snapshot length or magic");
        }
        if u16::from_le_bytes(bytes[4..6].try_into().unwrap()) != SNAPSHOT_VERSION {
            return Err("unsupported snapshot version");
        }
        if u16::from_le_bytes(bytes[6..8].try_into().unwrap()) as usize != SNAPSHOT_NQ
            || u16::from_le_bytes(bytes[8..10].try_into().unwrap()) as usize != SNAPSHOT_NV
        {
            return Err("snapshot dimensions mismatch");
        }
        let mut offset = 10;
        let take_u64 = |offset: &mut usize| {
            let value = u64::from_le_bytes(bytes[*offset..*offset + 8].try_into().unwrap());
            *offset += 8;
            value
        };
        let timeline = take_u64(&mut offset);
        let sequence = take_u64(&mut offset);
        let simulation_time_ns = take_u64(&mut offset);
        let capture_monotonic_ns = take_u64(&mut offset);
        let mut qpos = [0.0; SNAPSHOT_NQ];
        let mut qvel = [0.0; SNAPSHOT_NV];
        for value in qpos.iter_mut().chain(qvel.iter_mut()) {
            *value = f64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
            offset += 8;
            if !value.is_finite() {
                return Err("non-finite generalized state");
            }
        }
        Ok(Self {
            timeline,
            sequence,
            simulation_time_ns,
            capture_monotonic_ns,
            qpos,
            qvel,
        })
    }
}

#[cfg(feature = "viewer")]
pub struct ReachySimViewer {
    data: MjData<Arc<MjModel>>,
    viewer: MjViewer,
}

#[cfg(feature = "viewer")]
impl ReachySimViewer {
    pub fn launch(path: impl AsRef<Path>) -> Result<Self, SimError> {
        let model =
            Arc::new(MjModel::from_xml(path).map_err(|error| SimError::Load(error.to_string()))?);
        if (model.nq() as usize, model.nv() as usize) != (SNAPSHOT_NQ, SNAPSHOT_NV) {
            return Err(SimError::WrongModelSize {
                qpos: model.nq() as usize,
                qvel: model.nv() as usize,
                actuators: model.nu() as usize,
            });
        }
        let data = MjData::new(model.clone());
        let viewer = MjViewer::launch_passive(model, 0)
            .map_err(|error| SimError::Load(error.to_string()))?;
        Ok(Self { data, viewer })
    }

    pub fn running(&self) -> bool {
        self.viewer.running()
    }

    pub fn render(&mut self, snapshot: &ReachySimSnapshot) -> Result<(), SimError> {
        self.data.qpos_mut().copy_from_slice(&snapshot.qpos);
        self.data.qvel_mut().copy_from_slice(&snapshot.qvel);
        self.data.set_time(snapshot.simulation_time_ns as f64 / 1e9);
        self.data.forward();
        self.viewer.sync_data(&mut self.data);
        self.viewer
            .render()
            .map_err(|error| SimError::Load(error.to_string()))
    }
}

#[derive(Debug)]
pub enum SimError {
    Load(String),
    WrongModelSize {
        qpos: usize,
        qvel: usize,
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
        if (
            model.nq() as usize,
            model.nv() as usize,
            model.nu() as usize,
        ) != (SNAPSHOT_NQ, SNAPSHOT_NV, ACTUATOR_COUNT)
        {
            return Err(SimError::WrongModelSize {
                qpos: model.nq() as usize,
                qvel: model.nv() as usize,
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

    pub fn snapshot(&self, capture_monotonic_ns: u64) -> ReachySimSnapshot {
        let qpos = self.data.qpos().try_into().expect("pinned nq");
        let qvel = self.data.qvel().try_into().expect("pinned nv");
        ReachySimSnapshot {
            timeline: self.timeline,
            sequence: self.sequence,
            simulation_time_ns: (self.data.time() * 1_000_000_000.0) as u64,
            capture_monotonic_ns,
            qpos,
            qvel,
        }
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

    fn positions_for_application(
        &self,
        positions_rad: ActuatorPositions,
        application: PositionApplication,
    ) -> Result<ActuatorPositions, SimError> {
        if application == PositionApplication::Target {
            self.validate_positions(positions_rad)?;
            return Ok(positions_rad);
        }
        let mut applied = positions_rad;
        for (index, value) in positions_rad.into_iter().enumerate() {
            let [min, max] = self.limits[index];
            if !value.is_finite() {
                return Err(SimError::TargetOutOfRange {
                    name: ACTUATOR_NAMES[index],
                    value,
                    min,
                    max,
                });
            }
            if self.limited[index] {
                applied[index] = value.clamp(min, max);
            }
        }
        Ok(applied)
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

    fn apply_positions(
        &mut self,
        positions_rad: ActuatorPositions,
        application: PositionApplication,
    ) -> Result<(), Self::Error> {
        let applied = self.positions_for_application(positions_rad, application)?;
        self.data
            .ctrl_mut()
            .copy_from_slice(&applied.map(f64::from));
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
        plant
            .apply_positions(target, PositionApplication::Target)
            .unwrap();
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
            plant.apply_positions(target, PositionApplication::Target),
            Err(SimError::TargetOutOfRange {
                name: "yaw_body",
                ..
            })
        ));
    }

    #[test]
    fn measured_hold_clamps_model_limit_overshoot_but_target_remains_strict() {
        let plant = ReachySimPlant::load(scene()).unwrap();
        let mut overshoot = [0.0; ACTUATOR_COUNT];
        overshoot[0] = 2.8254294;

        assert!(matches!(
            plant.positions_for_application(overshoot, PositionApplication::Target),
            Err(SimError::TargetOutOfRange {
                name: "yaw_body",
                ..
            })
        ));
        let held = plant
            .positions_for_application(overshoot, PositionApplication::MeasuredPositionHold)
            .unwrap();
        assert!(held[0] <= 2.7925268);
        assert!(held[0].is_finite());
    }

    #[test]
    fn measured_hold_rejects_non_finite_values() {
        let plant = ReachySimPlant::load(scene()).unwrap();
        let mut positions = [0.0; ACTUATOR_COUNT];
        positions[0] = f32::NAN;
        assert!(plant
            .positions_for_application(positions, PositionApplication::MeasuredPositionHold)
            .is_err());
    }

    #[test]
    fn ttl_hold_survives_a_dynamics_overshoot_on_the_real_model() {
        let mut plant = ReachySimPlant::load(scene()).unwrap();
        let initial = plant.read_state().unwrap();
        let command = ReachyActuatorTarget {
            positions_rad: [0.0; ACTUATOR_COUNT],
            sequence: initial.sequence + 2,
            timeline: initial.timeline,
            issued_at_ns: 100,
            ttl_ns: 10,
        };
        let mut core = ControlCore::new();
        core.tick(&mut plant, Some(command), 105).unwrap();

        let yaw_joint = plant.data.model().joint("yaw_body").unwrap().id;
        let yaw_qpos = plant.data.model().jnt_qposadr()[yaw_joint] as usize;
        plant.data.qpos_mut()[yaw_qpos] = 2.8254294;
        plant.data.forward();
        let expired = core.tick(&mut plant, None, 110).unwrap();

        assert!(matches!(
            expired.applied.command,
            AppliedCommand::MeasuredPositionHold { .. }
        ));
        assert!(expired.applied.expiry_transition);
        assert!(plant.data.ctrl()[0].is_finite());
        assert!(plant.data.ctrl()[0] <= 2.7925268);
    }

    #[test]
    fn snapshot_round_trip_preserves_full_generalized_state() {
        let plant = ReachySimPlant::load(scene()).unwrap();
        let snapshot = plant.snapshot(123);
        let decoded = ReachySimSnapshot::decode(&snapshot.encode()).unwrap();
        assert_eq!(decoded, snapshot);
    }

    #[test]
    fn snapshot_rejects_bad_length_version_dimensions_and_non_finite_values() {
        let plant = ReachySimPlant::load(scene()).unwrap();
        let snapshot = plant.snapshot(123);
        let encoded = snapshot.encode();
        assert!(ReachySimSnapshot::decode(&encoded[..encoded.len() - 1]).is_err());
        let mut bad = encoded;
        bad[4] = 2;
        assert!(ReachySimSnapshot::decode(&bad).is_err());
        let mut bad = encoded;
        bad[6] = 0;
        assert!(ReachySimSnapshot::decode(&bad).is_err());
        let mut bad = encoded;
        bad[10 + 32 * 8..10 + 33 * 8].copy_from_slice(&f64::NAN.to_le_bytes());
        assert!(ReachySimSnapshot::decode(&bad).is_err());
    }
}
