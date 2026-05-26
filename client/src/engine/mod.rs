pub mod achievements;
pub mod audio;
pub mod bridge;
pub mod camera;
pub mod chunk;
pub mod codex;
pub mod controls;
pub mod creatures;
pub mod db;
pub mod foam;
pub mod gamepad;
pub mod inventory;
pub mod joystick;
pub mod minimap;
pub mod minerals;
pub mod particles;
pub mod portals;
pub mod structures;
pub mod terrain;
pub mod tour;
pub mod vegetation;
pub mod waterfall;

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::panic::AssertUnwindSafe;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use achievements::AchievementState;
use codex::Codex;
use foam::FoamSystem;
use particles::BubbleSystem;
use crate::state::{CameraMode, CharacterPreset, ParticleMode, WorldParams};
use terrain::{Zone, get_height};
use camera::Camera;
use chunk::{ChunkData, CHUNK_SIZE};
use controls::{Controls, MASK_1, MASK_2, MASK_3, MASK_4, MASK_5, MASK_6, MASK_7, MASK_8, MASK_9,
    MASK_A, MASK_B, MASK_D, MASK_E, MASK_G, MASK_LCLICK, MASK_Q, MASK_R, MASK_RCLICK, MASK_S, MASK_SHIFT, MASK_SPACE, MASK_T, MASK_W};
use creatures::{generate_creature_mesh, creature_animated_positions};
use gamepad::poll_gamepad;
use inventory::Inventory;
use minerals::generate_mineral_mesh;
use particles::ParticleSystem;
use portals::generate_portal_mesh;
use structures::{compute_chunk_structures, generate_road_mesh, generate_struct_mesh};
use tour::TourState;
use vegetation::{compute_chunk_vegetation, generate_veg_mesh_from_data, VegData, VegType};
use waterfall::WaterfallSystem;

const ARM_LENGTH: f64 = 5.0;
const ARM_HEIGHT: f64 = 2.5;
const ROT_SPEED: f64 = 2.5;
const WALK_AMP: f64 = 0.35;
const RUN_AMP: f64 = 0.6;
const WALK_FREQ: f64 = 5.0;
const RUN_FREQ: f64 = 8.0;

#[derive(Clone, Default)]
pub struct HudData {
    pub pos: [f64; 3],
    pub biome: String,
    pub height: f64,
    pub fps: u32,
    pub chunks: usize,
    pub yaw_deg: i32,
    pub speed: f64,
    pub fly_mode: bool,
    pub formula: String,
    pub observer_mode: bool,
    pub gamepad_connected: bool,
    pub waypoints: Vec<(f64, f64, f64, String)>,
    pub discovered_biomes: Vec<String>,
    pub discovery_message: Option<String>,
    pub near_portal: Option<String>,
    pub build_mode: bool,
    pub inventory: Vec<(u8, u32)>,
    pub minerals: Vec<(u8, u32)>,
    pub selected_slot: u8,
    pub season: u8,
    pub creature_count: u32,
    pub achievement_points: u32,
    pub vr_mode: bool,
    pub tour_mode: bool,
    pub weather_label: String,
    pub lightning: bool,
    pub craft_message: Option<String>,
    pub achievement_message: Option<String>,
    pub codex_biomes: (usize, usize),
    pub codex_structures: (usize, usize),
    pub codex_minerals: (usize, usize),
    pub codex_creatures: (usize, usize),
}

