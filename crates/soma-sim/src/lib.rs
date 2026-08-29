//! Direct headless MuJoCo Plant for the pinned Reachy Mini profile.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use mujoco_rs::prelude::{MjData, MjModel};
#[cfg(feature = "viewer")]
use mujoco_rs::viewer::MjViewer;
#[cfg(feature = "showcase")]
use mujoco_rs::{
    renderer::{png, MjRenderer},
    wrappers::mj_visualization::MjvCamera,
};
use soma_core::{
    ActuatorPositions, ActuatorState, Lifecycle, Plant, PlantHealth, PositionApplication,
    ReachyActuatorState, ACTUATOR_COUNT,
};

pub const OPEN_DUCK_ACTUATOR_COUNT: usize = 14;
pub const OPEN_DUCK_ACTUATOR_NAMES: [&str; OPEN_DUCK_ACTUATOR_COUNT] = [
    "left_hip_yaw",
    "left_hip_roll",
    "left_hip_pitch",
    "left_knee",
    "left_ankle",
    "neck_pitch",
    "head_pitch",
    "head_yaw",
    "head_roll",
    "right_hip_yaw",
    "right_hip_roll",
    "right_hip_pitch",
    "right_knee",
    "right_ankle",
];
pub const OPEN_DUCK_SCENE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/open-duck-mini-v2/xmls/scene_flat_terrain.xml"
);

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

#[cfg(feature = "showcase")]
pub struct ReachySimRenderer {
    data: MjData<Arc<MjModel>>,
    renderer: MjRenderer,
}

#[cfg(feature = "showcase")]
impl ReachySimRenderer {
    pub const WIDTH: usize = 640;
    pub const HEIGHT: usize = 480;

    pub fn launch(path: impl AsRef<Path>) -> Result<Self, SimError> {
        let model =
            Arc::new(MjModel::from_xml(path).map_err(|error| SimError::Load(error.to_string()))?);
        if model.camera("studio_close").map(|camera| camera.id) != Some(0) {
            return Err(SimError::Load(
                "fixed camera id 0 is not scene camera studio_close".into(),
            ));
        }
        let data = MjData::new(model.clone());
        let renderer = MjRenderer::builder()
            .width(Self::WIDTH as u32)
            .height(Self::HEIGHT as u32)
            .camera(MjvCamera::new_fixed(0))
            .png_compression(png::Compression::Balanced)
            .build(model)
            .map_err(|error| SimError::Load(error.to_string()))?;
        Ok(Self { data, renderer })
    }

    pub fn render_png(
        &mut self,
        snapshot: &ReachySimSnapshot,
        path: impl AsRef<Path>,
    ) -> Result<(), SimError> {
        self.data.qpos_mut().copy_from_slice(&snapshot.qpos);
        self.data.qvel_mut().copy_from_slice(&snapshot.qvel);
        self.data.set_time(snapshot.simulation_time_ns as f64 / 1e9);
        self.data.forward();
        self.renderer
            .sync_data(&mut self.data)
            .and_then(|_| self.renderer.render())
            .map_err(|error| SimError::Load(error.to_string()))?;
        self.renderer
            .save_rgb(path)
            .map_err(|error| SimError::Load(error.to_string()))
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PhysicsScheduleError {
    ZeroControlPeriod,
    InvalidPhysicsTimestep {
        physics_timestep_s: f64,
    },
    NonIntegralSubsteps {
        control_period_s: f64,
        physics_timestep_s: f64,
    },
}

/// Validated integer relationship between one control period and physics steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicsSchedule {
    substeps_per_control_period: usize,
}

impl PhysicsSchedule {
    pub fn new(
        control_period: Duration,
        physics_timestep_s: f64,
    ) -> Result<Self, PhysicsScheduleError> {
        if control_period.is_zero() {
            return Err(PhysicsScheduleError::ZeroControlPeriod);
        }
        if !physics_timestep_s.is_finite() || physics_timestep_s <= 0.0 {
            return Err(PhysicsScheduleError::InvalidPhysicsTimestep { physics_timestep_s });
        }
        let control_period_s = control_period.as_secs_f64();
        let ratio = control_period_s / physics_timestep_s;
        let rounded = ratio.round();
        let tolerance = 1e-9 * ratio.max(1.0);
        if rounded < 1.0 || rounded > usize::MAX as f64 || (ratio - rounded).abs() > tolerance {
            return Err(PhysicsScheduleError::NonIntegralSubsteps {
                control_period_s,
                physics_timestep_s,
            });
        }
        Ok(Self {
            substeps_per_control_period: rounded as usize,
        })
    }

