use crate::engine::audio;
use crate::engine::minimap::{canvas_to_world, render_full_map};
use crate::engine::terrain;
use crate::engine::terrain::Zone;
use crate::engine::{Engine, HudData};
use crate::state::{AppState, CameraMode, CharacterPreset, ParticleMode};
use leptos::html;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::window;
use web_sys::KeyboardEvent;

const SCALE_PRESETS: &[f64] = &[0.005, 0.008, 0.012, 0.015, 0.02, 0.03, 0.05];
const AMP_PRESETS: &[f64] = &[1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0];
const OCTAVE_PRESETS: &[u32] = &[2, 3, 4, 5, 6, 7, 8];
const WATER_PRESETS: &[f64] = &[-0.5, 0.0, 0.3, 0.5, 0.8, 1.2];
const CHAR_PRESETS: &[CharacterPreset] = &[
    CharacterPreset::Human, CharacterPreset::Robot,
    CharacterPreset::Beast, CharacterPreset::Ghost,
    CharacterPreset::Teddy, CharacterPreset::Panda,
    CharacterPreset::Kraken,
];
const CHAR_SCALES: &[f64] = &[0.7, 0.85, 1.0, 1.15, 1.3];

const ZONE_LIST: &[Zone] = &[
    Zone::Forest, Zone::Plains, Zone::Desert, Zone::Tundra, Zone::Jungle,
    Zone::Volcanic, Zone::Ocean, Zone::Crystal, Zone::Cave, Zone::Lava,
    Zone::Fungus, Zone::Abyss, Zone::Storm, Zone::Aurora, Zone::Magma,
];

fn zone_emoji(zone: &Zone) -> &'static str {
    match zone {
        Zone::Forest => "🌲", Zone::Plains => "🌾", Zone::Desert => "🏜️",
        Zone::Tundra => "❄️", Zone::Jungle => "🌴", Zone::Volcanic => "🌋",
        Zone::Ocean => "🌊", Zone::Crystal => "💎", Zone::Cave => "🕳️",
        Zone::Lava => "🔥", Zone::Fungus => "🍄", Zone::Abyss => "👁️",
        Zone::Storm => "⛈️", Zone::Aurora => "🌌", Zone::Magma => "🟠",
        _ => "🌍",
    }
}

fn zone_name(zone: &Zone) -> &'static str {
    zone.as_str()
}

const SLIDER: &str = "w-full h-2 appearance-none bg-white/30 rounded-full outline-none cursor-pointer accent-cyan-400 slider-thumb";
const PBTN: &str = "px-2 py-1 rounded-lg text-[10px] font-mono bg-white/15 border border-white/20 text-white/80 hover:text-white hover:bg-white/25 transition-all active:scale-85";
const TBTN_ON: &str = "flex-1 px-2 py-1.5 rounded-lg text-[10px] font-mono bg-cyan-500/40 border border-cyan-400/50 text-cyan-200 transition-all active:scale-85";
const TBTN_OFF: &str = "flex-1 px-2 py-1.5 rounded-lg text-[10px] font-mono bg-white/15 border border-white/20 text-white/80 hover:text-white hover:bg-white/25 transition-all active:scale-85";

const COLORS: &[([f32; 3], [f32; 3])] = &[
    ([0.2, 0.4, 0.8], [1.0, 0.85, 0.75]),
    ([0.1, 0.1, 0.3], [0.8, 0.7, 0.6]),
    ([0.7, 0.1, 0.1], [0.9, 0.7, 0.6]),
    ([0.1, 0.6, 0.2], [0.8, 0.9, 0.7]),
    ([0.8, 0.6, 0.1], [0.9, 0.85, 0.7]),
    ([0.4, 0.1, 0.6], [0.9, 0.8, 0.9]),
    ([0.9, 0.9, 0.9], [0.9, 0.9, 0.9]),
];

