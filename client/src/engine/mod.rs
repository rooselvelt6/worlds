pub mod audio;
pub mod bridge;
pub mod camera;
pub mod chunk;
pub mod controls;
pub mod creatures;
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

use std::cell::{Cell, RefCell};
use std::panic::AssertUnwindSafe;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::state::{CameraMode, CharacterPreset, ParticleMode, WorldParams};
use terrain::{Zone, get_height};
use camera::Camera;
use chunk::{ChunkData, CHUNK_SIZE};
use controls::{Controls, MASK_1, MASK_2, MASK_3, MASK_4, MASK_5, MASK_6, MASK_7, MASK_8, MASK_9,
    MASK_A, MASK_B, MASK_D, MASK_E, MASK_LCLICK, MASK_Q, MASK_RCLICK, MASK_S, MASK_SHIFT, MASK_SPACE, MASK_T, MASK_W};
use creatures::{generate_creature_mesh, creature_animated_positions};
use gamepad::poll_gamepad;
use inventory::Inventory;
use minerals::generate_mineral_mesh;
use particles::ParticleSystem;
use portals::generate_portal_mesh;
use structures::generate_struct_mesh;
use tour::TourState;
use vegetation::generate_veg_mesh;

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
    pub build_mode: bool,
    pub inventory: Vec<(u8, u32)>,
    pub minerals: Vec<(u8, u32)>,
    pub selected_slot: u8,
    pub season: u8,
    pub creature_count: u32,
    pub achievement_points: u32,
    pub vr_mode: bool,
    pub tour_mode: bool,
}

fn save_blocks(blocks: &std::collections::HashMap<(i32,i32,i32), u8>) {
    let data: Vec<[i32; 4]> = blocks.iter().map(|(&(x,y,z), &t)| [x, y, z, t as i32]).collect();
    if let Ok(json) = serde_json::to_string(&data) {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item("worlds_blocks", &json);
        }
    }
}

fn load_blocks() -> std::collections::HashMap<(i32,i32,i32), u8> {
    let mut map = std::collections::HashMap::new();
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(Some(json)) = storage.get_item("worlds_blocks") {
            if let Ok(data) = serde_json::from_str::<Vec<[i32; 4]>>(&json) {
                for arr in data {
                    map.insert((arr[0], arr[1], arr[2]), arr[3] as u8);
                }
            }
        }
    }
    map
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
        let terrain_h = get_height(params, x, z);
        if y < terrain_h && terrain_h > params.water_level {
            let adj_key = (bx, terrain_h.floor() as i32, bz);
            return Some((adj_key, false));
        }
        x += sx * step;
        y += sy * step;
        z += sz * step;
        dist += step;
    }
    None
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
    placed_blocks: std::collections::HashMap<(i32, i32, i32), u8>,
    block_inventory: Vec<(u8, u32)>,
    build_prev: bool,
    slot_prev: u32,
    save_timer: f64,
    params_dirty: bool,
    char_dirty: bool,
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
        bridge::remove_mesh(&format!("mineral_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("portal_{},{}", old.cx, old.cz));
    }
    let mut new_chunks: Vec<ChunkData> = Vec::new();
    for x in (pcx - d)..=(pcx + d) {
        for z in (pcz - d)..=(pcz + d) {
            let data = chunk::compute_chunk_data(&s.params, x, z);
            let key = format!("chunk_{},{}", data.cx, data.cz);
            let pos_arr = js_sys::Float32Array::from(&data.positions[..]);
            let norm_arr = js_sys::Float32Array::from(&data.normals[..]);
            let col_arr = js_sys::Float32Array::from(&data.colors[..]);
            let idx_arr = js_sys::Uint32Array::from(
                &data.indices.iter().map(|&i| i as u32).collect::<Vec<_>>()[..],
            );
            bridge::upload_mesh(&key, &pos_arr, &norm_arr, &idx_arr, &col_arr);
            if let Some((vpos, vnorm, vidx, vcol)) = generate_veg_mesh(&s.params, x, z) {
                let vkey = format!("veg_{},{}", x, z);
                let vp = js_sys::Float32Array::from(&vpos[..]);
                let vn = js_sys::Float32Array::from(&vnorm[..]);
                let vi = js_sys::Uint32Array::from(&vidx[..]);
                let vc = js_sys::Float32Array::from(&vcol[..]);
                bridge::upload_mesh(&vkey, &vp, &vn, &vi, &vc);
            }
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
            new_chunks.push(data);
        }
    }
    s.chunks = new_chunks;
    s.prev_cx = pcx;
    s.prev_cz = pcz;
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
        bridge::remove_mesh(&format!("mineral_{},{}", old.cx, old.cz));
        bridge::remove_mesh(&format!("portal_{},{}", old.cx, old.cz));
    }

    for &(cx, cz) in &to_compute {
        let data = chunk::compute_chunk_data(&s.params, cx, cz);
        let key = format!("chunk_{},{}", data.cx, data.cz);
        let pos_arr = js_sys::Float32Array::from(&data.positions[..]);
        let norm_arr = js_sys::Float32Array::from(&data.normals[..]);
        let col_arr = js_sys::Float32Array::from(&data.colors[..]);
        let idx_arr = js_sys::Uint32Array::from(
            &data.indices.iter().map(|&i| i as u32).collect::<Vec<_>>()[..],
        );
        bridge::upload_mesh(&key, &pos_arr, &norm_arr, &idx_arr, &col_arr);

        // Vegetation mesh for this chunk
        if let Some((vpos, vnorm, vidx, vcol)) = generate_veg_mesh(&s.params, cx, cz) {
            let vkey = format!("veg_{},{}", cx, cz);
            let vp = js_sys::Float32Array::from(&vpos[..]);
            let vn = js_sys::Float32Array::from(&vnorm[..]);
            let vi = js_sys::Uint32Array::from(&vidx[..]);
            let vc = js_sys::Float32Array::from(&vcol[..]);
            bridge::upload_mesh(&vkey, &vp, &vn, &vi, &vc);
        }

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

        new_chunks.push(data);
    }

    s.chunks = new_chunks;
}