    pub fn substeps_per_control_period(self) -> usize {
        self.substeps_per_control_period
    }
}

#[derive(Debug)]
pub enum SimError {
    Load(String),
    PhysicsSchedule(PhysicsScheduleError),
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
    physics_schedule: PhysicsSchedule,
    timeline: u64,
    sequence: u64,
    limits: [[f32; 2]; ACTUATOR_COUNT],
    limited: [bool; ACTUATOR_COUNT],
}

/// Fixed Open Duck Mini v2 simulation Plant. This profile intentionally does
/// not participate in Reachy's public protocol or runtime endpoints.
pub struct OpenDuckSimPlant {
    data: MjData<Arc<MjModel>>,
    physics_schedule: PhysicsSchedule,
    timeline: u64,
    sequence: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpenDuckPolicyFacts {
    pub positions_rad: [f32; OPEN_DUCK_ACTUATOR_COUNT],
    pub velocities_rad_s: [f32; OPEN_DUCK_ACTUATOR_COUNT],
    pub gyro_rad_s: [f32; 3],
    pub acceleration_m_s2: [f32; 3],
}

impl OpenDuckSimPlant {
    pub fn load(control_period: Duration) -> Result<Self, SimError> {
        let model = Arc::new(
            MjModel::from_xml(Path::new(OPEN_DUCK_SCENE_PATH))
                .map_err(|e| SimError::Load(e.to_string()))?,
        );
        if (
            model.nq() as usize,
            model.nv() as usize,
            model.nu() as usize,
        ) != (21, 20, OPEN_DUCK_ACTUATOR_COUNT)
        {
            return Err(SimError::WrongModelSize {
                qpos: model.nq() as usize,
                qvel: model.nv() as usize,
                actuators: model.nu() as usize,
            });
        }
        for (expected, name) in OPEN_DUCK_ACTUATOR_NAMES.into_iter().enumerate() {
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
        let physics_schedule = PhysicsSchedule::new(control_period, model.opt().timestep)
            .map_err(SimError::PhysicsSchedule)?;
        let mut data = MjData::new(model);
        data.reset_keyframe(0)
            .map_err(|e| SimError::Load(format!("reset Open Duck home keyframe: {e}")))?;
        Ok(Self {
            data,
            physics_schedule,
            timeline: 1,
            sequence: 0,
        })
    }

    pub fn policy_decimation(&self) -> usize {
        10
    }
    pub fn physics_schedule(&self) -> PhysicsSchedule {
        self.physics_schedule
    }
    pub fn advance_physics_step(&mut self) {
        self.data.step();
    }
    pub fn advance_control_period(&mut self) {
        for _ in 0..self.physics_schedule.substeps_per_control_period() {
            self.advance_physics_step();
        }
    }
    pub fn model_dimensions(&self) -> (usize, usize, usize) {
        (
            self.data.model().nq() as usize,
            self.data.model().nv() as usize,
            self.data.model().nu() as usize,
        )
    }
    fn positions(&self) -> [f32; OPEN_DUCK_ACTUATOR_COUNT] {
        std::array::from_fn(|i| {
            self.data
                .joint(OPEN_DUCK_ACTUATOR_NAMES[i])
                .expect("validated joint")
                .view(&self.data)
                .qpos[0] as f32
        })
    }
    pub fn policy_facts(&self) -> OpenDuckPolicyFacts {
        let velocities_rad_s = std::array::from_fn(|i| {
            self.data
                .joint(OPEN_DUCK_ACTUATOR_NAMES[i])
                .expect("validated joint")
                .view(&self.data)
                .qvel[0] as f32
        });
        let sensors = self.data.sensordata();
        OpenDuckPolicyFacts {
            positions_rad: self.positions(),
            velocities_rad_s,
            gyro_rad_s: std::array::from_fn(|i| sensors[i] as f32),
            acceleration_m_s2: std::array::from_fn(|i| sensors[6 + i] as f32),
        }
    }
    pub fn reset(&mut self) {
        self.data
            .reset_keyframe(0)
            .expect("validated Open Duck home keyframe");
        self.timeline = self.timeline.wrapping_add(1);
        self.sequence = 0;
    }
}

impl Plant<OPEN_DUCK_ACTUATOR_COUNT> for OpenDuckSimPlant {
    type Error = SimError;
    fn read_state(&mut self) -> Result<ActuatorState<OPEN_DUCK_ACTUATOR_COUNT>, Self::Error> {
        self.sequence = self.sequence.wrapping_add(1);
        Ok(ActuatorState {
            positions_rad: self.positions(),
            sequence: self.sequence,
            timeline: self.timeline,
            timestamp_ns: (self.data.time() * 1e9) as u64,
            lifecycle: Lifecycle::Enabled,
            health: PlantHealth::Healthy,
        })
    }
    fn apply_positions(
        &mut self,
        positions_rad: [f32; OPEN_DUCK_ACTUATOR_COUNT],
        _application: PositionApplication,
    ) -> Result<(), Self::Error> {
        for (i, value) in positions_rad.into_iter().enumerate() {
            if !value.is_finite() {
                return Err(SimError::TargetOutOfRange {
                    name: OPEN_DUCK_ACTUATOR_NAMES[i],
                    value,
                    min: f32::NEG_INFINITY,
                    max: f32::INFINITY,
                });
            }
            self.data.ctrl_mut()[i] = value as f64;
        }
        Ok(())
    }
}

impl ReachySimPlant {
    pub fn load(path: impl AsRef<Path>, control_period: Duration) -> Result<Self, SimError> {
        let model =
            Arc::new(MjModel::from_xml(path).map_err(|error| SimError::Load(error.to_string()))?);
        let physics_schedule = PhysicsSchedule::new(control_period, model.opt().timestep)
            .map_err(SimError::PhysicsSchedule)?;
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
            physics_schedule,
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

    pub fn advance_control_period(&mut self) {
        for _ in 0..self.physics_schedule.substeps_per_control_period() {
            self.data.step();
        }
    }

    pub fn physics_schedule(&self) -> PhysicsSchedule {
        self.physics_schedule
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

impl Plant<{ soma_core::ACTUATOR_COUNT }> for ReachySimPlant {
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

    const CONTROL_PERIOD: Duration = Duration::from_millis(20);

    fn scene() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/reachy-mini/scene.xml")
    }

    #[test]
    fn pinned_reachy_model_loads_with_the_expected_dimensions() {
        let plant = ReachySimPlant::load(scene(), CONTROL_PERIOD).unwrap();
        assert_eq!(plant.model_dimensions(), (37, 30, 9));
        assert_eq!(plant.physics_schedule().substeps_per_control_period(), 10);
    }

    #[test]
    fn physics_schedule_rejects_invalid_robot_cadence_before_execution() {
        assert_eq!(
            PhysicsSchedule::new(Duration::ZERO, 0.002),
            Err(PhysicsScheduleError::ZeroControlPeriod)
        );
        for timestep in [0.0, -0.002, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                PhysicsSchedule::new(CONTROL_PERIOD, timestep),
                Err(PhysicsScheduleError::InvalidPhysicsTimestep { .. })
            ));
        }
        for timestep in [0.03, 0.003] {
            assert!(matches!(
                PhysicsSchedule::new(CONTROL_PERIOD, timestep),
                Err(PhysicsScheduleError::NonIntegralSubsteps { .. })
            ));
        }
    }

