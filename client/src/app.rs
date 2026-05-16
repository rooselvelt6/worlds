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
            view! {
                <div class="flex items-center gap-1.5 group/slider">
                    <span class="text-[9px] shrink-0 w-4 text-center opacity-40 group-hover/slider:opacity-80 transition-opacity">{$icon}</span>
                    <span class="text-[8px] font-mono text-white/40 w-16 shrink-0 truncate">{$label}</span>
                    <div class="flex-1 relative">
                        <input type="range"
                            min=$min max=$max step=$step
                            prop:value=move || format!("{}", val())
                            on:input=move |ev| {
                                let i: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
                                on_input(i.value_as_number());
                            }
                            class="slider-glow flex-1 h-0.5 rounded-full bg-white/10 cursor-pointer w-full"
                        />
                    </div>
                    <span class="text-[8px] font-mono text-white/60 w-10 text-right tabular-nums">{move || display()}</span>
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
    let settings_tab = RwSignal::new(0u8);
    let simple_mode = RwSignal::new(false);

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
                mc.set_width(180);
                mc.set_height(180);
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
        (Zone::Forest, "forest", "#22c55e"),
        (Zone::Plains, "plains", "#a3e635"),
        (Zone::Desert, "desert", "#f59e0b"),
        (Zone::Tundra, "tundra", "#e0f2fe"),
        (Zone::Jungle, "jungle", "#166534"),
        (Zone::Volcanic, "volcanic", "#7c2d12"),
        (Zone::Ocean, "ocean", "#0ea5e9"),
        (Zone::Crystal, "crystal", "#a855f7"),
        (Zone::Cave, "cave", "#525252"),
        (Zone::Lava, "lava", "#ef4444"),
        (Zone::Fungus, "fungus", "#a855f7"),
        (Zone::Abyss, "abyss", "#1e1b4b"),
        (Zone::Storm, "storm", "#64748b"),
        (Zone::Aurora, "aurora", "#2dd4bf"),
        (Zone::Magma, "magma", "#ea580c"),
    ];

    let zone_svg = |zone: Zone| -> &'static str {
        match zone {
            Zone::Forest => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><path d='M12 2L4 10h16L12 2z'/><path d='M12 10v12'/><path d='M8 16h8'/></svg>",
            Zone::Plains => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><path d='M2 20L6 10l4 6 4-8 4 6 4-4v10H2z'/></svg>",
            Zone::Desert => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><path d='M2 20l4-8 4 4 4-6 4 4 4-6v12H2z'/><circle cx='6' cy='8' r='1.5' fill='currentColor'/></svg>",
            Zone::Tundra => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><path d='M4 20L8 4l4 16 4-16 4 16'/><line x1='2' y1='20' x2='22' y2='20'/></svg>",
            Zone::Jungle => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><path d='M12 2C8 6 4 12 4 18c0 3 2 4 8 4s8-1 8-4c0-6-4-12-8-16z'/><path d='M12 14a3 3 0 100-6 3 3 0 000 6z'/></svg>",
            Zone::Volcanic => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><path d='M8 20l4-18 4 18'/><path d='M4 20h16'/><circle cx='12' cy='12' r='2' fill='currentColor'/></svg>",
            Zone::Ocean => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><path d='M2 12c2-2 4-2 6 0s4 2 6 0 4-2 6 0'/><path d='M2 18c2-2 4-2 6 0s4 2 6 0 4-2 6 0'/></svg>",
            Zone::Crystal => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><polygon points='12,2 20,20 4,20'/><polygon points='12,8 16,18 8,18'/></svg>",
            Zone::Cave => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><path d='M6 20V8c0-2 2-4 6-4s6 2 6 4v12'/><path d='M4 20h16'/><path d='M10 12h4'/><path d='M10 16h4'/></svg>",
            Zone::Lava => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><circle cx='12' cy='12' r='8'/><path d='M12 4v4m0 8v4M4 12h4m8 0h4'/><circle cx='12' cy='12' r='2' fill='currentColor'/></svg>",
            Zone::Fungus => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><ellipse cx='12' cy='16' rx='5' ry='3'/><ellipse cx='12' cy='12' rx='4' ry='3'/><circle cx='12' cy='6' r='3' fill='currentColor'/></svg>",
            Zone::Abyss => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><path d='M2 22L12 2l10 20H2z'/><circle cx='12' cy='14' r='2' fill='currentColor'/></svg>",
            Zone::Storm => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><path d='M6 16c0-3 2-5 6-5s6 2 6 5'/><path d='M12 11V4'/><line x1='10' y1='7' x2='14' y2='7'/><line x1='12' y1='2' x2='12' y2='4'/></svg>",
            Zone::Aurora => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><path d='M2 16c3-4 6-4 8 0s5 4 8 0 4-4 4 0'/><path d='M4 20c2-3 5-3 7 0s4 3 7 0'/></svg>",
            Zone::Magma => "<svg viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' class='w-4 h-4'><path d='M12 2c-3 4-6 8-6 12a6 6 0 0012 0c0-4-3-8-6-12z'/><circle cx='12' cy='14' r='2' fill='currentColor'/></svg>",
        }
    };

    let formulas = FormulaType::all();

    let tabs = ["Mundo", "Formula", "Color", "Control"];
    let tab_icons = ["\u{1f30d}", "\u{1f4d0}", "\u{1f3a8}", "\u{1f3ae}"];

    view! {
        <div class="w-screen h-screen overflow-hidden relative select-none antialiased"
            style="font-family: 'Inter', 'Orbitron', system-ui, sans-serif; background: #0a0a12;">

            <canvas node_ref=canvas_ref
                class="absolute inset-0 w-full h-full outline-none touch-none"
                tabindex="0"
            />

            // ===== TOP-LEFT HUD =====
            <div class="absolute top-3 left-3 z-30 flex flex-col gap-1">
                <div class="px-2.5 py-1 rounded-lg bg-black/60 backdrop-blur-md border-l-2 border-violet-400 shadow-lg shadow-violet-500/5">
                    <div class="flex items-center gap-2 text-[9px] font-mono">
                        <span class="text-white/30 text-[7px]">X</span>
                        <span class="text-violet-300 font-bold tabular-nums tracking-wider">
                            {move || format!("{:04.0}", hud.get().pos[0])}
                        </span>
                        <span class="text-white/10 text-[7px]">|</span>
                        <span class="text-white/30 text-[7px]">Z</span>
                        <span class="text-violet-300 font-bold tabular-nums tracking-wider">
                            {move || format!("{:04.0}", hud.get().pos[2])}
                        </span>
                    </div>
                </div>

                <div class="flex gap-1">
                    <div class="px-2.5 py-1.5 rounded-lg bg-black/60 backdrop-blur-md border-l-2 border-cyan-400 shadow-lg shadow-cyan-500/5">
                        <div class="flex items-center gap-2 text-[10px] font-mono">
                            <span class="text-cyan-300 font-bold tabular-nums tracking-wider">
                                {move || format!("{:04.1}", hud.get().speed)}
                            </span>
                            <span class="text-white/30 text-[8px]">VEL</span>
                        </div>
                        <div class="mt-0.5" style="width: 56px;">
                            <div class="h-0.5 rounded-full bg-white/10 overflow-hidden">
                                <div class="h-full rounded-full bg-gradient-to-r from-cyan-400 to-purple-400 transition-all duration-150"
                                    style:width={move || format!("{}%", (hud.get().speed / 50.0 * 100.0).min(100.0))}>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="flex items-center gap-1 px-2 py-1.5 rounded-lg bg-black/60 backdrop-blur-md border-l-2 border-emerald-400 shadow-lg shadow-emerald-500/5">
                        <div class="flex flex-col items-center gap-0.5">
                            <div class="h-10 w-2 rounded-full bg-white/10 overflow-hidden relative">
                                <div class="absolute bottom-0 w-full rounded-full bg-gradient-to-t from-emerald-400 to-cyan-300 transition-all duration-200"
                                    style:height={move || format!("{}%", ((hud.get().pos[1] / 20.0).min(1.0) * 100.0).max(5.0))}>
                                </div>
                            </div>
                            <span class="text-emerald-300 font-bold text-[8px] font-mono tabular-nums">
                                {move || format!("{:03.0}", hud.get().pos[1])}
                            </span>
                        </div>
                    </div>
                </div>
            </div>

            // ===== TOP-RIGHT HUD =====
            <div class="absolute top-3 right-3 z-30 flex flex-col gap-1.5 items-end">
                <div class="px-2.5 py-1.5 rounded-lg bg-black/60 backdrop-blur-md border-r-2 border-amber-400 shadow-lg shadow-amber-500/5">
                    <div class="flex items-center gap-2 text-[10px] font-mono">
                        <span class="text-white/30 text-[8px]">RUMBO</span>
                        <span class="text-amber-300 font-bold tabular-nums tracking-wider">
                            {move || format!("{:03}\u{b0}", hud.get().yaw_deg)}
                        </span>
                    </div>
                </div>
                <div class="px-2.5 py-1.5 rounded-lg bg-black/60 backdrop-blur-md border-r-2 border-purple-400 shadow-lg shadow-purple-500/5">
                    <div class="text-purple-300 font-bold text-[10px] font-mono">{move || hud.get().biome}</div>
                </div>
                <div class="px-2.5 py-1 rounded-lg bg-black/40 backdrop-blur-md text-[9px] font-mono">
                    <span class="text-white/20">{move || {
                        let h = hud.get();
                        format!("{:03}fps {:02}ch", h.fps, h.chunks)
                    }}</span>
                </div>
            </div>

            // ===== COMPASS BAR =====
            <div class="absolute top-2 left-1/2 -translate-x-1/2 z-30 px-3 py-1 rounded-full bg-black/40 backdrop-blur-md border border-white/5">
                <div class="flex gap-6 text-[8px] font-mono text-white/20">
                    <span class={move || if hud.get().yaw_deg > 315 || hud.get().yaw_deg <= 45 { "text-cyan-300 font-bold" } else { "" }}>N</span>
                    <span class="text-white/10">|</span>
                    <span class={move || if hud.get().yaw_deg > 45 && hud.get().yaw_deg <= 135 { "text-cyan-300 font-bold" } else { "" }}>E</span>
                    <span class="text-white/10">|</span>
                    <span class={move || if hud.get().yaw_deg > 135 && hud.get().yaw_deg <= 225 { "text-cyan-300 font-bold" } else { "" }}>S</span>
                    <span class="text-white/10">|</span>
                    <span class={move || if hud.get().yaw_deg > 225 && hud.get().yaw_deg <= 315 { "text-cyan-300 font-bold" } else { "" }}>W</span>
                </div>
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

            // ===== MODE INDICATOR =====
            <div class="absolute bottom-24 left-1/2 -translate-x-1/2 z-30">
                <div class={move || {
                    let fly = hud.get().fly_mode;
                    let base = "px-3 py-1 rounded-full text-[9px] font-mono font-bold tracking-widest uppercase border backdrop-blur-md transition-all duration-300";
                    if fly {
                        format!("{} bg-cyan-900/30 text-cyan-300 border-cyan-500/30 shadow-lg shadow-cyan-500/5", base)
                    } else {
                        format!("{} bg-emerald-900/30 text-emerald-300 border-emerald-500/30 shadow-lg shadow-emerald-500/5", base)
                    }
                }}>
                    {move || if hud.get().fly_mode { "\u{2b06} VUELO" } else { "\u{1f6b6} CAMINAR" }}
                </div>
            </div>

            // ===== MINIMAP =====
            <canvas node_ref=minimap_canvas_ref
                class="absolute bottom-28 right-3 z-30 w-[180px] h-[180px] rounded-full pointer-events-none"
                style="box-shadow: 0 0 20px rgba(0,0,0,0.5), 0 0 40px rgba(0,255,255,0.05);"
                width=180 height=180
            />

            // ===== JOYSTICK (touch/gamepad) =====
            <canvas node_ref=joystick_canvas_ref
                class="absolute bottom-4 left-4 z-30 w-[100px] h-[100px] touch-none"
                style="display: none;"
                width=100 height=100
            />

            // ===== SETTINGS PANEL =====
            {move || settings_open.get().then(|| view! {
                <div class="absolute top-20 right-3 bottom-24 z-40 w-64 pointer-events-none">
                    <div class="pointer-events-auto w-full h-full bg-black/85 backdrop-blur-xl rounded-2xl border border-white/[0.06] p-2.5 space-y-1 overflow-y-auto shadow-2xl shadow-cyan-500/5">

                        // Header
                        <div class="flex items-center justify-between mb-1">
                            <span class="text-[9px] font-mono font-bold tracking-widest text-white/30 uppercase"
                                style="font-family: 'Orbitron', monospace;">Ajustes</span>
                            <button on:click=move |_| settings_open.set(false)
                                class="text-white/30 hover:text-white/70 text-[10px] px-1 rounded transition-colors">&times;</button>
                        </div>

                        // Simple/Advanced toggle
                        <div class="flex items-center gap-2 mb-1">
                            <div class="flex-1 h-px bg-white/5"></div>
                            <button on:click=move |_| simple_mode.update(|v| *v = !*v)
                                class={move || {
                                    let s = simple_mode.get();
                                    let base = "text-[7px] font-mono tracking-widest uppercase px-2 py-0.5 rounded-full border transition-all duration-300";
                                    if s {
                                        format!("{} bg-cyan-900/20 text-cyan-400 border-cyan-500/20", base)
                                    } else {
                                        format!("{} bg-white/5 text-white/30 border-white/10", base)
                                    }
                                }}>
                                {move || if simple_mode.get() { "Avanzado" } else { "Simple" }}
                            </button>
                        </div>

                        // Tabs
                        <div class="flex gap-0.5 mb-2 bg-white/[0.03] rounded-lg p-0.5">
                            {tabs.iter().enumerate().map(|(i, &name)| {
                                let i_u8 = i as u8;
                                view! {
                                    <button
                                        on:click=move |_| settings_tab.set(i_u8)
                                        class={move || {
                                            let active = settings_tab.get() == i_u8;
                                            let base = "flex-1 text-[8px] font-bold tracking-widest uppercase py-1 rounded-md transition-all duration-200";
                                            if active {
                                                format!("{} bg-white/10 text-white shadow-sm", base)
                                            } else {
                                                format!("{} text-white/30 hover:text-white/60", base)
                                            }
                                        }}
                                    >
                                        <span class="mr-1">{tab_icons[i]}</span>{name}
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>

                        // ===== TAB: MUNDO =====
                        {move || (settings_tab.get() == 0).then(|| view! {
                            {slider!("Semilla del Mundo", "\u{1f331}", 1, 9999, 1,
                                move || state.params.get().seed as f64,
                                move || format!("{:04}", state.params.get().seed),
                                move |v| state.params.update(|p| p.seed = v as u32)
                            )}
                            {slider!("Velocidad", "\u{1f3c3}", 1, 100, 1,
                                move || state.params.get().speed,
                                move || format!("{:02}", state.params.get().speed as u32),
                                move |v| state.params.update(|p| p.speed = v)
                            )}
                            {slider!("Sensibilidad", "\u{1f3af}", 0.1, 5.0, 0.1,
                                move || state.params.get().mouse_sensitivity,
                                move || format!("{:.1}", state.params.get().mouse_sensitivity),
                                move |v| state.params.update(|p| p.mouse_sensitivity = v)
                            )}
                            {slider!("Horizonte", "\u{1f441}", 2, 8, 1,
                                move || state.params.get().render_distance as f64,
                                move || format!("{}", state.params.get().render_distance),
                                move |v| state.params.update(|p| p.render_distance = v as u32)
                            )}
                        })}

                        // ===== TAB: FORMULA =====
                        {move || (settings_tab.get() == 1).then(|| view! {
                            <div class="px-1.5 py-1 rounded bg-white/[0.02] border border-white/[0.03] mb-1">
                                <div class="text-[7px] font-mono text-cyan-300/70 tabular-nums leading-relaxed">
                                    {move || {
                                        let p = state.params.get();
                                        format!("h = {} \u{d7} {:.1} + {:.1}", p.formula.formula_expr(p.scale, p.octaves), p.amplitude, p.water_level)
                                    }}
                                </div>
                            </div>
                            {slider!("Frecuencia Vital", "\u{1f4ca}", 0.002, 0.15, 0.001,
                                move || state.params.get().scale,
                                move || format!("{:.3}", state.params.get().scale),
                                move |v| state.params.update(|p| p.scale = v)
                            )}
                            {move || simple_mode.get().then(|| view! {
                                {slider!("Octavas", "\u{1f50a}", 1, 10, 1,
                                    move || state.params.get().octaves as f64,
                                    move || format!("{}", state.params.get().octaves),
                                    move |v| state.params.update(|p| p.octaves = v as u32)
                                )}
                            })}
                            {slider!("Gravedad Onirica", "\u{2696}\u{fe0f}", 0.2, 8.0, 0.1,
                                move || state.params.get().amplitude,
                                move || format!("{:.1}", state.params.get().amplitude),
                                move |v| state.params.update(|p| p.amplitude = v)
                            )}
                            {slider!("Oceano Interior", "\u{1f30a}", 0.0, 5.0, 0.1,
                                move || state.params.get().water_level,
                                move || format!("{:.1}", state.params.get().water_level),
                                move |v| state.params.update(|p| p.water_level = v)
                            )}
                            {move || simple_mode.get().then(|| view! {
                                {slider!(move || state.params.get().formula.param_a_label(), "\u{1f527}", 0.0, 2.0, 0.01,
                                    move || state.params.get().param_a,
                                    move || format!("{:.2}", state.params.get().param_a),
                                    move |v| state.params.update(|p| p.param_a = v)
                                )}
                                {slider!(move || state.params.get().formula.param_b_label(), "\u{1f527}", 0.0, 2.0, 0.01,
                                    move || state.params.get().param_b,
                                    move || format!("{:.2}", state.params.get().param_b),
                                    move |v| state.params.update(|p| p.param_b = v)
                                )}
                            })}
                        })}

                        // ===== TAB: COLOR =====
                        {move || (settings_tab.get() == 2).then(|| view! {
                            {slider!("Armonia", "\u{1f3a8}", 0, 360, 1,
                                move || state.params.get().hue_shift,
                                move || format!("{:03.0}", state.params.get().hue_shift),
                                move |v| state.params.update(|p| p.hue_shift = v)
                            )}
                            {slider!("Intensidad", "\u{2728}", 0.0, 2.0, 0.01,
                                move || state.params.get().saturation,
                                move || format!("{:.2}", state.params.get().saturation),
                                move |v| state.params.update(|p| p.saturation = v)
                            )}
                            {slider!("Luminosidad", "\u{2600}\u{fe0f}", 0.0, 2.0, 0.01,
                                move || state.params.get().lightness,
                                move || format!("{:.2}", state.params.get().lightness),
                                move |v| state.params.update(|p| p.lightness = v)
                            )}
                        })}

                        // ===== TAB: CONTROL =====
                        {move || (settings_tab.get() == 3).then(|| view! {
                            <div class="space-y-1.5">
                                <div class="flex items-center justify-between px-1.5 py-1 rounded bg-white/[0.02] border border-white/[0.03]">
                                    <span class="text-[8px] font-mono text-white/40">Modo Vuelo</span>
                                    <button on:click=move |_| state.params.update(|p| p.fly_mode = !p.fly_mode)
                                        class={move || {
                                            let fly = state.params.get().fly_mode;
                                            let base = "text-[8px] font-mono font-bold px-2 py-0.5 rounded-full border transition-all duration-200";
                                            if fly {
                                                format!("{} bg-cyan-900/20 text-cyan-300 border-cyan-500/20", base)
                                            } else {
                                                format!("{} bg-white/5 text-white/40 border-white/10", base)
                                            }
                                        }}>
                                        {move || if state.params.get().fly_mode { "FLY" } else { "WALK" }}
                                    </button>
                                </div>
                                {move || simple_mode.get().then(|| view! {
                                    <div class="flex items-center justify-between px-1.5 py-1 rounded bg-white/[0.02] border border-white/[0.03]">
                                        <span class="text-[8px] font-mono text-white/40">Control</span>
                                        <button on:click=move |_| state.params.update(|p| p.control_mode = match p.control_mode {
                                            crate::state::ControlMode::DPad => crate::state::ControlMode::Joystick,
                                            crate::state::ControlMode::Joystick => crate::state::ControlMode::DPad,
                                        })>
                                            <span class="text-[8px] font-mono font-bold px-2 py-0.5 rounded-full bg-white/10 text-white/60">
                                                {move || format!("{:?}", state.params.get().control_mode)}
                                            </span>
                                        </button>
                                    </div>
                                })}
                                <div class="px-1.5 py-1.5 rounded bg-white/[0.02] border border-white/[0.03]">
                                    <div class="text-[7px] font-mono text-white/25 leading-relaxed">
                                        WASD - Andar | Q/E - Giro<br/>
                                        Espacio - Saltar | Shift - Agacharse<br/>
                                        Click - Bloquear raton
                                    </div>
                                </div>
                            </div>
                        })}

                    </div>
                </div>
            })}

            // ===== BOTTOM CONTROLS BAR =====
            <div class="absolute bottom-0 left-0 right-0 z-30 pb-2 sm:pb-3 px-2 pointer-events-none">
                <div class="max-w-2xl mx-auto pointer-events-auto space-y-1.5">

                    // Zones row with SVG icons
                    <div class="flex justify-center gap-0.5 sm:gap-1 bg-black/40 backdrop-blur-lg rounded-2xl px-2 py-1.5 border border-white/[0.03] mx-auto w-fit">
                        {zones.iter().map(|&(zone, name, _)| {
                            let svg = zone_svg(zone);
                            view! {
                                <button
                                    class={move || {
                                        let active = state.params.get().zone == zone;
                                        let base = "shrink-0 text-sm sm:text-base px-1.5 sm:px-2.5 py-1 rounded-xl font-bold transition-all duration-200 active:scale-85 leading-none";
                                        if active {
                                            format!("{} bg-white/15 text-white border border-white/20 shadow-lg", base)
                                        } else {
                                            format!("{} text-white/30 hover:text-white/70 hover:bg-white/5 border border-transparent", base)
                                        }
                                    }}
                                    on:click=move |_| state.params.update(|p| p.zone = zone)
                                    title=name
                                    inner_html=svg
                                ></button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>

                    // Formulas row
                    <div class="flex justify-center gap-0.5 sm:gap-1 bg-black/40 backdrop-blur-lg rounded-2xl px-2 py-1.5 border border-white/[0.03] mx-auto w-fit">
                        {formulas.iter().map(|f| {
                            let f = *f;
                            view! {
                                <button
                                    class={move || {
                                        let active = state.params.get().formula == f;
                                        let base = "shrink-0 text-xs sm:text-sm px-1.5 sm:px-2 py-1 rounded-lg font-bold transition-all duration-200 active:scale-85 leading-none";
                                        if active {
                                            format!("{} bg-white/15 text-white border border-white/20 shadow-lg", base)
                                        } else {
                                            format!("{} text-white/25 hover:text-white/60 hover:bg-white/5 border border-transparent", base)
                                        }
                                    }}
                                    on:click=move |_| state.params.update(|p| p.formula = f)
                                    title={f.name()}
                                >
                                    <span>{f.emoji()}</span>
                                </button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>

                    // Action buttons
                    <div class="flex justify-center gap-1.5 sm:gap-2">
                        <button
                            class="text-[10px] sm:text-xs px-2.5 sm:px-3.5 py-1.5 rounded-xl font-bold transition-all duration-200 active:scale-85 bg-black/50 backdrop-blur-md border border-white/5 text-white/50 hover:text-white/80 hover:bg-white/5"
                            on:click={let s = state.clone(); move |_| {
                                s.params.update(|p| p.seed = (js_sys::Math::random() * 9999.0) as u32 + 1);
                            }}
                        >
                            <span class="mr-1">{"\u{1f3b2}"}</span>
                            <span class="hidden sm:inline">Nuevo Mundo</span>
                        </button>
                        <button
                            class={move || {
                                let fly = state.params.get().fly_mode;
                                let base = "text-[10px] sm:text-xs px-2.5 sm:px-3.5 py-1.5 rounded-xl font-bold transition-all duration-200 active:scale-85 backdrop-blur-md border";
                                if fly {
                                    format!("{} bg-cyan-500/10 text-cyan-300 border-cyan-500/20 shadow-lg shadow-cyan-500/5", base)
                                } else {
                                    format!("{} bg-black/50 text-white/60 hover:text-white/90 hover:bg-white/5 border border-white/5", base)
                                }
                            }}
                            on:click={let s = state.clone(); move |_| {
                                s.params.update(|p| p.fly_mode = !p.fly_mode);
                            }}
                        >
                            <span>{move || if state.params.get().fly_mode { "\u{1f9bd}" } else { "\u{1f6b6}" }}</span>
                            <span class="ml-1 hidden sm:inline">{move || if state.params.get().fly_mode { "Volar" } else { "Andar" }}</span>
                        </button>
                        <button
                            class={move || {
                                let sprint = state.params.get().speed > 25.0;
                                let base = "text-[10px] sm:text-xs px-2.5 sm:px-3.5 py-1.5 rounded-xl font-bold transition-all duration-200 active:scale-85 backdrop-blur-md border";
                                if sprint {
                                    format!("{} bg-amber-500/10 text-amber-300 border-amber-500/20 shadow-lg shadow-amber-500/5", base)
                                } else {
                                    format!("{} bg-black/50 text-white/60 hover:text-white/90 hover:bg-white/5 border border-white/5", base)
                                }
                            }}
                            on:click={let s = state.clone(); move |_| {
                                s.params.update(|p| p.speed = if p.speed > 25.0 { 18.0 } else { 45.0 });
                            }}
                        >
                            <span>{"\u{1f3c3}"}</span>
                            <span class="ml-1 hidden sm:inline">{move || if state.params.get().speed > 25.0 { "Sprint" } else { "Paso" }}</span>
                        </button>
                        <button
                            class={move || {
                                let open = settings_open.get();
                                let base = "text-[10px] sm:text-xs px-2.5 sm:px-3.5 py-1.5 rounded-xl font-bold transition-all duration-200 active:scale-85 backdrop-blur-md border";
                                if open {
                                    format!("{} bg-cyan-500/15 text-cyan-300 border-cyan-500/25 shadow-lg shadow-cyan-500/5", base)
                                } else {
                                    format!("{} bg-black/50 text-white/60 hover:text-white/90 hover:bg-white/5 border border-white/5", base)
                                }
                            }}
                            on:click=move |_| settings_open.update(|v| *v = !*v)
                        >
                            <span>{"\u{2699}\u{fe0f}"}</span>
                        </button>
                    </div>

                    <div class="text-center text-[6px] text-white/10 font-mono tracking-widest uppercase">
                        Click | Wasd | Qe | Espacio | Shift
                    </div>
                </div>
            </div>
        </div>
    }
}
