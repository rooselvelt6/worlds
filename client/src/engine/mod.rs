pub mod audio;
pub mod bridge;
pub mod camera;
pub mod chunk;
pub mod controls;
pub mod joystick;
pub mod minerals;
pub mod minimap;
pub mod particles;
pub mod structures;
pub mod terrain;
pub mod vegetation;

use std::cell::{Cell, RefCell};
use std::panic::AssertUnwindSafe;
use std::rc::Rc;



use camera::Camera;
use chunk::{ChunkData, CHUNK_SIZE};
use controls::{Controls, MASK_A, MASK_C, MASK_D, MASK_E, MASK_F12, MASK_G, MASK_H, MASK_M, MASK_Q, MASK_S, MASK_SHIFT, MASK_SPACE, MASK_T, MASK_W};
use minerals::MineralData;
use structures::StructData;
use vegetation::VegData;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::engine::terrain::Zone;
use crate::state::WorldParams;

const GRAVITY: f64 = 20.0;
const JUMP_SPEED: f64 = 12.0;

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
    pub waypoints: Vec<(f64, f64, f64, String)>,
    pub discovered_biomes: Vec<String>,
    pub discovery_message: Option<String>,
}

struct GameState {
    canvas: web_sys::HtmlCanvasElement,
    camera: Camera,
    controls: Controls,
    chunks: Vec<ChunkData>,
    veg_chunks: Vec<VegData>,
    struct_chunks: Vec<StructData>,
    mineral_chunks: Vec<MineralData>,
    params: WorldParams,
    prev_cx: i32,
    prev_cz: i32,
    last_time: f64,
    frame_count: u32,
    fps_timer: f64,
    fps: u32,
    vel_y: f64,
    joy_dx: Rc<Cell<f64>>,
    joy_dy: Rc<Cell<f64>>,
    time_of_day: f64,
    prev_zone: Option<Zone>,
    observer_mode: bool,
    orbit_radius: f64,
    night_mode: bool,
    waypoints: Vec<(f64, f64, f64, String)>,
    discovered_biomes: Vec<String>,
    discovery_message: Option<(String, f64)>,
}

pub struct Engine {
    state: Rc<RefCell<GameState>>,
    hud: Rc<RefCell<HudData>>,
    anim_id: Option<i32>,
    _closure: Option<Rc<RefCell<Option<Closure<dyn FnMut()>>>>>,
}

impl Engine {
    pub fn new(canvas: web_sys::HtmlCanvasElement, params: WorldParams) -> Result<Self, String> {
        let joy_dx = Rc::new(Cell::new(0.0));
        let joy_dy = Rc::new(Cell::new(0.0));
        canvas.set_tab_index(0);

        bridge::init(&canvas);

        let yaw = Rc::new(Cell::new(0.785));
        let pitch = Rc::new(Cell::new(-0.3));
        let mut controls = Controls::new(yaw.clone(), pitch.clone());
        controls.set_sensitivity(params.mouse_sensitivity);
        controls.attach(&canvas);
        let camera = Camera::new(yaw, pitch);

        let state = Rc::new(RefCell::new(GameState {
            canvas,
            camera,
            controls,
            chunks: Vec::new(),
            veg_chunks: Vec::new(),
            struct_chunks: Vec::new(),
            mineral_chunks: Vec::new(),
            params,
            prev_cx: i32::MAX,
            prev_cz: i32::MAX,
            last_time: 0.0,
            frame_count: 0,
            fps_timer: 0.0,
            fps: 0,
            vel_y: 0.0,
            joy_dx: joy_dx.clone(),
            joy_dy: joy_dy.clone(),
            time_of_day: 0.5,
            prev_zone: None,
            observer_mode: false,
            orbit_radius: 15.0,
            night_mode: false,
            waypoints: Vec::new(),
            discovered_biomes: Vec::new(),
            discovery_message: None,
        }));

        audio::init();

        {
            let mut s = state.borrow_mut();
            let cx = (s.camera.pos[0] / CHUNK_SIZE) as i32;
            let cz = (s.camera.pos[2] / CHUNK_SIZE) as i32;
            generate_chunks(&mut s, cx, cz);
        }

        let hud = Rc::new(RefCell::new(HudData::default()));

        Ok(Self { state, hud, anim_id: None, _closure: None })
    }

    pub fn update_params(&mut self, params: &WorldParams) {
        let mut s = self.state.borrow_mut();
        s.params = *params;
        s.controls.set_sensitivity(params.mouse_sensitivity);
        s.prev_cx = i32::MAX;
        s.prev_cz = i32::MAX;
        drop(s);
        self.regenerate_all();
    }

