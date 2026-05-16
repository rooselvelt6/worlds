use crate::engine::bridge;
use crate::engine::joystick::Joystick;
use crate::engine::minimap::Minimap;
use crate::engine::terrain::Zone;
use crate::engine::{Engine, HudData};
use crate::state::{AppState, FormulaType};
use leptos::html;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::window;

macro_rules! slider {
    ($label:expr, $icon:expr, $min:expr, $max:expr, $step:expr,
     $value:expr, $display:expr, $on_input:expr $(,)?) => {
        {
            let val = $value;
            let display = $display;
            let on_input = $on_input;
            let icon_str = $icon;
            view! {
                <div class="flex items-center gap-3 group/slider">
                    <span class="text-base shrink-0 w-6 text-center opacity-40 group-hover/slider:opacity-80 transition-opacity" inner_html=icon_str></span>
                    <span class="text-xs font-mono text-white/40 w-24 shrink-0 truncate">{$label}</span>
                    <div class="flex-1 relative">
                        <input type="range"
                            min=$min max=$max step=$step
                            prop:value=move || format!("{}", val())
                            on:input=move |ev| {
                                let i: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
                                on_input(i.value_as_number());
                            }
                            class="slider-glow flex-1 h-1.5 rounded-full bg-white/10 cursor-pointer w-full"
                        />
                    </div>
                    <span class="text-xs font-mono text-white/60 w-14 text-right tabular-nums">{move || display()}</span>
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
        (Zone::Fungus, "Fungus", "#a855f7", "fa-mushroom"),
        (Zone::Abyss, "Abismo", "#1e1b4b", "fa-skull"),
        (Zone::Storm, "Tormenta", "#64748b", "fa-cloud-bolt"),
        (Zone::Aurora, "Aurora", "#2dd4bf", "fa-aurora"),
        (Zone::Magma, "Magma", "#ea580c", "fa-meteor"),
    ];



    let simple_mode = RwSignal::new(false);
    let settings_tab = RwSignal::new(0u8);
    let tabs = ["Mundo", "Formula", "Color", "Control"];

    view! {
        <div class="w-screen h-screen overflow-hidden relative select-none antialiased"
            style="font-family: 'Inter', 'Orbitron', system-ui, sans-serif; background: #0a0a12;">

            // 3D Canvas
            <canvas node_ref=canvas_ref
                class="absolute inset-0 w-full h-full outline-none touch-none"
                tabindex="0"
            />

            // ===== TOP BAR =====
            <div class="absolute top-0 left-0 right-0 z-30 h-12 bg-black/70 backdrop-blur-xl border-b border-white/[0.04] flex items-center justify-between px-3">
                // Left: coords
                <div class="flex items-center gap-3">
                    <div class="flex items-center gap-1.5">
                        <i class="fa-solid fa-crosshairs text-[10px] text-cyan-400/60"></i>
                        <span class="text-cyan-300 font-bold text-sm font-mono tabular-nums tracking-wider">
                            {move || format!("{:04}", hud.get().pos[0])}
                        </span>
                        <span class="text-white/15 text-xs font-mono">/</span>
                        <span class="text-cyan-300 font-bold text-sm font-mono tabular-nums tracking-wider">
                            {move || format!("{:04}", hud.get().pos[2])}
                        </span>
                    </div>
                    <div class="w-px h-4 bg-white/5"></div>
                    <div class="flex items-center gap-1">
                        <i class="fa-solid fa-gauge-high text-[10px] text-cyan-400/60"></i>
                        <span class="text-cyan-300 font-bold text-sm font-mono tabular-nums">
                            {move || format!("{:04.1}", hud.get().speed)}
                        </span>
                        <span class="text-white/25 text-[9px] font-mono hidden sm:inline">VEL</span>
                    </div>
                    <div class="w-px h-4 bg-white/5 hidden sm:block"></div>
                    <div class="items-center gap-1 hidden sm:flex">
                        <div class="h-6 w-1.5 rounded-full bg-white/10 overflow-hidden relative">
                            <div class="absolute bottom-0 w-full rounded-full bg-gradient-to-t from-emerald-400 to-cyan-300 transition-all duration-200"
                                style:height={move || format!("{}%", ((hud.get().pos[1] / 20.0).min(1.0) * 100.0).max(5.0))}>
                            </div>
                        </div>
                        <span class="text-emerald-300 font-bold text-xs font-mono tabular-nums">
                            {move || format!("{:03.0}", hud.get().pos[1])}
                        </span>
                    </div>
                </div>

                // Center: compass
                <div class="flex items-center gap-0.5 bg-black/40 px-3 py-1 rounded-full border border-white/5">
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
                    <span class="text-purple-300/80 font-bold text-xs font-mono hidden sm:inline">{move || hud.get().biome}</span>
                    <span class="text-white/20 text-[10px] font-mono">{move || format!("{}fps", hud.get().fps)}</span>
                    <button on:click=move |_| settings_open.update(|v| *v = !*v)
                        class={move || {
                            let open = settings_open.get();
                            let base = "w-9 h-9 rounded-xl flex items-center justify-center transition-all duration-200 active:scale-90";
                            if open {
                                format!("{} bg-cyan-500/15 text-cyan-300 border border-cyan-400/25 shadow-lg shadow-cyan-500/10", base)
                            } else {
                                format!("{} text-white/40 hover:text-white/80 hover:bg-white/5 border border-transparent", base)
                            }
                        }}
                    >
                        <i class="fa-solid fa-sliders text-base"></i>
                    </button>
                </div>
            </div>

            // ===== LEFT SIDE: BIG JOYSTICK =====
            <div class="absolute left-3 top-1/2 -translate-y-1/2 z-20">
                <div class="bg-black/50 backdrop-blur-xl rounded-2xl p-2.5 border border-white/5 shadow-2xl shadow-cyan-500/5">
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
            <div class="absolute right-3 top-1/2 -translate-y-1/2 z-20 flex flex-col gap-2">
                // Jump
                <button
                    class="w-14 h-14 rounded-2xl bg-black/60 backdrop-blur-xl border border-white/8 text-white flex items-center justify-center active:scale-85 transition-all duration-100 hover:bg-white/10 shadow-lg shadow-black/30"
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
                        let base = "w-14 h-14 rounded-2xl backdrop-blur-xl border flex items-center justify-center active:scale-85 transition-all duration-100 shadow-lg shadow-black/30";
                        if fly {
                            format!("{} bg-cyan-500/15 text-cyan-300 border-cyan-400/25", base)
                        } else {
                            format!("{} bg-black/60 text-white/50 hover:text-white/85 hover:bg-white/8 border border-white/8", base)
                        }
                    }}
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
                        let base = "w-14 h-14 rounded-2xl backdrop-blur-xl border flex items-center justify-center active:scale-85 transition-all duration-100 shadow-lg shadow-black/30";
                        if sprint {
                            format!("{} bg-amber-500/15 text-amber-300 border-amber-400/25", base)
                        } else {
                            format!("{} bg-black/60 text-white/50 hover:text-white/85 hover:bg-white/8 border border-white/8", base)
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
                    class="w-14 h-14 rounded-2xl bg-black/60 backdrop-blur-xl border border-white/8 text-white flex items-center justify-center active:scale-85 transition-all duration-100 hover:bg-white/10 shadow-lg shadow-black/30"
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
            <div class="absolute bottom-4 left-3 z-20">
                <div class="bg-black/60 backdrop-blur-xl rounded-2xl p-2 border border-white/5 shadow-xl shadow-black/30">
                    <canvas node_ref=minimap_canvas_ref
                        class="w-[120px] h-[120px] rounded-full pointer-events-none"
                        width=120 height=120
                    />
                    <div class="flex items-center justify-center gap-1.5 mt-1">
                        <i class="fa-solid fa-location-dot text-[8px] text-purple-400/40"></i>
                        <span class="text-[7px] font-mono text-purple-300/40">{move || hud.get().biome}</span>
                    </div>
                </div>
            </div>

            // ===== MODE INDICATOR =====
            <div class="absolute bottom-4 left-1/2 -translate-x-1/2 z-20">
                <div class={move || {
                    let fly = hud.get().fly_mode;
                    let base = "px-4 py-1.5 rounded-full text-[11px] font-mono font-bold tracking-widest uppercase border backdrop-blur-xl transition-all duration-300 shadow-lg";
                    if fly {
                        format!("{} bg-cyan-900/30 text-cyan-300 border-cyan-500/25 shadow-cyan-500/10", base)
                    } else {
                        format!("{} bg-emerald-900/30 text-emerald-300 border-emerald-500/25 shadow-emerald-500/10", base)
                    }
                }}>
                    <i class={move || if hud.get().fly_mode { "fa-solid fa-wing mr-1.5" } else { "fa-solid fa-person-walking mr-1.5" }}></i>
                    {move || if hud.get().fly_mode { "VUELO" } else { "CAMINAR" }}
                </div>
            </div>

            // ===== MENU OVERLAY =====
            {move || {
                let forms = FormulaType::all();
                settings_open.get().then(|| view! {
                <div class="absolute inset-0 z-40 menu-overlay bg-black/80 flex items-center justify-center">
                    <div class="w-full max-w-lg max-h-[85vh] mx-4 bg-[#0d0d1a]/95 backdrop-blur-2xl rounded-3xl border border-white/[0.06] shadow-2xl shadow-cyan-500/5 overflow-hidden flex flex-col">

                        // Header
                        <div class="flex items-center justify-between px-5 py-3 border-b border-white/5">
                            <button on:click=move |_| settings_open.set(false)
                                class="flex items-center gap-1.5 text-white/40 hover:text-white/80 transition-colors text-sm"
                            >
                                <i class="fa-solid fa-arrow-left"></i>
                                <span class="font-mono text-xs tracking-wider">Volver</span>
                            </button>
                            <span class="text-xs font-mono font-bold tracking-widest text-white/30 uppercase"
                                style="font-family: 'Orbitron', monospace;">
                                Ajustes
                            </span>
                            <div class="w-16"></div>
                        </div>

                        // Tabs
                        <div class="flex gap-2 px-4 pt-3 pb-2">
                            <button on:click=move |_| menu_tab.set(0)
                                class={move || {
                                    let act = menu_tab.get() == 0;
                                    let base = "flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl text-xs font-bold tracking-wider uppercase transition-all duration-200";
                                    if act { format!("{} bg-cyan-500/10 text-cyan-300 border border-cyan-400/20 shadow-sm", base) }
                                    else { format!("{} text-white/30 hover:text-white/60 hover:bg-white/5 border border-transparent", base) }
                                }}
                            >
                                <i class="fa-solid fa-mountain"></i>
                                <span>Zonas</span>
                            </button>
                            <button on:click=move |_| menu_tab.set(1)
                                class={move || {
                                    let act = menu_tab.get() == 1;
                                    let base = "flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl text-xs font-bold tracking-wider uppercase transition-all duration-200";
                                    if act { format!("{} bg-cyan-500/10 text-cyan-300 border border-cyan-400/20 shadow-sm", base) }
                                    else { format!("{} text-white/30 hover:text-white/60 hover:bg-white/5 border border-transparent", base) }
                                }}
                            >
                                <i class="fa-solid fa-cube"></i>
                                <span>Formulas</span>
                            </button>
                            <button on:click=move |_| menu_tab.set(2)
                                class={move || {
                                    let act = menu_tab.get() == 2;
                                    let base = "flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl text-xs font-bold tracking-wider uppercase transition-all duration-200";
                                    if act { format!("{} bg-cyan-500/10 text-cyan-300 border border-cyan-400/20 shadow-sm", base) }
                                    else { format!("{} text-white/30 hover:text-white/60 hover:bg-white/5 border border-transparent", base) }
                                }}
                            >
                                <i class="fa-solid fa-gear"></i>
                                <span>Config</span>
                            </button>
                            <button on:click=move |_| menu_tab.set(3)
                                class={move || {
                                    let act = menu_tab.get() == 3;
                                    let base = "flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl text-xs font-bold tracking-wider uppercase transition-all duration-200";
                                    if act { format!("{} bg-cyan-500/10 text-cyan-300 border border-cyan-400/20 shadow-sm", base) }
                                    else { format!("{} text-white/30 hover:text-white/60 hover:bg-white/5 border border-transparent", base) }
                                }}
                            >
                                <i class="fa-solid fa-gamepad"></i>
                                <span>Control</span>
                            </button>
                        </div>

                        // Content
                        <div class="flex-1 overflow-y-auto px-4 pb-4 pt-1">
                            // ===== TAB: ZONAS =====
                            {move || (menu_tab.get() == 0).then(|| view! {
                                <div class="grid grid-cols-3 gap-2">
                                    {zones.iter().map(|&(zone, name, color, icon)| {
                                        view! {
                                            <button
                                                class={move || {
                                                    let active = state.params.get().zone == zone;
                                                    let base = "flex flex-col items-center gap-1.5 py-3 px-2 rounded-xl font-bold transition-all duration-200 active:scale-90 border";
                                                    if active {
                                                        format!("{} bg-white/10 text-white border-white/15 shadow-lg shadow-cyan-500/5", base)
                                                    } else {
                                                        format!("{} bg-white/[0.02] text-white/40 hover:text-white/70 hover:bg-white/5 border-white/[0.04]", base)
                                                    }
                                                }}
                                                on:click=move |_| state.params.update(|p| p.zone = zone)
                                            >
                                                <i class={format!("fa-solid {} text-xl", icon)} style={format!("color: {}", color)}></i>
                                                <span class="text-[9px] font-mono tracking-wider">{name}</span>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                // Random seed button
                                <button on:click=move |_| {
                                    state.params.update(|p| p.seed = (js_sys::Math::random() * 9999.0) as u32 + 1);
                                } class="mt-3 w-full py-2.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/8 text-white/60 hover:text-white/90 text-xs font-mono tracking-wider transition-all duration-200 active:scale-98 flex items-center justify-center gap-2">
                                    <i class="fa-solid fa-dice"></i>
                                    <span>Semilla Aleatoria</span>
                                </button>
                            })}

                            // ===== TAB: FORMULAS =====
                            {move || (menu_tab.get() == 1).then(|| view! {
                                <div class="grid grid-cols-4 gap-1.5">
                                    {forms.iter().map(|f| {
                                        let f = *f;
                                        view! {
                                            <button
                                                class={move || {
                                                    let active = state.params.get().formula == f;
                                                    let base = "flex flex-col items-center gap-0.5 py-2.5 px-1 rounded-xl font-bold transition-all duration-200 active:scale-90 border";
                                                    if active {
                                                        format!("{} bg-white/10 text-white border-white/15 shadow-sm", base)
                                                    } else {
                                                        format!("{} bg-white/[0.02] text-white/35 hover:text-white/65 hover:bg-white/5 border-white/[0.04]", base)
                                                    }
                                                }}
                                                on:click=move |_| state.params.update(|p| p.formula = f)
                                                title={f.name()}
                                            >
                                                <span class="text-lg">{f.emoji()}</span>
                                                <span class="text-[7px] font-mono truncate w-full text-center">{f.name()}</span>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                // Formula info
                                <div class="mt-2 p-2.5 rounded-xl bg-white/[0.02] border border-white/[0.04]">
                                    <div class="text-[10px] font-mono text-cyan-300/60 tabular-nums leading-relaxed">
                                        {move || {
                                            let p = state.params.get();
                                            format!("{}", p.formula.formula_expr(p.scale, p.octaves))
                                        }}
                                    </div>
                                </div>
                            })}

                            // ===== TAB: CONFIG =====
                            {move || (menu_tab.get() == 2).then(|| view! {
                                // Simple/Advanced toggle
                                <div class="flex items-center gap-3 mb-3">
                                    <span class="text-[10px] font-mono text-white/25 uppercase tracking-wider">Modo</span>
                                    <button on:click=move |_| simple_mode.update(|v| *v = !*v)
                                        class={move || {
                                            let s = simple_mode.get();
                                            let base = "text-[10px] font-mono tracking-widest uppercase px-3 py-1 rounded-full border transition-all duration-300";
                                            if s {
                                                format!("{} bg-cyan-900/20 text-cyan-400 border-cyan-500/20", base)
                                            } else {
                                                format!("{} bg-white/5 text-white/40 border-white/10", base)
                                            }
                                        }}>
                                        {move || if simple_mode.get() { "Avanzado" } else { "Simple" }}
                                    </button>
                                </div>

                                // Sub-tabs
                                <div class="flex gap-1 mb-3 bg-white/[0.02] rounded-lg p-0.5">
                                    {tabs.iter().enumerate().map(|(i, &name)| {
                                        let i_u8 = i as u8;
                                        view! {
                                            <button
                                                on:click=move |_| settings_tab.set(i_u8)
                                                class={move || {
                                                    let active = settings_tab.get() == i_u8;
                                                    let base = "flex-1 text-[9px] font-bold tracking-widest uppercase py-1.5 rounded-md transition-all duration-200";
                                                    if active {
                                                        format!("{} bg-white/10 text-white shadow-sm", base)
                                                    } else {
                                                        format!("{} text-white/30 hover:text-white/60", base)
                                                    }
                                                }}
                                            >
                                                {name}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>

                                // Mundo
                                {move || (settings_tab.get() == 0).then(|| view! {
                                    {slider!("Semilla", "<i class='fa-solid fa-seedling'></i>", 1, 9999, 1,
                                        move || state.params.get().seed as f64,
                                        move || format!("{:04}", state.params.get().seed),
                                        move |v| state.params.update(|p| p.seed = v as u32)
                                    )}
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
                                })}

                                // Formula
                                {move || (settings_tab.get() == 1).then(|| view! {
                                    {slider!("Frecuencia", "<i class='fa-solid fa-chart-line'></i>", 0.002, 0.15, 0.001,
                                        move || state.params.get().scale,
                                        move || format!("{:.3}", state.params.get().scale),
                                        move |v| state.params.update(|p| p.scale = v)
                                    )}
                                    {move || simple_mode.get().then(|| view! {
                                        {slider!("Octavas", "<i class='fa-solid fa-layer-group'></i>", 1, 10, 1,
                                            move || state.params.get().octaves as f64,
                                            move || format!("{}", state.params.get().octaves),
                                            move |v| state.params.update(|p| p.octaves = v as u32)
                                        )}
                                    })}
                                    {slider!("Amplitud", "<i class='fa-solid fa-arrow-up-wide-short'></i>", 0.2, 8.0, 0.1,
                                        move || state.params.get().amplitude,
                                        move || format!("{:.1}", state.params.get().amplitude),
                                        move |v| state.params.update(|p| p.amplitude = v)
                                    )}
                                    {slider!("Agua", "<i class='fa-solid fa-water'></i>", 0.0, 5.0, 0.1,
                                        move || state.params.get().water_level,
                                        move || format!("{:.1}", state.params.get().water_level),
                                        move |v| state.params.update(|p| p.water_level = v)
                                    )}
                                    {move || simple_mode.get().then(|| view! {
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
                                    })}
                                })}

                                // Color
                                {move || (settings_tab.get() == 2).then(|| view! {
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
                                })}

                                // Control
                                {move || (settings_tab.get() == 3).then(|| view! {
                                    <div class="space-y-2">
                                        <div class="flex items-center justify-between px-3 py-2 rounded-xl bg-white/[0.02] border border-white/[0.04]">
                                            <span class="text-xs font-mono text-white/40 flex items-center gap-2">
                                                <i class="fa-solid fa-wing text-cyan-400/60"></i>
                                                Vuelo
                                            </span>
                                            <button on:click=move |_| state.params.update(|p| p.fly_mode = !p.fly_mode)
                                                class={move || {
                                                    let fly = state.params.get().fly_mode;
                                                    let base = "text-[10px] font-mono font-bold px-3 py-1 rounded-full border transition-all duration-200";
                                                    if fly {
                                                        format!("{} bg-cyan-900/20 text-cyan-300 border-cyan-500/20", base)
                                                    } else {
                                                        format!("{} bg-white/5 text-white/40 border-white/10", base)
                                                    }
                                                }}>
                                                {move || if state.params.get().fly_mode { "SI" } else { "NO" }}
                                            </button>
                                        </div>
                                        {move || simple_mode.get().then(|| view! {
                                            <div class="flex items-center justify-between px-3 py-2 rounded-xl bg-white/[0.02] border border-white/[0.04]">
                                                <span class="text-xs font-mono text-white/40 flex items-center gap-2">
                                                    <i class="fa-solid fa-gamepad text-cyan-400/60"></i>
                                                    Control
                                                </span>
                                                <button on:click=move |_| state.params.update(|p| p.control_mode = match p.control_mode {
                                                    crate::state::ControlMode::DPad => crate::state::ControlMode::Joystick,
                                                    crate::state::ControlMode::Joystick => crate::state::ControlMode::DPad,
                                                })>
                                                    <span class="text-[10px] font-mono font-bold px-3 py-1 rounded-full bg-white/10 text-white/60">
                                                        {move || format!("{:?}", state.params.get().control_mode)}
                                                    </span>
                                                </button>
                                            </div>
                                        })}
                                        <div class="px-3 py-2.5 rounded-xl bg-white/[0.02] border border-white/[0.04]">
                                            <div class="text-[10px] font-mono text-white/25 leading-relaxed flex flex-wrap gap-x-4 gap-y-1">
                                                <span><i class="fa-solid fa-keyboard text-white/15 mr-1"></i>WASD</span>
                                                <span><i class="fa-solid fa-arrows-rotate text-white/15 mr-1"></i>Q/E</span>
                                                <span><i class="fa-solid fa-arrow-up text-white/15 mr-1"></i>Saltar</span>
                                                <span><i class="fa-solid fa-arrow-down text-white/15 mr-1"></i>Agachar</span>
                                            </div>
                                        </div>
                                    </div>
                                })}
                            })}

                            // ===== TAB: CONTROL (ayuda) =====
                            {move || (menu_tab.get() == 3).then(|| view! {
                                <div class="space-y-3">
                                    <div class="p-4 rounded-xl bg-white/[0.02] border border-white/[0.04]">
                                        <h3 class="text-xs font-mono font-bold text-white/40 uppercase tracking-wider mb-3 flex items-center gap-2">
                                            <i class="fa-solid fa-keyboard text-cyan-400/60"></i>
                                            Teclado
                                        </h3>
                                        <div class="grid grid-cols-2 gap-2 text-[10px] font-mono">
                                            <div class="flex items-center gap-2 text-white/30"><span class="px-2 py-0.5 rounded bg-white/5 text-white/60 font-bold text-[9px]">W</span> Avanzar</div>
                                            <div class="flex items-center gap-2 text-white/30"><span class="px-2 py-0.5 rounded bg-white/5 text-white/60 font-bold text-[9px]">S</span> Retroceder</div>
                                            <div class="flex items-center gap-2 text-white/30"><span class="px-2 py-0.5 rounded bg-white/5 text-white/60 font-bold text-[9px]">A</span> Izquierda</div>
                                            <div class="flex items-center gap-2 text-white/30"><span class="px-2 py-0.5 rounded bg-white/5 text-white/60 font-bold text-[9px]">D</span> Derecha</div>
                                            <div class="flex items-center gap-2 text-white/30"><span class="px-2 py-0.5 rounded bg-white/5 text-white/60 font-bold text-[9px]">Q</span> Giro Izq</div>
                                            <div class="flex items-center gap-2 text-white/30"><span class="px-2 py-0.5 rounded bg-white/5 text-white/60 font-bold text-[9px]">E</span> Giro Der</div>
                                            <div class="flex items-center gap-2 text-white/30"><span class="px-2 py-0.5 rounded bg-white/5 text-white/60 font-bold text-[9px]">Esp</span> Saltar</div>
                                            <div class="flex items-center gap-2 text-white/30"><span class="px-2 py-0.5 rounded bg-white/5 text-white/60 font-bold text-[9px]">May</span> Agachar</div>
                                        </div>
                                    </div>
                                    <div class="p-4 rounded-xl bg-white/[0.02] border border-white/[0.04]">
                                        <h3 class="text-xs font-mono font-bold text-white/40 uppercase tracking-wider mb-3 flex items-center gap-2">
                                            <i class="fa-solid fa-tablet-screen text-cyan-400/60"></i>
                                            Tactil
                                        </h3>
                                        <div class="text-[10px] font-mono text-white/30 leading-relaxed space-y-1">
                                            <p><i class="fa-regular fa-circle-dot text-cyan-400/60 mr-1.5"></i>Joystick izquierdo para mover</p>
                                            <p><i class="fa-regular fa-circle-dot text-cyan-400/60 mr-1.5"></i>Botones derecho para acciones</p>
                                            <p><i class="fa-regular fa-circle-dot text-cyan-400/60 mr-1.5"></i>Deslizar para mirar alrededor</p>
                                        </div>
                                    </div>
                                </div>
                            })}
                        </div>
                    </div>
                </div>
            })}}
        </div>
    }
}