    #[test]
    fn advancing_once_covers_one_validated_control_period() {
        let mut plant = ReachySimPlant::load(scene(), CONTROL_PERIOD).unwrap();
        let before = plant.snapshot(0).simulation_time_ns;
        plant.advance_control_period();
        let after = plant.snapshot(0).simulation_time_ns;
        assert_eq!(after - before, CONTROL_PERIOD.as_nanos() as u64);
    }

    #[test]
    fn target_moves_the_model_and_reset_changes_the_timeline() {
        let mut plant = ReachySimPlant::load(scene(), CONTROL_PERIOD).unwrap();
        let before = plant.read_state().unwrap();
        let mut target = before.positions_rad;
        target[0] += 0.2;
        plant
            .apply_positions(target, PositionApplication::Target)
            .unwrap();
        for _ in 0..100 {
            plant.advance_control_period();
        }
        let moved = plant.read_state().unwrap();
        assert!((moved.positions_rad[0] - before.positions_rad[0]).abs() > 0.01);

        plant.reset();
        let reset = plant.read_state().unwrap();
        assert_ne!(reset.timeline, moved.timeline);
        assert_eq!(reset.sequence, 1);
    }

    #[test]
    fn open_duck_profile_validates_dimensions_order_and_cadence() {
        let plant = OpenDuckSimPlant::load(Duration::from_millis(20)).unwrap();
        assert_eq!(plant.model_dimensions(), (21, 20, 14));
        assert_eq!(plant.physics_schedule().substeps_per_control_period(), 10);
        assert_eq!(plant.policy_decimation(), 10);
        assert_eq!(
            plant.positions(),
            [
                0.002, 0.053, -0.63, 1.368, -0.784, 0.0, 0.0, 0.0, 0.0, -0.003, -0.065, 0.635,
                1.379, -0.796
            ]
        );
    }