    pub fn regenerate_all(&self) {
        let mut s = self.state.borrow_mut();
        for chunk in s.chunks.drain(..) {
            let key = format!("{},{}", chunk.cx, chunk.cz);
            bridge::remove_chunk(&key);
            bridge::remove_vegetation(&key);
            bridge::remove_structure(&key);
            bridge::remove_minerals(&key);
        }
        s.veg_chunks.clear();
        s.struct_chunks.clear();
        s.mineral_chunks.clear();
        s.prev_cx = i32::MAX;
        s.prev_cz = i32::MAX;
        drop(s);
        let cx = (self.state.borrow().camera.pos[0] / CHUNK_SIZE) as i32;
        let cz = (self.state.borrow().camera.pos[2] / CHUNK_SIZE) as i32;
        let mut s = self.state.borrow_mut();
        generate_chunks(&mut s, cx, cz);
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
                if s.last_time == 0.0 { s.last_time = now; }
                let delta = ((now - s.last_time) / 1000.0).min(0.1);
                s.last_time = now;

                s.frame_count += 1;
                if s.frame_count == 1 || s.frame_count % 120 == 0 {
                    web_sys::console::log_1(&format!("[engine] frame {}", s.frame_count).into());
                }
                if now - s.fps_timer > 1000.0 {
                    s.fps = s.frame_count;
                    s.frame_count = 0;
                    s.fps_timer = now;
                }

                let mut keys_mask = s.controls.keys.get();
                let joy_dx = s.joy_dx.get();
                let joy_dy = s.joy_dy.get();
                if joy_dx.abs() > 0.3 || joy_dy.abs() > 0.3 {
                    if joy_dy < -0.3 { keys_mask |= MASK_W; }
                    if joy_dy > 0.3 { keys_mask |= MASK_S; }
                    if joy_dx < -0.3 { keys_mask |= MASK_A; }
                    if joy_dx > 0.3 { keys_mask |= MASK_D; }
                }
                let yaw_val = s.controls.yaw.get();
                let pitch_val = s.controls.pitch.get();
                s.camera.yaw.set(yaw_val);
                s.camera.pitch.set(pitch_val);

                let speed = s.params.speed * delta;
                let (sy, cy) = (yaw_val.sin(), yaw_val.cos());

                if keys_mask & MASK_W != 0 { s.camera.pos[0] -= sy * speed; s.camera.pos[2] -= cy * speed; }
                if keys_mask & MASK_S != 0 { s.camera.pos[0] += sy * speed; s.camera.pos[2] += cy * speed; }
                if keys_mask & MASK_A != 0 { s.camera.pos[0] -= cy * speed; s.camera.pos[2] += sy * speed; }
                if keys_mask & MASK_D != 0 { s.camera.pos[0] += cy * speed; s.camera.pos[2] -= sy * speed; }
                if keys_mask & MASK_Q != 0 { s.controls.yaw.set(yaw_val + speed * 1.2); }
                if keys_mask & MASK_E != 0 { s.controls.yaw.set(yaw_val - speed * 1.2); }

                if s.params.fly_mode {
                    if keys_mask & MASK_SPACE != 0 { s.camera.pos[1] += speed * 2.0; }
                    if keys_mask & MASK_SHIFT != 0 { s.camera.pos[1] -= speed * 2.0; }
                } else {
                    if keys_mask & MASK_SPACE != 0 && (s.camera.pos[1] - terrain::get_height(&s.params, s.camera.pos[0], s.camera.pos[2]) <= 0.1) {
                        s.vel_y = JUMP_SPEED;
                    }
                    s.vel_y -= GRAVITY * delta;
                    s.camera.pos[1] += s.vel_y * delta;

                    let ground_y = terrain::get_height(&s.params, s.camera.pos[0], s.camera.pos[2]);
                    if s.camera.pos[1] < ground_y + 1.8 {
                        s.camera.pos[1] = ground_y + 1.8;
                        s.vel_y = 0.0;
                    }
                    if keys_mask & MASK_SHIFT != 0 {
                        s.camera.pos[1] = s.camera.pos[1].min(ground_y + 1.2);
                    }
                }

                // Observer mode toggle (C key)
                if keys_mask & MASK_C != 0 && !s.observer_mode {
                    s.observer_mode = true;
                    s.controls.keys.set(s.controls.keys.get() & !MASK_C);
                } else if keys_mask & MASK_C != 0 && s.observer_mode {
                    s.observer_mode = false;
                    s.orbit_radius = 15.0;
                    s.controls.keys.set(s.controls.keys.get() & !MASK_C);
                }

                // Screenshot (F12)
                if keys_mask & MASK_F12 != 0 {
                    let angle = (s.camera.yaw.get() * 180.0 / std::f64::consts::PI) as i32;
                    let formula_name = s.params.formula.name();
                    let zone_name = terrain::get_zone(&s.params, s.camera.pos[0], s.camera.pos[2]).as_str();
                    bridge::capture_screenshot(s.params.seed, formula_name, zone_name,
                        s.camera.pos[0], s.camera.pos[1], s.camera.pos[2]);
                    s.controls.keys.set(s.controls.keys.get() & !MASK_F12);
                }

                // Export OBJ (G key)
                if keys_mask & MASK_G != 0 {
                    bridge::export_obj();
                    s.controls.keys.set(s.controls.keys.get() & !MASK_G);
                }

                // Save preset (H key) - save current seed to localStorage
                if keys_mask & MASK_H != 0 {
                    let seed_str = format!("{}", s.params.seed);
                    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
                        let _ = storage.set_item("worlds_last_seed", &seed_str);
                    }
                    s.controls.keys.set(s.controls.keys.get() & !MASK_H);
                }