fn collides_with_veg(x: f64, z: f64, veg_chunks: &std::collections::HashMap<(i32, i32), VegData>) -> bool {
    let cx = (x / CHUNK_SIZE) as i32;
    let cz = (z / CHUNK_SIZE) as i32;
    for dcx in -1..=1 {
        for dcz in -1..=1 {
            if let Some(veg) = veg_chunks.get(&(cx + dcx, cz + dcz)) {
                for inst in &veg.instances {
                    let radius = match inst.veg_type {
                        VegType::Tree | VegType::DeadTree => inst.size as f64 * 0.5,
                        VegType::Cactus => inst.size as f64 * 0.15,
                        _ => 0.0,
                    };
                    if radius <= 0.0 { continue; }
                    let dx = x - inst.x as f64;
                    let dz = z - inst.z as f64;
                    if dx * dx + dz * dz < radius * radius {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn auto_save_full(s: &GameState) {
    let inventory_json = serde_json::to_string(&s.inventory).ok();
    let codex_json = serde_json::to_string(&s.codex).ok();
    let achievements_json = serde_json::to_string(&s.achievements).ok();
    let placed: Vec<[i32; 4]> = s.placed_blocks.iter().map(|(&(x,y,z), &t)| [x, y, z, t as i32]).collect();
    let block_inv: Vec<(u8, u32)> = s.block_inventory.iter().map(|(t, c)| (*t, *c)).collect();
    let discovered = s.discovered_biomes.clone();
    let mut data = crate::state::SaveData::new(
        "Auto", &s.params, s.char_pos,
        s.controls.yaw.get(), s.controls.pitch.get(),
        &[], &discovered,
        s.day_time, s.params.fly_mode, false,
    );
    data.inventory_json = inventory_json;
    data.codex_json = codex_json;
    data.achievements_json = achievements_json;
    data.placed_blocks = placed;
    data.block_inventory = block_inv;
    if let Ok(json) = serde_json::to_string(&data) {
        db::set_async("worlds_autosave", &json);
    }
}

fn save_blocks(blocks: &std::collections::HashMap<(i32,i32,i32), u8>) {
    let data: Vec<[i32; 4]> = blocks.iter().map(|(&(x,y,z), &t)| [x, y, z, t as i32]).collect();
    if let Ok(json) = serde_json::to_string(&data) {
        db::set_async("worlds_blocks", &json);
    }
}

fn load_blocks() -> std::collections::HashMap<(i32,i32,i32), u8> {
    let mut map = std::collections::HashMap::new();
    if let Some(json) = db::get("worlds_blocks") {
        if let Ok(data) = serde_json::from_str::<Vec<[i32; 4]>>(&json) {
            for arr in data {
                map.insert((arr[0], arr[1], arr[2]), arr[3] as u8);
            }
        }
    }
    map
}

fn save_mined(mined: &HashSet<(i32,i32,i32)>) {
    let data: Vec<[i32; 3]> = mined.iter().map(|&(x,y,z)| [x, y, z]).collect();
    if let Ok(json) = serde_json::to_string(&data) {
        db::set_async("worlds_mined", &json);
    }
}

fn load_mined() -> HashSet<(i32,i32,i32)> {
    let mut set = HashSet::new();
    if let Some(json) = db::get("worlds_mined") {
        if let Ok(data) = serde_json::from_str::<Vec<[i32; 3]>>(&json) {
            for arr in data {
                set.insert((arr[0], arr[1], arr[2]));
            }
        }
    }
    set
}

fn collides_with_blocks(x: f64, y: f64, z: f64,
    blocks: &std::collections::HashMap<(i32,i32,i32), u8>) -> bool
{
    let bx = x.floor() as i32;
    let bz = z.floor() as i32;
    let by = y.floor() as i32;
    for dy in 0..2 {
        if blocks.contains_key(&(bx, by + dy, bz)) {
            return true;
        }
    }
    false
}

fn raycast_block(ox: f64, oy: f64, oz: f64, dx: f64, dy: f64, dz: f64, max_dist: f64,
    placed: &std::collections::HashMap<(i32,i32,i32), u8>,
    mined: &HashSet<(i32,i32,i32)>,
    params: &WorldParams) -> Option<((i32,i32,i32), bool)>
{
    let len = (dx*dx + dy*dy + dz*dz).sqrt();
    if len < 0.001 { return None; }
    let (sx, sy, sz) = (dx/len, dy/len, dz/len);
    let mut x = ox;
    let mut y = oy;
    let mut z = oz;
    let step = 0.5;
    let mut dist = 0.0;
    while dist < max_dist {
        let bx = x.floor() as i32;
        let by = y.floor() as i32;
        let bz = z.floor() as i32;
        let key = (bx, by, bz);
        if placed.contains_key(&key) {
            return Some((key, true));
        }
        if mined.contains(&key) {
            x += sx * step;
            y += sy * step;
            z += sz * step;
            dist += step;
            continue;
        }
        let mut terrain_h = get_height(params, x, z);
        terrain::zone_effects(params, x, z, &mut terrain_h);
        if y < terrain_h && terrain_h > params.water_level {
            return Some((key, false));
        }
        x += sx * step;
        y += sy * step;
        z += sz * step;
        dist += step;
    }
    None
}

struct BreakParticle {
    key: String,
    positions: Vec<f64>,
    velocities: Vec<[f64; 3]>,
    lifetime: f64,
    max_lifetime: f64,
}

impl BreakParticle {
    fn new(key: String, origin: [f64; 3], count: usize) -> Self {
        let mut rng: u64 = origin[0] as u64 ^ (origin[1] as u64).wrapping_mul(374761393) ^ (origin[2] as u64).wrapping_mul(668265263);
        let mut rng_f = || {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            rng as f64 * 4.6566129e-10
        };
        let mut positions = Vec::with_capacity(count * 3);
        let mut velocities = Vec::with_capacity(count);
        for _ in 0..count {
            let theta = rng_f() * std::f64::consts::TAU;
            let phi = rng_f() * std::f64::consts::PI;
            let speed = 2.0 + rng_f() * 4.0;
            positions.push(origin[0]);
            positions.push(origin[1]);
            positions.push(origin[2]);
            velocities.push([theta.cos() * phi.sin() * speed, phi.cos() * speed, theta.sin() * phi.sin() * speed]);
        }
        let flat: Vec<f32> = positions.iter().map(|&v| v as f32).collect();
        let arr = js_sys::Float32Array::from(&flat[..]);
        bridge::create_particles(&key, count as u32, 1.0, 1.0, 1.0, 0.06);
        bridge::update_particles(&key, &arr);
        Self { key, positions, velocities, lifetime: 0.6, max_lifetime: 0.6 }
    }

    fn update(&mut self, delta: f64) {
        self.lifetime -= delta;
        if self.lifetime <= 0.0 { return; }
        let gray = (self.lifetime / self.max_lifetime) as f32;
        bridge::set_particles_opacity(&self.key, (gray * 0.8) as f64);
        for i in 0..self.velocities.len() {
            let i3 = i * 3;
            self.positions[i3] += self.velocities[i][0] * delta;
            self.positions[i3 + 1] += self.velocities[i][1] * delta;
            self.positions[i3 + 2] += self.velocities[i][2] * delta;
            self.velocities[i][1] -= 9.8 * delta;
        }
        let flat: Vec<f32> = self.positions.iter().map(|&v| v as f32).collect();
        bridge::update_particles(&self.key, &js_sys::Float32Array::from(&flat[..]));
    }

    fn remove(&self) {
        bridge::remove_mesh(&self.key);
    }
}

struct RemotePlayerData {
    name: String,
    x: f64, y: f64, z: f64,
}

#[allow(dead_code)]
struct GameState {
    canvas: web_sys::HtmlCanvasElement,
    camera: Camera,
    controls: Controls,
    params: WorldParams,
    chunks: Vec<ChunkData>,
    prev_cx: i32,
    prev_cz: i32,
    last_time: f64,
    frame_count: u32,
    fps_timer: f64,
    fps: u32,
    joy_dx: Rc<Cell<f64>>,
    joy_dy: Rc<Cell<f64>>,
    speed: f64,
    char_pos: [f64; 3],
    vel_x: f64,
    vel_z: f64,
    vel_y: f64,
    cam_pos: [f64; 3],
    time: f64,
    day_time: f64,
    walk_time: f64,
    char_yaw: f64,
    particles: Option<ParticleSystem>,
    tour: TourState,
    gamepad_state: gamepad::GamepadState,
    inventory: Inventory,
    tour_prev: bool,
    spawned: bool,
    weather_power: f64,
    weather_target: f64,
    weather_timer: f64,
    lightning_flash: f64,
    veg_chunks: std::collections::HashMap<(i32, i32), VegData>,
    placed_blocks: std::collections::HashMap<(i32, i32, i32), u8>,
    block_inventory: Vec<(u8, u32)>,
    build_prev: bool,
    slot_prev: u32,
    save_timer: f64,
    params_dirty: bool,
    char_dirty: bool,
    portal_prev: bool,
    g_prev: bool,
    waterfalls: WaterfallSystem,
    foam: FoamSystem,
    bubbles: BubbleSystem,
    codex: codex::Codex,
    achievements: AchievementState,
    discovered_biomes: Vec<String>,
    total_distance: f64,
    weather_power_idx: u8,
    weather_cooldown: f64,
    mined_blocks: HashSet<(i32, i32, i32)>,
    break_counter: u32,
    break_particles: Vec<BreakParticle>,
    ws_connected: bool,
    ws_player_id: String,
    ws_send_timer: f64,
    chat_messages: std::collections::VecDeque<(String, String)>,
    remote_players: std::collections::HashMap<String, RemotePlayerData>,
}

pub struct Engine {
    state: Rc<RefCell<GameState>>,
    hud: Rc<RefCell<HudData>>,
    anim_id: Option<i32>,
    _closure: Option<Rc<RefCell<Option<Closure<dyn FnMut()>>>>>,
}

fn uv_sphere(radius: f64, rings: u32, segments: u32) -> (Vec<f32>, Vec<f32>, Vec<u32>) {
    let verts_per_ring = segments + 1;
    let total = verts_per_ring * (rings + 1);
    let mut positions = Vec::with_capacity(total as usize * 3);
    let mut normals = Vec::with_capacity(total as usize * 3);
    for ring in 0..=rings {
        let phi = ring as f64 / rings as f64 * std::f64::consts::PI;
        for seg in 0..=segments {
            let theta = seg as f64 / segments as f64 * std::f64::consts::TAU;
            let x = radius * phi.sin() * theta.cos();
            let y = radius * phi.cos();
            let z = radius * phi.sin() * theta.sin();
            positions.push(x as f32);
            positions.push(y as f32);
            positions.push(z as f32);
            normals.push(x as f32 / radius as f32);
            normals.push(y as f32 / radius as f32);
            normals.push(z as f32 / radius as f32);
        }
    }
    let mut indices = Vec::with_capacity(rings as usize * segments as usize * 6);
    for ring in 0..rings {
        for seg in 0..segments {
            let a = ring * verts_per_ring + seg;
            let b = a + 1;
            let c = (ring + 1) * verts_per_ring + seg;
            let d = c + 1;
            indices.push(a);
            indices.push(c);
            indices.push(b);
            indices.push(b);
            indices.push(c);
            indices.push(d);
        }
    }
    (positions, normals, indices)
}

fn box_mesh(w: f64, h: f64, d: f64) -> (Vec<f32>, Vec<f32>, Vec<u32>) {
    let hw = (w * 0.5) as f32;
    let hh = (h * 0.5) as f32;
    let hd = (d * 0.5) as f32;
    let positions = vec![
        -hw, -hh,  hd,  hw, -hh,  hd,  hw,  hh,  hd, -hw,  hh,  hd,
        -hw, -hh, -hd, -hw,  hh, -hd,  hw,  hh, -hd,  hw, -hh, -hd,
        -hw,  hh, -hd, -hw,  hh,  hd,  hw,  hh,  hd,  hw,  hh, -hd,
        -hw, -hh, -hd,  hw, -hh, -hd,  hw, -hh,  hd, -hw, -hh,  hd,
         hw, -hh, -hd,  hw,  hh, -hd,  hw,  hh,  hd,  hw, -hh,  hd,
        -hw, -hh, -hd, -hw, -hh,  hd, -hw,  hh,  hd, -hw,  hh, -hd,
    ];
    let normals = vec![
        0.0f32, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
        0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0,
        0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0,
        0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0,
        1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0,
        -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0,
    ];
    let indices = vec![
        0u32, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7,
        8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15,
        16, 17, 18, 16, 18, 19, 20, 21, 22, 20, 22, 23,
    ];
    (positions, normals, indices)
}

fn box_pivot_top(w: f64, h: f64, d: f64) -> (Vec<f32>, Vec<f32>, Vec<u32>) {
    let hw = (w * 0.5) as f32;
    let hh = (h * 0.5) as f32;
    let hd = (d * 0.5) as f32;
    let shift = -hh;
    let positions = vec![
        -hw, shift-hh,  hd,  hw, shift-hh,  hd,  hw, shift+hh,  hd, -hw, shift+hh,  hd,
        -hw, shift-hh, -hd, -hw, shift+hh, -hd,  hw, shift+hh, -hd,  hw, shift-hh, -hd,
        -hw, shift+hh, -hd, -hw, shift+hh,  hd,  hw, shift+hh,  hd,  hw, shift+hh, -hd,
        -hw, shift-hh, -hd,  hw, shift-hh, -hd,  hw, shift-hh,  hd, -hw, shift-hh,  hd,
         hw, shift-hh, -hd,  hw, shift+hh, -hd,  hw, shift+hh,  hd,  hw, shift-hh,  hd,
        -hw, shift-hh, -hd, -hw, shift-hh,  hd, -hw, shift+hh,  hd, -hw, shift+hh, -hd,
    ];
    let normals = vec![
        0.0f32, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
        0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0,
        0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0,
        0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0,
        1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0,
        -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0,
    ];
    let indices = vec![
        0u32, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7,
        8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15,
        16, 17, 18, 16, 18, 19, 20, 21, 22, 20, 22, 23,
    ];
    (positions, normals, indices)
}

fn gerstner_y(wx: f64, wz: f64, time: f64) -> f64 {
    // 4 Gerstner waves with varying direction, amplitude, frequency, speed
    let waves = [
        (0.6, 1.2, 1.0, [1.0f64, 0.0]),
        (0.4, 0.8, 0.7, [0.0, 1.0]),
        (0.3, 1.5, 1.2, [0.7071, 0.7071]),
        (0.25, 0.6, 0.9, [-0.5, 0.8660]),
    ];
    let mut y = 0.0;
    for &(amp, freq, speed, dir) in &waves {
        let theta = freq * (dir[0] * wx + dir[1] * wz) + speed * time;
        y += amp * theta.sin();
    }
    y
}

fn generate_sky_dome(radius: f64) -> (Vec<f32>, Vec<f32>, Vec<u32>, Vec<f32>) {
    let rings = 24;
    let segments = 32;
    let vpr = segments + 1;
    let nv = vpr * (rings + 1);
    let mut positions = Vec::with_capacity(nv as usize * 3);
    let mut normals = Vec::with_capacity(nv as usize * 3);
    let mut colors = Vec::with_capacity(nv as usize * 3);
    for ring in 0..=rings {
        let phi = ring as f64 / rings as f64 * std::f64::consts::PI;
        let y = radius * phi.cos();
        let r_sin = radius * phi.sin();
        let t = (y / radius + 1.0) * 0.5; // 0 bottom, 0.5 horizon, 1 top
        let col = if t > 0.5 {
            let tt = (t - 0.5) * 2.0;
            let a = 0.7 + (0.15 - 0.7) * tt;
            let b = 0.85 + (0.30 - 0.85) * tt;
            let c = 0.95 + (0.70 - 0.95) * tt;
            [a as f32, b as f32, c as f32]
        } else {
            let tt = t * 2.0;
            let a = 0.15 + (0.7 - 0.15) * tt;
            let b = 0.12 + (0.85 - 0.12) * tt;
            let c = 0.15 + (0.95 - 0.15) * tt;
            [a as f32, b as f32, c as f32]
        };
        for seg in 0..=segments {
            let theta = seg as f64 / segments as f64 * std::f64::consts::TAU;
            let x = r_sin * theta.cos();
            let z = r_sin * theta.sin();
            positions.push(x as f32);
            positions.push(y as f32);
            positions.push(z as f32);
            let len = (x * x + y * y + z * z).sqrt() as f32;
            normals.push(x as f32 / len);
            normals.push(y as f32 / len);
            normals.push(z as f32 / len);
            colors.push(col[0]);
            colors.push(col[1]);
            colors.push(col[2]);
        }
    }
    let ni = rings as usize * segments as usize * 6;
    let mut indices = Vec::with_capacity(ni);
    for ring in 0..rings {
        for seg in 0..segments {
            let a = ring * vpr + seg;
            let b = a + 1;
            let c = (ring + 1) * vpr + seg;
            let d = c + 1;
            indices.push(a);
            indices.push(c);
            indices.push(b);
            indices.push(b);
            indices.push(c);
            indices.push(d);
        }
    }
    (positions, normals, indices, colors)
}

fn generate_water_grid() -> (Vec<f32>, Vec<f32>, Vec<u32>) {
    let half = 120.0;
    let spacing = 3.0;
    let nx = 81u32;
    let nz = 81u32;
    let nv = nx * nz;
    let mut positions = Vec::with_capacity(nv as usize * 3);
    let mut normals = Vec::with_capacity(nv as usize * 3);
    for iz in 0..nz {
        for ix in 0..nx {
            let lx = -half + ix as f64 * spacing;
            let lz = -half + iz as f64 * spacing;
            positions.push(lx as f32);
            positions.push(0.0);
            positions.push(lz as f32);
            normals.push(0.0);
            normals.push(1.0);
            normals.push(0.0);
        }
    }
    let ni = ((nx - 1) * (nz - 1) * 6) as usize;
    let mut indices = Vec::with_capacity(ni);
    for iz in 0..nz - 1 {
        for ix in 0..nx - 1 {
            let a = iz * nx + ix;
            let b = a + 1;
            let c = (iz + 1) * nx + ix;
            let d = c + 1;
            indices.push(a);
            indices.push(c);
            indices.push(b);
            indices.push(b);
            indices.push(c);
            indices.push(d);
        }
    }
    (positions, normals, indices)
}

fn fill_color(n: usize, r: f32, g: f32, b: f32) -> Vec<f32> {
    let mut v = Vec::with_capacity(n * 3);
    for _ in 0..n { v.push(r); v.push(g); v.push(b); }
    v
}

fn upload_part(key: &str, pos: &[f32], norm: &[f32], idx: &[u32], col: &[f32]) {
    let p_arr = js_sys::Float32Array::from(pos);
    let n_arr = js_sys::Float32Array::from(norm);
    let i_arr = js_sys::Uint32Array::from(idx);
    let c_arr = js_sys::Float32Array::from(col);
    bridge::upload_mesh(key, &p_arr, &n_arr, &i_arr, &c_arr);
}

fn part_position(part_key: &str, char_pos: [f64; 3], ox: f64, oy: f64, oz: f64) {
    bridge::set_mesh_position(part_key, char_pos[0] + ox, char_pos[1] + oy, char_pos[2] + oz);
}

fn regenerate_all(s: &mut GameState) {
    let d = s.params.render_distance as i32;
    let pcx = (s.char_pos[0] / CHUNK_SIZE) as i32;
    let pcz = (s.char_pos[2] / CHUNK_SIZE) as i32;
    for old in s.chunks.drain(..) {
        bridge::remove_mesh(&format!("chunk_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("veg_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("crea_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("struct_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("road_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("mineral_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("portal_{},{}", old.cx, old.cz));
    }
    s.waterfalls.remove_all();
    s.foam.remove();
    s.bubbles.remove();
    s.veg_chunks.clear();
    let mut new_chunks: Vec<ChunkData> = Vec::new();
    for x in (pcx - d)..=(pcx + d) {
        for z in (pcz - d)..=(pcz + d) {
            let data = chunk::compute_chunk_data(&s.params, x, z, &s.mined_blocks);
            let key = format!("chunk_{},{}", data.cx, data.cz);
            let pos_arr = js_sys::Float32Array::from(&data.positions[..]);
            let norm_arr = js_sys::Float32Array::from(&data.normals[..]);
            let col_arr = js_sys::Float32Array::from(&data.colors[..]);
            let idx_arr = js_sys::Uint32Array::from(&data.indices[..]);
            bridge::upload_mesh(&key, &pos_arr, &norm_arr, &idx_arr, &col_arr);
            let veg_data = compute_chunk_vegetation(&s.params, x, z);
            if !veg_data.instances.is_empty() {
                let vkey = format!("veg_{},{}", x, z);
                let (vpos, vnorm, vidx, vcol) = generate_veg_mesh_from_data(&veg_data, 1);
                let vp = js_sys::Float32Array::from(&vpos[..]);
                let vn = js_sys::Float32Array::from(&vnorm[..]);
                let vi = js_sys::Uint32Array::from(&vidx[..]);
                let vc = js_sys::Float32Array::from(&vcol[..]);
                bridge::upload_mesh(&vkey, &vp, &vn, &vi, &vc);
            }
            s.veg_chunks.insert((x, z), veg_data);
            if let Some((cpos, cnorm, cidx, ccol)) = generate_creature_mesh(&s.params, x, z) {
                let ckey = format!("crea_{},{}", x, z);
                let cp = js_sys::Float32Array::from(&cpos[..]);
                let cn = js_sys::Float32Array::from(&cnorm[..]);
                let ci = js_sys::Uint32Array::from(&cidx[..]);
                let cc = js_sys::Float32Array::from(&ccol[..]);
                bridge::upload_mesh(&ckey, &cp, &cn, &ci, &cc);
            }
            if let Some((spos, snorm, sidx, scol)) = generate_struct_mesh(&s.params, x, z) {
                let skey = format!("struct_{},{}", x, z);
                let sp = js_sys::Float32Array::from(&spos[..]);
                let sn = js_sys::Float32Array::from(&snorm[..]);
                let si = js_sys::Uint32Array::from(&sidx[..]);
                let sc = js_sys::Float32Array::from(&scol[..]);
                bridge::upload_mesh(&skey, &sp, &sn, &si, &sc);
            }
            if let Some((mpos, mnorm, midx, mcol)) = generate_mineral_mesh(&s.params, x, z) {
                let mkey = format!("mineral_{},{}", x, z);
                let mp = js_sys::Float32Array::from(&mpos[..]);
                let mn = js_sys::Float32Array::from(&mnorm[..]);
                let mi = js_sys::Uint32Array::from(&midx[..]);
                let mc = js_sys::Float32Array::from(&mcol[..]);
                bridge::upload_mesh(&mkey, &mp, &mn, &mi, &mc);
            }
            if let Some((ppos, pnorm, pidx, pcol)) = generate_portal_mesh(&s.params, x, z) {
                let pkey = format!("portal_{},{}", x, z);
                let pp = js_sys::Float32Array::from(&ppos[..]);
                let pn = js_sys::Float32Array::from(&pnorm[..]);
                let pi = js_sys::Uint32Array::from(&pidx[..]);
                let pc = js_sys::Float32Array::from(&pcol[..]);
                bridge::upload_mesh(&pkey, &pp, &pn, &pi, &pc);
            }
            if let Some((rpos, rnorm, ridx, rcol)) = generate_road_mesh(&s.params, x, z) {
                let rkey = format!("road_{},{}", x, z);
                let rp = js_sys::Float32Array::from(&rpos[..]);
                let rn = js_sys::Float32Array::from(&rnorm[..]);
                let ri = js_sys::Uint32Array::from(&ridx[..]);
                let rc = js_sys::Float32Array::from(&rcol[..]);
                bridge::upload_mesh(&rkey, &rp, &rn, &ri, &rc);
            }
            new_chunks.push(data);
        }
    }
    s.chunks = new_chunks;
    s.prev_cx = pcx;
    s.prev_cz = pcz;
}

fn lod_for_distance(dist: i32) -> u32 {
    if dist <= 2 { 0 }
    else if dist <= 4 { 1 }
    else { 2 }
}

fn generate_chunks(s: &mut GameState, cx: i32, cz: i32) {
    s.prev_cx = cx;
    s.prev_cz = cz;
    let d = s.params.render_distance as i32;
    let mut new_chunks: Vec<ChunkData> = Vec::new();
    let mut to_compute: Vec<(i32, i32)> = Vec::new();

    for x in (cx - d)..=(cx + d) {
        for z in (cz - d)..=(cz + d) {
            let key = (x, z);
            if let Some(idx) = s.chunks.iter().position(|c| c.key() == key) {
                new_chunks.push(s.chunks.swap_remove(idx));
            } else {
                to_compute.push(key);
            }
        }
    }

    for old in s.chunks.drain(..) {
        bridge::remove_mesh(&format!("chunk_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("veg_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("crea_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("struct_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("road_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("mineral_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("portal_{},{}", old.cx, old.cz));
        s.veg_chunks.remove(&(old.cx, old.cz));
    }

    for &(cx, cz) in &to_compute {
        let dx = (cx - s.prev_cx).abs().max((cz - s.prev_cz).abs());
        let lod = lod_for_distance(dx);

        // Use LOD chunk generation for distant chunks
        let data = if lod > 0 {
            chunk::compute_chunk_data_lod(&s.params, cx, cz, &s.mined_blocks, lod)
        } else {
            chunk::compute_chunk_data(&s.params, cx, cz, &s.mined_blocks)
        };
        let key = format!("chunk_{},{}", data.cx, data.cz);
        let pos_arr = js_sys::Float32Array::from(&data.positions[..]);
        let norm_arr = js_sys::Float32Array::from(&data.normals[..]);
        let col_arr = js_sys::Float32Array::from(&data.colors[..]);
        let idx_arr = js_sys::Uint32Array::from(&data.indices[..]);
        bridge::upload_mesh(&key, &pos_arr, &norm_arr, &idx_arr, &col_arr);
        bridge::set_mesh_frustum_culled(&key, false);

        // Skip vegetation/creatures/structures/minerals for far LOD chunks (LOD 2)
        if lod < 2 {
            // Vegetation mesh for this chunk
            let veg_data = compute_chunk_vegetation(&s.params, cx, cz);
            if !veg_data.instances.is_empty() {
                let vkey = format!("veg_{},{}", cx, cz);
                let (vpos, vnorm, vidx, vcol) = generate_veg_mesh_from_data(&veg_data, 1);
                let vp = js_sys::Float32Array::from(&vpos[..]);
                let vn = js_sys::Float32Array::from(&vnorm[..]);
                let vi = js_sys::Uint32Array::from(&vidx[..]);
                let vc = js_sys::Float32Array::from(&vcol[..]);
                bridge::upload_mesh(&vkey, &vp, &vn, &vi, &vc);
            }
            s.veg_chunks.insert((cx, cz), veg_data);

            // Creature mesh for this chunk
            if let Some((cpos, cnorm, cidx, ccol)) = generate_creature_mesh(&s.params, cx, cz) {
                let ckey = format!("crea_{},{}", cx, cz);
                let cp = js_sys::Float32Array::from(&cpos[..]);
                let cn = js_sys::Float32Array::from(&cnorm[..]);
                let ci = js_sys::Uint32Array::from(&cidx[..]);
                let cc = js_sys::Float32Array::from(&ccol[..]);
                bridge::upload_mesh(&ckey, &cp, &cn, &ci, &cc);
            }

            // Structure mesh for this chunk
            if let Some((spos, snorm, sidx, scol)) = generate_struct_mesh(&s.params, cx, cz) {
                let skey = format!("struct_{},{}", cx, cz);
                let sp = js_sys::Float32Array::from(&spos[..]);
                let sn = js_sys::Float32Array::from(&snorm[..]);
                let si = js_sys::Uint32Array::from(&sidx[..]);
                let sc = js_sys::Float32Array::from(&scol[..]);
                bridge::upload_mesh(&skey, &sp, &sn, &si, &sc);
            }

            // Mineral mesh for this chunk
            if let Some((mpos, mnorm, midx, mcol)) = generate_mineral_mesh(&s.params, cx, cz) {
                let mkey = format!("mineral_{},{}", cx, cz);
                let mp = js_sys::Float32Array::from(&mpos[..]);
                let mn = js_sys::Float32Array::from(&mnorm[..]);
                let mi = js_sys::Uint32Array::from(&midx[..]);
                let mc = js_sys::Float32Array::from(&mcol[..]);
                bridge::upload_mesh(&mkey, &mp, &mn, &mi, &mc);
            }

            // Portal mesh for this chunk
            if let Some((ppos, pnorm, pidx, pcol)) = generate_portal_mesh(&s.params, cx, cz) {
                let pkey = format!("portal_{},{}", cx, cz);
                let pp = js_sys::Float32Array::from(&ppos[..]);
                let pn = js_sys::Float32Array::from(&pnorm[..]);
                let pi = js_sys::Uint32Array::from(&pidx[..]);
                let pc = js_sys::Float32Array::from(&pcol[..]);
                bridge::upload_mesh(&pkey, &pp, &pn, &pi, &pc);
            }

            // Road mesh for this chunk
            if let Some((rpos, rnorm, ridx, rcol)) = generate_road_mesh(&s.params, cx, cz) {
                let rkey = format!("road_{},{}", cx, cz);
                let rp = js_sys::Float32Array::from(&rpos[..]);
                let rn = js_sys::Float32Array::from(&rnorm[..]);
                let ri = js_sys::Uint32Array::from(&ridx[..]);
                let rc = js_sys::Float32Array::from(&rcol[..]);
                bridge::upload_mesh(&rkey, &rp, &rn, &ri, &rc);
            }
        }

        new_chunks.push(data);
    }

    s.chunks = new_chunks;
}

fn regenerate_character(s: &mut GameState) {
    for key in &["char_body", "char_head", "char_arm_l", "char_arm_r", "char_leg_l", "char_leg_r", "char_tent_1", "char_tent_2", "char_tent_3", "char_tent_4"] {
        bridge::remove_mesh(key);
    }
    let scale = s.params.char_scale;
    let (preset_body, preset_head, preset_arm, preset_leg, body_col, head_col) = match s.params.character {
        CharacterPreset::Human => (
            (0.7, 1.0, 0.4), (0.5, 10, 10, 0.0),
            (0.2, 0.7, 0.2), (0.25, 0.7, 0.25),
            [0.2, 0.4, 0.8], [1.0, 0.85, 0.75],
        ),
        CharacterPreset::Robot => (
            (0.8, 1.0, 0.5), (0.4, 8, 8, 1.0),
            (0.3, 0.6, 0.3), (0.3, 0.6, 0.3),
            [0.5, 0.5, 0.55], [0.7, 0.7, 0.75],
        ),
        CharacterPreset::Beast => (
            (0.8, 0.6, 1.2), (0.45, 8, 8, 0.0),
            (0.2, 0.5, 0.2), (0.3, 0.55, 0.3),
            [0.45, 0.35, 0.25], [0.55, 0.4, 0.25],
        ),
        CharacterPreset::Ghost => (
            (0.6, 0.9, 0.3), (0.4, 10, 10, 0.0),
            (0.15, 0.6, 0.15), (0.2, 0.6, 0.2),
            [0.3, 0.5, 0.7], [0.5, 0.7, 0.9],
        ),
        CharacterPreset::Teddy => (
            (0.85, 0.8, 0.6), (0.48, 10, 10, 0.0),
            (0.22, 0.5, 0.22), (0.28, 0.5, 0.28),
            [0.55, 0.35, 0.15], [0.75, 0.65, 0.45],
        ),
        CharacterPreset::Panda => (
            (0.9, 0.85, 0.65), (0.52, 10, 10, 0.0),
            (0.28, 0.55, 0.28), (0.33, 0.55, 0.33),
            [0.95, 0.95, 0.9], [0.95, 0.95, 0.9],
        ),
        CharacterPreset::Kraken => (
            (0.7, 0.5, 0.7), (0.55, 12, 12, 0.0),
            (0.15, 0.55, 0.15), (0.12, 0.7, 0.12),
            [0.35, 0.15, 0.4], [0.2, 0.7, 0.3],
        ),
    };
    let schemes: &[([f32; 3], [f32; 3])] = &[
        ([0.2, 0.4, 0.8], [1.0, 0.85, 0.75]),
        ([0.1, 0.1, 0.3], [0.8, 0.7, 0.6]),
        ([0.7, 0.1, 0.1], [0.9, 0.7, 0.6]),
        ([0.1, 0.6, 0.2], [0.8, 0.9, 0.7]),
        ([0.8, 0.6, 0.1], [0.9, 0.85, 0.7]),
        ([0.4, 0.1, 0.6], [0.9, 0.8, 0.9]),
        ([0.9, 0.9, 0.9], [0.9, 0.9, 0.9]),
    ];
    let si = (s.params.color_scheme as usize) % schemes.len();
    let (body_col, head_col) = if s.params.color_scheme > 0 {
        schemes[si]
    } else {
        (body_col, head_col)
    };
    let (bw, bh, bd) = preset_body;
    let (hs, hr, hseg, hbox) = preset_head;
    let (aw, ah, ad) = preset_arm;
    let (lw, lh, ld) = preset_leg;

    let (b_pos, b_norm, b_idx) = box_mesh(bw * scale, bh * scale, bd * scale);
    let b_col = fill_color(b_pos.len() / 3, body_col[0], body_col[1], body_col[2]);
    upload_part("char_body", &b_pos, &b_norm, &b_idx, &b_col);

    let (h_pos, h_norm, h_idx) = if hbox > 0.0 {
        box_mesh(hs * scale * 0.8, hs * scale, hs * scale * 0.7)
    } else {
        uv_sphere(hr as f64 * scale, hseg as u32, hseg as u32)
    };
    let h_col = fill_color(h_pos.len() / 3, head_col[0], head_col[1], head_col[2]);
    upload_part("char_head", &h_pos, &h_norm, &h_idx, &h_col);

    let (a_pos, a_norm, a_idx) = box_pivot_top(aw * scale, ah * scale, ad * scale);
    let a_col = fill_color(a_pos.len() / 3, head_col[0], head_col[1], head_col[2]);
    upload_part("char_arm_l", &a_pos, &a_norm, &a_idx, &a_col);
    upload_part("char_arm_r", &a_pos, &a_norm, &a_idx, &a_col);

    let (l_pos, l_norm, l_idx) = box_pivot_top(lw * scale, lh * scale, ld * scale);
    let l_col = fill_color(l_pos.len() / 3, body_col[0], body_col[1], body_col[2]);
    upload_part("char_leg_l", &l_pos, &l_norm, &l_idx, &l_col);
    upload_part("char_leg_r", &l_pos, &l_norm, &l_idx, &l_col);

    if s.params.character == CharacterPreset::Kraken {
        let (t_pos, t_norm, t_idx) = box_pivot_top(0.1 * scale, 0.7 * scale, 0.1 * scale);
        let t_col = fill_color(t_pos.len() / 3, 0.2, 0.7, 0.3);
        for i in 1..=4 {
            upload_part(&format!("char_tent_{}", i), &t_pos, &t_norm, &t_idx, &t_col);
        }
    }
}

impl Engine {
    pub fn new(canvas: web_sys::HtmlCanvasElement, params: WorldParams) -> Result<Self, String> {
        let joy_dx = Rc::new(Cell::new(0.0));
        let joy_dy = Rc::new(Cell::new(0.0));
        canvas.set_tab_index(0);

        bridge::init(&canvas);

        let yaw = Rc::new(Cell::new(0.0));
        let pitch = Rc::new(Cell::new(0.0));
        let mut controls = Controls::new(yaw.clone(), pitch.clone());
        controls.attach(&canvas);
        let camera = Camera::new(yaw.clone(), pitch.clone());

        let (b_pos, b_norm, b_idx) = box_mesh(0.7, 1.0, 0.4);
        let b_col = fill_color(b_pos.len() / 3, 0.2, 0.4, 0.8);
        upload_part("char_body", &b_pos, &b_norm, &b_idx, &b_col);

        let (h_pos, h_norm, h_idx) = uv_sphere(0.5, 10, 10);
        let h_col = fill_color(h_pos.len() / 3, 1.0, 0.85, 0.75);
        upload_part("char_head", &h_pos, &h_norm, &h_idx, &h_col);

        let (a_pos, a_norm, a_idx) = box_pivot_top(0.2, 0.7, 0.2);
        let a_col = fill_color(a_pos.len() / 3, 1.0, 0.85, 0.75);
        upload_part("char_arm_l", &a_pos, &a_norm, &a_idx, &a_col);
        upload_part("char_arm_r", &a_pos, &a_norm, &a_idx, &a_col);

        let (l_pos, l_norm, l_idx) = box_pivot_top(0.25, 0.7, 0.25);
        let l_col = fill_color(l_pos.len() / 3, 0.2, 0.4, 0.8);
        upload_part("char_leg_l", &l_pos, &l_norm, &l_idx, &l_col);
        upload_part("char_leg_r", &l_pos, &l_norm, &l_idx, &l_col);

        // Sky dome
        let (sk_pos, sk_norm, sk_idx, sk_col) = generate_sky_dome(200.0);
        let sk_p = js_sys::Float32Array::from(&sk_pos[..]);
        let sk_n = js_sys::Float32Array::from(&sk_norm[..]);
        let sk_i = js_sys::Uint32Array::from(&sk_idx[..]);
        let sk_c = js_sys::Float32Array::from(&sk_col[..]);
        bridge::upload_sky_mesh("sky_dome", &sk_p, &sk_n, &sk_i, &sk_c);

        // Water grid
        let (w_pos, w_norm, w_idx) = generate_water_grid();
        let w_p = js_sys::Float32Array::from(&w_pos[..]);
        let w_n = js_sys::Float32Array::from(&w_norm[..]);
        let w_i = js_sys::Uint32Array::from(&w_idx[..]);
        bridge::upload_water_mesh("water", &w_p, &w_n, &w_i);

        // Fog and sun
        bridge::set_fog(0.6, 0.75, 0.92, 0.006);
        bridge::set_sun_light(50.0, 80.0, 50.0, 1.0, 0.95, 0.85, 2.0);

        // Particle system (initialized with default zone, will update on first frame)
        let zone_init = terrain::get_zone(&params, 0.0, 0.0);
        let (pr, pg, pb, ps) = particles::particle_color_size(zone_init);
        let pcount = particles::particle_count(zone_init);
        let particles_sys = if pcount > 0 {
            let psys = ParticleSystem::new("particles", pcount, pr, pg, pb, ps);
            Some(psys)
        } else {
            None
        };

        // Star field
        let star_count = 1500u32;
        let mut star_pos = Vec::with_capacity(star_count as usize * 3);
        for i in 0..star_count {
            let theta = (i as f64 / star_count as f64) * std::f64::consts::TAU;
            let phi = ((i as f64 * 7.0 / star_count as f64) * std::f64::consts::PI).sin().asin();
            let r = 180.0;
            star_pos.push((r * phi.cos() * theta.cos()) as f32);
            star_pos.push((r * phi.sin()) as f32);
            star_pos.push((r * phi.cos() * theta.sin()) as f32);
        }
        let sp_arr = js_sys::Float32Array::from(&star_pos[..]);
        bridge::create_particles("stars", star_count, 1.0, 1.0, 1.0, 0.5);
        bridge::update_particles("stars", &sp_arr);
        bridge::set_particles_opacity("stars", 0.0);

        let init_ground = terrain::get_height(&params, 0.0, 0.0);
        let char_pos = [0.0, init_ground, 0.0];
        let init_yaw = yaw.get();
        let init_pitch = pitch.get();
        let init_cam_x = char_pos[0] + ARM_LENGTH * init_pitch.cos() * init_yaw.sin();
        let init_cam_z = char_pos[2] + ARM_LENGTH * init_pitch.cos() * init_yaw.cos();
        let init_cam_y = char_pos[1] + ARM_HEIGHT + ARM_LENGTH * init_pitch.sin();
        let cam_pos = [init_cam_x, init_cam_y.max(init_ground + 0.5), init_cam_z];

        let state = Rc::new(RefCell::new(GameState {
            canvas,
            camera,
            controls,
            params,
            chunks: Vec::new(),
            prev_cx: i32::MAX,
            prev_cz: i32::MAX,
            last_time: 0.0,
            frame_count: 0,
            fps_timer: 0.0,
            fps: 0,
            joy_dx: joy_dx.clone(),
            joy_dy: joy_dy.clone(),
            speed: 300.0,
            char_pos,
            vel_x: 0.0,
            vel_z: 0.0,
            vel_y: 0.0,
            cam_pos,
            time: 0.0,
            day_time: 1.5, // start near noon
            walk_time: 0.0,
            char_yaw: 0.0,
            particles: particles_sys,
            tour: TourState::new(),
            gamepad_state: gamepad::GamepadState::default(),
            inventory: Inventory::new(),
            tour_prev: false,
            spawned: false,
            weather_power: 0.0,
            weather_target: 0.0,
            weather_timer: 0.0,
            lightning_flash: 0.0,
            veg_chunks: std::collections::HashMap::new(),
            placed_blocks: load_blocks(),
            mined_blocks: load_mined(),
            block_inventory: vec![(0, 64), (1, 32), (2, 16), (3, 16), (4, 8), (5, 8), (6, 8), (7, 8), (8, 8)],
            build_prev: false,
            slot_prev: 0,
            save_timer: 0.0,
            params_dirty: false,
            char_dirty: false,
            portal_prev: false,
            g_prev: false,
            waterfalls: WaterfallSystem::new(),
            foam: FoamSystem::new(),
            bubbles: BubbleSystem::new(),
            codex: Codex::new(),
            achievements: AchievementState::new(),
            discovered_biomes: Vec::new(),
            total_distance: 0.0,
            weather_power_idx: 0,
            weather_cooldown: 0.0,
            break_counter: 0,
            break_particles: Vec::new(),
            ws_connected: false,
            ws_player_id: String::new(),
            ws_send_timer: 0.0,
            chat_messages: std::collections::VecDeque::new(),
            remote_players: std::collections::HashMap::new(),
        }));

        {
            let mut s = state.borrow_mut();
            let cx = (s.char_pos[0] / CHUNK_SIZE) as i32;
            let cz = (s.char_pos[2] / CHUNK_SIZE) as i32;
            generate_chunks(&mut s, cx, cz);
        }

        let hud = Rc::new(RefCell::new(HudData::default()));

        Ok(Self {
            state,
            hud,
            anim_id: None,
            _closure: None,
        })
    }

    pub fn update_params(&mut self, params: &WorldParams) {
        let mut s = self.state.borrow_mut();
        s.controls.set_sensitivity(params.mouse_sensitivity);
        s.speed = params.speed;
        let dirty = s.params.seed != params.seed || s.params.scale != params.scale
            || s.params.amplitude != params.amplitude
            || s.params.octaves != params.octaves || s.params.water_level != params.water_level
            || s.params.zone != params.zone || s.params.mutation != params.mutation
            || s.params.render_distance != params.render_distance
            || s.params.canyons != params.canyons
            || s.params.hue_shift != params.hue_shift || s.params.saturation != params.saturation
            || s.params.lightness != params.lightness || s.params.param_a != params.param_a
            || s.params.param_b != params.param_b;
        let part_dirty = s.params.particle_mode != params.particle_mode;
        let char_dirty = s.params.character != params.character
            || s.params.color_scheme != params.color_scheme
            || s.params.char_scale != params.char_scale;
        s.params = *params;
        if dirty {
            s.params_dirty = true;
        }
        if char_dirty {
            s.char_dirty = true;
        }
        if part_dirty {
            if let Some(p) = s.particles.take() { p.remove(); }
        }
    }

    pub fn start(&mut self) {
        let state = self.state.clone();
        let hud = self.hud.clone();
        let closure = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
        let closure2 = closure.clone();

        *closure.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
            let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
                let mut s = state.borrow_mut();

                let now = web_sys::window().unwrap().performance().unwrap().now();
                if s.last_time == 0.0 {
                    s.last_time = now;
                }
                let delta = ((now - s.last_time) / 1000.0).min(0.1);
                s.last_time = now;

                s.frame_count += 1;
                if now - s.fps_timer > 1000.0 {
                    s.fps = s.frame_count;
                    s.frame_count = 0;
                    s.fps_timer = now;
                }

                s.time += delta;

                // Gamepad input
                s.gamepad_state = poll_gamepad();
                if s.gamepad_state.connected {
                    let gp = &s.gamepad_state;
                    let mut k = s.controls.keys.get();
                    k &= !(MASK_W | MASK_A | MASK_S | MASK_D | MASK_SPACE | MASK_SHIFT);
                    if gp.axes[1] < -0.3 { k |= MASK_W; }
                    if gp.axes[1] > 0.3 { k |= MASK_S; }
                    if gp.axes[0] < -0.3 { k |= MASK_A; }
                    if gp.axes[0] > 0.3 { k |= MASK_D; }
                    if gp.a { k |= MASK_SPACE; }
                    if gp.rb { k |= MASK_SHIFT; }
                    if gp.start { let _ = s.canvas.request_pointer_lock(); }
                    s.controls.keys.set(k);
                    // Right stick -> camera
                    let sens = s.controls.sensitivity.get();
                    s.controls.yaw.set(s.controls.yaw.get() + gp.axes[2] * 0.03 * sens);
                    let p = (s.controls.pitch.get() - gp.axes[3] * 0.03 * sens).max(-1.5).min(1.5);
                    s.controls.pitch.set(p);
                }

                // Tour mode toggle (edge detection)
                let keys = s.controls.keys.get();
                let tour_pressed = keys & MASK_T != 0;
                let tour_yaw_toggle = s.controls.yaw.get();
                let tour_pitch_toggle = s.controls.pitch.get();
                if tour_pressed && !s.tour_prev {
                    if s.tour.active == tour::TourMode::None {
                        s.tour.start_orbit(tour_yaw_toggle, tour_pitch_toggle, 25.0);
                    } else {
                        s.tour.stop();
                    }
                }
                s.tour_prev = tour_pressed;

                // Build mode toggle (edge detection)
                let build_pressed = keys & MASK_B != 0;
                if build_pressed && !s.build_prev {
                    s.params.build_mode = !s.params.build_mode;
                }
                s.build_prev = build_pressed;

                // Slot selection with number keys (edge detection)
                let slot_masks = [MASK_1, MASK_2, MASK_3, MASK_4, MASK_5, MASK_6, MASK_7, MASK_8, MASK_9];
                for (i, &mask) in slot_masks.iter().enumerate() {
                    if keys & mask != 0 && s.slot_prev & mask == 0 {
                        s.inventory.selected_slot = i as u8;
                    }
                }
                s.slot_prev = keys;

                let cam_yaw = s.controls.yaw.get();
                let keys = s.controls.keys.get();
                let water_surface = s.params.water_level;
                let in_water = !s.params.fly_mode && s.char_pos[1] + 1.0 < water_surface;
                let speed = s.speed * delta * if in_water { 0.5 } else { 1.0 };
                let move_yaw = if s.params.camera_mode == CameraMode::FirstPerson { -cam_yaw } else { cam_yaw };
                let (sy, cy) = move_yaw.sin_cos();
                let mut mx = 0.0;
                let mut mz = 0.0;
                if keys & MASK_W != 0 { mx -= sy; mz -= cy; }
                if keys & MASK_S != 0 { mx += sy; mz += cy; }
                if keys & MASK_A != 0 { mx -= cy; mz += sy; }
                if keys & MASK_D != 0 { mx += cy; mz -= sy; }

                let moving = keys & (MASK_W | MASK_S | MASK_A | MASK_D) != 0;
                let running = moving && keys & MASK_SHIFT != 0;

                if moving {
                    let len = (mx * mx + mz * mz).sqrt();
                    if len > 0.0 { mx /= len; mz /= len; }
                    s.vel_x += mx * s.params.movement_accel * delta;
                    s.vel_z += mz * s.params.movement_accel * delta;
                    let target_yaw = (-mx).atan2(-mz);
                    s.char_yaw += (target_yaw - s.char_yaw) * 0.12;
                } else {
                    let spd = (s.vel_x * s.vel_x + s.vel_z * s.vel_z).sqrt();
                    if spd > 0.0 {
                        let f = (s.params.movement_friction * delta).min(spd);
                        s.vel_x -= (s.vel_x / spd) * f;
                        s.vel_z -= (s.vel_z / spd) * f;
                    }
                }

                let spd = (s.vel_x * s.vel_x + s.vel_z * s.vel_z).sqrt();
                if spd > speed {
                    s.vel_x = (s.vel_x / spd) * speed;
                    s.vel_z = (s.vel_z / spd) * speed;
                }

                let steps = ((s.vel_x.abs() + s.vel_z.abs()) * delta / 0.8).ceil().max(1.0) as u32;
                let step_vx = s.vel_x * delta / steps as f64;
                let step_vz = s.vel_z * delta / steps as f64;
                let old_x = s.char_pos[0];
                let old_z = s.char_pos[2];
                for _ in 0..steps {
                    let nx = s.char_pos[0] + step_vx;
                    let nz = s.char_pos[2] + step_vz;
                    let gh2 = terrain::get_height(&s.params, s.char_pos[0], s.char_pos[2]);
                    let h_t2 = terrain::get_height(&s.params, nx, nz);
                    let diff2 = h_t2 - gh2;
                    if diff2 <= s.params.step_height
                        && !collides_with_veg(nx, nz, &s.veg_chunks)
                    {
                        s.char_pos[0] = nx;
                        s.char_pos[2] = nz;
                    } else {
                        let h_x = terrain::get_height(&s.params, nx, s.char_pos[2]);
                        if h_x - gh2 <= s.params.step_height
                            && !collides_with_veg(nx, s.char_pos[2], &s.veg_chunks)
                        {
                            s.char_pos[0] = nx;
                        }
                        let h_z = terrain::get_height(&s.params, s.char_pos[0], nz);
                        if h_z - gh2 <= s.params.step_height
                            && !collides_with_veg(s.char_pos[0], nz, &s.veg_chunks)
                        {
                            s.char_pos[2] = nz;
                        }
                        break;
                    }
                }
                if s.char_pos[0] == old_x && s.char_pos[2] == old_z {
                    s.vel_x = 0.0;
                    s.vel_z = 0.0;
                } else {
                    let dx = s.char_pos[0] - old_x;
                    let dz = s.char_pos[2] - old_z;
                    s.total_distance += (dx * dx + dz * dz).sqrt();
                    let td = s.total_distance;
                    s.achievements.check_distance(td);
                }

                if keys & MASK_Q != 0 {
                    s.controls.yaw.set(cam_yaw - ROT_SPEED * delta);
                }
                if keys & MASK_E != 0 {
                    s.controls.yaw.set(cam_yaw + ROT_SPEED * delta);
                }

                let ground_y = terrain::get_height(&s.params, s.char_pos[0], s.char_pos[2]);
                if s.params.fly_mode {
                    // Flying: no gravity, space=up, shift=down
                    if keys & MASK_SPACE != 0 { s.char_pos[1] += speed * 1.5; }
                    if keys & MASK_SHIFT != 0 { s.char_pos[1] -= speed * 1.5; }
                    s.vel_y = 0.0;
                } else if s.char_pos[1] + 1.0 < water_surface {
                    // Swimming / underwater
                    if keys & MASK_SPACE != 0 { s.vel_y = 3.0; }
                    if keys & MASK_SHIFT != 0 { s.vel_y = -2.0; }
                    let buoyancy = (water_surface - s.char_pos[1]).max(0.0) * 0.5;
                    s.vel_y += buoyancy * delta;
                    s.vel_y -= s.params.gravity * 0.3 * delta;
                    s.char_pos[1] += s.vel_y * delta;
                    if s.char_pos[1] > water_surface {
                        s.char_pos[1] = water_surface;
                        s.vel_y = 0.0;
                    }
                    if s.char_pos[1] < ground_y {
                        s.char_pos[1] = ground_y;
                        s.vel_y = 0.0;
                    }
                } else {
                    // Normal ground movement
                    if keys & MASK_SPACE != 0 && s.char_pos[1] <= ground_y + 0.1 {
                        s.vel_y = s.params.jump_speed;
                        audio::play_tone(400.0, 0.1);
                    }
                    s.vel_y -= s.params.gravity * delta;
                    s.char_pos[1] += s.vel_y * delta;
                    if s.char_pos[1] < ground_y {
                        s.char_pos[1] = ground_y;
                        s.vel_y = 0.0;
                    }
                }

                // Build mode: block interaction with raycast
                if s.params.build_mode {
                    let sel_slot = s.inventory.selected_slot;
                    let rdx = s.char_pos[0] - s.cam_pos[0];
                    let rdy = s.char_pos[1] - s.cam_pos[1];
                    let rdz = s.char_pos[2] - s.cam_pos[2];
                    if let Some(hit) = raycast_block(s.cam_pos[0], s.cam_pos[1], s.cam_pos[2], rdx, rdy, rdz, 12.0, &s.placed_blocks, &s.mined_blocks, &s.params) {
                        let (hx, hy, hz) = hit.0;
                        let key = (hx, hy, hz);
                        if keys & MASK_LCLICK != 0 {
                            // Try to place a block at the targeted position
                            let can_place = match hit.1 {
                                true => false,
                                false => {
                                    let dist2 = (s.char_pos[0] - (hx as f64 + 0.5)).powi(2) + (s.char_pos[1] - (hy as f64 + 0.5)).powi(2) + (s.char_pos[2] - (hz as f64 + 0.5)).powi(2);
                                    dist2 > 1.5
                                }
                            };
                            if can_place && !s.placed_blocks.contains_key(&key) {
                                if let Some(slot) = s.block_inventory.iter().find(|(t, _)| *t == sel_slot) {
                                    if slot.1 > 0 {
                                        s.placed_blocks.insert(key, sel_slot);
                                        let bkey = format!("b_{}_{}_{}", hx, hy, hz);
                                        let block_colors = [[0.6,0.45,0.3],[0.5,0.5,0.5],[0.55,0.35,0.15],[0.2,0.6,0.2],[0.7,0.4,1.0],[0.8,0.3,0.05],[0.7,0.9,1.0],[0.85,0.75,0.5],[0.3,0.5,0.2]];
                                        let bcol = if sel_slot == 2 { [0.9, 0.6, 0.1] } else { block_colors[sel_slot as usize % block_colors.len()] };
                                        let is_torch = sel_slot == 2;
                                        let (pos, nrm, idx) = box_mesh(1.0, 1.0, 1.0);
                                        let col = fill_color(24, bcol[0], bcol[1], bcol[2]);
                                        let bp = js_sys::Float32Array::from(&pos[..]);
                                        let bn = js_sys::Float32Array::from(&nrm[..]);
                                        let bi = js_sys::Uint32Array::from(&idx[..]);
                                        let bc_arr = js_sys::Float32Array::from(&col[..]);
                                        bridge::upload_mesh(&bkey, &bp, &bn, &bi, &bc_arr);
                                        bridge::set_mesh_position(&bkey, hx as f64 + 0.5, hy as f64 + 0.5, hz as f64 + 0.5);
                                        if let Some(slot) = s.block_inventory.iter_mut().find(|(t, _)| *t == sel_slot) {
                                            slot.1 -= 1;
                                        }
                                        s.achievements.blocks_placed += 1;
                                        let bp = s.achievements.blocks_placed;
                                        s.achievements.check_build(bp);
                                    }
                                }
                            }
                        }
                        if keys & MASK_RCLICK != 0 {
                            let broke = if hit.1 {
                                s.placed_blocks.remove(&key).is_some()
                            } else {
                                !s.mined_blocks.contains(&key)
                            };
                            if broke {
                                if hit.1 {
                                    let bkey = format!("b_{}_{}_{}", hx, hy, hz);
                                    bridge::remove_mesh(&bkey);
                                } else {
                                    let wx = hx as f64 + 0.5;
                                    let wz = hz as f64 + 0.5;
                                    let zone = terrain::get_zone(&s.params, wx, wz);
                                    let mut surface_h = terrain::get_height(&s.params, wx, wz);
                                    terrain::zone_effects(&s.params, wx, wz, &mut surface_h);
                                    let bt = terrain::get_block_type(&s.params, wx, hy as f64 + 0.5, wz, surface_h, zone);
                                    // Drop mineral based on block type
                                    let mineral = match bt {
                                        terrain::BLK_STONE | terrain::BLK_DIRT | terrain::BLK_GRAVEL => Some(0),
                                        terrain::BLK_COAL_ORE => Some(2),
                                        terrain::BLK_IRON_ORE => Some(0),
                                        terrain::BLK_GOLD_ORE => Some(4),
                                        terrain::BLK_DIAMOND_ORE => Some(3),
                                        terrain::BLK_SAND => Some(7),
                                        _ => None,
                                    };
                                    let dist2 = (s.char_pos[0] - wx).powi(2) + (s.char_pos[1] - (hy as f64 + 0.5)).powi(2) + (s.char_pos[2] - wz).powi(2);
                                    if dist2 < 6.0 {
                                        if let Some(mt) = mineral {
                                            s.inventory.add_mineral(mt, 1);
                                            s.codex.discover_mineral(mt);
                                        }
                                    }
                                    s.mined_blocks.insert(key);
                                    s.params_dirty = true;
                                    s.achievements.try_unlock(achievements::Achievement::FirstMine);
                                }
                                // Particle burst
                                let pkey = format!("break_{}", s.break_counter);
                                s.break_counter += 1;
                                s.break_particles.push(BreakParticle::new(
                                    pkey,
                                    [hx as f64 + 0.5, hy as f64 + 0.5, hz as f64 + 0.5],
                                    20,
                                ));
                            }
                        }
                    }
                } else {
                    // Inventory: auto-collect nearby minerals
                    if keys & MASK_LCLICK != 0 {
                        let cx = (s.char_pos[0] / CHUNK_SIZE) as i32;
                        let cz = (s.char_pos[2] / CHUNK_SIZE) as i32;
                        let md = minerals::compute_chunk_minerals(&s.params, cx, cz);
                        for d in &md.deposits {
                            let dx = d.x as f64 - s.char_pos[0];
                            let dz = d.z as f64 - s.char_pos[2];
                            if dx * dx + dz * dz < 4.0 {
                                s.inventory.add_mineral(d.mineral_type, 1);
                                s.codex.discover_mineral(d.mineral_type);
                                s.achievements.try_unlock(achievements::Achievement::FirstMine);
                            }
                        }
                    }
                    // Examine nearby creatures
                    if keys & MASK_RCLICK != 0 {
                        for c in &s.chunks {
                            let data = creatures::compute_chunk_creatures(&s.params, c.cx, c.cz);
                            for cr in &data.creatures {
                                let dx = cr.x - s.char_pos[0];
                                let dz = cr.z - s.char_pos[2];
                                if dx * dx + dz * dz < 9.0 {
                                    let name = creatures::creature_name(cr.creature_type);
                                    s.codex.discover_creature(cr.creature_type);
                                    s.achievements.check_creatures(1);
                                    let msg = format!("🔍 {} — {}", name, crate::engine::terrain::get_zone(&s.params, cr.x, cr.z).as_str());
                                    if s.chat_messages.len() > 50 { s.chat_messages.pop_front(); }
                                    s.chat_messages.push_back(("Sistema".into(), msg));
                                    break;
                                }
                            }
                        }
                    }
                }

                // Position and rotate character parts
                part_position("char_body", s.char_pos, 0.0, 1.0, 0.0);
                part_position("char_head", s.char_pos, 0.0, 1.8, 0.0);
                part_position("char_arm_l", s.char_pos, -0.45, 1.5, 0.0);
                part_position("char_arm_r", s.char_pos, 0.45, 1.5, 0.0);
                part_position("char_leg_l", s.char_pos, -0.2, 0.9, 0.0);
                part_position("char_leg_r", s.char_pos, 0.2, 0.9, 0.0);

                if s.params.character == CharacterPreset::Kraken {
                    let tent_offsets = [(-0.35, 0.7, -0.3), (0.35, 0.7, -0.3), (-0.35, 0.7, 0.3), (0.35, 0.7, 0.3)];
                    for (i, &(ox, oy, oz)) in tent_offsets.iter().enumerate() {
                        part_position(&format!("char_tent_{}", i + 1), s.char_pos, ox, oy, oz);
                    }
                }

                // Walk/run animation
                if moving {
                    s.walk_time += delta;
                }
                let amp = if running { RUN_AMP } else if moving { WALK_AMP } else { 0.0 };
                let freq = if running { RUN_FREQ } else { WALK_FREQ };
                let t = s.walk_time * freq;
                let cyaw = s.char_yaw;
                let leg_l = amp * t.sin();
                let leg_r = amp * (t + std::f64::consts::PI).sin();
                bridge::set_mesh_rotation("char_body", 0.0, cyaw, 0.0);
                bridge::set_mesh_rotation("char_head", 0.0, cyaw, 0.0);
                bridge::set_mesh_rotation("char_leg_l", leg_l, cyaw, 0.0);
                bridge::set_mesh_rotation("char_leg_r", leg_r, cyaw, 0.0);
                bridge::set_mesh_rotation("char_arm_l", -leg_r, cyaw, 0.0);
                bridge::set_mesh_rotation("char_arm_r", -leg_l, cyaw, 0.0);

                if s.params.character == CharacterPreset::Kraken {
                    let tent_phases = [0.0, 1.57, 3.14, 4.71];
                    for i in 0..4 {
                        let sway = (s.time * 2.0 + tent_phases[i]).sin() * 0.3;
                        bridge::set_mesh_rotation(&format!("char_tent_{}", i + 1), sway, cyaw, 0.0);
                    }
                }

                if !moving {
                    s.walk_time *= 0.9;
                }

                // Hide character in first-person mode
                let fp = s.params.camera_mode == CameraMode::FirstPerson;
                bridge::set_mesh_visible("char_body", !fp);
                bridge::set_mesh_visible("char_head", !fp);
                bridge::set_mesh_visible("char_arm_l", !fp);
                bridge::set_mesh_visible("char_arm_r", !fp);
                bridge::set_mesh_visible("char_leg_l", !fp);
                bridge::set_mesh_visible("char_leg_r", !fp);
                if s.params.character == CharacterPreset::Kraken {
                    bridge::set_mesh_visible("char_tent_1", !fp);
                    bridge::set_mesh_visible("char_tent_2", !fp);
                    bridge::set_mesh_visible("char_tent_3", !fp);
                    bridge::set_mesh_visible("char_tent_4", !fp);
                }

                if s.char_dirty {
                    s.char_dirty = false;
                    regenerate_character(&mut s);
                }

                if s.params_dirty {
                    s.params_dirty = false;
                    regenerate_all(&mut s);
                }

                let target_cx = (s.char_pos[0] / CHUNK_SIZE) as i32;
                let target_cz = (s.char_pos[2] / CHUNK_SIZE) as i32;
                if target_cx != s.prev_cx || target_cz != s.prev_cz {
                    generate_chunks(&mut s, target_cx, target_cz);
                }

                // Camera: first-person or third-person orbital (or tour)
                let tour_params_cp = s.params;
                let tour_pos_cp = s.char_pos;
                let tour_yaw_cp = s.controls.yaw.get();
                let tour_pitch_cp = s.controls.pitch.get();
                let (cam_x, cam_y, cam_z, look_yaw, look_pitch) = if let Some(tu) = s.tour.update(delta, &tour_params_cp, &tour_pos_cp, tour_yaw_cp, tour_pitch_cp) {
                    (tu.pos[0], tu.pos[1], tu.pos[2], tu.yaw, tu.pitch)
                } else if s.params.camera_mode == CameraMode::FirstPerson {
                    let pitch = tour_pitch_cp.max(-1.5).min(1.5);
                    let eye_y = s.char_pos[1] + 1.6;
                    s.cam_pos = [s.char_pos[0], eye_y, s.char_pos[2]];
                    (s.cam_pos[0], s.cam_pos[1], s.cam_pos[2], -tour_yaw_cp, pitch)
                } else {
                    let pitch_clamped = tour_pitch_cp.max(-0.6).min(1.0);
                    let (sp_c, cp_c) = pitch_clamped.sin_cos();
                    let target_x = s.char_pos[0] + ARM_LENGTH * cp_c * cam_yaw.sin();
                    let target_z = s.char_pos[2] + ARM_LENGTH * cp_c * cam_yaw.cos();
                    let target_y = (s.char_pos[1] + ARM_HEIGHT + ARM_LENGTH * sp_c)
                        .max(terrain::get_height(&s.params, target_x, target_z) + 0.5);
                    s.cam_pos = [target_x, target_y, target_z];
                    let dx = s.char_pos[0] - s.cam_pos[0];
                    let dy = s.char_pos[1] + 1.0 - s.cam_pos[1];
                    let dz = s.char_pos[2] - s.cam_pos[2];
                    let dist_h = (dx * dx + dz * dz).sqrt().max(0.001);
                    let ly = dx.atan2(-dz);
                    let lp = (-dy / dist_h).atan();
                    (s.cam_pos[0], s.cam_pos[1], s.cam_pos[2], ly, lp)
                };
                bridge::set_camera(cam_x, cam_y, cam_z, look_yaw, look_pitch);

                // Update sky dome position to follow camera
                bridge::set_mesh_position("sky_dome", s.cam_pos[0], s.cam_pos[1], s.cam_pos[2]);

                // Creature AI update (wander + flee) every other frame
                if s.frame_count & 1 == 0 {
                    for c in &s.chunks {
                        if let Some((positions, rotations)) = creatures::update_creature_positions(
                            &s.params, c.cx, c.cz, s.time,
                            s.char_pos[0], s.char_pos[2],
                        ) {
                            let key = format!("crea_{},{}", c.cx, c.cz);
                            let arr = js_sys::Float32Array::from(&positions[..]);
                            bridge::update_mesh_positions(&key, &arr);
                        }
                    }
                }

                // Update water with Gerstner waves
                let water_level = s.params.water_level;
                let half = 120.0;
                let spacing = 3.0;
                let nx = 81u32;
                let nz = 81u32;
                let nv = nx * nz;
                let mut wp = Vec::with_capacity(nv as usize * 3);
                for iz in 0..nz {
                    for ix in 0..nx {
                        let lx = -half + ix as f64 * spacing;
                        let lz = -half + iz as f64 * spacing;
                        let wx = s.char_pos[0] + lx;
                        let wz = s.char_pos[2] + lz;
                        let wy = gerstner_y(wx, wz, s.time);
                        wp.push(lx as f32);
                        wp.push(wy as f32);
                        wp.push(lz as f32);
                    }
                }
                let wp_arr = js_sys::Float32Array::from(&wp[..]);
                bridge::update_mesh_positions("water", &wp_arr);
                bridge::set_mesh_position("water", s.char_pos[0], water_level, s.char_pos[2]);

                // Day/night cycle
                s.day_time += delta * s.params.day_speed;
                let sun_angle = s.day_time;
                let (sun_sin, sun_cos) = sun_angle.sin_cos();
                let sun_elev = sun_sin; // >0 = day, <0 = night
                let day_factor = sun_elev.max(0.0).min(1.0);
                let night_factor = (-sun_elev).max(0.0).min(1.0);
                let sunset_factor = (1.0 - (sun_elev - 0.05).abs() * 5.0).clamp(0.0, 1.0) * day_factor;

                // Weather effects on sun/sky/fog
                let weather_dim = 1.0 - s.weather_power * 0.4;
                let weather_gray = s.weather_power * 0.3;

                // Sun position and color
                let sun_x = 80.0 * sun_cos;
                let sun_y = (80.0 * sun_sin).max(-15.0);
                let sun_r = (1.0 - sunset_factor * 0.4) * weather_dim + weather_gray;
                let sun_g = (0.95 - sunset_factor * 0.5) * weather_dim + weather_gray;
                let sun_b = (0.85 - sunset_factor * 0.7) * weather_dim + weather_gray;
                let sun_intensity = (0.3 + day_factor * 1.7) * (1.0 - s.weather_power * 0.6);
                bridge::set_sun_light(sun_x, sun_y, 50.0, sun_r.max(0.0), sun_g.max(0.0), sun_b.max(0.0), sun_intensity.max(0.1));

                // Sky tint
                let sky_r = 1.0 - night_factor * 0.95;
                let sky_g = 1.0 - night_factor * 0.95;
                let sky_b = 1.0 - night_factor * 0.85;
                let mut sr = (sky_r - sunset_factor * 0.3) * weather_dim + weather_gray;
                let mut sg = (sky_g - sunset_factor * 0.4) * weather_dim + weather_gray;
                let mut sb = (sky_b - sunset_factor * 0.6) * weather_dim + weather_gray;

                // Lightning flash
                if s.lightning_flash > 0.01 {
                    let flash = s.lightning_flash * 0.8;
                    sr = (sr + flash * 2.0).min(1.0);
                    sg = (sg + flash * 2.0).min(1.0);
                    sb = (sb + flash * 2.5).min(1.0);
                }
                bridge::set_mesh_color("sky_dome", sr.max(0.0), sg.max(0.0), sb.max(0.0));

                // Fog color and density
                let fog_r = 0.6 - night_factor * 0.58 + sunset_factor * 0.25;
                let fog_g = 0.75 - night_factor * 0.73 - sunset_factor * 0.25;
                let fog_b = 0.92 - night_factor * 0.90 - sunset_factor * 0.62;
                let fog_density = 0.006 + s.weather_power * 0.025;
                let fog_r = fog_r * (1.0 - s.weather_power * 0.5) + 0.4 * s.weather_power;
                let fog_g = fog_g * (1.0 - s.weather_power * 0.5) + 0.4 * s.weather_power;
                let fog_b = fog_b * (1.0 - s.weather_power * 0.4) + 0.45 * s.weather_power;
                bridge::set_fog(fog_r.max(0.0), fog_g.max(0.0), fog_b.max(0.0), fog_density);

                // Stars opacity (reduced by weather)
                let stars_opac = (sun_elev * -3.0 - 0.5).clamp(0.0, 1.0) * (1.0 - s.weather_power * 0.8);
                bridge::set_particles_opacity("stars", stars_opac);

                // Weather system — zone-specific targets
                let weather_zone = terrain::get_zone(&s.params, s.char_pos[0], s.char_pos[2]);
                let zone_target = match weather_zone {
                    Zone::Storm => 1.0,
                    Zone::Tundra => 0.5,
                    Zone::Jungle => 0.45,
                    Zone::Volcanic | Zone::Lava | Zone::Magma => 0.35,
                    Zone::Ocean => 0.2,
                    Zone::Desert => 0.05,
                    _ => -1.0, // random
                };
                if zone_target >= 0.0 {
                    s.weather_target = zone_target;
                } else {
                    s.weather_timer += delta;
                    if s.weather_timer > 8.0 {
                        s.weather_timer = 0.0;
                        let r = ((s.time * 1000.0) as u64).wrapping_mul(1103515245).wrapping_add(12345) as f64 / u64::MAX as f64;
                        if r < 0.3 {
                            let r2 = ((s.time * 1000.0 + 1.0) as u64).wrapping_mul(1103515245).wrapping_add(12345) as f64 / u64::MAX as f64;
                            s.weather_target = r2 * 0.6;
                        } else {
                            s.weather_target = 0.0;
                        }
                    }
                }
                s.weather_power += (s.weather_target - s.weather_power) * delta * 0.5;
                let particle_opacity = if s.params.particle_mode == ParticleMode::Off {
                    0.0
                } else {
                    (0.5 + (s.weather_power * 0.5).min(1.0)).min(1.0)
                };
                bridge::set_particles_opacity("particles", particle_opacity);

                // Lightning flashes (Storm zone or high weather)
                s.lightning_flash *= 0.92; // decay
                if s.weather_power > 0.6 {
                    let strike_chance = (s.weather_power - 0.6) * 0.5 * delta;
                    if ((s.time * 1000.0 + 777.0) as u64).wrapping_mul(1103515245) as f64 / (u64::MAX as f64) < strike_chance {
                        s.lightning_flash = 1.0;
                        if ((s.time * 1000.0 + 777.0) as u64).wrapping_mul(1103515245) as f64 / (u64::MAX as f64) < 0.3 {
                            audio::play_effect("thunder");
                        }
                    }
                }

                // Update break particles
                let mut i = 0;
                while i < s.break_particles.len() {
                    if s.break_particles[i].lifetime <= 0.0 {
                        let dead = s.break_particles.remove(i);
                        dead.remove();
                    } else {
                        s.break_particles[i].update(delta);
                        i += 1;
                    }
                }
                if s.break_particles.len() > 10 {
                    let dead = s.break_particles.remove(0);
                    dead.remove();
                }

                // Weather label for HUD
                let weather_label = if s.weather_power < 0.05 {
                    "Despejado"
                } else if s.weather_power < 0.2 {
                    "Nublado"
                } else if s.weather_power < 0.4 {
                    match weather_zone {
                        Zone::Tundra => "Nevando",
                        Zone::Volcanic | Zone::Lava | Zone::Magma => "Ceniza",
                        _ => "Lluvia",
                    }
                } else if s.weather_power < 0.7 {
                    match weather_zone {
                        Zone::Tundra => "Tormenta de nieve",
                        Zone::Storm => "Tormenta eléctrica",
                        Zone::Volcanic | Zone::Lava | Zone::Magma => "Lluvia de ceniza",
                        _ => "Tormenta",
                    }
                } else {
                    match weather_zone {
                        Zone::Storm => "Tormenta severa",
                        Zone::Tundra => "Ventisca",
                        _ => "Tormenta intensa",
                    }
                };
                let power_names = ["☀️", "🌧️", "⛈️", "❄️"];
                let weather_label_power = format!("{} [{}]", weather_label, power_names[s.weather_power_idx as usize]);

                // Weather powers (G key cycle + activate)
                s.weather_cooldown = (s.weather_cooldown - delta).max(0.0);
                let g_pressed = keys & MASK_G != 0;
                if g_pressed && !s.g_prev && s.weather_cooldown <= 0.0 {
                    let weather_powers = [0.0, 0.6, 1.0, 0.8];
                    s.weather_power_idx = (s.weather_power_idx + 1) % 4;
                    s.weather_target = weather_powers[s.weather_power_idx as usize];
                    s.weather_cooldown = 8.0;
                    audio::play_tone(500.0 + s.weather_power_idx as f32 * 100.0, 0.15);
                }
                s.g_prev = g_pressed;

                // Send position to multiplayer server
                if s.ws_connected {
                    s.ws_send_timer += delta;
                    if s.ws_send_timer > 0.1 {
                        s.ws_send_timer = 0.0;
                        bridge::ws_send_pos(s.char_pos[0], s.char_pos[1], s.char_pos[2],
                            s.controls.yaw.get(), s.controls.pitch.get());
                    }
                }

                // Update particles
                let zone = terrain::get_zone(&s.params, s.char_pos[0], s.char_pos[2]);
                let mode = s.params.particle_mode;
                let (new_count, mode_active) = if mode != ParticleMode::Off {
                    (particles::mode_count(mode), true)
                } else {
                    (0, false)
                };
                let should_have = new_count > 0;
                if should_have && s.particles.is_none() {
                    let (pr, pg, pb, ps) = if mode_active {
                        particles::mode_color_size(mode)
                    } else {
                        particles::particle_color_size(zone)
                    };
                    s.particles = Some(ParticleSystem::new("particles", new_count, pr, pg, pb, ps));
                } else if !should_have && s.particles.is_some() {
                    if let Some(p) = s.particles.take() { p.remove(); }
                }
                if s.particles.is_some() {
                    let p_params = s.params;
                    let p_px = s.char_pos[0];
                    let p_py = s.char_pos[1];
                    let p_pz = s.char_pos[2];
                    if let Some(ref mut p) = s.particles {
                        p.update(delta, zone, &p_params, p_px, p_py, p_pz, water_level, mode);
                    }
                }

                // Waterfall particles
                let wf_params = s.params;
                let wf_px = s.char_pos[0];
                let wf_py = s.char_pos[1];
                let wf_pz = s.char_pos[2];
                let foam_px = s.char_pos[0];
                let foam_pz = s.char_pos[2];
                let foam_wl = wf_params.water_level;
                let bubble_x = s.char_pos[0];
                let bubble_y = s.char_pos[1];
                let bubble_z = s.char_pos[2];
                let bubble_wl = wf_params.water_level;
                s.waterfalls.update(delta, &wf_params, wf_px, wf_py, wf_pz);
                s.foam.update(delta, &wf_params, foam_px, foam_pz, foam_wl);
                s.bubbles.update(delta, &wf_params, bubble_x, bubble_y, bubble_z, bubble_wl);

                let surface_type = {
                    let zone = terrain::get_zone(&s.params, s.char_pos[0], s.char_pos[2]);
                    if s.char_pos[1] < s.params.water_level - 0.5 { 4 }
                    else {
                        match zone {
                            Zone::Desert => 3,
                            Zone::Tundra | Zone::Crystal | Zone::Aurora => 5,
                            Zone::Cave | Zone::Abyss => 1,
                            Zone::Volcanic | Zone::Lava | Zone::Magma => 1,
                            Zone::Ocean | Zone::CoralReef | Zone::KelpForest | Zone::RockyReef | Zone::SandyPlain | Zone::DeepOcean => 3,
                            _ => 2,
                        }
                    }
                };
                audio::update(
                    terrain::get_zone(&s.params, s.char_pos[0], s.char_pos[2]),
                    s.params.seed,
                    s.char_pos[1] <= ground_y + 0.5 && keys & (MASK_W | MASK_S | MASK_A | MASK_D) != 0,
                    s.speed,
                    s.weather_power,
                    surface_type,
                );

                // Auto-save placed blocks + mined blocks every 15s
                s.save_timer += delta;
                if s.save_timer > 15.0 {
                    s.save_timer = 0.0;
                    save_blocks(&s.placed_blocks);
                    save_mined(&s.mined_blocks);
                    auto_save_full(&s);
                }

                // Portal detection and teleportation
                let mut near_portal = None;
                let portal_data = portals::compute_portals(&s.params);
                for p in &portal_data.portals {
                    let dx = s.char_pos[0] - p.x;
                    let dz = s.char_pos[2] - p.z;
                    let dist = (dx * dx + dz * dz).sqrt();
                    if dist < p.radius {
                        near_portal = Some(p.id.clone());
                        let portal_key = keys & MASK_R != 0;
                        if portal_key && !s.portal_prev {
                            audio::play_effect("portal_open");
                            s.params.seed = p.target_seed;
                            s.params_dirty = true;
                            s.char_pos = [0.0, terrain::get_height(&s.params, 0.0, 0.0).max(0.0), 0.0];
                            s.vel_x = 0.0;
                            s.vel_z = 0.0;
                            s.vel_y = 0.0;
                        }
                        break;
                    }
                }
                s.portal_prev = keys & MASK_R != 0;

                // Biome discovery
                let zone_name = zone.as_str().to_string();
                if !s.discovered_biomes.contains(&zone_name) {
                    s.discovered_biomes.push(zone_name.clone());
                    let discovered = s.discovered_biomes.clone();
                    let all_biome_names: &[&str] = &[];
                    s.achievements.check_biomes(&discovered, all_biome_names);
                    s.codex.discover_biome(&zone_name);
                }

                // Structure discovery: check proximity to structures every few frames
                if s.frame_count & 15 == 0 {
                    let pcx = (s.char_pos[0] / CHUNK_SIZE) as i32;
                    let pcz = (s.char_pos[2] / CHUNK_SIZE) as i32;
                    for dx in -1..=1 {
                        for dz in -1..=1 {
                            let data = structures::compute_chunk_structures(&s.params, pcx + dx, pcz + dz);
                            for inst in &data.instances {
                                let ddx = s.char_pos[0] - inst.x as f64;
                                let ddz = s.char_pos[2] - inst.z as f64;
                                if ddx * ddx + ddz * ddz < 36.0 {
                                    let name = match inst.struct_type {
                                        structures::StructType::Hut => "Hut",
                                        structures::StructType::Tower => "Tower",
                                        structures::StructType::Ruins => "Ruins",
                                        structures::StructType::Arch => "Arch",
                                        structures::StructType::Pillar => "Pillar",
                                        structures::StructType::Dome => "Dome",
                                        structures::StructType::Pyramid => "Pyramid",
                                        structures::StructType::CrystalSpire => "Crystal Spire",
                                        structures::StructType::MushroomHut => "Mushroom Hut",
                                        structures::StructType::Obelisk => "Obelisk",
                                        structures::StructType::Plaza => "Plaza",
                                        structures::StructType::Muralla => "Muralla",
                                        structures::StructType::DungeonEntrance => "Dungeon Entrance",
                                    };
                                    s.codex.discover_structure(name);
                                }
                            }
                        }
                    }
                }

                let achievement_msg = s.achievements.pending.pop().map(|a| {
                    achievements::achievement_icon(&a).to_string() + " " + achievements::achievement_name(&a) + ": ¡Logro!"
                });

                bridge::render_frame();

                let angle = (cam_yaw * 180.0 / std::f64::consts::PI) as i32;
                let angle = if angle < 0 { angle + 360 } else { angle % 360 };
                let zone = terrain::get_zone(&s.params, s.char_pos[0], s.char_pos[2]);
                let hud_build_mode = s.params.build_mode;
                let hud_selected = s.inventory.selected_slot;
                let hud_blocks = s.block_inventory.clone();
                *hud.borrow_mut() = HudData {
                    pos: s.char_pos,
                    biome: zone.as_str().to_string(),
                    height: ground_y,
                    fps: s.fps,
                    chunks: s.chunks.len(),
                    yaw_deg: angle,
                    speed: s.speed,
                    build_mode: hud_build_mode,
                    selected_slot: hud_selected,
                    inventory: hud_blocks,
                    minerals: s.inventory.items.iter().filter(|i| i.count > 0).map(|i| (i.mineral_type, i.count)).collect(),
                    near_portal,
                    weather_label: weather_label_power.clone(),
                    lightning: s.lightning_flash > 0.1,
                    craft_message: None,
                    achievement_message: achievement_msg.clone(),
                    achievement_points: s.achievements.unlocked.len() as u32,
                    codex_biomes: (s.codex.biomes.iter().filter(|e| e.discovered).count(), s.codex.biomes.len()),
                    codex_structures: (s.codex.structures.iter().filter(|e| e.discovered).count(), s.codex.structures.len()),
                    codex_minerals: (s.codex.minerals.iter().filter(|e| e.discovered).count(), s.codex.minerals.len()),
                    codex_creatures: (s.codex.creatures.iter().filter(|e| e.discovered).count(), s.codex.creatures.len()),
                    ..Default::default()
                };
            }));

            if let Some(ref c) = *closure2.borrow() {
                web_sys::window()
                    .unwrap()
                    .request_animation_frame(c.as_ref().unchecked_ref())
                    .ok();
            }
        }));

        if let Some(ref c) = *closure.borrow() {
            let id = web_sys::window()
                .unwrap()
                .request_animation_frame(c.as_ref().unchecked_ref())
                .ok();
            self.anim_id = id;
        }
        self._closure = Some(closure);
    }

    pub fn get_hud(&self) -> HudData {
        self.hud.borrow().clone()
    }

    pub fn joystick_cells(&self) -> (Rc<Cell<f64>>, Rc<Cell<f64>>) {
        let s = self.state.borrow();
        (s.joy_dx.clone(), s.joy_dy.clone())
    }

    pub fn save_to_slot(&self, slot: u32, name: &str,
        waypoints: &[(f64, f64, f64, String)],
        discovered: &[String]) -> Result<(), String>
    {
        let s = self.state.borrow();
        let inventory_json = serde_json::to_string(&s.inventory).ok();
        let codex_json = serde_json::to_string(&s.codex).ok();
        let achievements_json = serde_json::to_string(&s.achievements).ok();
        let placed_blocks: Vec<[i32; 4]> = s.placed_blocks.iter().map(|(&(x,y,z), &t)| [x, y, z, t as i32]).collect();
        let block_inventory: Vec<(u8, u32)> = s.block_inventory.iter().map(|(t, c)| (*t, *c)).collect();
        let discovered_vec = discovered.to_vec();
        let mut data = crate::state::SaveData::new(
            name,
            &s.params,
            s.char_pos,
            s.controls.yaw.get(),
            s.controls.pitch.get(),
            waypoints,
            &discovered_vec,
            s.day_time,
            s.params.fly_mode,
            false,
        );
        data.inventory_json = inventory_json;
        data.codex_json = codex_json;
        data.achievements_json = achievements_json;
        data.placed_blocks = placed_blocks;
        data.block_inventory = block_inventory;
        let json = serde_json::to_string(&data).map_err(|e| e.to_string())?;
        let key = format!("worlds_save_{}", slot);
        db::set_async(&key, &json);
        Ok(())
    }

    pub fn load_from_slot(slot: u32) -> Option<crate::state::SaveData> {
        let key = format!("worlds_save_{}", slot);
        if let Some(json) = db::get(&key) {
            return serde_json::from_str(&json).ok();
        }
        None
    }

    pub fn apply_save(&mut self, data: &crate::state::SaveData) {
        let mut s = self.state.borrow_mut();
        s.params = data.params;
        s.char_pos = data.pos;
        s.controls.yaw.set(data.yaw);
        s.controls.pitch.set(data.pitch);
        s.day_time = data.time_of_day;
        s.params.fly_mode = data.fly_mode;
        s.params_dirty = true;

        // Restore inventory
        if let Some(ref json) = data.inventory_json {
            if let Ok(inv) = serde_json::from_str::<crate::engine::inventory::Inventory>(json) {
                s.inventory = inv;
            }
        }
        // Restore codex
        if let Some(ref json) = data.codex_json {
            if let Ok(codex) = serde_json::from_str::<crate::engine::codex::Codex>(json) {
                s.codex = codex;
            }
        }
        // Restore achievements
        if let Some(ref json) = data.achievements_json {
            if let Ok(ach) = serde_json::from_str::<crate::engine::achievements::AchievementState>(json) {
                s.achievements = ach;
            }
        }
        // Restore placed blocks
        let mut blocks = std::collections::HashMap::new();
        for arr in &data.placed_blocks {
            blocks.insert((arr[0], arr[1], arr[2]), arr[3] as u8);
        }
        s.placed_blocks = blocks;
        // Restore block inventory
        if !data.block_inventory.is_empty() {
            s.block_inventory = data.block_inventory.clone();
        }
        // Restore discovered biomes (from legacy field or data)
        if !data.discovered_biomes.is_empty() {
            s.discovered_biomes = data.discovered_biomes.clone();
        }

        // Respawn blocks in 3D scene
        let block_colors: [[f32; 3]; 9] = [
            [0.6,0.45,0.3],[0.5,0.5,0.5],[0.55,0.35,0.15],
            [0.2,0.6,0.2],[0.7,0.4,1.0],[0.8,0.3,0.05],
            [0.7,0.9,1.0],[0.85,0.75,0.5],[0.3,0.5,0.2],
        ];
        for (&(bx, by, bz), &bt) in &s.placed_blocks {
            let bkey = format!("b_{}_{}_{}", bx, by, bz);
            let bcol = block_colors[bt as usize % block_colors.len()];
            let (pos, nrm, idx) = box_mesh(1.0, 1.0, 1.0);
            let col = fill_color(24, bcol[0], bcol[1], bcol[2]);
            let bp = js_sys::Float32Array::from(&pos[..]);
            let bn = js_sys::Float32Array::from(&nrm[..]);
            let bi = js_sys::Uint32Array::from(&idx[..]);
            let bc_arr = js_sys::Float32Array::from(&col[..]);
            bridge::upload_mesh(&bkey, &bp, &bn, &bi, &bc_arr);
            bridge::set_mesh_position(&bkey, bx as f64 + 0.5, by as f64 + 0.5, bz as f64 + 0.5);
        }
    }

    pub fn craft_recipe(&mut self, recipe_index: usize) -> String {
        let mut s = self.state.borrow_mut();
        if recipe_index < inventory::RECIPES.len() {
            let result = inventory::perform_craft(&inventory::RECIPES[recipe_index], &mut s.inventory);
            if !result.is_empty() {
                s.achievements.items_crafted += 1;
                let count = s.achievements.items_crafted;
                s.achievements.check_craft(count);
                audio::play_effect("craft");
                // Apply immediate effects for certain items
                let result_id = inventory::RECIPES[recipe_index].result.1;
                match result_id {
                    14 => { // Heal potion
                        audio::play_effect("heal");
                        // Green particles burst around player
                        let pkey = format!("heal_{}", s.break_counter);
                        s.break_counter += 1;
                        s.break_particles.push(BreakParticle::new(
                            pkey,
                            [s.char_pos[0], s.char_pos[1] + 1.0, s.char_pos[2]],
                            30,
                        ));
                    }
                    17 => { // Power ring: temp speed boost
                        audio::play_effect("power_up");
                        s.speed = s.speed * 2.0;
                    }
                    13 => { // Crystal wand: small light
                        audio::play_tone(880.0, 0.2);
                    }
                    _ => {}
                }
            }
            result
        } else {
            String::new()
        }
    }

    pub fn inventory_minerals(&self) -> Vec<(u8, u32)> {
        let s = self.state.borrow();
        s.inventory.items.iter().filter(|i| i.count > 0).map(|i| (i.mineral_type, i.count)).collect()
    }

    pub fn delete_slot(slot: u32) {
        let key = format!("worlds_save_{}", slot);
        db::delete_async(&key);
    }

    pub fn list_slots() -> Vec<(u32, String)> {
        let mut slots = Vec::new();
        for key in db::keys_with_prefix("worlds_save_") {
            if let Some(json) = db::get(&key) {
                if let Ok(data) = serde_json::from_str::<crate::state::SaveData>(&json) {
                    if let Some(num_str) = key.strip_prefix("worlds_save_") {
                        if let Ok(i) = num_str.parse::<u32>() {
                            let label = format!("{} — slot {}", data.slot_name, i);
                            slots.push((i, label));
                        }
                    }
                }
            }
        }
        slots
    }

    pub fn load_autosave() -> Option<crate::state::SaveData> {
        if let Some(json) = db::get("worlds_autosave") {
            if let Ok(data) = serde_json::from_str::<crate::state::SaveData>(&json) {
                return Some(data);
            }
        }
        None
    }

    // ── Multiplayer ──
    pub fn connect_multiplayer(&mut self, server_url: &str, seed: u32) {
        self.disconnect_multiplayer();
        let state = self.state.clone();
        let cb = Closure::<dyn FnMut(String)>::new(move |json: String| {
            let mut s = state.borrow_mut();
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&json) {
                let msg_type = msg["type"].as_str().unwrap_or("");
                match msg_type {
                    "welcome" => {
                        if let Some(your_id) = msg["your_id"].as_str() {
                            s.ws_player_id = your_id.to_string();
                            s.chat_messages.push_back(("Sistema".into(), format!("Conectado. ID: {}", &your_id[..6])));
                            if s.chat_messages.len() > 50 { s.chat_messages.pop_front(); }
                        }
                    }
                    "pos" => {
                        if let Some(players) = msg["players"].as_array() {
                            for p in players {
                                let pid = p["id"].as_str().unwrap_or("");
                                if pid == s.ws_player_id { continue; }
                                let name = p["name"].as_str().unwrap_or("Player").to_string();
                                let x = p["x"].as_f64().unwrap_or(0.0);
                                let y = p["y"].as_f64().unwrap_or(0.0);
                                let z = p["z"].as_f64().unwrap_or(0.0);
                                let yaw = p["yaw"].as_f64().unwrap_or(0.0);
                                let pitch = p["pitch"].as_f64().unwrap_or(0.0);
                                s.remote_players.insert(pid.to_string(), RemotePlayerData {
                                    name: name.clone(), x, y, z,
                                });
                                bridge::ws_update_remote_player(pid, &name, x, y, z, yaw, pitch);
                            }
                        }
                    }
                    "chat" => {
                        let chat_data = &msg["chat"];
                        let player_name = chat_data["player_name"].as_str().unwrap_or("?").to_string();
                        let text = chat_data["text"].as_str().unwrap_or("").to_string();
                        s.chat_messages.push_back((player_name, text));
                        if s.chat_messages.len() > 50 { s.chat_messages.pop_front(); }
                    }
                    _ => {}
                }
            }
        });
        let url_js = js_sys::JsString::from(server_url);
        let seed_js = js_sys::JsValue::from(seed);
        bridge::ws_connect(&url_js, seed, cb.as_ref().unchecked_ref());
        cb.forget();
        self.state.borrow_mut().ws_connected = true;
    }

    pub fn disconnect_multiplayer(&mut self) {
        bridge::ws_disconnect();
        let mut s = self.state.borrow_mut();
        s.ws_connected = false;
        s.ws_player_id.clear();
        s.remote_players.clear();
        s.chat_messages.clear();
    }

    pub fn send_chat(&mut self, text: &str) {
        bridge::ws_send_chat(text);
    }

    pub fn chat_messages(&self) -> Vec<(String, String)> {
        self.state.borrow().chat_messages.iter().cloned().collect()
    }

    pub fn is_ws_connected(&self) -> bool {
        self.state.borrow().ws_connected
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if let Some(id) = self.anim_id {
            web_sys::window().unwrap().cancel_animation_frame(id).ok();
        }
    }
}
