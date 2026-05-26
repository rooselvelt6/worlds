use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlMode {
    DPad,
    Joystick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterPreset {
    Human,
    Robot,
    Beast,
    Ghost,
    Teddy,
    Panda,
    Kraken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticleMode {
    Off,
    Rain,
    Snow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson,
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct WorldParams {
    pub seed: u32,
    pub scale: f64,
    pub octaves: u32,
    pub amplitude: f64,
    pub water_level: f64,
    pub render_distance: u32,
    pub zone: crate::engine::terrain::Zone,
    pub mutation: f64,
    pub speed: f64,
    pub mouse_sensitivity: f64,
    pub fly_mode: bool,
    pub build_mode: bool,
    pub camera_mode: CameraMode,
    pub tour_speed: f64,
    pub tour_radius: f64,
    pub control_mode: ControlMode,
    pub hue_shift: f64,
    pub saturation: f64,
    pub lightness: f64,
    pub param_a: f64,
    pub param_b: f64,
    pub canyons: bool,
    pub particle_mode: ParticleMode,
    pub character: CharacterPreset,
    pub color_scheme: u32,
    pub char_scale: f64,
    pub volume: f64,
    pub day_speed: f64,
    pub season_speed: f64,
    pub season: u8,
    pub gravity: f64,
    pub jump_speed: f64,
    pub step_height: f64,
    pub movement_accel: f64,
    pub movement_friction: f64,
}

impl Default for WorldParams {
    fn default() -> Self {
        Self {
            seed: 42,
            scale: 0.015,
            octaves: 5,
            amplitude: 4.0,
            water_level: 0.0,
            render_distance: 2,
            zone: crate::engine::terrain::Zone::Forest,
            mutation: 0.0,
            speed: 300.0,
            mouse_sensitivity: 1.0,
            fly_mode: false,
            build_mode: false,
            camera_mode: CameraMode::ThirdPerson,
            tour_speed: 8.0,
            tour_radius: 20.0,
            control_mode: ControlMode::DPad,
            canyons: false,
            particle_mode: ParticleMode::Off,
            character: CharacterPreset::Human,
            color_scheme: 0,
            char_scale: 1.0,
            hue_shift: 0.0,
            saturation: 1.0,
            lightness: 1.0,
            param_a: 0.5,
            param_b: 0.5,
            volume: 0.3,
            day_speed: 0.0,
            season_speed: 0.01,
            season: 0,
            gravity: 20.0,
            jump_speed: 10.0,
            step_height: 0.7,
            movement_accel: 200.0,
            movement_friction: 30.0,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub params: RwSignal<WorldParams>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            params: RwSignal::new(WorldParams::default()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub slot_name: String,
    pub params: WorldParams,
    pub pos: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub waypoints: Vec<(f64, f64, f64, String)>,
    pub discovered_biomes: Vec<String>,
    pub time_of_day: f64,
    pub fly_mode: bool,
    pub observer_mode: bool,
    pub created_at: f64,
    pub inventory_json: Option<String>,
    pub codex_json: Option<String>,
    pub achievements_json: Option<String>,
    pub placed_blocks: Vec<[i32; 4]>,
    pub block_inventory: Vec<(u8, u32)>,
}

impl SaveData {
    pub fn new(slot_name: &str, params: &WorldParams, pos: [f64; 3], yaw: f64, pitch: f64,
               waypoints: &[(f64, f64, f64, String)], discovered: &[String],
               time_of_day: f64, fly_mode: bool, observer_mode: bool) -> Self {
        Self {
            slot_name: slot_name.to_string(),
            params: *params,
            pos,
            yaw,
            pitch,
            waypoints: waypoints.to_vec(),
            discovered_biomes: discovered.to_vec(),
            time_of_day,
            fly_mode,
            observer_mode,
            created_at: js_sys::Date::now(),
            inventory_json: None,
            codex_json: None,
            achievements_json: None,
            placed_blocks: Vec::new(),
            block_inventory: Vec::new(),
        }
    }
}