                // Waypoint (T key)
                if keys_mask & MASK_T != 0 {
                    let wp_name = format!("WP{}", s.waypoints.len() + 1);
                    let px = s.camera.pos[0];
                    let py = s.camera.pos[1];
                    let pz = s.camera.pos[2];
                    s.waypoints.push((px, py, pz, wp_name));
                    s.controls.keys.set(s.controls.keys.get() & !MASK_T);
                }

                // Remove last waypoint (M key)
                if keys_mask & MASK_M != 0 {
                    s.waypoints.pop();
                    s.controls.keys.set(s.controls.keys.get() & !MASK_M);
                }

                let cx = (s.camera.pos[0] / CHUNK_SIZE) as i32;
                let cz = (s.camera.pos[2] / CHUNK_SIZE) as i32;
                if cx != s.prev_cx || cz != s.prev_cz {
                    generate_chunks(&mut s, cx, cz);
                }

                if s.observer_mode {
                    let ox = s.camera.pos[0];
                    let oy = s.camera.pos[1];
                    let oz = s.camera.pos[2];
                    let r = s.orbit_radius;
                    let cam_x = ox + r * pitch_val.cos() * yaw_val.sin();
                    let cam_y = oy + r * pitch_val.sin();
                    let cam_z = oz + r * pitch_val.cos() * yaw_val.cos();
                    bridge::update_camera(cam_x, cam_y, cam_z, yaw_val, pitch_val);
                } else {
                    bridge::update_camera(
                        s.camera.pos[0], s.camera.pos[1], s.camera.pos[2],
                        yaw_val, pitch_val,
                    );
                }

                // Day/night cycle
                if s.night_mode {
                    s.time_of_day = 0.0;
                } else {
                    s.time_of_day += delta * 0.02;
                    if s.time_of_day > 1.0 { s.time_of_day -= 1.0; }
                }
                bridge::set_time(s.time_of_day);
                bridge::set_water_level(s.params.water_level);

                // Ambient particles + discovery per zone
                let zone = terrain::get_zone(&s.params, s.camera.pos[0], s.camera.pos[2]);
                if s.prev_zone.map(|z| z != zone).unwrap_or(true) {
                    s.prev_zone = Some(zone);
                    particles::remove_ambient_particles();
                    particles::spawn_zone_particles(zone, s.camera.pos[0], s.camera.pos[2], s.params.water_level);
                    // Discovery tracking
                    let zone_name = zone.as_str().to_string();
                    if !s.discovered_biomes.contains(&zone_name) {
                        s.discovered_biomes.push(zone_name.clone());
                        s.discovery_message = Some((format!("✨ Discovered: {}!", zone_name), now + 3.0));
                    }
                }

                // Discovery message timeout
                let disc_expired = s.discovery_message.as_ref().map(|(_, e)| *e).unwrap_or(f64::MAX) < now;
                if disc_expired {
                    s.discovery_message = None;
                }

                // Hidden structure proximity check
                let found_name = bridge::check_discovery(s.camera.pos[0] as f32, s.camera.pos[1] as f32, s.camera.pos[2] as f32);
                if !found_name.is_empty() {
                    s.discovery_message = Some((format!("🏛️ Found: {}!", found_name), now + 4.0));
                }

