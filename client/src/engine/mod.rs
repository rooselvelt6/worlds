pub mod bridge;
pub mod camera;
pub mod chunk;
pub mod controls;
pub mod terrain;

use std::cell::{Cell, RefCell};
use std::panic::AssertUnwindSafe;
use std::rc::Rc;

use camera::Camera;
use chunk::{ChunkData, CHUNK_SIZE};
use controls::{Controls, MASK_A, MASK_D, MASK_E, MASK_Q, MASK_S, MASK_SHIFT, MASK_SPACE, MASK_W};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::state::WorldParams;

const RENDER_DISTANCE: i32 = 3;
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
}

struct GameState {
    canvas: web_sys::HtmlCanvasElement,
    camera: Camera,
    controls: Controls,
    chunks: Vec<ChunkData>,
    params: WorldParams,
    prev_cx: i32,
    prev_cz: i32,
    last_time: f64,
    frame_count: u32,
    fps_timer: f64,
    fps: u32,
    vel_y: f64,
}

pub struct Engine {
    state: Rc<RefCell<GameState>>,
    hud: Rc<RefCell<HudData>>,
    anim_id: Option<i32>,
    _closure: Option<Rc<RefCell<Option<Closure<dyn FnMut()>>>>>,
}

impl Engine {
    pub fn new(canvas: web_sys::HtmlCanvasElement, params: WorldParams) -> Result<Self, String> {
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
            params,
            prev_cx: i32::MAX,
            prev_cz: i32::MAX,
            last_time: 0.0,
            frame_count: 0,
            fps_timer: 0.0,
            fps: 0,
            vel_y: 0.0,
        }));

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
        }
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

                let keys_mask = s.controls.keys.get();
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

                let cx = (s.camera.pos[0] / CHUNK_SIZE) as i32;
                let cz = (s.camera.pos[2] / CHUNK_SIZE) as i32;
                if cx != s.prev_cx || cz != s.prev_cz {
                    generate_chunks(&mut s, cx, cz);
                }

                bridge::update_camera(
                    s.camera.pos[0], s.camera.pos[1], s.camera.pos[2],
                    yaw_val, pitch_val,
                );

                bridge::render();

                let ground_y = terrain::get_height(&s.params, s.camera.pos[0], s.camera.pos[2]);
                let zone = terrain::get_zone(&s.params, s.camera.pos[0], s.camera.pos[2]);
                let angle = (s.camera.yaw.get() * 180.0 / std::f64::consts::PI) as i32;
                let angle = if angle < 0 { angle + 360 } else { angle % 360 };
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
}

fn generate_chunks(s: &mut GameState, cx: i32, cz: i32) {
    s.prev_cx = cx;
    s.prev_cz = cz;

    let d = RENDER_DISTANCE;
    let mut new_chunks: Vec<ChunkData> = Vec::new();
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

    // Remove chunks that are no longer in range
    for old in s.chunks.drain(..) {
        let key = format!("{},{}", old.cx, old.cz);
        bridge::remove_chunk(&key);
    }

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

    // Upload to Three.js (main thread only)
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

    new_chunks.extend(new_data);
    s.chunks = new_chunks;
}

impl Drop for Engine {
    fn drop(&mut self) {
        if let Some(id) = self.anim_id {
            web_sys::window().unwrap().cancel_animation_frame(id).ok();
        }
    }
}
