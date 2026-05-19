pub mod audio;
pub mod bridge;
pub mod camera;
pub mod chunk;
pub mod controls;
pub mod creatures;
pub mod gamepad;
pub mod inventory;
pub mod joystick;
pub mod minerals;
pub mod minimap;
pub mod particles;
pub mod portals;
pub mod structures;
pub mod terrain;
pub mod vegetation;

use std::cell::{Cell, RefCell};
use std::panic::AssertUnwindSafe;
use std::rc::Rc;



use camera::Camera;
use chunk::{ChunkData, CHUNK_SIZE};
use controls::{Controls, MASK_A, MASK_B, MASK_C, MASK_D, MASK_E, MASK_F12, MASK_G, MASK_H, MASK_LCLICK, MASK_M, MASK_Q, MASK_S, MASK_SHIFT, MASK_SPACE, MASK_T, MASK_W};
use creatures::CreatureData;
use inventory::Inventory;
use minerals::MineralData;
use portals::PortalData;
use structures::StructData;
use vegetation::VegData;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::engine::terrain::Zone;
use crate::state::{SaveData, WorldParams};

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
    pub gamepad_connected: bool,
    pub waypoints: Vec<(f64, f64, f64, String)>,
    pub discovered_biomes: Vec<String>,
    pub discovery_message: Option<String>,
    pub build_mode: bool,
    pub inventory: Vec<(u8, u32)>,
    pub selected_slot: u8,
    pub season: u8,
    pub creature_count: u32,
    pub achievement_points: u32,
    pub vr_mode: bool,
}

struct GameState {
    canvas: web_sys::HtmlCanvasElement,
    camera: Camera,
    controls: Controls,
    chunks: Vec<ChunkData>,
    veg_chunks: Vec<VegData>,
    struct_chunks: Vec<StructData>,
    mineral_chunks: Vec<MineralData>,
    creature_chunks: Vec<CreatureData>,
    portal_data: PortalData,
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
    gamepad_connected: bool,
    inventory: Inventory,
    build_mode: bool,
    mine_cooldown: f64,
    place_cooldown: f64,
    season: u8,
    season_timer: f64,
    tree_growth: Vec<(i32, i32, f32)>,
    creature_spawn_timer: f64,
    creature_count: u32,
    portal_cooldown: f64,
    last_seed: u32,
    vr_enabled: bool,
    achievements: Vec<String>,
    achievement_points: u32,
    weather_power_cooldown: f64,
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

        let portal_data = portals::compute_portals(&params);
        for p in &portal_data.portals {
            bridge::spawn_portal(&p.id, p.x, p.y, p.z, p.target_seed);
        }

        let state = Rc::new(RefCell::new(GameState {
            canvas,
            camera,
            controls,
            chunks: Vec::new(),
            veg_chunks: Vec::new(),
            struct_chunks: Vec::new(),
            mineral_chunks: Vec::new(),
            creature_chunks: Vec::new(),
            portal_data,
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
            gamepad_connected: false,
            inventory: Inventory::new(),
            build_mode: false,
            mine_cooldown: 0.0,
            place_cooldown: 0.0,
            season: 0,
            season_timer: 0.0,
            tree_growth: Vec::new(),
            creature_spawn_timer: 0.0,
            creature_count: 0,
            portal_cooldown: 0.0,
            last_seed: params.seed,
            vr_enabled: false,
            achievements: Vec::new(),
            achievement_points: 0,
            weather_power_cooldown: 0.0,
        }));

        audio::init();

        // Auto-load from save slot 0 if exists
        if let Some(data) = Self::load_from_slot(0) {
            let mut s = state.borrow_mut();
            s.params = data.params;
            s.camera.pos = data.pos;
            s.camera.yaw.set(data.yaw);
            s.camera.pitch.set(data.pitch);
            s.waypoints = data.waypoints;
            s.discovered_biomes = data.discovered_biomes;
            s.time_of_day = data.time_of_day;
            s.params.fly_mode = data.fly_mode;
            s.observer_mode = data.observer_mode;
            drop(s);
        }