                // Audio + weather
                let walking = !s.params.fly_mode
                    && (s.camera.pos[1] - terrain::get_height(&s.params, s.camera.pos[0], s.camera.pos[2])).abs() < 0.5
                    && (keys_mask & (MASK_W | MASK_S | MASK_A | MASK_D)) != 0;
                audio::update(zone, s.params.seed, walking, s.params.speed);

                // Wind animation
                bridge::update_wind(now as f32 * 0.001);

                bridge::render();

                let ground_y = terrain::get_height(&s.params, s.camera.pos[0], s.camera.pos[2]);
                let zone = terrain::get_zone(&s.params, s.camera.pos[0], s.camera.pos[2]);
                let angle = (s.camera.yaw.get() * 180.0 / std::f64::consts::PI) as i32;
                let angle = if angle < 0 { angle + 360 } else { angle % 360 };
                let disc_msg = s.discovery_message.as_ref().map(|m| m.0.clone());
                *hud.borrow_mut() = HudData {
                    pos: s.camera.pos,
                    biome: zone.as_str().to_string(),
                    height: ground_y,
                    fps: s.fps,
                    chunks: s.chunks.len(),
                    yaw_deg: angle,
                    speed: s.params.speed,
                    fly_mode: s.params.fly_mode,
                    formula: s.params.formula.name().to_string(),
                    observer_mode: s.observer_mode,
                    waypoints: s.waypoints.clone(),
                    discovered_biomes: s.discovered_biomes.clone(),
                    discovery_message: disc_msg,
                };
            }));

            if let Some(ref c) = *closure2.borrow() {
                web_sys::window().unwrap()
                    .request_animation_frame(c.as_ref().unchecked_ref()).ok();
            }
        }));

        if let Some(ref c) = *closure.borrow() {
            let id = web_sys::window().unwrap()
                .request_animation_frame(c.as_ref().unchecked_ref()).ok();
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
}