    #[test]
    fn open_duck_profile_advances_and_resets_timeline() {
        let mut plant = OpenDuckSimPlant::load(Duration::from_millis(2)).unwrap();
        let before = plant.read_state().unwrap();
        plant.advance_physics_step();
        let after = plant.read_state().unwrap();
        assert!(after.timestamp_ns > before.timestamp_ns);
        plant.reset();
        assert_eq!(plant.read_state().unwrap().timeline, before.timeline + 1);
    }

    #[test]
    fn open_duck_control_core_applies_target_on_fixed_physics_path() {
        use soma_core::{ActuatorTarget, ControlCore};
        let mut plant = OpenDuckSimPlant::load(Duration::from_millis(2)).unwrap();
        let timeline = plant.read_state().unwrap().timeline;
        let mut core = ControlCore::<OPEN_DUCK_ACTUATOR_COUNT>::new();
        let target = ActuatorTarget {
            positions_rad: [0.0; OPEN_DUCK_ACTUATOR_COUNT],
            sequence: 3,
            timeline,
            issued_at_ns: 0,
            ttl_ns: 1_000_000_000,
        };
        let tick = core.tick(&mut plant, Some(target), 1).unwrap();
        assert_eq!(
            tick.command_result,
            soma_core::CommandResult::Accepted { sequence: 3 }
        );
        for _ in 0..10 {
            plant.advance_physics_step();
        }
        assert!(plant.read_state().unwrap().timestamp_ns >= 20_000_000);
    }

    #[test]
    fn control_core_expires_to_measured_position_hold_on_the_real_model() {
        let mut plant = ReachySimPlant::load(scene(), CONTROL_PERIOD).unwrap();
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
        plant.advance_control_period();
        let expired = core.tick(&mut plant, None, 10).unwrap();
        assert!(matches!(
            expired.applied.command,
            AppliedCommand::MeasuredPositionHold { .. }
        ));
        assert!(expired.applied.expiry_transition);
    }

    #[test]
    fn plant_rejects_targets_outside_the_pinned_model_limits() {
        let mut plant = ReachySimPlant::load(scene(), CONTROL_PERIOD).unwrap();
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
        let plant = ReachySimPlant::load(scene(), CONTROL_PERIOD).unwrap();
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
        let plant = ReachySimPlant::load(scene(), CONTROL_PERIOD).unwrap();
        let mut positions = [0.0; ACTUATOR_COUNT];
        positions[0] = f32::NAN;
        assert!(plant
            .positions_for_application(positions, PositionApplication::MeasuredPositionHold)
            .is_err());
    }

    #[test]
    fn ttl_hold_survives_a_dynamics_overshoot_on_the_real_model() {
        let mut plant = ReachySimPlant::load(scene(), CONTROL_PERIOD).unwrap();
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
        let plant = ReachySimPlant::load(scene(), CONTROL_PERIOD).unwrap();
        let snapshot = plant.snapshot(123);
        let decoded = ReachySimSnapshot::decode(&snapshot.encode()).unwrap();
        assert_eq!(decoded, snapshot);
    }

    #[test]
    fn snapshot_rejects_bad_length_version_dimensions_and_non_finite_values() {
        let plant = ReachySimPlant::load(scene(), CONTROL_PERIOD).unwrap();
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