#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();
    let canvas_ref: NodeRef<html::Canvas> = NodeRef::new();
    let engine: Rc<RefCell<Option<Engine>>> = Rc::new(RefCell::new(None));
    let hud = RwSignal::new(HudData::default());
    let glow_rgb = RwSignal::new((34u8, 211u8, 238u8));
    let open_menu = RwSignal::new(None::<&'static str>);

    let map_open = RwSignal::new(false);
    let chat_input = RwSignal::new(String::new());
    let chat_msgs = RwSignal::new(Vec::<(String, String)>::new());
    let chat_send = RwSignal::new(None::<String>);
    let mp_connected = RwSignal::new(false);
    let waypoints = RwSignal::new(Vec::<(f64, f64, f64, String)>::new());
    let waypoint_counter = RwSignal::new(0u32);
    let map_canvas_ref: NodeRef<html::Canvas> = NodeRef::new();

    let toggle_menu = move |key: &'static str| {
        open_menu.update(|m| {
            if *m == Some(key) { *m = None; } else { *m = Some(key); }
        });
    };

    {
        let canvas_ref = canvas_ref.clone();
        let engine = engine.clone();
        let mut params = state.params.get();

        // Parse URL params for sharing (?seed=...&formula=...&zone=...&mod=...)
        let mut mod_url: Option<String> = None;
        if let Some(search) = web_sys::window().and_then(|w| w.location().search().ok()) {
            if !search.is_empty() {
                let stripped = search.trim_start_matches('?');
                for pair in stripped.split('&') {
                    let mut parts = pair.splitn(2, '=');
                    let key = parts.next().unwrap_or("");
                    let val = parts.next().unwrap_or("");
                    match key {
                        "seed" => {
                            if let Ok(s) = val.parse::<u32>() {
                                params.seed = s.max(1).min(9999);
                            }
                        }
                        "zone" => {
                            let z = crate::engine::terrain::Zone::from_str(val);
                            if z != crate::engine::terrain::Zone::Forest {
                                params.zone = z;
                            }
                        }
                        "scale" => {
                            if let Ok(s) = val.parse::<f64>() {
                                params.scale = s.max(0.001).min(0.1);
                            }
                        }
                        "amplitude" => {
                            if let Ok(a) = val.parse::<f64>() {
                                params.amplitude = a.max(0.5).min(20.0);
                            }
                        }
                        "fly" => {
                            params.fly_mode = val == "1" || val == "true";
                        }
                        "speed" => {
                            if let Ok(s) = val.parse::<f64>() {
                                params.speed = s.max(10.0).min(1000.0);
                            }
                        }
                        "water" => {
                            if let Ok(w) = val.parse::<f64>() {
                                params.water_level = w.max(-1.0).min(2.0);
                            }
                        }
                        "octaves" => {
                            if let Ok(o) = val.parse::<u32>() {
                                params.octaves = o.max(1).min(10);
                            }
                        }
                        "canyons" => {
                            params.canyons = val == "1" || val == "true";
                        }
                        "mutation" => {
                            if let Ok(m) = val.parse::<f64>() {
                                params.mutation = m.clamp(0.0, 1.0);
                            }
                        }
                        "hue" => {
                            if let Ok(h) = val.parse::<f64>() {
                                params.hue_shift = h.clamp(-180.0, 180.0);
                            }
                        }
                        "saturation" => {
                            if let Ok(s) = val.parse::<f64>() {
                                params.saturation = s.clamp(0.0, 2.0);
                            }
                        }
                        "char" => {
                            for (_i, c) in CHAR_PRESETS.iter().enumerate() {
                                if format!("{:?}", c).to_lowercase() == val.to_lowercase() {
                                    params.character = *c;
                                    break;
                                }
                            }
                        }
                        "particles" => {
                            match val {
                                "rain" => params.particle_mode = ParticleMode::Rain,
                                "snow" => params.particle_mode = ParticleMode::Snow,
                                _ => params.particle_mode = ParticleMode::Off,
                            }
                        }
                        "mod" => {
                            if !val.is_empty() {
                                mod_url = Some(val.to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Auto fullscreen on mobile + quality detection
        if let Some(win) = web_sys::window() {
            let is_mobile = web_sys::window().and_then(|w| w.navigator().user_agent().ok())
                .map(|ua| {
                    ua.contains("Mobile") || ua.contains("Android") || ua.contains("iPhone")
                }).unwrap_or(false);
            if is_mobile {
                // Lower settings for mobile
                params.render_distance = 2;
                params.particle_mode = ParticleMode::Off;
                // Request fullscreen on tap
                let canvas_clone = canvas_ref.clone();
                let fs_cb = Closure::<dyn Fn()>::new(move || {
                    if let Some(canvas) = canvas_clone.get() {
                        let el: &web_sys::Element = canvas.as_ref();
                        let _ = el.request_fullscreen();
                    }
                });
                let doc = win.document().unwrap();
                doc.add_event_listener_with_callback("click", fs_cb.as_ref().unchecked_ref()).ok();
                fs_cb.forget();
            }
        }

        // Register service worker for PWA
        if let Some(win) = web_sys::window() {
            let _ = win.navigator().service_worker().register("/service-worker.js");
        }

        wasm_bindgen_futures::spawn_local(async move {
            // Inicializar IndexedDB y migrar datos existentes de localStorage
            let _ = crate::engine::db::init_async().await;
            crate::engine::db::migrate_from_local_storage().await;

            // Cargar mod desde URL si se especificó (?mod=...)
            if let Some(mod_url) = mod_url.clone() {
                let _ = crate::engine::modding::fetch_and_apply_mod(&mod_url).await;
            }

            let canvas_ref = canvas_ref.clone();
            let engine = engine.clone();
            let params = params;
            let init_cb = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
            let init_cb2 = init_cb.clone();
            let cb = Closure::<dyn FnMut()>::new(move || {
                audio::init();
                if let Some(canvas) = canvas_ref.get() {
                    let autosave = Engine::load_autosave();
                    let engine_params = autosave.as_ref().map(|d| d.params).unwrap_or(params);
                    match Engine::new(canvas, engine_params) {
                        Ok(mut e) => {
                            if let Some(ref data) = autosave {
                                e.apply_save(data);
                            }
                            e.start();
                            *engine.borrow_mut() = Some(e);
                        }
                        Err(msg) => leptos::logging::error!("Engine init error: {}", msg),
                    }
                }
                *init_cb2.borrow_mut() = None;
            });
            if let Some(win) = window() {
                win.request_animation_frame(cb.as_ref().unchecked_ref()).ok();
            }
            *init_cb.borrow_mut() = Some(cb);
        });
    }

    {
        let engine = engine.clone();
        let hud = hud;
        let mp_connected = mp_connected.clone();
        let cb = Closure::<dyn FnMut()>::new(move || {
            if let Some(ref eng) = *engine.borrow() { 
                hud.set(eng.get_hud());
                mp_connected.set(eng.is_ws_connected());
                chat_msgs.set(eng.chat_messages());
            }
        });
        let raw: &js_sys::Function = cb.as_ref().unchecked_ref();
        window().unwrap().set_interval_with_callback_and_timeout_and_arguments_0(raw, 50).ok();
        cb.forget();
    }

    // Map rendering interval (only active when map_open)
    {
        let engine = engine.clone();
        let state = state.clone();
        let hud = hud;
        let waypoints = waypoints;
        let map_open = map_open;
        let map_canvas_ref = map_canvas_ref.clone();
        let cb = Closure::<dyn FnMut()>::new(move || {
            if !map_open.get_untracked() { return; }
            if let Some(canvas) = map_canvas_ref.get() {
                if let Some(ref _eng) = *engine.borrow() {
                    let hud_data = hud.get_untracked();
                    let params = state.params.get_untracked();
                    let wps = waypoints.get_untracked();
                    if let Some(ctx) = canvas.get_context("2d").ok().flatten()
                        .and_then(|c| c.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
                    {
                        render_full_map(&ctx, canvas.width() as f64, canvas.height() as f64,
                            &params, hud_data.pos[0], hud_data.pos[2], hud_data.yaw_deg, &wps);
                    }
                }
            }
        });
        let raw: &js_sys::Function = cb.as_ref().unchecked_ref();
        web_sys::window().unwrap().set_interval_with_callback_and_timeout_and_arguments_0(raw, 200).ok();
        cb.forget();
    }

    // Save/Load action signals (avoid capturing !Send Rc in view closures)
    let save_action = RwSignal::new(None::<u32>);
    let load_action = RwSignal::new(None::<u32>);
    let del_action = RwSignal::new(None::<u32>);
    let save_trigger = {
        let engine = engine.clone();
        let waypoints = waypoints.clone();
        let open_menu = open_menu;
        move |slot: u32| {
            if let Some(ref eng) = *engine.borrow() {
                let wps = waypoints.get_untracked();
                let _ = eng.save_to_slot(slot, &format!("Slot {}", slot + 1), &wps, &[]);
                open_menu.set(None);
            }
        }
    };
    Effect::new({
        let save_trigger = save_trigger.clone();
        move || {
            if let Some(slot) = save_action.get() { save_trigger(slot); save_action.set(None); }
        }
    });
    Effect::new({
        let engine = engine.clone();
        let waypoints = waypoints.clone();
        let open_menu = open_menu;
        move || {
            if let Some(slot) = load_action.get() {
                load_action.set(None);
                if let Some(data) = Engine::load_from_slot(slot) {
                    if let Some(ref mut eng) = *engine.borrow_mut() {
                        eng.apply_save(&data);
                        waypoints.set(data.waypoints);
                        open_menu.set(None);
                    }
                }
            }
        }
    });
    Effect::new(move || {
        if let Some(slot) = del_action.get() {
            del_action.set(None);
            Engine::delete_slot(slot);
            open_menu.set(None);
        }
    });

    let _params_effect = {
        let engine = engine.clone();
        let state = state.clone();
        Effect::new(move || {
            let params = state.params.get();
            if let Some(ref mut eng) = *engine.borrow_mut() { eng.update_params(&params); }
        })
    };

    // Multiplayer action signals + effects (avoid capturing !Send Rc in view closures)
    let mp_connect_url = RwSignal::new(None::<String>);
    let mp_disconnect = RwSignal::new(false);
    Effect::new({
        let engine = engine.clone();
        let state = state.clone();
        let open_menu = open_menu;
        move || {
            if let Some(url) = mp_connect_url.get() {
                mp_connect_url.set(None);
                if let Some(ref mut eng) = *engine.borrow_mut() {
                    eng.connect_multiplayer(&url, state.params.get_untracked().seed);
                }
                open_menu.set(None);
            }
        }
    });
    Effect::new({
        let engine = engine.clone();
        let open_menu = open_menu;
        move || {
            if mp_disconnect.get() {
                mp_disconnect.set(false);
                if let Some(ref mut eng) = *engine.borrow_mut() {
                    eng.disconnect_multiplayer();
                }
                open_menu.set(None);
            }
        }
    });
    // Chat keydown handler (signal-based, avoids capturing !Send Rc)
    let on_chat_keydown = {
        let chat_input = chat_input.clone();
        move |ev: KeyboardEvent| {
            if ev.key() == "Enter" {
                let text = chat_input.get_untracked();
                if !text.is_empty() {
                    chat_send.set(Some(text));
                    chat_input.set(String::new());
                }
            }
        }
    };
    Effect::new({
        let engine = engine.clone();
        move || {
            if let Some(text) = chat_send.get() {
                chat_send.set(None);
                if let Some(ref mut eng) = *engine.borrow_mut() {
                    eng.send_chat(&text);
                }
            }
        }
    });

    view! {
        <div class="w-screen h-screen overflow-hidden relative select-none antialiased"
            style="font-family: 'Inter', 'Orbitron', system-ui, sans-serif; background: #0a0a12;">

            <canvas node_ref=canvas_ref
                class="absolute inset-0 w-full h-full outline-none touch-none"
                tabindex="0" role="application" aria-label="WORLDS game canvas"
            />

            <div class="absolute top-0 left-0 right-0 z-10 h-12 bg-gradient-to-b from-black/60 via-black/30 to-transparent flex items-center justify-between px-4 max-sm:px-2 max-sm:h-10">
                <div class="flex items-center gap-3 max-sm:gap-1">
                    <span class="text-white font-bold text-sm max-sm:text-[10px] font-mono tabular-nums tracking-wider"
                        style={move || format!("color: rgb({},{},{})", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}>
                        {move || format!("{:04}", hud.get().pos[0])}
                    </span>
                    <span class="text-white/15 text-xs font-mono">/</span>
                    <span class="text-white font-bold text-sm max-sm:text-[10px] font-mono tabular-nums tracking-wider"
                        style={move || format!("color: rgb({},{},{})", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}>
                        {move || format!("{:04}", hud.get().pos[2])}
                    </span>
                    <span class="text-white/20 text-sm max-sm:hidden">|</span>
                    <span class="text-xs max-sm:text-[9px] font-mono text-white/40 truncate max-w-[100px] max-sm:max-w-[60px]">{move || hud.get().biome}</span>
                </div>
                <div class="flex items-center gap-2 max-sm:gap-1">
                    <span class="text-[10px] max-sm:text-[8px] font-mono text-white/70 bg-white/[0.12] px-2 py-0.5 rounded-full max-sm:px-1 max-sm:py-0"
                        title="Scale/Ampl/Oct">
                        {move || {
                            let p = state.params.get();
                            format!("⚙️{:.3} 📏{:.0} 🔢{}", p.scale, p.amplitude, p.octaves)
                        }}
                    </span>
                    <span class="text-[10px] max-sm:text-[8px] font-mono text-white/70 bg-white/[0.12] px-2 py-0.5 rounded-full max-sm:hidden"
                        title="Water level / Nivel de agua">
                        {move || format!("💧{:.1}", state.params.get().water_level)}
                    </span>
                    <span class="text-[10px] max-sm:text-[8px] font-mono text-white/70 bg-white/[0.12] px-2 py-0.5 rounded-full max-sm:px-1 max-sm:py-0"
                        title="Zone / Zona">
                        {move || {
                            let z = state.params.get().zone;
                            format!("{}{}", zone_emoji(&z), zone_name(&z))
                        }}
                    </span>
                    <span class="text-[10px] max-sm:text-[8px] font-mono text-white/70 bg-white/[0.12] px-2 py-0.5 rounded-full max-sm:hidden"
                        title="Character/Particles / Personaje/Partículas">
                        {move || {
                            let p = state.params.get();
                            let part = match p.particle_mode {
                                ParticleMode::Off => "",
                                ParticleMode::Rain => "🌧️",
                                ParticleMode::Snow => "❄️",
                            };
                            format!("🧑{:.1}{}", p.char_scale, part)
                        }}
                    </span>
                    <span class="text-[10px] max-sm:text-[8px] font-mono text-white/70 bg-white/[0.12] px-2 py-0.5 rounded-full max-sm:px-1 max-sm:py-0"
                        title="Seed / Semilla">
                        {move || format!("🌱{}", state.params.get().seed)}
                    </span>
                    <span class="text-[10px] max-sm:text-[8px] font-mono text-white/70 bg-white/[0.12] px-2 py-0.5 rounded-full max-sm:hidden"
                        title="Weather / Clima">
                        {move || {
                            let h = hud.get();
                            let icon = if h.lightning { "⚡" } else { "" };
                            format!("{}{}", icon, h.weather_label)
                        }}
                    </span>
                    <span class="text-[10px] max-sm:text-[8px] font-mono text-white/80">{move || format!("{}fps", hud.get().fps)}</span>
                    {move || {
                        if hud.get().mounted {
                            view! { <span class="text-[10px] max-sm:text-[8px] font-mono text-amber-300 bg-amber-900/40 px-2 py-0.5 rounded-full max-sm:px-1">{String::from("🐎")}</span> }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }
                    }}
                    <span class="text-[10px] max-sm:text-[8px] font-mono text-white/70 bg-white/[0.12] px-2 py-0.5 rounded-full max-sm:hidden"
                        title="Season / Estación">
                        {move || {
                            let s = hud.get().season;
                            match s { 0 => "🌸Spr", 1 => "☀️Sum", 2 => "🍂Aut", _ => "❄️Win" }
                        }}
                    </span>
                </div>
            </div>

            {move || open_menu.get().map(|_| {
                view! {
                    <div class="fixed inset-0 z-15" on:click=move |_| open_menu.set(None) />
                }
            })}

            <div class="absolute left-3 top-1/2 -translate-y-1/2 z-20 flex flex-col gap-2.5 max-sm:gap-1.5 max-sm:left-1.5">
                <button aria-label="Seed" on:click=move |_| toggle_menu("seed") class="max-sm:w-9 max-sm:h-9 max-sm:text-base w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12] focus:outline-none focus:ring-2 focus:ring-cyan-400/50" title="Seed">{ "🌱" }</button>
                <button aria-label="Movement" on:click=move |_| toggle_menu("movement") class="max-sm:w-9 max-sm:h-9 max-sm:text-base w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12] focus:outline-none focus:ring-2 focus:ring-cyan-400/50" title="Movement">{ "🏃" }</button>
                <button aria-label="Camera" on:click=move |_| toggle_menu("camera") class="max-sm:w-9 max-sm:h-9 max-sm:text-base w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12] focus:outline-none focus:ring-2 focus:ring-cyan-400/50" title="Camera">{ "🎥" }</button>
                <button aria-label="Day and Night" on:click=move |_| toggle_menu("daynight") class="max-sm:w-9 max-sm:h-9 max-sm:text-base w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12] focus:outline-none focus:ring-2 focus:ring-cyan-400/50" title="Day/Night">{ "☀️" }</button>
                <button aria-label="Map" on:click=move |_| map_open.set(!map_open.get()) class="max-sm:w-9 max-sm:h-9 max-sm:text-base w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12] focus:outline-none focus:ring-2 focus:ring-cyan-400/50" title="Map">{ "🧭" }</button>
                <button aria-label="Save and Load" on:click=move |_| toggle_menu("saves") class="max-sm:w-9 max-sm:h-9 max-sm:text-base w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12] focus:outline-none focus:ring-2 focus:ring-cyan-400/50" title="Save/Load">{ "💾" }</button>
                <button aria-label="Crafting" on:click=move |_| toggle_menu("crafting") class="max-sm:hidden w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12] focus:outline-none focus:ring-2 focus:ring-cyan-400/50" title="Crafting">{ "⚒️" }</button>
                <button aria-label="Codex" on:click=move |_| toggle_menu("codex") class="max-sm:hidden w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12] focus:outline-none focus:ring-2 focus:ring-cyan-400/50" title="Codex">{ "📖" }</button>
                <button aria-label="Multiplayer" on:click=move |_| toggle_menu("multiplayer") class="max-sm:hidden w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12] focus:outline-none focus:ring-2 focus:ring-cyan-400/50" title="Multiplayer">{ "🌐" }</button>
            </div>

            <div class="absolute left-[78px] top-1/2 -translate-y-1/2 z-20 flex-col gap-2.5 max-sm:hidden max-sm:left-[52px] hidden sm:flex">
                <button on:click=move |_| toggle_menu("scale") class="w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12]" title="Scale / Escala">{ "⚙️" }</button>
                <button on:click=move |_| toggle_menu("amplitude") class="w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12]" title="Amplitude / Amplitud">{ "📏" }</button>
                <button on:click=move |_| toggle_menu("octaves") class="w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12]" title="Octaves / Octavas">{ "🔢" }</button>
                <button on:click=move |_| toggle_menu("water") class="w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12]" title="Water / Agua">{ "💧" }</button>
                <button on:click=move |_| toggle_menu("zone") class="w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12]" title="Zone / Zona">{ "🌍" }</button>
                <button on:click=move |_| toggle_menu("canyons") class="w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12]" title="Canyons / Cañones">{ "🏔️" }</button>
                <button on:click=move |_| toggle_menu("particles") class="w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12]" title="Particles / Partículas">{ "🌧️" }</button>
                <button on:click=move |_| toggle_menu("profiling") class="w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12]" title="Profiling / Rendimiento">{ "📊" }</button>
            </div>

            <div class="absolute left-[156px] top-1/2 -translate-y-1/2 z-20 flex-col gap-2.5 max-sm:hidden hidden sm:flex">
                <button on:click=move |_| toggle_menu("character") class="w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12]" title="Character / Personaje">{ "🧑" }</button>
                <button on:click=move |_| toggle_menu("color") class="w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12]" title="Color / Esquema de color">{ "🎨" }</button>
                <button on:click=move |_| toggle_menu("charscale") class="w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12]" title="Char Size / Tamaño">{ "📐" }</button>
                <button on:click=move |_| toggle_menu("season") class="w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/20 text-white/80 hover:text-white hover:bg-white/[0.12]" title="Seasons / Estaciones">{ "🍂" }</button>
            </div>

            {move || open_menu.get().map(|_| {
                view! {
                    <div class="absolute left-[236px] max-sm:left-3 top-1/2 -translate-y-1/2 z-25 max-sm:max-w-[calc(100vw-16px)]">
                        <div class="bg-black/60 backdrop-blur-xl border border-white/20 rounded-2xl p-4 min-w-[200px] shadow-2xl max-sm:max-h-[60vh] max-sm:overflow-y-auto">
                            <div class="text-white/80 text-[10px] font-mono mb-3 uppercase tracking-widest">
                                {move || match open_menu.get() {
                                    Some("seed") => "Semilla",
                                    Some("movement") => "Movimiento y Física",
                                    Some("camera") => "Cámara",
                                    Some("daynight") => "Ciclo Día/Noche",
                                    Some("scale") => "Escala del Terreno",
                                    Some("amplitude") => "Amplitud",
                                    Some("octaves") => "Octavas",
                                    Some("water") => "Nivel del Agua",
                                    Some("zone") => "Zona",
                                    Some("canyons") => "Cañones",
                                    Some("particles") => "Partículas",
                                    Some("character") => "Personaje",
                                    Some("color") => "Esquema de Color",
                                    Some("charscale") => "Tamaño del Personaje",
                                    Some("saves") => "Guardar / Cargar",
                                    Some("season") => "Estaciones",
                                    Some("codex") => "Codex / Bestiario",
                                    Some("profiling") => "Profiling / Rendimiento",
                                    _ => "",
                                }}
                            </div>
                            {move || if open_menu.get() == Some("seed") { Some(view! {
                                <div class="flex flex-col gap-3">
                                    <input type="number" min="1" max="9999"
                                        prop:value={move || state.params.get().seed.to_string()}
                                        on:input=move |ev| { state.params.update(|p| p.seed = event_target_value(&ev).parse::<u32>().unwrap_or(1).max(1).min(9999)); }
                                        class="w-full px-3 py-2 rounded-xl bg-white/[0.06] border border-white/[0.08] text-white/80 text-sm font-mono outline-none focus:border-cyan-400/30 transition-all"
                                    />
                                    <button on:click=move |_| {
                                        state.params.update(|p| p.seed = (js_sys::Math::random() * 9999.0) as u32 + 1);
                                        open_menu.set(None);
                                    } class=PBTN>"🎲 Aleatorio"</button>
                                    <button on:click=move |_| {
                                        let p = state.params.get_untracked();
                                        let base = web_sys::window().and_then(|w| w.location().href().ok()).unwrap_or_default().split('?').next().unwrap_or("").to_string();
                                        let url = format!("{}?seed={}&zone={}&scale={:.3}&amplitude={:.1}&water={:.1}&octaves={}&canyons={}&mutation={:.2}&speed={:.0}&fly={}&hue={:.0}&saturation={:.2}&char={:?}&particles={}",
                                            base, p.seed, p.zone.as_str(), p.scale, p.amplitude, p.water_level,
                                            p.octaves, if p.canyons {1} else {0}, p.mutation, p.speed,
                                            if p.fly_mode {1} else {0}, p.hue_shift, p.saturation,
                                            p.character,
                                            match p.particle_mode { ParticleMode::Rain => "rain", ParticleMode::Snow => "snow", _ => "off" });
                                        let _ = web_sys::window().map(|w| {
                                            w.navigator().clipboard().write_text(&url)
                                        });
                                        open_menu.set(None);
                                    } class=PBTN>"🔗 Compartir"</button>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("movement") { Some(view! {
                                <div class="flex flex-col gap-2.5" style="min-width:220px">
                                    <div class="flex gap-2">
                                        <button on:click=move |_| state.params.update(|p| p.fly_mode = false)
                                            class={move || if !state.params.get().fly_mode { TBTN_ON } else { TBTN_OFF }}>"Normal"</button>
                                        <button on:click=move |_| state.params.update(|p| p.fly_mode = true)
                                            class={move || if state.params.get().fly_mode { TBTN_ON } else { TBTN_OFF }}>"Vuelo"</button>
                                    </div>
                                    <div><div class="text-white/80 text-[10px] font-mono mb-1">Velocidad <span class="text-white/60 ml-1">{move || format!("{:.0}", state.params.get().speed)}</span></div>
                                        <input type="range" min="10" max="1000" step="10" prop:value={move || format!("{}", state.params.get().speed as u32)} on:input=move |ev| { state.params.update(|p| p.speed = event_target_value(&ev).parse::<f64>().unwrap_or(300.0).max(10.0).min(1000.0)); } class=SLIDER/></div>
                                    <div><div class="text-white/80 text-[10px] font-mono mb-1">Gravedad <span class="text-white/60 ml-1">{move || format!("{:.1}", state.params.get().gravity)}</span></div>
                                        <input type="range" min="5" max="40" step="0.5" prop:value={move || format!("{:.1}", state.params.get().gravity)} on:input=move |ev| { state.params.update(|p| p.gravity = event_target_value(&ev).parse::<f64>().unwrap_or(20.0).max(5.0).min(40.0)); } class=SLIDER/></div>
                                    <div><div class="text-white/80 text-[10px] font-mono mb-1">Salto <span class="text-white/60 ml-1">{move || format!("{:.1}", state.params.get().jump_speed)}</span></div>
                                        <input type="range" min="2" max="50" step="1" prop:value={move || format!("{:.1}", state.params.get().jump_speed)} on:input=move |ev| { state.params.update(|p| p.jump_speed = event_target_value(&ev).parse::<f64>().unwrap_or(10.0).max(2.0).min(50.0)); } class=SLIDER/></div>
                                    <div><div class="text-white/80 text-[10px] font-mono mb-1">Paso máx. <span class="text-white/60 ml-1">{move || format!("{:.2}", state.params.get().step_height)}</span></div>
                                        <input type="range" min="0.1" max="2.0" step="0.05" prop:value={move || format!("{:.2}", state.params.get().step_height)} on:input=move |ev| { state.params.update(|p| p.step_height = event_target_value(&ev).parse::<f64>().unwrap_or(0.7).max(0.1).min(2.0)); } class=SLIDER/></div>
                                    <div><div class="text-white/80 text-[10px] font-mono mb-1">Aceleración <span class="text-white/60 ml-1">{move || format!("{:.0}", state.params.get().movement_accel)}</span></div>
                                        <input type="range" min="10" max="500" step="5" prop:value={move || format!("{}", state.params.get().movement_accel as u32)} on:input=move |ev| { state.params.update(|p| p.movement_accel = event_target_value(&ev).parse::<f64>().unwrap_or(200.0).max(10.0).min(500.0)); } class=SLIDER/></div>
                                    <div><div class="text-white/80 text-[10px] font-mono mb-1">Fricción <span class="text-white/60 ml-1">{move || format!("{:.0}", state.params.get().movement_friction)}</span></div>
                                        <input type="range" min="1" max="100" step="1" prop:value={move || format!("{}", state.params.get().movement_friction as u32)} on:input=move |ev| { state.params.update(|p| p.movement_friction = event_target_value(&ev).parse::<f64>().unwrap_or(30.0).max(1.0).min(100.0)); } class=SLIDER/></div>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("camera") { Some(view! {
                                <div class="flex flex-col gap-2.5" style="min-width:200px">
                                    <div class="flex gap-2">
                                        <button on:click=move |_| state.params.update(|p| p.camera_mode = CameraMode::FirstPerson)
                                            class={move || if state.params.get().camera_mode == CameraMode::FirstPerson { TBTN_ON } else { TBTN_OFF }}>"1ª Persona"</button>
                                        <button on:click=move |_| state.params.update(|p| p.camera_mode = CameraMode::ThirdPerson)
                                            class={move || if state.params.get().camera_mode == CameraMode::ThirdPerson { TBTN_ON } else { TBTN_OFF }}>"3ª Persona"</button>
                                    </div>
                                    <div><div class="text-white/80 text-[10px] font-mono mb-1">Distancia de renderizado <span class="text-white/60 ml-1">{move || format!("{}", state.params.get().render_distance)}</span></div>
                                        <input type="range" min="1" max="6" step="1" prop:value={move || format!("{}", state.params.get().render_distance)} on:input=move |ev| { state.params.update(|p| p.render_distance = event_target_value(&ev).parse::<u32>().unwrap_or(4).max(1).min(6)); } class=SLIDER/></div>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("daynight") { Some(view! {
                                <div class="flex flex-col gap-2">
                                    <div class="flex gap-2">
                                        <button on:click=move |_| state.params.update(|p| p.day_speed = 0.0)
                                            class={move || if state.params.get().day_speed == 0.0 { TBTN_ON } else { TBTN_OFF }}>"Estático"</button>
                                        <button on:click=move |_| state.params.update(|p| p.day_speed = 0.05)
                                            class={move || if (state.params.get().day_speed - 0.05).abs() < 0.01 { TBTN_ON } else { TBTN_OFF }}>"Lento"</button>
                                        <button on:click=move |_| state.params.update(|p| p.day_speed = 0.15)
                                            class={move || if (state.params.get().day_speed - 0.15).abs() < 0.01 { TBTN_ON } else { TBTN_OFF }}>"Rápido"</button>
                                    </div>
                                    <div><div class="text-white/80 text-[10px] font-mono mb-1">Velocidad <span class="text-white/60 ml-1">{move || format!("{:.2}", state.params.get().day_speed)}</span></div>
                                        <input type="range" min="0" max="0.5" step="0.01" prop:value={move || format!("{:.2}", state.params.get().day_speed)} on:input=move |ev| { state.params.update(|p| p.day_speed = event_target_value(&ev).parse::<f64>().unwrap_or(0.0).max(0.0).min(0.5)); } class=SLIDER/></div>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("scale") { Some(view! {
                                <div class="flex flex-col gap-2">
                                    <input type="range" min="0.001" max="0.1" step="0.001" prop:value={move || format!("{:.3}", state.params.get().scale)} on:input=move |ev| { state.params.update(|p| p.scale = event_target_value(&ev).parse::<f64>().unwrap_or(0.015).max(0.001).min(0.1)); } class=SLIDER/>
                                    <div class="text-center text-white/70 text-xs font-mono">{move || format!("{:.3}", state.params.get().scale)}</div>
                                    <div class="flex gap-1 flex-wrap justify-center">
                                        {SCALE_PRESETS.iter().map(|&v| { let sv = v; view! {
                                            <button on:click=move |_| state.params.update(|p| p.scale = sv) class={move || format!("{} {}", PBTN, if (state.params.get().scale - sv).abs() < 0.0005 { "bg-cyan-500/15 border-cyan-400/20 text-cyan-300" } else { "" })}>{format!("{:.3}", v)}</button>
                                        }}).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("amplitude") { Some(view! {
                                <div class="flex flex-col gap-2">
                                    <input type="range" min="0.5" max="20" step="0.5" prop:value={move || format!("{:.1}", state.params.get().amplitude)} on:input=move |ev| { state.params.update(|p| p.amplitude = event_target_value(&ev).parse::<f64>().unwrap_or(4.0).max(0.5).min(20.0)); } class=SLIDER/>
                                    <div class="text-center text-white/70 text-xs font-mono">{move || format!("{:.1}", state.params.get().amplitude)}</div>
                                    <div class="flex gap-1 flex-wrap justify-center">
                                        {AMP_PRESETS.iter().map(|&v| { let av = v; view! {
                                            <button on:click=move |_| state.params.update(|p| p.amplitude = av) class={move || format!("{} {}", PBTN, if (state.params.get().amplitude - av).abs() < 0.25 { "bg-cyan-500/15 border-cyan-400/20 text-cyan-300" } else { "" })}>{format!("{:.0}", v)}</button>
                                        }}).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("octaves") { Some(view! {
                                <div class="flex flex-col gap-2">
                                    <div class="flex items-center justify-center gap-3">
                                        <button on:click=move |_| state.params.update(|p| if p.octaves > 1 { p.octaves -= 1; }) class="w-8 h-8 rounded-xl bg-white/[0.06] border border-white/[0.08] text-white/60 hover:text-white/90 flex items-center justify-center text-lg font-mono transition-all active:scale-85">"-"</button>
                                        <span class="text-white/90 text-lg font-mono min-w-[2ch] text-center">{move || state.params.get().octaves.to_string()}</span>
                                        <button on:click=move |_| state.params.update(|p| if p.octaves < 10 { p.octaves += 1; }) class="w-8 h-8 rounded-xl bg-white/[0.06] border border-white/[0.08] text-white/60 hover:text-white/90 flex items-center justify-center text-lg font-mono transition-all active:scale-85">"+"</button>
                                    </div>
                                    <div class="flex gap-1 flex-wrap justify-center">
                                        {OCTAVE_PRESETS.iter().map(|&v| { let ov = v; view! {
                                            <button on:click=move |_| state.params.update(|p| p.octaves = ov) class={move || format!("{} {}", PBTN, if state.params.get().octaves == ov { "bg-cyan-500/15 border-cyan-400/20 text-cyan-300" } else { "" })}>{format!("{}", v)}</button>
                                        }}).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("water") { Some(view! {
                                <div class="flex flex-col gap-2">
                                    <input type="range" min="-1.0" max="2.0" step="0.1" prop:value={move || format!("{:.1}", state.params.get().water_level)} on:input=move |ev| { state.params.update(|p| p.water_level = event_target_value(&ev).parse::<f64>().unwrap_or(0.0).max(-1.0).min(2.0)); } class=SLIDER/>
                                    <div class="text-center text-white/70 text-xs font-mono">{move || format!("{:.1}", state.params.get().water_level)}</div>
                                    <div class="flex gap-1 flex-wrap justify-center">
                                        {WATER_PRESETS.iter().map(|&v| { let wv = v; view! {
                                            <button on:click=move |_| state.params.update(|p| p.water_level = wv) class={move || format!("{} {}", PBTN, if (state.params.get().water_level - wv).abs() < 0.05 { "bg-cyan-500/15 border-cyan-400/20 text-cyan-300" } else { "" })}>{format!("{:.1}", v)}</button>
                                        }}).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("zone") { Some(view! {
                                <div class="grid grid-cols-3 gap-1.5 max-h-[260px] overflow-y-auto pr-1">
                                    {ZONE_LIST.iter().map(|z| {
                                        let zone = *z;
                                        view! {
                                            <button on:click=move |_| state.params.update(|p| p.zone = zone) class={move || {
                                                let active = state.params.get().zone == zone;
                                                format!("flex flex-col items-center gap-0.5 px-2 py-2 rounded-xl text-[10px] font-mono transition-all active:scale-85 border {}",
                                                    if active { "bg-cyan-500/15 border-cyan-400/20 text-cyan-300" } else { "bg-white/[0.04] border-white/[0.04] text-white/50 hover:text-white/80 hover:bg-white/[0.08]" })
                                            }}>
                                                <span class="text-base">{zone_emoji(&zone)}</span>
                                                <span>{zone_name(&zone)}</span>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("canyons") { Some(view! {
                                <div class="flex gap-2">
                                    <button on:click=move |_| state.params.update(|p| p.canyons = false) class={move || if !state.params.get().canyons { TBTN_ON } else { TBTN_OFF }}>"Off"</button>
                                    <button on:click=move |_| state.params.update(|p| p.canyons = true) class={move || if state.params.get().canyons { TBTN_ON } else { TBTN_OFF }}>"On"</button>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("particles") { Some(view! {
                                <div class="flex gap-2">
                                    <button on:click=move |_| state.params.update(|p| p.particle_mode = ParticleMode::Off) class={move || if state.params.get().particle_mode == ParticleMode::Off { TBTN_ON } else { TBTN_OFF }}>"Off"</button>
                                    <button on:click=move |_| state.params.update(|p| p.particle_mode = ParticleMode::Rain) class={move || if state.params.get().particle_mode == ParticleMode::Rain { TBTN_ON } else { TBTN_OFF }}>"🌧️ Lluvia"</button>
                                    <button on:click=move |_| state.params.update(|p| p.particle_mode = ParticleMode::Snow) class={move || if state.params.get().particle_mode == ParticleMode::Snow { TBTN_ON } else { TBTN_OFF }}>"❄️ Nieve"</button>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("character") { Some(view! {
                                <div class="flex flex-col gap-1.5">
                                    {CHAR_PRESETS.iter().map(|c| {
                                        let preset = *c;
                                        let name = match preset { CharacterPreset::Human => "Humano", CharacterPreset::Robot => "Robot", CharacterPreset::Beast => "Bestia", CharacterPreset::Ghost => "Fantasma", CharacterPreset::Teddy => "Teddy Bear", CharacterPreset::Panda => "Panda", CharacterPreset::Kraken => "Kraken", };
                                        view! {
                                            <button on:click=move |_| state.params.update(|p| p.character = preset) class={move || {
                                                let active = state.params.get().character == preset;
                                                format!("w-full px-3 py-2 rounded-xl text-xs font-mono text-left transition-all active:scale-85 border {}",
                                                    if active { "bg-cyan-500/15 border-cyan-400/20 text-cyan-300" } else { "bg-white/[0.04] border-white/[0.04] text-white/50 hover:text-white/80 hover:bg-white/[0.08]" })
                                            }}>{name}</button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("color") { Some(view! {
                                <div class="flex gap-2 flex-wrap justify-center">
                                    {(0..7).map(|i| { let (body, _) = COLORS[i];
                                        let (r, g, b) = ((body[0]*255.0) as u8, (body[1]*255.0) as u8, (body[2]*255.0) as u8);
                                        view! {
                                            <button on:click=move |_| state.params.update(|p| p.color_scheme = i as u32) class={move || {
                                                let active = state.params.get().color_scheme as usize == i;
                                                format!("w-9 h-9 rounded-xl border transition-all active:scale-85 {}", if active { "border-cyan-400/40 ring-2 ring-cyan-400/20" } else { "border-white/[0.08]" })
                                            }} style={format!("background: rgb({},{},{})", r, g, b)}></button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("charscale") { Some(view! {
                                <div class="flex flex-col gap-2">
                                    <input type="range" min="0.5" max="1.5" step="0.05" prop:value={move || format!("{:.2}", state.params.get().char_scale)} on:input=move |ev| { state.params.update(|p| p.char_scale = event_target_value(&ev).parse::<f64>().unwrap_or(1.0).max(0.5).min(1.5)); } class=SLIDER/>
                                    <div class="text-center text-white/70 text-xs font-mono">{move || format!("{:.2}", state.params.get().char_scale)}</div>
                                    <div class="flex gap-1 flex-wrap justify-center">
                                        {CHAR_SCALES.iter().map(|&v| { let cv = v; view! {
                                            <button on:click=move |_| state.params.update(|p| p.char_scale = cv) class={move || format!("{} {}", PBTN, if (state.params.get().char_scale - cv).abs() < 0.025 { "bg-cyan-500/15 border-cyan-400/20 text-cyan-300" } else { "" })}>{format!("{:.2}", v)}</button>
                                        }}).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("saves") { Some(view! {
                                <div class="flex flex-col gap-2 min-w-[280px]">
                                    {move || {
                                        let slots = Engine::list_slots();
                                        (0u32..5).map(|slot| {
                                            let saved = slots.iter().any(|(s, _)| *s == slot);
                                            let s_save = save_action;
                                            let s_load = load_action;
                                            let s_del = del_action;
                                            let slot_data = if saved {
                                                Engine::load_from_slot(slot)
                                            } else { None };
                                            let label = slot_data.as_ref().map(|d| {
                                                format!("{} — {} | seed:{}", d.slot_name, d.params.zone.as_str(), d.params.seed)
                                            }).unwrap_or_default();
                                            view! {
                                                <div class="flex items-center gap-2 py-1.5 px-2 rounded-xl bg-white/[0.04] border border-white/[0.06]">
                                                    <span class="text-white/40 text-[10px] font-mono min-w-[3ch]">{slot + 1}.</span>
                                                    <span class="flex-1 text-white/50 text-[10px] font-mono truncate">
                                                        {if saved { label.clone() } else { "⬜ Vacío".to_string() }}
                                                    </span>
                                                    <button on:click=move |_| s_save.set(Some(slot)) class={format!("{} text-[9px] {}", PBTN, if saved { "text-cyan-400/70" } else { "" })}>"G"</button>
                                                    <button on:click=move |_| s_load.set(Some(slot)) class={format!("{} text-[9px] {}", PBTN, if saved { "" } else { "opacity-30 pointer-events-none" })}>"C"</button>
                                                    <button on:click=move |_| s_del.set(Some(slot)) class={format!("{} text-[9px] {}", PBTN, if saved { "text-red-400/60" } else { "opacity-30 pointer-events-none" })}>"X"</button>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()
                                    }}
                                    <div class="text-[9px] font-mono text-white/20 text-center mt-1">
                                        Auto-save cada 15s. G=Guardar C=Cargar X=Eliminar
                                    </div>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("multiplayer") { Some(view! {
                                <div class="flex flex-col gap-2 min-w-[220px]">
                                    <div class="text-white/80 text-[10px] font-mono uppercase tracking-widest mb-2">
                                        {move || if mp_connected.get() { "🌐 Conectado" } else { "🌐 Multijugador" }}
                                    </div>
                                    {move || if !mp_connected.get() {
                                        view! {
                                            <input id="mp-url" type="text"
                                                placeholder="ws://servidor:3000/ws"
                                                class="w-full px-2 py-1.5 rounded-xl bg-white/[0.06] border border-white/[0.08] text-white/80 text-xs font-mono outline-none focus:border-cyan-400/30"
                                            />
                                            <button on:click=move |_| {
                                                let url = web_sys::window().and_then(|w| {
                                                    let input = w.document().and_then(|d| d.get_element_by_id("mp-url"))?;
                                                    let el = input.dyn_into::<web_sys::HtmlInputElement>().ok()?;
                                                    Some(el.value())
                                                }).unwrap_or_default();
                                                if !url.is_empty() { mp_connect_url.set(Some(url)); }
                                            } class=PBTN>"Conectar"</button>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <button on:click=move |_| mp_disconnect.set(true) class=PBTN>"Desconectar"</button>
                                        }.into_any()
                                    }}
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("season") { Some(view! {
                                <div class="flex flex-col gap-3 min-w-[220px]">
                                    <div class="flex gap-2 items-center">
                                        {move || {
                                            let s = state.params.get().season;
                                            let icons = ["🌸", "☀️", "🍂", "❄️"];
                                            let names = ["Primavera", "Verano", "Otoño", "Invierno"];
                                            format!("{} {}", icons[s as usize], names[s as usize])
                                        }}
                                    </div>
                                    <div><div class="text-white/80 text-[10px] font-mono mb-1">Velocidad <span class="text-white/60 ml-1">{move || format!("{:.3}", state.params.get().season_speed)}</span></div>
                                        <input type="range" min="0" max="0.1" step="0.001" prop:value={move || format!("{:.3}", state.params.get().season_speed)} on:input=move |ev| { state.params.update(|p| p.season_speed = event_target_value(&ev).parse::<f64>().unwrap_or(0.01).max(0.0).min(0.1)); } class=SLIDER/></div>
                                    <div class="flex gap-2">
                                        <button on:click=move |_| state.params.update(|p| p.season = 0) class={move || if state.params.get().season == 0 { TBTN_ON } else { TBTN_OFF }}>"🌸Prim"</button>
                                        <button on:click=move |_| state.params.update(|p| p.season = 1) class={move || if state.params.get().season == 1 { TBTN_ON } else { TBTN_OFF }}>"☀️Ver"</button>
                                        <button on:click=move |_| state.params.update(|p| p.season = 2) class={move || if state.params.get().season == 2 { TBTN_ON } else { TBTN_OFF }}>"🍂Oto"</button>
                                        <button on:click=move |_| state.params.update(|p| p.season = 3) class={move || if state.params.get().season == 3 { TBTN_ON } else { TBTN_OFF }}>"❄️Inv"</button>
                                    </div>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("codex") { Some(view! {
                                <div class="flex flex-col gap-2 min-w-[220px]">
                                    <div class="text-white/80 text-[10px] font-mono uppercase tracking-widest mb-2">"📖 Codex / Bestiario"</div>
                                    {move || {
                                        let h = hud.get();
                                        vec![
                                            ("🌲 Biomas", h.codex_biomes),
                                            ("🏛️ Estructuras", h.codex_structures),
                                            ("💎 Minerales", h.codex_minerals),
                                            ("🦁 Criaturas", h.codex_creatures),
                                        ].into_iter().map(|(label, (found, total))| {
                                            let pct = if total > 0 { found * 100 / total } else { 0 };
                                            view! {
                                                <div class="flex items-center gap-2 py-1.5 px-2 rounded-xl bg-white/[0.04] border border-white/[0.06]">
                                                    <span class="text-[11px] font-mono text-white/60 min-w-[5ch]">{label}</span>
                                                    <span class="flex-1 text-right text-[11px] font-mono text-white/80">{found}{" / "}{total}</span>
                                                    <div class="w-16 h-1.5 rounded-full bg-white/[0.06] overflow-hidden">
                                                        <div class="h-full rounded-full bg-cyan-400/40 transition-all" style={format!("width: {}%", pct)}></div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()
                                    }}
                                    <div class="text-[9px] font-mono text-white/20 text-center mt-1">
                                        Presiona G para cambiar clima
                                    </div>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("profiling") { Some(view! {
                                <div class="flex flex-col gap-2 min-w-[200px]">
                                    <div class="text-white/80 text-[10px] font-mono mb-1">"📊 Rendimiento"</div>
                                    {move || {
                                        let h = hud.get();
                                        let items = vec![
                                            ("FPS", format!("{}", h.profiling_fps), h.profiling_fps >= 30),
                                            ("Chunks", format!("{}", h.profiling_chunks), true),
                                            ("Worker cola", format!("{}", h.profiling_worker_pending), h.profiling_worker_pending < 8),
                                            ("Draw calls", format!("{}", h.profiling_draw_calls), h.profiling_draw_calls < 500),
                                            ("Memoria", format!("{:.1} MB", h.profiling_memory_mb), h.profiling_memory_mb < 200.0),
                                        ];
                                        items.into_iter().map(|(label, value, ok)| {
                                            let color = if ok { "text-green-400" } else { "text-red-400" };
                                            view! {
                                                <div class="flex items-center gap-2 py-1 px-2 rounded-xl bg-white/[0.04] border border-white/[0.06]">
                                                    <span class="text-[10px] font-mono text-white/50 min-w-[9ch]">{label}</span>
                                                    <span class={format!("flex-1 text-right text-[11px] font-mono {}", color)}>{value}</span>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()
                                    }}
                                </div>
                            }) } else { None }}
                        </div>
                    </div>
                }
            })}

            {move || {
                let h = hud.get();
                h.near_portal.map(|_name| {
                    let target_seed = h.portal_target_seed.unwrap_or(0);
                    let hue = ((target_seed as f64 % 200.0) / 200.0) * 0.8;
                    let border_style = format!("border-color: hsl({}, 80%, 50%)", hue * 360.0);
                    let text_style = format!("color: hsl({}, 80%, 60%)", hue * 360.0);
                    let visited = h.visited_seeds;
                    view! {
                        <div class="absolute left-1/2 top-1/3 -translate-x-1/2 z-20 pointer-events-none flex flex-col items-center gap-2">
                            <div class="bg-black/70 backdrop-blur-md rounded-2xl px-6 py-3 shadow-2xl border border-cyan-400/30" style=border_style>
                                <div class="text-sm font-mono text-center" style=text_style>
                                    <span class="text-lg">{"🔮"}</span>
                                    <br/>{"Portal — presiona R para viajar"}
                                    <br/><span class="text-[10px] text-white/40">{"Destino: mundo #"}{target_seed}</span>
                                </div>
                            </div>
                            {(!visited.is_empty()).then(|| view! {
                                <div class="bg-black/70 backdrop-blur-md rounded-2xl px-4 py-2 shadow-2xl border border-white/10 max-w-[200px]">
                                    <div class="text-[10px] font-mono text-white/40 text-center mb-1">"🌐 Mundos visitados"</div>
                                    <div class="flex flex-wrap gap-1 justify-center">
                                        {visited.iter().rev().take(8).map(|s| {
                                            let seed = *s;
                                            let shue = ((seed as f64 % 200.0) / 200.0) * 0.8;
                                            let scolor = format!("color: hsl({}, 80%, 60%)", shue * 360.0);
                                            view! {
                                                <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-white/[0.06]" style=scolor>
                                                    {"#"}{seed}
                                                </span>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            })}
                        </div>
                    }
                })
            }}

            {move || hud.get().achievement_message.clone().map(|msg| {
                view! {
                    <div class="absolute right-6 bottom-24 z-20 pointer-events-none ach-fade-in">
                        <div class="bg-yellow-500/20 backdrop-blur-md border border-yellow-400/30 rounded-2xl px-5 py-3 shadow-2xl shadow-yellow-500/10">
                            <div class="text-yellow-300 text-sm font-mono text-center">
                                {msg}
                            </div>
                        </div>
                    </div>
                }
            })}

            // Chat overlay (visible when multiplayer connected)
            {move || {
                if !mp_connected.get() { return None; }
                let msgs = chat_msgs.get();
                Some(view! {
                    <div class="absolute bottom-4 right-4 z-20 w-80">
                        <div class="flex flex-col-reverse gap-1 max-h-32 overflow-hidden mb-1 pointer-events-none">
                            {msgs.into_iter().rev().take(8).map(|(name, text)| {
                                view! {
                                    <div class="text-[10px] font-mono text-white/90 bg-black/60 backdrop-blur-sm px-2 py-1 rounded-lg border border-white/10">
                                        <span class="text-cyan-400/80">{name}</span>
                                        <span class="text-white/40 mx-0.5">:</span>
                                        <span>{text}</span>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                        <input type="text" placeholder="Chat..."
                            prop:value=chat_input
                            on:input=move |ev| chat_input.set(event_target_value(&ev))
                            on:keydown=on_chat_keydown
                            class="w-full px-3 py-1.5 rounded-xl bg-black/60 backdrop-blur-sm border border-white/20 text-white/80 text-xs font-mono outline-none focus:border-cyan-400/40"
                        />
                    </div>
                })
            }}

            {move || map_open.get().then(|| {
                view! {
                    <div class="fixed inset-0 z-30 flex items-center justify-center bg-black/60 backdrop-blur-sm"
                        on:click=move |_| map_open.set(false)>
                        <div class="relative rounded-2xl overflow-hidden shadow-2xl border border-white/[0.08]"
                            style="width: min(90vw, 800px); height: min(90vh, 800px);"
                            on:click=move |ev| ev.stop_propagation()>
                            <canvas node_ref=map_canvas_ref
                                class="w-full h-full block"
                                width="800"
                                height="800"
                                style="image-rendering: pixelated;"
                                on:click=move |ev| {
                                    if let Some(canvas) = map_canvas_ref.get() {
                                        let rect = canvas.get_bounding_client_rect();
                                        let cw = canvas.width() as f64;
                                        let ch = canvas.height() as f64;
                                        let sx = (ev.client_x() as f64 - rect.left()) * (cw / rect.width());
                                        let sy = (ev.client_y() as f64 - rect.top()) * (ch / rect.height());
                                        let hud_data = hud.get_untracked();
                                        let params = state.params.get_untracked();
                                        let (wx, wz) = canvas_to_world(sx, sy,
                                            hud_data.pos[0], hud_data.pos[2], cw, ch);
                                        if ev.shift_key() {
                                            let mut wps = waypoints.get_untracked();
                                            if let Some(idx) = wps.iter().position(|(px, _, pz, _)| {
                                                ((px - wx).powi(2) + (pz - wz).powi(2)).sqrt() < 10.0
                                            }) {
                                                wps.remove(idx);
                                                waypoints.set(wps);
                                            }
                                        } else {
                                            let mut wps = waypoints.get_untracked();
                                            let id = waypoint_counter.get();
                                            waypoint_counter.set(id + 1);
                                            let h = terrain::get_height(&params, wx, wz);
                                            wps.push((wx, h, wz, format!("WP {}", id)));
                                            waypoints.set(wps);
                                        }
                                    }
                                }
                            />
                            <div class="absolute top-3 right-3 flex gap-2">
                                <span class="text-[10px] font-mono text-white/30 bg-black/60 px-2 py-1 rounded-lg backdrop-blur-sm">
                                    {move || format!("📍 {} waypoints", waypoints.get().len())}
                                </span>
                                <button on:click=move |_| map_open.set(false)
                                    class="w-8 h-8 rounded-xl bg-black/60 border border-white/[0.12] text-white/60 hover:text-white/90 flex items-center justify-center text-sm transition-all active:scale-85 backdrop-blur-sm"
                                >"✕"</button>
                            </div>
                            <div class="absolute bottom-3 left-3 right-3 flex justify-center gap-4">
                                <span class="text-[10px] font-mono text-white/20 bg-black/40 px-3 py-1 rounded-lg backdrop-blur-sm">Click para añadir waypoint</span>
                                <span class="text-[10px] font-mono text-white/20 bg-black/40 px-3 py-1 rounded-lg backdrop-blur-sm">Shift+Click para eliminar</span>
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
