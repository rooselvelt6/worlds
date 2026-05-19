use crate::engine::audio;
use crate::engine::bridge;
use crate::engine::joystick::Joystick;
use crate::engine::minimap::Minimap;
use crate::engine::terrain::Zone;
use crate::engine::{Engine, HudData};
use crate::state::{AppState, FormulaType, SaveData};
use leptos::html;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::window;

/// Safe wrapper for Rc<RefCell<Option<Engine>>> to satisfy Send bound.
/// WASM is single-threaded, so this is safe.
#[derive(Clone)]
struct SendEngine(Rc<RefCell<Option<Engine>>>);
unsafe impl Send for SendEngine {}

impl SendEngine {
    fn save_to_slot(&self, slot: u32, name: &str) {
        if let Some(ref eng) = *self.0.borrow() {
            eng.save_to_slot(slot, name);
        }
    }
    fn apply_save(&self, data: &SaveData) {
        if let Some(ref mut eng) = *self.0.borrow_mut() {
            eng.apply_save(data);
        }
    }
}

fn parse_hex(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    if h.len() == 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(255);
        (r, g, b)
    } else {
        (255, 255, 255)
    }
}

macro_rules! slider {
    ($label:expr, $icon:expr, $min:expr, $max:expr, $step:expr,
     $value:expr, $display:expr, $on_input:expr $(,)?) => {
        {
            let val = $value;
            let display = $display;
            let on_input = $on_input;
            let icon_str = $icon;
            view! {
                <div class="flex items-center gap-2.5 group/slider">
                    <span class="text-sm shrink-0 w-5 text-center opacity-30 group-hover/slider:opacity-70 transition-opacity" inner_html=icon_str></span>
                    <span class="text-[10px] font-mono text-white/25 w-[72px] shrink-0 truncate">{$label}</span>
                    <div class="flex-1 relative">
                        <input type="range"
                            min=$min max=$max step=$step
                            prop:value=move || format!("{}", val())
                            on:input=move |ev| {
                                let i: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
                                on_input(i.value_as_number());
                            }
                            class="slider-thumb w-full h-1 rounded-full cursor-pointer"
                            style="background: linear-gradient(to right, rgba(var(--glow-r), var(--glow-g), var(--glow-b), 0.2), rgba(255,255,255,0.04));"
                        />
                    </div>
                    <span class="text-[10px] font-mono text-white/45 w-12 text-right tabular-nums">{move || display()}</span>
                </div>
            }
        }
    };
}