        {
            let mut s = state.borrow_mut();
            bridge::ws_connect(s.params.seed);
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
        s.creature_chunks.clear();
        // Remove portals from JS
        for p in &s.portal_data.portals {
            bridge::remove_portal(&p.id);
        }
        s.portal_data = portals::compute_portals(&s.params);
        for p in &s.portal_data.portals {
            bridge::spawn_portal(&p.id, p.x, p.y, p.z, p.target_seed);
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

                let mut keys_mask = s.controls.keys.get();
                let joy_dx = s.joy_dx.get();
                let joy_dy = s.joy_dy.get();
                if joy_dx.abs() > 0.3 || joy_dy.abs() > 0.3 {
                    if joy_dy < -0.3 { keys_mask |= MASK_W; }
                    if joy_dy > 0.3 { keys_mask |= MASK_S; }
                    if joy_dx < -0.3 { keys_mask |= MASK_A; }
                    if joy_dx > 0.3 { keys_mask |= MASK_D; }
                }

                // Gamepad polling
                let gp = gamepad::poll_gamepad();
                s.gamepad_connected = gp.connected;
                if gp.connected {
                    let sens = s.params.mouse_sensitivity;
                    if gp.axes[0].abs() > 0.15 {
                        if gp.axes[0] < -0.15 { keys_mask |= MASK_A; }
                        if gp.axes[0] > 0.15 { keys_mask |= MASK_D; }
                    }
                    if gp.axes[1].abs() > 0.15 {
                        if gp.axes[1] < -0.15 { keys_mask |= MASK_W; }
                        if gp.axes[1] > 0.15 { keys_mask |= MASK_S; }
                    }
                    // Right stick -> camera (axes[2] = horizontal, axes[3] = vertical)
                    let yaw_now = s.controls.yaw.get();
                    let pitch_now = s.controls.pitch.get();
                    s.controls.yaw.set(yaw_now - gp.axes[2] * 0.05 * sens);
                    let new_pitch = (pitch_now - gp.axes[3] * 0.05 * sens).max(-1.5).min(1.5);
                    s.controls.pitch.set(new_pitch);
                    // Buttons
                    if gp.a { keys_mask |= MASK_SPACE; }
                    if gp.b { keys_mask |= MASK_SHIFT; }
                    if gp.x { keys_mask |= MASK_C; }
                    if gp.y { keys_mask |= MASK_F12; }
                    if gp.lb { keys_mask |= MASK_Q; }
                    if gp.rb { keys_mask |= MASK_E; }
                    if gp.dpad_up { keys_mask |= MASK_W; }
                    if gp.dpad_down { keys_mask |= MASK_S; }
                    if gp.dpad_left { keys_mask |= MASK_A; }
                    if gp.dpad_right { keys_mask |= MASK_D; }
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

                // F6: Build mode toggle (B key)
                if keys_mask & MASK_B != 0 {
                    s.build_mode = !s.build_mode;
                    if !s.build_mode {
                        let _ = s.canvas.request_pointer_lock();
                    }
                    s.controls.keys.set(s.controls.keys.get() & !MASK_B);
                }

                // F6: Mining (left click in normal mode) and building (left click in build mode)
                if keys_mask & MASK_LCLICK != 0 && s.mine_cooldown <= 0.0 {
                    if s.build_mode {
                        // Place block
                        let sel = s.inventory.selected_item().cloned();
                        if let Some(item) = sel {
                            if item.count > 0 {
                                if bridge::place_block(s.camera.pos[0], s.camera.pos[1], s.camera.pos[2], s.camera.yaw.get(), s.camera.pitch.get(), item.mineral_type) {
                                    if let Some(inv) = s.inventory.items.iter_mut().find(|i| i.mineral_type == item.mineral_type) {
                                        inv.count = inv.count.saturating_sub(1);
                                    }
                                    s.mine_cooldown = 0.3;
                                }
                            }
                        }
                    } else {
                        // Mine
                        let hit = bridge::mine_at(s.camera.pos[0], s.camera.pos[1], s.camera.pos[2], s.camera.yaw.get(), s.camera.pitch.get());
                        if hit >= 0.0 {
                            let mt = (hit as u32) & 0xFF;
                            s.inventory.add_mineral(1, mt);
                            s.mine_cooldown = 0.3;
                            audio::play_effect("mine");
                            // Achievement: first mined block
                            if !s.achievements.contains(&"first_mine".to_string()) {
                                s.achievements.push("first_mine".to_string());
                                s.achievement_points += 10;
                                s.discovery_message = Some(("⛏️ Achievement: First Mine! (+10 pts)".to_string(), now + 3.0));
                            }
                        }
                    }
                    s.controls.keys.set(s.controls.keys.get() & !MASK_LCLICK);
                }
                if s.mine_cooldown > 0.0 { s.mine_cooldown -= delta; }

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

                // F8: Seasons (every 120 seconds = ~1 hour in-game)
                s.season_timer += delta;
                if s.season_timer > 120.0 {
                    s.season_timer = 0.0;
                    s.season = (s.season + 1) % 4;
                    let season_names = ["spring", "summer", "autumn", "winter"];
                    bridge::set_season(season_names[s.season as usize]);
                    bridge::audio_play_effect("season");
                    s.discovery_message = Some((format!("🍂 Season: {}!", season_names[s.season as usize]), now + 3.0));
                }

                // F8: Tree growth tick
                for (gcx, gcz, growth) in &mut s.tree_growth {
                    if *growth < 1.0 {
                        *growth += delta as f32 * 0.01;
                        if *growth > 1.0 { *growth = 1.0; }
                        bridge::set_tree_growth(&format!("{},{}", gcx, gcz), *growth);
                    }
                }

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
                        s.achievement_points += 5;
                        if s.discovered_biomes.len() >= 5 && !s.achievements.contains(&"explorer".to_string()) {
                            s.achievements.push("explorer".to_string());
                            s.achievement_points += 25;
                            s.discovery_message = Some(("🌍 Achievement: Explorer! (5 biomes, +25 pts)".to_string(), now + 3.0));
                        }
                        if s.discovered_biomes.len() >= 10 && !s.achievements.contains(&"adventurer".to_string()) {
                            s.achievements.push("adventurer".to_string());
                            s.achievement_points += 50;
                            s.discovery_message = Some(("🏆 Achievement: Adventurer! (10 biomes, +50 pts)".to_string(), now + 4.0));
                        }
                    }
                }

                // Discovery message timeout
                let disc_expired = s.discovery_message.as_ref().map(|(_, e)| *e).unwrap_or(f64::MAX) < now;
                if disc_expired {
                    s.discovery_message = None;
                }

                // Hidden structure proximity check
                let found_name = bridge::check_discovery(s.camera.pos[0] as f32, s.camera.pos[1] as f32, s.camera.pos[2] as f32);
                if !found_name.is_empty() && s.discovery_message.is_none() {
                    s.discovery_message = Some((format!("🏛️ Found: {}!", found_name), now + 4.0));
                    if !s.achievements.contains(&"structure".to_string()) {
                        s.achievements.push("structure".to_string());
                        s.achievement_points += 20;
                    }
                }

                // F6: Crafting (press C while in build mode with something selected)
                if s.build_mode && keys_mask & MASK_C != 0 {
                    let result = s.inventory.craft();
                    if !result.is_empty() {
                        s.discovery_message = Some((format!("🔧 Crafted {}!", result), now + 3.0));
                        if !s.achievements.contains(&"craft".to_string()) {
                            s.achievements.push("craft".to_string());
                            s.achievement_points += 15;
                        }
                    }
                    s.controls.keys.set(s.controls.keys.get() & !MASK_C);
                }

                // F9: Creature spawning
                    s.creature_spawn_timer += delta;
                if s.creature_spawn_timer > 5.0 {
                    s.creature_spawn_timer = 0.0;
                    let mut max_type = s.creature_count;
                    for chunk in &s.creature_chunks {
                        for creature in &chunk.creatures {
                            bridge::spawn_creature(&creature.id, creature.x, creature.y, creature.z, creature.creature_type, zone.as_str());
                            max_type = max_type.max(creature.creature_type as u32 + 1);
                        }
                    }
                    s.creature_count = max_type;
                }

                // F11: Portal proximity check
                s.portal_cooldown -= delta;
                if s.portal_cooldown <= 0.0 {
                    for p in &s.portal_data.portals {
                        let dx = s.camera.pos[0] - p.x;
                        let dz = s.camera.pos[2] - p.z;
                        let dist = (dx * dx + dz * dz).sqrt();
                        if dist < p.radius {
                            let new_seed = p.target_seed;
                            if new_seed != s.params.seed {
                                s.discovery_message = Some((format!("🌀 Traveling to seed {}!", new_seed), now + 2.0));
                                s.params.seed = new_seed;
                                s.portal_cooldown = 3.0;
                                s.last_seed = new_seed;
                                // Trigger world regeneration
                                s.prev_cx = i32::MAX;
                                s.prev_cz = i32::MAX;
                            }
                            break;
                        }
                    }
                }

                // F14: Weather powers (press X when holding breath / special interaction)
                if keys_mask & MASK_F12 == 0 {
                    s.weather_power_cooldown -= delta;
                }

                // Audio + weather
                let walking = !s.params.fly_mode
                    && (s.camera.pos[1] - terrain::get_height(&s.params, s.camera.pos[0], s.camera.pos[2])).abs() < 0.5
                    && (keys_mask & (MASK_W | MASK_S | MASK_A | MASK_D)) != 0;
                audio::update(zone, s.params.seed, walking, s.params.speed);

                // Wind animation
                bridge::update_wind(now as f32 * 0.001);

                // Multiplayer: send position every frame
                bridge::ws_send_position(
                    s.camera.pos[0], s.camera.pos[1], s.camera.pos[2],
                    s.camera.yaw.get(), s.camera.pitch.get(),
                );

                // F13: Water features (spawn waterfalls near player)
                if s.frame_count % 300 == 0 {
                    let wf_zone = zone;
                    let is_water_nearby = wf_zone == Zone::Ocean || wf_zone == Zone::Jungle;
                    if is_water_nearby {
                        let wfx = s.camera.pos[0];
                        let wfz = s.camera.pos[2];
                        bridge::spawn_waterfall("near", wfx, s.params.water_level + 8.0, wfz, 6.0);
                    }
                }

                bridge::render();

                // F18: VR check
                if s.frame_count % 60 == 0 && !s.vr_enabled {
                    if bridge::is_vr_supported() && !s.vr_enabled {
                        s.vr_enabled = true;
                        if !s.achievements.contains(&"vr_ready".to_string()) {
                            s.achievements.push("vr_ready".to_string());
                            s.achievement_points += 30;
                        }
                    }
                }

                let ground_y = terrain::get_height(&s.params, s.camera.pos[0], s.camera.pos[2]);
                let zone = terrain::get_zone(&s.params, s.camera.pos[0], s.camera.pos[2]);
                let angle = (s.camera.yaw.get() * 180.0 / std::f64::consts::PI) as i32;
                let angle = if angle < 0 { angle + 360 } else { angle % 360 };
                let disc_msg = s.discovery_message.as_ref().map(|m| m.0.clone());
                let inv_vec: Vec<(u8, u32)> = s.inventory.items.iter().map(|i| (i.mineral_type, i.count)).collect();
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
                    gamepad_connected: s.gamepad_connected,
                    waypoints: s.waypoints.clone(),
                    discovered_biomes: s.discovered_biomes.clone(),
                    discovery_message: disc_msg,
                    build_mode: s.build_mode,
                    inventory: inv_vec,
                    selected_slot: s.inventory.selected_slot,
                    season: s.season,
                    creature_count: s.creature_count,
                    achievement_points: s.achievement_points,
                    vr_mode: s.vr_enabled,
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

    pub fn save_to_slot(&self, slot: u32, name: &str) {
        let s = self.state.borrow();
        let data = SaveData::new(
            name, &s.params, s.camera.pos, s.camera.yaw.get(), s.camera.pitch.get(),
            &s.waypoints, &s.discovered_biomes,
            s.time_of_day, s.params.fly_mode, s.observer_mode,
        );
        if let Ok(json) = serde_json::to_string(&data) {
            bridge::save_slot(slot, &json);
        }
    }

    pub fn load_from_slot(slot: u32) -> Option<SaveData> {
        let json = bridge::load_slot(slot);
        if json.is_empty() { return None; }
        serde_json::from_str(&json).ok()
    }

    pub fn delete_slot(slot: u32) {
        bridge::delete_slot(slot);
    }

    pub fn apply_save(&mut self, data: &SaveData) {
        let mut s = self.state.borrow_mut();
        s.params = data.params;
        s.camera.pos = data.pos;
        s.camera.yaw.set(data.yaw);
        s.camera.pitch.set(data.pitch);
        s.waypoints = data.waypoints.clone();
        s.discovered_biomes = data.discovered_biomes.clone();
        s.time_of_day = data.time_of_day;
        s.params.fly_mode = data.fly_mode;
        s.observer_mode = data.observer_mode;
        s.prev_cx = i32::MAX;
        s.prev_cz = i32::MAX;
        drop(s);
        self.regenerate_all();
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
    let mut new_creatures: Vec<CreatureData> = Vec::new();
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

    // Retain creatures for existing chunks
    for x in (cx - d)..=(cx + d) {
        for z in (cz - d)..=(cz + d) {
            if let Some(idx) = s.creature_chunks.iter().position(|v| v.cx == x && v.cz == z) {
                let cr = s.creature_chunks.swap_remove(idx);
                new_creatures.push(cr);
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
    s.creature_chunks.clear();

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

    // Compute creatures for new chunks
    let new_creature_data: Vec<CreatureData> = to_compute.iter()
        .map(|&(cx, cz)| creatures::compute_chunk_creatures(&params, cx, cz))
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
    new_creatures.extend(new_creature_data);
    s.creature_chunks = new_creatures;
}

impl Drop for Engine {
    fn drop(&mut self) {
        if let Some(id) = self.anim_id {
            web_sys::window().unwrap().cancel_animation_frame(id).ok();
        }
        audio::stop();
        bridge::ws_disconnect();
    }
}
