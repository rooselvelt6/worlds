use crate::engine::audio;
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

const SCALE_PRESETS: &[f64] = &[0.005, 0.008, 0.012, 0.015, 0.02, 0.03, 0.05];
const AMP_PRESETS: &[f64] = &[1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0];
const OCTAVE_PRESETS: &[u32] = &[2, 3, 4, 5, 6, 7, 8];
const WATER_PRESETS: &[f64] = &[-0.5, 0.0, 0.3, 0.5, 0.8, 1.2];
const CHAR_PRESETS: &[CharacterPreset] = &[
    CharacterPreset::Human, CharacterPreset::Robot,
    CharacterPreset::Beast, CharacterPreset::Ghost,
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

const BTN: &str = "w-12 h-12 rounded-2xl flex items-center justify-center text-xl transition-all duration-200 active:scale-85 shadow-lg backdrop-blur-xl border border-white/[0.06] text-white/40 hover:text-white/80 hover:bg-white/[0.04]";
const SLIDER: &str = "w-full h-1.5 appearance-none bg-white/[0.08] rounded-full outline-none cursor-pointer accent-cyan-400";
const PBTN: &str = "px-2 py-1 rounded-lg text-[10px] font-mono bg-white/[0.06] border border-white/[0.06] text-white/50 hover:text-white/80 hover:bg-white/[0.1] transition-all active:scale-85";
const TBTN_ON: &str = "flex-1 px-2 py-1.5 rounded-lg text-[10px] font-mono bg-cyan-500/20 border border-cyan-400/30 text-cyan-300 transition-all active:scale-85";
const TBTN_OFF: &str = "flex-1 px-2 py-1.5 rounded-lg text-[10px] font-mono bg-white/[0.06] border border-white/[0.06] text-white/50 hover:text-white/80 hover:bg-white/[0.1] transition-all active:scale-85";

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

    let toggle_menu = move |key: &'static str| {
        open_menu.update(|m| {
            if *m == Some(key) { *m = None; } else { *m = Some(key); }
        });
    };

    {
        let canvas_ref = canvas_ref.clone();
        let engine = engine.clone();
        let params = state.params.get();
        let init_cb = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
        let init_cb2 = init_cb.clone();
        let cb = Closure::<dyn FnMut()>::new(move || {
            audio::init();
            if let Some(canvas) = canvas_ref.get() {
                match Engine::new(canvas, params) {
                    Ok(mut e) => { e.start(); *engine.borrow_mut() = Some(e); }
                    Err(msg) => leptos::logging::error!("Engine init error: {}", msg),
                }
            }
            *init_cb2.borrow_mut() = None;
        });
        if let Some(win) = window() {
            win.request_animation_frame(cb.as_ref().unchecked_ref()).ok();
        }
        *init_cb.borrow_mut() = Some(cb);
    }

    {
        let engine = engine.clone();
        let hud = hud;
        let cb = Closure::<dyn FnMut()>::new(move || {
            if let Some(ref eng) = *engine.borrow() { hud.set(eng.get_hud()); }
        });
        let raw: &js_sys::Function = cb.as_ref().unchecked_ref();
        window().unwrap().set_interval_with_callback_and_timeout_and_arguments_0(raw, 50).ok();
        cb.forget();
    }

    let _params_effect = {
        let engine = engine.clone();
        let state = state.clone();
        Effect::new(move || {
            let params = state.params.get();
            if let Some(ref mut eng) = *engine.borrow_mut() { eng.update_params(&params); }
        })
    };

    view! {
        <div class="w-screen h-screen overflow-hidden relative select-none antialiased"
            style="font-family: 'Inter', 'Orbitron', system-ui, sans-serif; background: #0a0a12;">

            <canvas node_ref=canvas_ref
                class="absolute inset-0 w-full h-full outline-none touch-none"
                tabindex="0"
            />

            <div class="absolute top-0 left-0 right-0 z-10 h-12 bg-gradient-to-b from-black/60 via-black/30 to-transparent flex items-center justify-between px-4">
                <div class="flex items-center gap-3">
                    <span class="text-white font-bold text-sm font-mono tabular-nums tracking-wider"
                        style={move || format!("color: rgb({},{},{})", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}>
                        {move || format!("{:04}", hud.get().pos[0])}
                    </span>
                    <span class="text-white/15 text-xs font-mono">/</span>
                    <span class="text-white font-bold text-sm font-mono tabular-nums tracking-wider"
                        style={move || format!("color: rgb({},{},{})", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}>
                        {move || format!("{:04}", hud.get().pos[2])}
                    </span>
                    <span class="text-white/20 text-sm">|</span>
                    <span class="text-xs font-mono text-white/40">{move || hud.get().biome}</span>
                </div>
                <div class="flex items-center gap-2">
                    <span class="text-[10px] font-mono text-white/15 bg-white/[0.04] px-2 py-0.5 rounded-full"
                        title="Escala / Amplitud / Octavas">
                        {move || {
                            let p = state.params.get();
                            format!("⚙️{:.3} 📏{:.0} 🔢{}", p.scale, p.amplitude, p.octaves)
                        }}
                    </span>
                    <span class="text-[10px] font-mono text-white/15 bg-white/[0.04] px-2 py-0.5 rounded-full"
                        title="Nivel de agua">
                        {move || format!("💧{:.1}", state.params.get().water_level)}
                    </span>
                    <span class="text-[10px] font-mono text-white/15 bg-white/[0.04] px-2 py-0.5 rounded-full"
                        title="Zona actual">
                        {move || {
                            let z = state.params.get().zone;
                            format!("{}{}", zone_emoji(&z), zone_name(&z))
                        }}
                    </span>
                    <span class="text-[10px] font-mono text-white/15 bg-white/[0.04] px-2 py-0.5 rounded-full"
                        title="Personaje / Partículas">
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
                    <span class="text-[10px] font-mono text-white/15 bg-white/[0.04] px-2 py-0.5 rounded-full"
                        title="Semilla">
                        {move || format!("🌱{}", state.params.get().seed)}
                    </span>
                    <span class="text-[10px] font-mono text-white/25">{move || format!("{}fps", hud.get().fps)}</span>
                </div>
            </div>

            {move || open_menu.get().map(|_| {
                view! {
                    <div class="fixed inset-0 z-15" on:click=move |_| open_menu.set(None) />
                }
            })}

            <div class="absolute left-3 top-1/2 -translate-y-1/2 z-20 flex flex-col gap-2.5">
                <button on:click=move |_| toggle_menu("seed") class=BTN title="Semilla del mundo">{ "🌱" }</button>
                <button on:click=move |_| toggle_menu("movement") class=BTN title="Movimiento y física">{ "🏃" }</button>
                <button on:click=move |_| toggle_menu("camera") class=BTN title="Modo de cámara">{ "🎥" }</button>
                <button on:click=move |_| toggle_menu("daynight") class=BTN title="Ciclo día/noche">{ "☀️" }</button>
            </div>

            <div class="absolute left-[78px] top-1/2 -translate-y-1/2 z-20 flex flex-col gap-2.5">
                <button on:click=move |_| toggle_menu("scale") class=BTN title="Escala del terreno">{ "⚙️" }</button>
                <button on:click=move |_| toggle_menu("amplitude") class=BTN title="Amplitud del terreno">{ "📏" }</button>
                <button on:click=move |_| toggle_menu("octaves") class=BTN title="Octavas de ruido">{ "🔢" }</button>
                <button on:click=move |_| toggle_menu("water") class=BTN title="Nivel del agua">{ "💧" }</button>
                <button on:click=move |_| toggle_menu("zone") class=BTN title="Zona / preset de terreno">{ "🌍" }</button>
                <button on:click=move |_| toggle_menu("canyons") class=BTN title="Cañones profundos">{ "🏔️" }</button>
                <button on:click=move |_| toggle_menu("particles") class=BTN title="Partículas">{ "🌧️" }</button>
            </div>

            <div class="absolute left-[156px] top-1/2 -translate-y-1/2 z-20 flex flex-col gap-2.5">
                <button on:click=move |_| toggle_menu("character") class=BTN title="Personaje">{ "🧑" }</button>
                <button on:click=move |_| toggle_menu("color") class=BTN title="Esquema de color">{ "🎨" }</button>
                <button on:click=move |_| toggle_menu("charscale") class=BTN title="Tamaño del personaje">{ "📐" }</button>
            </div>

            {move || open_menu.get().map(|_| {
                view! {
                    <div class="absolute left-[236px] top-1/2 -translate-y-1/2 z-25">
                        <div class="bg-black/80 backdrop-blur-xl border border-white/[0.08] rounded-2xl p-4 min-w-[200px] shadow-2xl">
                            <div class="text-white/40 text-[10px] font-mono mb-3 uppercase tracking-widest">
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
                                    <div><div class="text-white/35 text-[10px] font-mono mb-1">Velocidad <span class="text-white/60 ml-1">{move || format!("{:.0}", state.params.get().speed)}</span></div>
                                        <input type="range" min="5" max="60" step="1" prop:value={move || format!("{}", state.params.get().speed as u32)} on:input=move |ev| { state.params.update(|p| p.speed = event_target_value(&ev).parse::<f64>().unwrap_or(18.0).max(5.0).min(60.0)); } class=SLIDER/></div>
                                    <div><div class="text-white/35 text-[10px] font-mono mb-1">Gravedad <span class="text-white/60 ml-1">{move || format!("{:.1}", state.params.get().gravity)}</span></div>
                                        <input type="range" min="5" max="40" step="0.5" prop:value={move || format!("{:.1}", state.params.get().gravity)} on:input=move |ev| { state.params.update(|p| p.gravity = event_target_value(&ev).parse::<f64>().unwrap_or(20.0).max(5.0).min(40.0)); } class=SLIDER/></div>
                                    <div><div class="text-white/35 text-[10px] font-mono mb-1">Salto <span class="text-white/60 ml-1">{move || format!("{:.1}", state.params.get().jump_speed)}</span></div>
                                        <input type="range" min="2" max="25" step="0.5" prop:value={move || format!("{:.1}", state.params.get().jump_speed)} on:input=move |ev| { state.params.update(|p| p.jump_speed = event_target_value(&ev).parse::<f64>().unwrap_or(10.0).max(2.0).min(25.0)); } class=SLIDER/></div>
                                    <div><div class="text-white/35 text-[10px] font-mono mb-1">Paso máx. <span class="text-white/60 ml-1">{move || format!("{:.2}", state.params.get().step_height)}</span></div>
                                        <input type="range" min="0.1" max="2.0" step="0.05" prop:value={move || format!("{:.2}", state.params.get().step_height)} on:input=move |ev| { state.params.update(|p| p.step_height = event_target_value(&ev).parse::<f64>().unwrap_or(0.7).max(0.1).min(2.0)); } class=SLIDER/></div>
                                    <div><div class="text-white/35 text-[10px] font-mono mb-1">Aceleración <span class="text-white/60 ml-1">{move || format!("{:.0}", state.params.get().movement_accel)}</span></div>
                                        <input type="range" min="10" max="150" step="5" prop:value={move || format!("{}", state.params.get().movement_accel as u32)} on:input=move |ev| { state.params.update(|p| p.movement_accel = event_target_value(&ev).parse::<f64>().unwrap_or(50.0).max(10.0).min(150.0)); } class=SLIDER/></div>
                                    <div><div class="text-white/35 text-[10px] font-mono mb-1">Fricción <span class="text-white/60 ml-1">{move || format!("{:.0}", state.params.get().movement_friction)}</span></div>
                                        <input type="range" min="2" max="50" step="1" prop:value={move || format!("{}", state.params.get().movement_friction as u32)} on:input=move |ev| { state.params.update(|p| p.movement_friction = event_target_value(&ev).parse::<f64>().unwrap_or(15.0).max(2.0).min(50.0)); } class=SLIDER/></div>
                                </div>
                            }) } else { None }}

                            {move || if open_menu.get() == Some("camera") { Some(view! {
                                <div class="flex gap-2">
                                    <button on:click=move |_| state.params.update(|p| p.camera_mode = CameraMode::FirstPerson)
                                        class={move || if state.params.get().camera_mode == CameraMode::FirstPerson { TBTN_ON } else { TBTN_OFF }}>"1ª Persona"</button>
                                    <button on:click=move |_| state.params.update(|p| p.camera_mode = CameraMode::ThirdPerson)
                                        class={move || if state.params.get().camera_mode == CameraMode::ThirdPerson { TBTN_ON } else { TBTN_OFF }}>"3ª Persona"</button>
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
                                    <div><div class="text-white/35 text-[10px] font-mono mb-1">Velocidad <span class="text-white/60 ml-1">{move || format!("{:.2}", state.params.get().day_speed)}</span></div>
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
                                        let name = match preset { CharacterPreset::Human => "Humano", CharacterPreset::Robot => "Robot", CharacterPreset::Beast => "Bestia", CharacterPreset::Ghost => "Fantasma", };
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
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