#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();
    let canvas_ref: NodeRef<html::Canvas> = NodeRef::new();
    let minimap_canvas_ref: NodeRef<html::Canvas> = NodeRef::new();
    let joystick_canvas_ref: NodeRef<html::Canvas> = NodeRef::new();
    let engine: Rc<RefCell<Option<Engine>>> = Rc::new(RefCell::new(None));
    let minimap: Rc<RefCell<Option<Minimap>>> = Rc::new(RefCell::new(None));
    let joystick: Rc<RefCell<Option<Joystick>>> = Rc::new(RefCell::new(None));
    let hud = RwSignal::new(HudData::default());
    let settings_open = RwSignal::new(false);
    let menu_tab = RwSignal::new(0u8);
    let glow_rgb = RwSignal::new((34u8, 211u8, 238u8));

    {
        let canvas_ref = canvas_ref.clone();
        let minimap_canvas_ref = minimap_canvas_ref.clone();
        let joystick_canvas_ref = joystick_canvas_ref.clone();
        let engine = engine.clone();
        let minimap = minimap.clone();
        let joystick = joystick.clone();
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
            if let Some(ref eng) = *engine.borrow() {
                if let Some(jc) = joystick_canvas_ref.get() {
                    let (jd, jdy) = eng.joystick_cells();
                    match Joystick::new(jc, jd, jdy) {
                        Ok(mut j) => { j.start_render(); *joystick.borrow_mut() = Some(j); }
                        Err(msg) => leptos::logging::error!("Joystick init error: {}", msg),
                    }
                }
            }
            if let Some(mc) = minimap_canvas_ref.get() {
                mc.set_width(160);
                mc.set_height(160);
                match Minimap::new(mc) {
                    Ok(m) => { *minimap.borrow_mut() = Some(m); }
                    Err(msg) => leptos::logging::error!("Minimap init error: {}", msg),
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

    {
        let state = state.clone();
        let minimap = minimap.clone();
        let hud = hud;
        let cb = Closure::<dyn FnMut()>::new(move || {
            let h = hud.get();
            let params = state.params.get();
            if let Some(ref m) = *minimap.borrow() {
                m.render(&params, h.pos[0], h.pos[2], h.yaw_deg);
            }
        });
        let raw: &js_sys::Function = cb.as_ref().unchecked_ref();
        window().unwrap().set_interval_with_callback_and_timeout_and_arguments_0(raw, 100).ok();
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

    let _volume_effect = {
        Effect::new(move || {
            let vol = state.params.get().volume;
            bridge::audio_set_master_volume(vol as f32);
        })
    };

    let _glow_effect = {
        let state = state.clone();
        Effect::new(move || {
            let hex = state.params.get().formula.color_hex();
            let (r, g, b) = parse_hex(hex);
            glow_rgb.set((r, g, b));
            if let Some(doc) = window().and_then(|w| w.document()) {
                if let Some(el) = doc.document_element() {
                    let html_el: &web_sys::HtmlElement = el.unchecked_ref();
                    html_el.style().set_property("--glow-r", &r.to_string()).ok();
                    html_el.style().set_property("--glow-g", &g.to_string()).ok();
                    html_el.style().set_property("--glow-b", &b.to_string()).ok();
                }
            }
        })
    };

    let zones = [
        (Zone::Forest, "Bosque", "#22c55e", "fa-tree"),
        (Zone::Plains, "Llanura", "#a3e635", "fa-leaf"),
        (Zone::Desert, "Desierto", "#f59e0b", "fa-sun"),
        (Zone::Tundra, "Tundra", "#e0f2fe", "fa-snowflake"),
        (Zone::Jungle, "Jungla", "#166534", "fa-seedling"),
        (Zone::Volcanic, "Volcan", "#7c2d12", "fa-volcano"),
        (Zone::Ocean, "Oceano", "#0ea5e9", "fa-water"),
        (Zone::Crystal, "Cristal", "#a855f7", "fa-gem"),
        (Zone::Cave, "Cueva", "#525252", "fa-mountain"),
        (Zone::Lava, "Lava", "#ef4444", "fa-fire"),
        (Zone::Fungus, "Fungus", "#a855f7", "fa-spa"),
        (Zone::Abyss, "Abismo", "#1e1b4b", "fa-skull"),
        (Zone::Storm, "Tormenta", "#64748b", "fa-cloud-bolt"),
        (Zone::Aurora, "Aurora", "#2dd4bf", "fa-wand-sparkles"),
        (Zone::Magma, "Magma", "#ea580c", "fa-star"),
    ];

    let simple_mode = RwSignal::new(false);

    let tabs = [
        (0u8, "fa-earth-americas", "Mundo"),
        (1, "fa-cube", "Fórmula"),
        (2, "fa-palette", "Color"),
        (3, "fa-gamepad", "Control"),
        (4, "fa-sliders", "Avanzado"),
    ];

    let send_engine = SendEngine(engine.clone());
    view! {
        <div class="w-screen h-screen overflow-hidden relative select-none antialiased"
            style="font-family: 'Inter', 'Orbitron', system-ui, sans-serif; background: #0a0a12;">

            // 3D Canvas
            <canvas node_ref=canvas_ref
                class="absolute inset-0 w-full h-full outline-none touch-none"
                tabindex="0"
            />

            // Overlay
            {move || settings_open.get().then(|| view! {
                <div class="absolute inset-0 z-30 bg-black/40 backdrop-blur-sm transition-opacity duration-300"
                     on:click=move |_| settings_open.set(false)>
                </div>
            })}

            // ===== TOP BAR =====
            <div class="absolute top-0 left-0 right-0 z-20 h-12 bg-[#0d0d1a]/60 backdrop-blur-glass border-b border-white/[0.04] flex items-center justify-between px-3">
                // Left: coords
                <div class="flex items-center gap-3">
                    <div class="flex items-center gap-1.5">
                        <i class="fa-solid fa-crosshairs text-[10px]" style={move || format!("color: rgba({},{},{},0.5)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}></i>
                        <span class="text-white font-bold text-sm font-mono tabular-nums tracking-wider"
                            style={move || format!("color: rgb({},{},{})", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}>
                            {move || format!("{:04}", hud.get().pos[0])}
                        </span>
                        <span class="text-white/15 text-xs font-mono">/</span>
                        <span class="text-white font-bold text-sm font-mono tabular-nums tracking-wider"
                            style={move || format!("color: rgb({},{},{})", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}>
                            {move || format!("{:04}", hud.get().pos[2])}
                        </span>
                    </div>
                    <div class="w-px h-4 bg-white/5"></div>
                    <div class="flex items-center gap-1">
                        <i class="fa-solid fa-gauge-high text-[10px] text-white/40"></i>
                        <span class="text-white/80 font-bold text-sm font-mono tabular-nums">
                            {move || format!("{:04.1}", hud.get().speed)}
                        </span>
                        <span class="text-white/20 text-[9px] font-mono hidden sm:inline">VEL</span>
                    </div>
                    <div class="w-px h-4 bg-white/5 hidden sm:block"></div>
                    <div class="items-center gap-1 hidden sm:flex">
                        <div class="h-6 w-1.5 rounded-full bg-white/10 overflow-hidden relative">
                            <div class="absolute bottom-0 w-full rounded-full transition-all duration-200"
                                style:height={move || format!("{}%", ((hud.get().pos[1] / 20.0).min(1.0) * 100.0).max(5.0))}
                                style={move || format!("background: linear-gradient(to top, rgba({},{},{},0.6), rgba({},{},{},0.3))",
                                    glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2,
                                    glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}>
                            </div>
                        </div>
                        <span class="text-white/80 font-bold text-xs font-mono tabular-nums"
                            style={move || format!("color: rgb({},{},{})", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}>
                            {move || format!("{:03.0}", hud.get().pos[1])}
                        </span>
                    </div>
                </div>

                // Center: compass
                <div class="flex items-center gap-0.5 bg-white/[0.04] backdrop-blur-xl px-3 py-1 rounded-full border border-white/[0.06]">
                    <span class={move || if hud.get().yaw_deg > 315 || hud.get().yaw_deg <= 45 { "text-cyan-300 font-bold text-xs" } else { "text-white/20 text-xs" }}>N</span>
                    <span class="text-white/8 text-[9px]">|</span>
                    <span class={move || if hud.get().yaw_deg > 45 && hud.get().yaw_deg <= 135 { "text-cyan-300 font-bold text-xs" } else { "text-white/20 text-xs" }}>E</span>
                    <span class="text-white/8 text-[9px]">|</span>
                    <span class={move || if hud.get().yaw_deg > 135 && hud.get().yaw_deg <= 225 { "text-cyan-300 font-bold text-xs" } else { "text-white/20 text-xs" }}>S</span>
                    <span class="text-white/8 text-[9px]">|</span>
                    <span class={move || if hud.get().yaw_deg > 225 && hud.get().yaw_deg <= 315 { "text-cyan-300 font-bold text-xs" } else { "text-white/20 text-xs" }}>W</span>
                    <span class="text-white/25 text-[9px] font-mono ml-1">{move || format!("{:03}\u{b0}", hud.get().yaw_deg)}</span>
                </div>

                // Right: biome, fps, menu button
                <div class="flex items-center gap-2">
                    <span class="text-white/40 font-bold text-xs font-mono hidden sm:inline"
                        style={move || format!("color: rgba({},{},{},0.7)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}>
                        {move || hud.get().biome}
                    </span>
                    <span class="text-white/20 text-[10px] font-mono">{move || format!("{}fps", hud.get().fps)}</span>
                    <Show when={move || hud.get().gamepad_connected}>
                        <span class="text-emerald-400/60 text-[10px]"><i class="fa-solid fa-gamepad"></i></span>
                    </Show>
                    <button on:click=move |_| settings_open.update(|v| *v = !*v)
                        class={move || {
                            let open = settings_open.get();
                            let base = "w-9 h-9 rounded-xl flex items-center justify-center transition-all duration-200 active:scale-90 border";
                            if open {
                                format!("{} border", base)
                            } else {
                                format!("{} text-white/40 hover:text-white/80 hover:bg-white/5 border-transparent", base)
                            }
                        }}
                        style={move || if settings_open.get() {
                            let (r, g, b) = glow_rgb.get();
                            format!("background-color: rgba({},{},{},0.15); color: rgb({},{},{}); border-color: rgba({},{},{},0.25)",
                                r, g, b, r, g, b, r, g, b)
                        } else {
                            "".to_string()
                        }}
                    >
                        <i class="fa-solid fa-sliders text-base"></i>
                    </button>
                </div>
            </div>

            // ===== LEFT SIDE: JOYSTICK =====
            <div class="absolute left-3 top-1/2 -translate-y-1/2 z-10">
                <div class="bg-white/[0.03] backdrop-blur-2xl rounded-2xl p-2.5 border border-white/[0.06] shadow-lg"
                    style={move || format!("box-shadow: 0 8px 32px rgba({},{},{},0.06)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}>
                    <canvas node_ref=joystick_canvas_ref
                        class="w-[160px] h-[160px] touch-none block"
                        width=160 height=160
                    />
                    <div class="flex justify-center gap-3 mt-1.5 text-[7px] font-mono text-white/15 tracking-widest uppercase">
                        <span><i class="fa-solid fa-arrow-up text-[8px] mr-0.5"></i>W</span>
                        <span><i class="fa-solid fa-arrow-down text-[8px] mr-0.5"></i>S</span>
                        <span><i class="fa-solid fa-arrow-left text-[8px] mr-0.5"></i>A</span>
                        <span><i class="fa-solid fa-arrow-right text-[8px] mr-0.5"></i>D</span>
                    </div>
                </div>
            </div>

            // ===== RIGHT SIDE: ACTION BUTTONS =====
            <div class="absolute right-3 top-1/2 -translate-y-1/2 z-10 flex flex-col gap-2">
                // Jump
                <button
                    class="w-14 h-14 rounded-2xl bg-white/[0.03] backdrop-blur-2xl border border-white/[0.06] text-white/60 hover:text-white hover:bg-white/[0.06] flex items-center justify-center active:scale-85 transition-all duration-150 shadow-lg"
                    on:pointerdown={let s = state.clone(); move |_| {
                        let fly = s.params.get().fly_mode;
                        if fly { s.params.update(|p| p.speed = 30.0); }
                    }}
                >
                    <i class="fa-solid fa-arrow-up text-xl"></i>
                </button>
                <span class="text-[7px] font-mono text-white/15 text-center tracking-widest uppercase -mt-0.5">Saltar</span>

                // Fly
                <button
                    class={move || {
                        let fly = state.params.get().fly_mode;
                        let base = "w-14 h-14 rounded-2xl backdrop-blur-2xl border flex items-center justify-center active:scale-85 transition-all duration-150 shadow-lg";
                        if fly {
                            format!("{} bg-white/[0.06] text-white border-white/[0.12]", base)
                        } else {
                            format!("{} bg-white/[0.03] text-white/60 hover:text-white hover:bg-white/[0.06] border-white/[0.06]", base)
                        }
                    }}
                    style={move || if state.params.get().fly_mode {
                        let (r, g, b) = glow_rgb.get();
                        format!("box-shadow: 0 4px 20px rgba({},{},{},0.15)", r, g, b)
                    } else { "".to_string() }}
                    on:click={let s = state.clone(); move |_| {
                        s.params.update(|p| p.fly_mode = !p.fly_mode);
                    }}
                >
                    <i class="fa-solid fa-wing text-xl"></i>
                </button>
                <span class="text-[7px] font-mono text-white/15 text-center tracking-widest uppercase -mt-0.5">
                    {move || if state.params.get().fly_mode { "Volando" } else { "Volar" }}
                </span>

                // Sprint
                <button
                    class={move || {
                        let sprint = state.params.get().speed > 25.0;
                        let base = "w-14 h-14 rounded-2xl backdrop-blur-2xl border flex items-center justify-center active:scale-85 transition-all duration-150 shadow-lg";
                        if sprint {
                            format!("{} bg-amber-500/10 text-amber-300 border-amber-400/20", base)
                        } else {
                            format!("{} bg-white/[0.03] text-white/60 hover:text-white hover:bg-white/[0.06] border-white/[0.06]", base)
                        }
                    }}
                    on:click={let s = state.clone(); move |_| {
                        s.params.update(|p| p.speed = if p.speed > 25.0 { 18.0 } else { 45.0 });
                    }}
                >
                    <i class="fa-solid fa-bolt text-xl"></i>
                </button>
                <span class="text-[7px] font-mono text-white/15 text-center tracking-widest uppercase -mt-0.5">
                    {move || if state.params.get().speed > 25.0 { "Sprint" } else { "Paso" }}
                </span>

                // Screenshot
                <button
                    class="w-14 h-14 rounded-2xl bg-white/[0.03] backdrop-blur-2xl border border-white/[0.06] text-white/60 hover:text-white hover:bg-white/[0.06] flex items-center justify-center active:scale-85 transition-all duration-150 shadow-lg"
                    on:click=move |_| {
                        let h = hud.get();
                        bridge::capture_screenshot(
                            state.params.get().seed,
                            h.formula.as_str(),
                            h.biome.as_str(),
                            h.pos[0], h.pos[1], h.pos[2],
                        );
                    }
                >
                    <i class="fa-solid fa-camera text-lg"></i>
                </button>
                <span class="text-[7px] font-mono text-white/15 text-center tracking-widest uppercase -mt-0.5">Foto</span>
            </div>

            // ===== CENTER CROSSHAIR =====
            <svg class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-10 pointer-events-none"
                width=24 height=24 viewBox="0 0 24 24">
                <defs>
                    <filter id="cg"><feGaussianBlur stdDeviation="0.5" result="b"/>
                        <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
                </defs>
                <circle cx="12" cy="12" r="1" fill="#00ffff" opacity="0.7" filter="url(#cg)"/>
                <circle cx="12" cy="12" r="4" fill="none" stroke="#00ffff" stroke-width="0.5" opacity="0.12"/>
                <g transform={move || format!("rotate({} 12 12)", -(hud.get().yaw_deg as f64))}>
                    <line x1="12" y1="1.5" x2="12" y2="5" stroke="#ff3366" stroke-width="2" stroke-linecap="round" filter="url(#cg)"/>
                    <polygon points="12,0 10.5,5 13.5,5" fill="#ff3366" filter="url(#cg)"/>
                </g>
            </svg>

            // ===== MINIMAP =====
            <div class="absolute bottom-4 left-3 z-10">
                <div class="bg-white/[0.03] backdrop-blur-2xl rounded-2xl p-2 border border-white/[0.06] shadow-lg"
                    style={move || format!("box-shadow: 0 8px 32px rgba({},{},{},0.06)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}>
                    <canvas node_ref=minimap_canvas_ref
                        class="w-[120px] h-[120px] rounded-full pointer-events-none"
                        width=120 height=120
                    />
                    <div class="flex items-center justify-center gap-1.5 mt-1">
                        <i class="fa-solid fa-location-dot text-[8px]"
                            style={move || format!("color: rgba({},{},{},0.4)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}></i>
                        <span class="text-[7px] font-mono text-white/40" style={move || format!("color: rgba({},{},{},0.6)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}>
                            {move || hud.get().biome}
                        </span>
                    </div>
                </div>
            </div>

            // ===== DISCOVERY MESSAGE =====
            <Show when={move || hud.get().discovery_message.is_some()}>
                <div class="absolute top-20 left-1/2 -translate-x-1/2 z-50 pointer-events-none animate-bounce">
                    <div class="px-5 py-2 rounded-xl bg-white/[0.06] backdrop-blur-2xl border border-white/10 text-sm font-bold text-white/90 shadow-xl">
                        {move || hud.get().discovery_message.clone().unwrap_or_default()}
                    </div>
                </div>
            </Show>

            // ===== WAYPOINTS COUNTER =====
            <div class="absolute bottom-4 right-4 z-10">
                <div class="px-3 py-1.5 rounded-lg bg-white/[0.03] backdrop-blur-xl border border-white/[0.06] text-[10px] font-mono text-white/30 flex items-center gap-2">
                    <i class="fa-solid fa-flag mr-1"></i>
                    {move || format!("{} WP | {} biomas", hud.get().waypoints.len(), hud.get().discovered_biomes.len())}
                    <span class="text-white/15">|</span>
                    <i class="fa-solid fa-star text-amber-400/40"></i>
                    {move || format!("{} pts", hud.get().achievement_points)}
                    <span class="text-white/15">|</span>
                    <span class={move || {
                        let s = hud.get().season;
                        match s { 0 => "🌸", 1 => "☀️", 2 => "🍂", _ => "❄️" }
                    }}></span>
                    {move || match hud.get().season { 0 => "Spring", 1 => "Summer", 2 => "Autumn", _ => "Winter" }}
                    <Show when={move || hud.get().vr_mode}>
                        <span class="text-purple-400/60"><i class="fa-solid fa-vr-cardboard"></i>VR</span>
                    </Show>
                </div>
            </div>

            // ===== MODE INDICATOR =====
            <div class="absolute bottom-4 left-1/2 -translate-x-1/2 z-10">
                <div class={move || {
                    let fly = hud.get().fly_mode;
                    let base = "px-4 py-1.5 rounded-full text-[11px] font-mono font-bold tracking-widest uppercase backdrop-blur-2xl border transition-all duration-300 shadow-lg bg-white/[0.03]";
                    if fly {
                        format!("{} text-cyan-300 border-cyan-500/20", base)
                    } else {
                        format!("{} text-emerald-300 border-emerald-500/20", base)
                    }
                }}
                style={move || if hud.get().fly_mode {
                    let (r, g, b) = glow_rgb.get();
                    format!("box-shadow: 0 4px 20px rgba({},{},{},0.1)", r, g, b)
                } else {
                    "box-shadow: 0 4px 20px rgba(52,211,153,0.1)".to_string()
                }}>
                    <i class={move || if hud.get().fly_mode { "fa-solid fa-wing mr-1.5" } else { "fa-solid fa-person-walking mr-1.5" }}></i>
                    {move || if hud.get().fly_mode { "VUELO" } else { "CAMINAR" }}
                    {move || if hud.get().build_mode { " | CONSTRUIR" } else { "" }}
                </div>
            </div>

            // ===== BUILD MODE INVENTORY BAR =====
            <Show when={move || hud.get().build_mode}>
                <div class="absolute bottom-16 left-1/2 -translate-x-1/2 z-10">
                    <div class="flex gap-1 px-3 py-2 rounded-xl bg-white/[0.06] backdrop-blur-2xl border border-white/[0.08] shadow-xl">
                        {move || hud.get().inventory.iter().enumerate().map(|(i, (t, c))| {
                            let colors = ["#998866","#888888","#885533","#339933","#9966ff","#cc5500","#88ccff","#d4b87a","#558833"];
                            let names = ["Dirt","Stone","Wood","Leaves","Crystal","Lava Stone","Ice","Sand","Moss"];
                            let sel = hud.get().selected_slot == i as u8;
                            let idx = *t as usize;
                            let color = colors.get(idx).unwrap_or(&"#888888").to_string();
                            view! {
                                <div class={if sel { "w-10 h-10 rounded-lg bg-white/[0.12] border border-white/20 flex flex-col items-center justify-center" } else { "w-10 h-10 rounded-lg bg-white/[0.03] border border-white/[0.04] flex flex-col items-center justify-center" }}>
                                    <div class="w-4 h-4 rounded-sm" style={format!("background-color: {}", color)}></div>
                                    <span class="text-[7px] font-mono text-white/50">{*c}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </Show>

            // ===== SETTINGS PANEL (Right slide-in) =====
            <div class="absolute top-0 right-0 h-full z-40 flex flex-col
                        bg-[#0d0d1a]/80 backdrop-blur-glass border-l border-white/[0.06] shadow-2xl
                        transition-transform duration-400 ease-out overflow-hidden
                        w-full sm:w-[420px] lg:w-[520px]"
                style:transform={move || if settings_open.get() { "translateX(0%)" } else { "translateX(100%)" }}
                style:pointer-events={move || if settings_open.get() { "auto" } else { "none" }}>

                // Header
                <div class="flex items-center justify-between px-4 lg:px-5 py-3 border-b border-white/[0.04] shrink-0 glass-shimmer"
                    style={move || format!("box-shadow: 0 1px 0 rgba({},{},{},0.04)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}>
                    <span class="text-[11px] font-bold tracking-[0.2em] text-white/40 uppercase"
                        style="font-family: 'Orbitron', monospace;">
                        Ajustes
                    </span>
                    <button on:click=move |_| settings_open.set(false)
                        class="w-8 h-8 rounded-lg flex items-center justify-center text-white/30 hover:text-white hover:bg-white/[0.06] transition-all duration-200 active:scale-90">
                        <i class="fa-solid fa-xmark text-lg"></i>
                    </button>
                </div>

                // Tabs + Content
                <div class="flex flex-1 overflow-hidden">
                    // Vertical Tabs
                    <div class="flex flex-col gap-0.5 p-1.5 lg:p-2 w-14 lg:w-40 border-r border-white/[0.04] shrink-0">
                        {tabs.iter().map(|&(id, icon, label)| {
                            view! {
                                <button on:click=move |_| menu_tab.set(id)
                                    class={move || {
                                        let active = menu_tab.get() == id;
                                        let base = "flex items-center justify-center lg:justify-start gap-2 lg:gap-3 px-2 lg:px-3 py-2.5 rounded-lg transition-all duration-200 border-l-2 text-[11px] font-bold tracking-wider";
                                        if active {
                                            format!("{} bg-white/[0.05] text-white", base)
                                        } else {
                                            format!("{} text-white/30 hover:text-white/60 hover:bg-white/[0.02] border-l-transparent", base)
                                        }
                                    }}
                                    style={move || if menu_tab.get() == id {
                                        let (r, g, b) = glow_rgb.get();
                                        format!("border-left-color: rgba({},{},{},0.6); box-shadow: 0 0 20px rgba({},{},{},0.04)", r, g, b, r, g, b)
                                    } else { "".to_string() }}
                                >
                                    <i class={format!("fa-solid {} text-base shrink-0", icon)}
                                        style={move || if menu_tab.get() == id {
                                            let (r, g, b) = glow_rgb.get();
                                            format!("color: rgb({},{},{})", r, g, b)
                                        } else { "".to_string() }}>
                                    </i>
                                    <span class="hidden lg:inline truncate">{label}</span>
                                </button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>

                    // Content Area
                    <div class="flex-1 overflow-y-auto p-3 lg:p-4 scrollbar-thin">

                        // ===== TAB 0: MUNDO =====
                        {move || (menu_tab.get() == 0).then(|| view! {
                            <div class="tab-enter space-y-0.5">
                                {slider!("Semilla", "<i class='fa-solid fa-seedling'></i>", 1, 9999, 1,
                                    move || state.params.get().seed as f64,
                                    move || format!("{:04}", state.params.get().seed),
                                    move |v| state.params.update(|p| p.seed = v as u32)
                                )}
                                <button on:click=move |_| {
                                    state.params.update(|p| p.seed = (js_sys::Math::random() * 9999.0) as u32 + 1);
                                } class="w-full py-2 rounded-lg bg-white/[0.02] hover:bg-white/[0.05] border border-white/[0.04] text-white/40 hover:text-white/70 text-[10px] font-mono tracking-wider transition-all duration-200 active:scale-[0.98] flex items-center justify-center gap-2 mb-2">
                                    <i class="fa-solid fa-dice"></i>
                                    <span>Aleatoria</span>
                                </button>
                                {slider!("Velocidad", "<i class='fa-solid fa-gauge-high'></i>", 1, 100, 1,
                                    move || state.params.get().speed,
                                    move || format!("{:02}", state.params.get().speed as u32),
                                    move |v| state.params.update(|p| p.speed = v)
                                )}
                                {slider!("Sensibilidad", "<i class='fa-solid fa-crosshairs'></i>", 0.1, 5.0, 0.1,
                                    move || state.params.get().mouse_sensitivity,
                                    move || format!("{:.1}", state.params.get().mouse_sensitivity),
                                    move |v| state.params.update(|p| p.mouse_sensitivity = v)
                                )}
                                {slider!("Horizonte", "<i class='fa-solid fa-eye'></i>", 2, 8, 1,
                                    move || state.params.get().render_distance as f64,
                                    move || format!("{}", state.params.get().render_distance),
                                    move |v| state.params.update(|p| p.render_distance = v as u32)
                                )}
                                {slider!("Agua", "<i class='fa-solid fa-water'></i>", 0.0, 5.0, 0.1,
                                    move || state.params.get().water_level,
                                    move || format!("{:.1}", state.params.get().water_level),
                                    move |v| state.params.update(|p| p.water_level = v)
                                )}
                            </div>
                        })}

                        // ===== TAB 1: FÓRMULA =====
                        {move || (menu_tab.get() == 1).then(|| view! {
                            <div class="tab-enter space-y-3">
                                <div class="grid grid-cols-4 gap-1.5">
                                    {FormulaType::all().iter().map(|f| {
                                        let f = *f;
                                        view! {
                                            <button
                                                class={move || {
                                                    let active = state.params.get().formula == f;
                                                    let base = "flex flex-col items-center gap-0.5 py-2.5 px-1 rounded-xl font-bold transition-all duration-200 active:scale-90 border";
                                                    if active {
                                                        format!("{} bg-white/[0.06] text-white border-white/[0.12]", base)
                                                    } else {
                                                        format!("{} bg-white/[0.02] text-white/35 hover:text-white/65 hover:bg-white/[0.04] border-white/[0.04]", base)
                                                    }
                                                }}
                                                style={move || if state.params.get().formula == f {
                                                    let h = f.color_hex();
                                                    let (r, g, b) = parse_hex(h);
                                                    format!("box-shadow: 0 0 20px rgba({},{},{},0.1); border-color: rgba({},{},{},0.3)", r, g, b, r, g, b)
                                                } else { "".to_string() }}
                                                on:click=move |_| state.params.update(|p| p.formula = f)
                                                title={f.name()}
                                            >
                                                <span class="text-lg">{f.emoji()}</span>
                                                <span class="text-[7px] font-mono truncate w-full text-center">{f.name()}</span>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                <div class="p-2.5 rounded-xl bg-white/[0.02] border border-white/[0.04]">
                                    <div class="text-[10px] font-mono tabular-nums leading-relaxed"
                                        style={move || { let (r, g, b) = glow_rgb.get(); format!("color: rgba({},{},{},0.6)", r, g, b) }}>
                                        {move || {
                                            let p = state.params.get();
                                            if p.blend_a > 0.01 && p.formula_b != p.formula {
                                                format!("{} → {} ({}%)", p.formula.name(), p.formula_b.name(), (p.blend_a * 100.0) as u32)
                                            } else {
                                                format!("{}", p.formula.formula_expr(p.scale, p.octaves))
                                            }
                                        }}
                                    </div>
                                </div>
                                {slider!("Frecuencia", "<i class='fa-solid fa-chart-line'></i>", 0.002, 0.15, 0.001,
                                    move || state.params.get().scale,
                                    move || format!("{:.3}", state.params.get().scale),
                                    move |v| state.params.update(|p| p.scale = v)
                                )}
                                {slider!("Amplitud", "<i class='fa-solid fa-arrow-up-wide-short'></i>", 0.2, 8.0, 0.1,
                                    move || state.params.get().amplitude,
                                    move || format!("{:.1}", state.params.get().amplitude),
                                    move |v| state.params.update(|p| p.amplitude = v)
                                )}
                                <div class="border-t border-white/[0.04] pt-3 space-y-2">
                                    <div class="flex items-center gap-2 mb-1">
                                        <i class="fa-solid fa-shuffle text-[10px]"
                                            style={move || format!("color: rgba({},{},{},0.5)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}></i>
                                        <span class="text-[10px] font-mono text-white/30 uppercase tracking-wider">Mezcla</span>
                                    </div>
                                    {slider!("Mezcla", "<i class='fa-solid fa-circle-half-stroke'></i>", 0.0, 1.0, 0.01,
                                        move || state.params.get().blend_a,
                                        move || format!("{:.0}%", state.params.get().blend_a * 100.0),
                                        move |v| state.params.update(|p| p.blend_a = v)
                                    )}
                                    <Show when={move || state.params.get().blend_a > 0.01}>
                                        <div class="flex items-center gap-2 mt-1.5 px-1">
                                            <span class="text-[9px] font-mono text-white/25 uppercase tracking-wider shrink-0">Formula B</span>
                                            <select
                                                class="flex-1 bg-white/[0.04] border border-white/[0.08] rounded-lg px-2 py-1.5 text-[10px] font-mono text-white/70 outline-none cursor-pointer appearance-none"
                                                style={move || format!("border-color: rgba({},{},{},0.15)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}
                                                on:change=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    if let Some(f) = FormulaType::all().iter().find(|f| f.name() == val) {
                                                        state.params.update(|p| p.formula_b = *f);
                                                    }
                                                }
                                                prop:value={move || state.params.get().formula_b.name()}
                                            >
                                                {FormulaType::all().iter().map(|f| {
                                                    let f = *f;
                                                    let selected = state.params.get().formula_b == f;
                                                    view! {
                                                        <option value={f.name()} selected=selected class="bg-[#1a1a2e] text-white/80">
                                                            {f.emoji()} {f.name()}
                                                        </option>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </select>
                                        </div>
                                    </Show>
                                </div>
                            </div>
                        })}

                        // ===== TAB 2: COLOR =====
                        {move || (menu_tab.get() == 2).then(|| view! {
                            <div class="tab-enter space-y-3">
                                <div class="grid grid-cols-3 gap-2">
                                    {zones.iter().map(|&(zone, name, color, icon)| {
                                        view! {
                                            <button
                                                class={move || {
                                                    let active = state.params.get().zone == zone;
                                                    let base = "flex flex-col items-center gap-1.5 py-3 px-2 rounded-xl font-bold transition-all duration-200 active:scale-90 border";
                                                    if active {
                                                        format!("{} bg-white/[0.06] text-white border-white/[0.12]", base)
                                                    } else {
                                                        format!("{} bg-white/[0.02] text-white/40 hover:text-white/70 hover:bg-white/[0.04] border-white/[0.04]", base)
                                                    }
                                                }}
                                                style={move || if state.params.get().zone == zone {
                                                    let (r, g, b) = parse_hex(color);
                                                    format!("box-shadow: 0 0 20px rgba({},{},{},0.12)", r, g, b)
                                                } else { "".to_string() }}
                                                on:click=move |_| state.params.update(|p| p.zone = zone)
                                            >
                                                <i class={format!("fa-solid {} text-xl", icon)} style={format!("color: {}", color)}></i>
                                                <span class="text-[9px] font-mono tracking-wider">{name}</span>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                <div class="border-t border-white/[0.04] pt-3 space-y-0.5">
                                    {slider!("Tono", "<i class='fa-solid fa-palette'></i>", 0, 360, 1,
                                        move || state.params.get().hue_shift,
                                        move || format!("{:03.0}", state.params.get().hue_shift),
                                        move |v| state.params.update(|p| p.hue_shift = v)
                                    )}
                                    {slider!("Saturacion", "<i class='fa-solid fa-droplet'></i>", 0.0, 2.0, 0.01,
                                        move || state.params.get().saturation,
                                        move || format!("{:.2}", state.params.get().saturation),
                                        move |v| state.params.update(|p| p.saturation = v)
                                    )}
                                    {slider!("Brillo", "<i class='fa-solid fa-sun'></i>", 0.0, 2.0, 0.01,
                                        move || state.params.get().lightness,
                                        move || format!("{:.2}", state.params.get().lightness),
                                        move |v| state.params.update(|p| p.lightness = v)
                                    )}
                                </div>
                            </div>
                        })}

                        // ===== TAB 3: CONTROL =====
                        {move || (menu_tab.get() == 3).then(|| view! {
                            <div class="tab-enter space-y-2">
                                <div class="flex items-center justify-between px-3 py-2.5 rounded-xl bg-white/[0.02] border border-white/[0.04]">
                                    <span class="text-[11px] font-mono text-white/40 flex items-center gap-2">
                                        <i class="fa-solid fa-wing"
                                            style={move || format!("color: rgba({},{},{},0.5)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}></i>
                                        Vuelo
                                    </span>
                                    <button on:click=move |_| state.params.update(|p| p.fly_mode = !p.fly_mode)
                                        class={move || {
                                            let fly = state.params.get().fly_mode;
                                            let base = "text-[10px] font-mono font-bold px-3 py-1 rounded-full border transition-all duration-200";
                                            if fly {
                                                format!("{} text-cyan-300 border-cyan-500/20", base)
                                            } else {
                                                format!("{} text-white/40 border-white/10 bg-white/[0.02]", base)
                                            }
                                        }}
                                        style={move || if state.params.get().fly_mode {
                                            let (r, g, b) = glow_rgb.get();
                                            format!("background-color: rgba({},{},{},0.1); box-shadow: 0 0 12px rgba({},{},{},0.08)", r, g, b, r, g, b)
                                        } else { "".to_string() }}>
                                        {move || if state.params.get().fly_mode { "SI" } else { "NO" }}
                                    </button>
                                </div>
                                <div class="flex items-center justify-between px-3 py-2.5 rounded-xl bg-white/[0.02] border border-white/[0.04]">
                                    <span class="text-[11px] font-mono text-white/40 flex items-center gap-2">
                                        <i class="fa-solid fa-gamepad"
                                            style={move || format!("color: rgba({},{},{},0.5)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}></i>
                                        Control
                                    </span>
                                    <button on:click=move |_| state.params.update(|p| p.control_mode = match p.control_mode {
                                        crate::state::ControlMode::DPad => crate::state::ControlMode::Joystick,
                                        crate::state::ControlMode::Joystick => crate::state::ControlMode::DPad,
                                    })>
                                        <span class="text-[10px] font-mono font-bold px-3 py-1 rounded-full bg-white/[0.05] text-white/60 border border-white/[0.06]">
                                            {move || format!("{:?}", state.params.get().control_mode)}
                                        </span>
                                    </button>
                                </div>
                                <div class="px-3 py-2.5 rounded-xl bg-white/[0.02] border border-white/[0.04]">
                                    <div class="text-[10px] font-mono text-white/25 leading-relaxed flex flex-wrap gap-x-4 gap-y-1">
                                        <span><i class="fa-solid fa-keyboard text-white/15 mr-1"></i>WASD</span>
                                        <span><i class="fa-solid fa-arrows-rotate text-white/15 mr-1"></i>Q/E</span>
                                        <span><i class="fa-solid fa-arrow-up text-white/15 mr-1"></i>Saltar</span>
                                        <span><i class="fa-solid fa-arrow-down text-white/15 mr-1"></i>Agachar</span>
                                    </div>
                                </div>
                                <div class="px-3 py-3 rounded-xl bg-white/[0.02] border border-white/[0.04]">
                                    <div class="text-[11px] font-mono font-bold text-white/30 uppercase tracking-wider mb-2 flex items-center gap-2">
                                        <i class="fa-solid fa-tablet-screen"
                                            style={move || format!("color: rgba({},{},{},0.5)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}></i>
                                        Tactil
                                    </div>
                                    <div class="text-[10px] font-mono text-white/30 leading-relaxed space-y-1">
                                        <p><i class="fa-regular fa-circle-dot mr-1.5"
                                            style={move || format!("color: rgba({},{},{},0.5)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}></i>Joystick para mover</p>
                                        <p><i class="fa-regular fa-circle-dot mr-1.5"
                                            style={move || format!("color: rgba({},{},{},0.5)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}></i>Botones para acciones</p>
                                        <p><i class="fa-regular fa-circle-dot mr-1.5"
                                            style={move || format!("color: rgba({},{},{},0.5)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}></i>Deslizar para mirar</p>
                                    </div>
                                </div>
                            </div>
                            <div class="border-t border-white/[0.04] pt-2 mt-2 space-y-0.5">
                                {slider!("Volumen", "<i class='fa-solid fa-volume-high'></i>", 0.0, 1.0, 0.05,
                                    move || state.params.get().volume,
                                    move || format!("{:.0}%", state.params.get().volume * 100.0),
                                    move |v| state.params.update(|p| p.volume = v)
                                )}
                            </div>
                        })}

                        // ===== TAB 4: AVANZADO =====
                        {move || (menu_tab.get() == 4).then(|| view! {
                            <div class="tab-enter space-y-2">
                                <div class="flex items-center gap-3 mb-1">
                                    <span class="text-[10px] font-mono text-white/25 uppercase tracking-wider">Modo</span>
                                    <button on:click=move |_| simple_mode.update(|v| *v = !*v)
                                        class={move || {
                                            let s = simple_mode.get();
                                            let base = "text-[10px] font-mono tracking-widest uppercase px-3 py-1 rounded-full border transition-all duration-300";
                                            if s {
                                                format!("{} bg-white/[0.06] text-white border-white/[0.12]", base)
                                            } else {
                                                format!("{} bg-white/[0.02] text-white/40 border-white/[0.06]", base)
                                            }
                                        }}
                                        style={move || if simple_mode.get() {
                                            let (r, g, b) = glow_rgb.get();
                                            format!("box-shadow: 0 0 16px rgba({},{},{},0.1)", r, g, b)
                                        } else { "".to_string() }}>
                                        {move || if simple_mode.get() { "Avanzado" } else { "Simple" }}
                                    </button>
                                </div>
                                <div class="text-[10px] font-mono text-white/20 mb-2">
                                    {move || if simple_mode.get() {
                                        "Controles avanzados visibles"
                                    } else {
                                        "Activa el modo avanzado para controles extra"
                                    }}
                                </div>
                                {move || simple_mode.get().then(|| view! {
                                    <div class="space-y-0.5">
                                        {slider!("Octavas", "<i class='fa-solid fa-layer-group'></i>", 1, 10, 1,
                                            move || state.params.get().octaves as f64,
                                            move || format!("{}", state.params.get().octaves),
                                            move |v| state.params.update(|p| p.octaves = v as u32)
                                        )}
                                        {slider!(move || state.params.get().formula.param_a_label(), "<i class='fa-solid fa-sliders'></i>", 0.0, 2.0, 0.01,
                                            move || state.params.get().param_a,
                                            move || format!("{:.2}", state.params.get().param_a),
                                            move |v| state.params.update(|p| p.param_a = v)
                                        )}
                                        {slider!(move || state.params.get().formula.param_b_label(), "<i class='fa-solid fa-sliders'></i>", 0.0, 2.0, 0.01,
                                            move || state.params.get().param_b,
                                            move || format!("{:.2}", state.params.get().param_b),
                                            move |v| state.params.update(|p| p.param_b = v)
                                        )}
                                        {slider!("Mutacion", "<i class='fa-solid fa-dna'></i>", 0.0, 1.0, 0.01,
                                            move || state.params.get().mutation,
                                            move || format!("{:.0}%", state.params.get().mutation * 100.0),
                                            move |v| state.params.update(|p| p.mutation = v)
                                        )}
                                    </div>
                                })}

                                // Save/Load section
                                <div class="border-t border-white/[0.04] pt-3 mt-4">
                                    <div class="flex items-center gap-2 mb-2">
                                        <i class="fa-solid fa-floppy-disk text-[10px]"
                                            style={move || format!("color: rgba({},{},{},0.5)", glow_rgb.get().0, glow_rgb.get().1, glow_rgb.get().2)}></i>
                                        <span class="text-[10px] font-mono text-white/30 uppercase tracking-wider">Guardar / Cargar</span>
                                    </div>
                                    {let eng = send_engine.clone(); (0u32..3).map(|slot| {
                                        let slot_name = match slot { 0 => "Auto", 1 => "Slot 2", _ => "Slot 3" };
                                        let state_load = state.clone();
                                        let save_eng = eng.clone();
                                        let load_eng = eng.clone();
                                        view! {
                                            <div class="flex items-center gap-1.5 mb-1">
                                                <span class="text-[9px] font-mono text-white/20 w-12">{slot_name}</span>
                                                <button
                                                    class="flex-1 py-1.5 rounded-lg bg-white/[0.03] hover:bg-white/[0.06] border border-white/[0.06] text-[10px] font-mono text-white/40 hover:text-white/70 transition-all duration-200 active:scale-95"
                                                    on:click=move |_| {
                                                        save_eng.save_to_slot(slot, &format!("Slot {}", slot + 1));
                                                    }>
                                                    <i class="fa-solid fa-floppy-disk mr-1"></i>Guardar
                                                </button>
                                                <button
                                                    class="flex-1 py-1.5 rounded-lg bg-white/[0.03] hover:bg-white/[0.06] border border-white/[0.06] text-[10px] font-mono text-white/40 hover:text-white/70 transition-all duration-200 active:scale-95"
                                                    on:click=move |_| {
                                                        if let Some(data) = Engine::load_from_slot(slot) {
                                                            state_load.params.set(data.params);
                                                            load_eng.apply_save(&data);
                                                        }
                                                    }>
                                                    <i class="fa-solid fa-upload mr-1"></i>Cargar
                                                </button>
                                                <button
                                                    class="w-7 h-7 rounded-lg bg-white/[0.02] hover:bg-red-500/20 border border-white/[0.04] text-white/20 hover:text-red-400 text-[9px] transition-all duration-200 active:scale-90"
                                                    on:click=move |_| { Engine::delete_slot(slot); }>
                                                    <i class="fa-solid fa-trash-can"></i>
                                                </button>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        })}

                    </div>
                </div>
            </div>

        </div>
    }
}