fn generate_chunks(s: &mut GameState, cx: i32, cz: i32) {
    s.prev_cx = cx;
    s.prev_cz = cz;

    let d = s.params.render_distance as i32;
    let mut new_chunks: Vec<ChunkData> = Vec::new();
    let mut new_veg: Vec<VegData> = Vec::new();
    let mut new_structs: Vec<StructData> = Vec::new();
    let mut new_minerals: Vec<MineralData> = Vec::new();
    let mut to_compute: Vec<(i32, i32)> = Vec::new();

    for x in (cx - d)..=(cx + d) {
        for z in (cz - d)..=(cz + d) {
            let key = (x, z);
            if let Some(idx) = s.chunks.iter().position(|c| c.key() == key) {
                let chunk = s.chunks.swap_remove(idx);
                new_chunks.push(chunk);
            } else {
                to_compute.push(key);
            }
        }
    }

    // Retain vegetation for existing chunks
    for x in (cx - d)..=(cx + d) {
        for z in (cz - d)..=(cz + d) {
            if let Some(idx) = s.veg_chunks.iter().position(|v| v.cx == x && v.cz == z) {
                let veg = s.veg_chunks.swap_remove(idx);
                new_veg.push(veg);
            }
        }
    }

    // Retain structures for existing chunks
    for x in (cx - d)..=(cx + d) {
        for z in (cz - d)..=(cz + d) {
            if let Some(idx) = s.struct_chunks.iter().position(|v| v.cx == x && v.cz == z) {
                let st = s.struct_chunks.swap_remove(idx);
                new_structs.push(st);
            }
        }
    }

    // Retain minerals for existing chunks
    for x in (cx - d)..=(cx + d) {
        for z in (cz - d)..=(cz + d) {
            if let Some(idx) = s.mineral_chunks.iter().position(|v| v.cx == x && v.cz == z) {
                let m = s.mineral_chunks.swap_remove(idx);
                new_minerals.push(m);
            }
        }
    }

    // Remove chunks + sub-systems that are no longer in range
    for old in s.chunks.drain(..) {
        let key = format!("{},{}", old.cx, old.cz);
        bridge::remove_chunk(&key);
        bridge::remove_vegetation(&key);
        bridge::remove_structure(&key);
        bridge::remove_minerals(&key);
    }
    s.veg_chunks.clear();
    s.struct_chunks.clear();
    s.mineral_chunks.clear();

    // Compute new chunk data (parallel via Rayon with `--features parallel`)
    let params = s.params;
    let new_data: Vec<ChunkData> = if to_compute.len() > 1 {
        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            to_compute.par_iter()
                .map(|&(cx, cz)| chunk::compute_chunk_data(&params, cx, cz))
                .collect()
        }
        #[cfg(not(feature = "parallel"))]
        {
            to_compute.iter()
                .map(|&(cx, cz)| chunk::compute_chunk_data(&params, cx, cz))
                .collect()
        }
    } else {
        to_compute.iter()
            .map(|&(cx, cz)| chunk::compute_chunk_data(&params, cx, cz))
            .collect()
    };

    // Compute vegetation for new chunks
    let new_veg_data: Vec<VegData> = to_compute.iter()
        .map(|&(cx, cz)| vegetation::compute_chunk_vegetation(&params, cx, cz))
        .collect();

    // Compute structures for new chunks
    let new_struct_data: Vec<StructData> = to_compute.iter()
        .map(|&(cx, cz)| structures::compute_chunk_structures(&params, cx, cz))
        .collect();

    // Compute minerals for new chunks
    let new_mineral_data: Vec<MineralData> = to_compute.iter()
        .map(|&(cx, cz)| minerals::compute_chunk_minerals(&params, cx, cz))
        .collect();

    // Upload chunks + sub-systems to Three.js
    for data in &new_data {
        let key = format!("{},{}", data.cx, data.cz);
        let positions = js_sys::Float32Array::from(&data.positions[..]);
        let colors = js_sys::Float32Array::from(&data.colors[..]);
        let indices = js_sys::Uint16Array::from(&data.indices[..]);
        bridge::add_chunk(
            &key,
            &positions,
            &colors,
            &indices,
            data.cx as f32 * CHUNK_SIZE as f32,
            data.cz as f32 * CHUNK_SIZE as f32,
        );
    }

    for veg in &new_veg_data {
        if veg.instances.is_empty() { continue; }
        let key = format!("{},{}", veg.cx, veg.cz);
        let mut pos_data = Vec::with_capacity(veg.instances.len() * 3);
        let mut size_data = Vec::with_capacity(veg.instances.len());
        let mut type_data = Vec::with_capacity(veg.instances.len());
        for inst in &veg.instances {
            pos_data.push(inst.x);
            pos_data.push(inst.y);
            pos_data.push(inst.z);
            size_data.push(inst.size);
            type_data.push(inst.veg_type as u8);
        }
        let pos_arr = js_sys::Float32Array::from(&pos_data[..]);
        let size_arr = js_sys::Float32Array::from(&size_data[..]);
        let type_arr = js_sys::Uint8Array::from(&type_data[..]);
        let zone_name = terrain::get_zone(&params, veg.cx as f64 * CHUNK_SIZE + 12.0, veg.cz as f64 * CHUNK_SIZE + 12.0);
        bridge::spawn_vegetation(&key, &pos_arr, &size_arr, &type_arr, veg.instances.len() as u32, zone_name.as_str());
    }

    for st in &new_struct_data {
        if st.instances.is_empty() { continue; }
        let key = format!("{},{}", st.cx, st.cz);
        let mut struct_arr = Vec::with_capacity(st.instances.len() * 6);
        for inst in &st.instances {
            struct_arr.push(inst.x);
            struct_arr.push(inst.y);
            struct_arr.push(inst.z);
            struct_arr.push(inst.rotation);
            struct_arr.push(inst.scale);
            struct_arr.push(inst.struct_type as u8 as f32);
        }
        let struct_arr_f32 = js_sys::Float32Array::from(&struct_arr[..]);
        bridge::spawn_structure(&key, &struct_arr_f32, st.instances.len() as u32, "");
    }

    for m in &new_mineral_data {
        if m.deposits.is_empty() { continue; }
        let key = format!("{},{}", m.cx, m.cz);
        let mut min_arr = Vec::with_capacity(m.deposits.len() * 5);
        for d in &m.deposits {
            min_arr.push(d.x);
            min_arr.push(d.y);
            min_arr.push(d.z);
            min_arr.push(d.mineral_type as f32);
            min_arr.push(d.size);
        }
        let min_arr_f32 = js_sys::Float32Array::from(&min_arr[..]);
        bridge::spawn_minerals(&key, &min_arr_f32, m.deposits.len() as u32);
    }

    new_chunks.extend(new_data);
    s.chunks = new_chunks;
    new_veg.extend(new_veg_data);
    s.veg_chunks = new_veg;
    new_structs.extend(new_struct_data);
    s.struct_chunks = new_structs;
    new_minerals.extend(new_mineral_data);
    s.mineral_chunks = new_minerals;
}

impl Drop for Engine {
    fn drop(&mut self) {
        if let Some(id) = self.anim_id {
            web_sys::window().unwrap().cancel_animation_frame(id).ok();
        }
    }
}