fn regenerate_character(s: &mut GameState) {
    for key in &["char_body", "char_head", "char_arm_l", "char_arm_r", "char_leg_l", "char_leg_r"] {
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
            speed: 18.0,
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
            weather_power: 0.5,
            weather_target: 0.5,
            weather_timer: 0.0,
            placed_blocks: load_blocks(),
            block_inventory: vec![(0, 64), (1, 32), (2, 16), (3, 16), (4, 8), (5, 8), (6, 8), (7, 8), (8, 8)],
            build_prev: false,
            slot_prev: 0,
            save_timer: 0.0,
            params_dirty: false,
            char_dirty: false,
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

                let new_x = s.char_pos[0] + s.vel_x * delta;
                let new_z = s.char_pos[2] + s.vel_z * delta;
                let old_x = s.char_pos[0];
                let old_z = s.char_pos[2];
                let gh = terrain::get_height(&s.params, s.char_pos[0], s.char_pos[2]);
                let h_t = terrain::get_height(&s.params, new_x, new_z);
                let diff = h_t - gh;

                let blocked = diff > s.params.step_height
                    || collides_with_blocks(new_x, s.char_pos[1], new_z, &s.placed_blocks);

                if !blocked {
                    s.char_pos[0] = new_x;
                    s.char_pos[2] = new_z;
                } else {
                    let h_x = terrain::get_height(&s.params, new_x, s.char_pos[2]);
                    if h_x - gh <= s.params.step_height
                        && !collides_with_blocks(new_x, s.char_pos[1], s.char_pos[2], &s.placed_blocks)
                    {
                        s.char_pos[0] = new_x;
                    }
                    let h_z = terrain::get_height(&s.params, s.char_pos[0], new_z);
                    if h_z - gh <= s.params.step_height
                        && !collides_with_blocks(s.char_pos[0], s.char_pos[1], new_z, &s.placed_blocks)
                    {
                        s.char_pos[2] = new_z;
                    }
                    if s.char_pos[0] == old_x && s.char_pos[2] == old_z {
                        s.vel_x = 0.0;
                        s.vel_z = 0.0;
                    }
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
                    if let Some(hit) = raycast_block(s.cam_pos[0], s.cam_pos[1], s.cam_pos[2], rdx, rdy, rdz, 12.0, &s.placed_blocks, &s.params) {
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
                                        let bcol = block_colors[sel_slot as usize % block_colors.len()];
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
                                    }
                                }
                            }
                        }
                        if keys & MASK_RCLICK != 0 {
                            if s.placed_blocks.remove(&key).is_some() {
                                let bkey = format!("b_{}_{}_{}", hx, hy, hz);
                                bridge::remove_mesh(&bkey);
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

                // Creature animation (Y-bob every other frame)
                if s.frame_count & 1 == 0 {
                    for c in &s.chunks {
                        if let Some(positions) = creature_animated_positions(&s.params, c.cx, c.cz, s.time) {
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

                // Sun position and color
                let sun_x = 80.0 * sun_cos;
                let sun_y = (80.0 * sun_sin).max(-15.0);
                let sun_r = 1.0 - sunset_factor * 0.4 + night_factor * 0.2 * 0.0;
                let sun_g = 0.95 - sunset_factor * 0.5 + night_factor * 0.2 * 0.0;
                let sun_b = 0.85 - sunset_factor * 0.7 + night_factor * 0.8 * 0.0;
                let sun_intensity = 0.3 + day_factor * 1.7;
                bridge::set_sun_light(sun_x, sun_y, 50.0, sun_r, sun_g, sun_b, sun_intensity);

                // Sky tint
                let sky_r = 1.0 - night_factor * 0.95;
                let sky_g = 1.0 - night_factor * 0.95;
                let sky_b = 1.0 - night_factor * 0.85;
                let sr = sky_r - sunset_factor * 0.3;
                let sg = sky_g - sunset_factor * 0.4;
                let sb = sky_b - sunset_factor * 0.6;
                bridge::set_mesh_color("sky_dome", sr.max(0.0), sg.max(0.0), sb.max(0.0));

                // Fog color
                let fog_r = 0.6 - night_factor * 0.58 + sunset_factor * 0.25;
                let fog_g = 0.75 - night_factor * 0.73 - sunset_factor * 0.25;
                let fog_b = 0.92 - night_factor * 0.90 - sunset_factor * 0.62;
                bridge::set_fog(fog_r.max(0.0), fog_g.max(0.0), fog_b.max(0.0), 0.006);

                // Stars opacity
                let stars_opac = (sun_elev * -3.0 - 0.5).clamp(0.0, 1.0);
                bridge::set_particles_opacity("stars", stars_opac);

                // Weather system
                let weather_zone = terrain::get_zone(&s.params, s.char_pos[0], s.char_pos[2]);
                if weather_zone == Zone::Storm {
                    s.weather_target = 1.0;
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
                bridge::set_particles_opacity("particles", 0.5 + s.weather_power * 0.5);

                // Update particles
                let zone = terrain::get_zone(&s.params, s.char_pos[0], s.char_pos[2]);
                let mode = s.params.particle_mode;
                let (new_count, mode_active) = if mode != ParticleMode::Off {
                    (particles::mode_count(mode), true)
                } else {
                    (particles::particle_count(zone), false)
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

                audio::update(
                    terrain::get_zone(&s.params, s.char_pos[0], s.char_pos[2]),
                    s.params.seed,
                    s.char_pos[1] <= ground_y + 0.5 && keys & (MASK_W | MASK_S | MASK_A | MASK_D) != 0,
                    s.speed,
                );

                // Auto-save placed blocks every 15s
                s.save_timer += delta;
                if s.save_timer > 15.0 {
                    s.save_timer = 0.0;
                    save_blocks(&s.placed_blocks);
                }

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

    pub fn save_to_slot(&self, _slot: u32, _name: &str) {}
    pub fn load_from_slot(_slot: u32) -> Option<crate::state::SaveData> {
        None
    }
    pub fn apply_save(&mut self, _data: &crate::state::SaveData) {}
    pub fn delete_slot(_slot: u32) {}
}

impl Drop for Engine {
    fn drop(&mut self) {
        if let Some(id) = self.anim_id {
            web_sys::window().unwrap().cancel_animation_frame(id).ok();
        }
    }
}
