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

#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();
    let canvas_ref: NodeRef<html::Canvas> = NodeRef::new();
    let engine: Rc<RefCell<Option<Engine>>> = Rc::new(RefCell::new(None));
    let hud = RwSignal::new(HudData::default());
    let settings_open = RwSignal::new(false);

    {
        let canvas_ref = canvas_ref.clone();
        let engine = engine.clone();
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

    let zones = [
        (Zone::Forest, "\u{1f332}", "Forest"),
        (Zone::Plains, "\u{1f33e}", "Plains"),
        (Zone::Desert, "\u{1f3dc}", "Desert"),
        (Zone::Tundra, "\u{2744}", "Tundra"),
        (Zone::Jungle, "\u{1f334}", "Jungle"),
        (Zone::Volcanic, "\u{1f30b}", "Volcanic"),
        (Zone::Ocean, "\u{1f30a}", "Ocean"),
        (Zone::Crystal, "\u{1f48e}", "Crystal"),
        (Zone::Cave, "\u{1f573}", "Cave"),
        (Zone::Lava, "\u{1f525}", "Lava"),
    ];

    let formulas = FormulaType::all();

    view! {
        <div class="w-screen h-screen overflow-hidden relative select-none antialiased"
            style="font-family: 'Inter', 'Orbitron', system-ui, sans-serif; background: #0a0a12;">

            <canvas node_ref=canvas_ref
                class="absolute inset-0 w-full h-full outline-none touch-none"
                tabindex="0"
            />

            // ===== TOP-LEFT HUD =====
            <div class="absolute top-3 left-3 z-30 flex flex-col gap-1.5">
                <div class="px-2.5 py-1.5 rounded-lg bg-black/60 backdrop-blur-md border-l-2 border-cyan-400 shadow-lg shadow-cyan-500/5">
                    <div class="flex items-center gap-2 text-[10px] font-mono">
                        <span class="text-cyan-300 font-bold tabular-nums tracking-wider">
                            {move || format!("{:04.1}", hud.get().speed)}
                        </span>
                        <span class="text-white/30 text-[8px]">SPD</span>
                    </div>
                    <div class="mt-0.5" style="width: 56px;">
                        <div class="h-0.5 rounded-full bg-white/10 overflow-hidden">
                            <div class="h-full rounded-full bg-gradient-to-r from-cyan-400 to-purple-400 transition-all duration-150"
                                style:width={move || format!("{}%", (hud.get().speed / 50.0 * 100.0).min(100.0))}>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="px-2.5 py-1.5 rounded-lg bg-black/60 backdrop-blur-md border-l-2 border-emerald-400 shadow-lg shadow-emerald-500/5">
                    <div class="flex items-center gap-2 text-[10px] font-mono">
                        <span class="text-emerald-300 font-bold tabular-nums tracking-wider">
                            {move || format!("{:03.0}", hud.get().pos[1])}
                        </span>
                        <span class="text-white/30 text-[8px]">ALT</span>
                    </div>
                </div>
            </div>

            // ===== TOP-RIGHT HUD =====
            <div class="absolute top-3 right-3 z-30 flex flex-col gap-1.5 items-end">
                <div class="px-2.5 py-1.5 rounded-lg bg-black/60 backdrop-blur-md border-r-2 border-amber-400 shadow-lg shadow-amber-500/5">
                    <div class="flex items-center gap-2 text-[10px] font-mono">
                        <span class="text-white/30 text-[8px]">YAW</span>
                        <span class="text-amber-300 font-bold tabular-nums tracking-wider">
                            {move || format!("{:03}\u{b0}", hud.get().yaw_deg)}
                        </span>
                    </div>
                </div>
                <div class="px-2.5 py-1.5 rounded-lg bg-black/60 backdrop-blur-md border-r-2 border-purple-400 shadow-lg shadow-purple-500/5">
                    <div class="flex items-center gap-2 text-[10px] font-mono">
                        <span class="text-purple-300 font-bold">{move || hud.get().biome}</span>
                    </div>
                </div>
                <div class="px-2.5 py-1 rounded-lg bg-black/40 backdrop-blur-md text-[9px] font-mono">
                    <span class="text-white/20">{move || {
                        let h = hud.get();
                        format!("{:03}fps {:02}ch", h.fps, h.chunks)
                    }}</span>
                </div>
            </div>

            // ===== COMPASS BAR (yaw indicator at top center) =====
            <div class="absolute top-2 left-1/2 -translate-x-1/2 z-30 px-3 py-1 rounded-full bg-black/40 backdrop-blur-md border border-white/5">
                <div class="flex gap-6 text-[8px] font-mono text-white/20">
                    <span>N</span>
                    <span class="text-white/10">|</span>
                    <span>E</span>
                    <span class="text-white/10">|</span>
                    <span>S</span>
                    <span class="text-white/10">|</span>
                    <span>W</span>
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

            // ===== MODE INDICATOR (bottom center) =====
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
                    {move || if hud.get().fly_mode { "\u{2b06} FLY" } else { "\u{1f6b6} WALK" }}
                </div>
            </div>

            // ===== SETTINGS PANEL =====
            {move || settings_open.get().then(|| view! {
                <div class="absolute bottom-28 left-1/2 -translate-x-1/2 z-40 w-full max-w-lg px-2 pointer-events-none">
                    <div class="pointer-events-auto bg-black/80 backdrop-blur-xl rounded-2xl border border-white/[0.04] p-3 space-y-2 overflow-y-auto max-h-[45vh] shadow-2xl">

                        // Formula params
                        <div class="flex items-center gap-2 mb-0.5">
                            <span class="text-[9px] font-mono font-bold tracking-widest text-cyan-400 uppercase">Formula</span>
                            <div class="flex-1 h-px bg-white/5"></div>
                        </div>

                        <div class="flex items-center gap-2">
                            <span class="text-[9px] font-mono text-white/50 w-14 shrink-0">Scale</span>
                            <input type="range" min="0.005" max="0.1" step="0.001"
                                prop:value=move || format!("{}", state.params.get().scale)
                                on:input=move |ev| { let i: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into(); state.params.update(|p| p.scale = i.value_as_number()); }
                                class="flex-1 h-1 accent-cyan-400 rounded-full bg-white/10 cursor-pointer"
                            />
                            <span class="text-[9px] font-mono text-white/70 w-12 text-right tabular-nums">{move || format!("{:.3}", state.params.get().scale)}</span>
                        </div>

                        <div class="flex items-center gap-2">
                            <span class="text-[9px] font-mono text-white/50 w-14 shrink-0">Octaves</span>
                            <input type="range" min="1" max="8" step="1"
                                prop:value=move || format!("{}", state.params.get().octaves)
                                on:input=move |ev| { let i: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into(); state.params.update(|p| p.octaves = i.value_as_number() as u32); }
                                class="flex-1 h-1 accent-cyan-400 rounded-full bg-white/10 cursor-pointer"
                            />
                            <span class="text-[9px] font-mono text-white/70 w-12 text-right tabular-nums">{move || format!("{}", state.params.get().octaves)}</span>
                        </div>

                        <div class="flex items-center gap-2">
                            <span class="text-[9px] font-mono text-white/50 w-14 shrink-0">Amp</span>
                            <input type="range" min="0.2" max="5.0" step="0.1"
                                prop:value=move || format!("{}", state.params.get().amplitude)
                                on:input=move |ev| { let i: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into(); state.params.update(|p| p.amplitude = i.value_as_number()); }
                                class="flex-1 h-1 accent-cyan-400 rounded-full bg-white/10 cursor-pointer"
                            />
                            <span class="text-[9px] font-mono text-white/70 w-12 text-right tabular-nums">{move || format!("{:.1}", state.params.get().amplitude)}</span>
                        </div>

                        <div class="flex items-center gap-2">
                            <span class="text-[9px] font-mono text-white/50 w-14 shrink-0">Water</span>
                            <input type="range" min="0.0" max="3.0" step="0.1"
                                prop:value=move || format!("{}", state.params.get().water_level)
                                on:input=move |ev| { let i: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into(); state.params.update(|p| p.water_level = i.value_as_number()); }
                                class="flex-1 h-1 accent-cyan-400 rounded-full bg-white/10 cursor-pointer"
                            />
                            <span class="text-[9px] font-mono text-white/70 w-12 text-right tabular-nums">{move || format!("{:.1}", state.params.get().water_level)}</span>
                        </div>

                        // Color controls
                        <div class="flex items-center gap-2 mb-0.5 mt-1">
                            <span class="text-[9px] font-mono font-bold tracking-widest text-purple-400 uppercase">Color</span>
                            <div class="flex-1 h-px bg-white/5"></div>
                        </div>

                        <div class="flex items-center gap-2">
                            <span class="text-[9px] font-mono text-white/50 w-14 shrink-0">Hue</span>
                            <input type="range" min="0" max="360" step="1"
                                prop:value=move || format!("{}", state.params.get().hue_shift)
                                on:input=move |ev| { let i: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into(); state.params.update(|p| p.hue_shift = i.value_as_number()); }
                                class="flex-1 h-1 accent-purple-400 rounded-full bg-white/10 cursor-pointer"
                            />
                            <span class="text-[9px] font-mono text-white/70 w-12 text-right tabular-nums">{move || format!("{:03.0}\u{b0}", state.params.get().hue_shift)}</span>
                        </div>

                        <div class="flex items-center gap-2">
                            <span class="text-[9px] font-mono text-white/50 w-14 shrink-0">Sat</span>
                            <input type="range" min="0.0" max="2.0" step="0.01"
                                prop:value=move || format!("{}", state.params.get().saturation)
                                on:input=move |ev| { let i: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into(); state.params.update(|p| p.saturation = i.value_as_number()); }
                                class="flex-1 h-1 accent-purple-400 rounded-full bg-white/10 cursor-pointer"
                            />
                            <span class="text-[9px] font-mono text-white/70 w-12 text-right tabular-nums">{move || format!("{:.2}", state.params.get().saturation)}</span>
                        </div>

                        <div class="flex items-center gap-2">
                            <span class="text-[9px] font-mono text-white/50 w-14 shrink-0">Light</span>
                            <input type="range" min="0.0" max="2.0" step="0.01"
                                prop:value=move || format!("{}", state.params.get().lightness)
                                on:input=move |ev| { let i: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into(); state.params.update(|p| p.lightness = i.value_as_number()); }
                                class="flex-1 h-1 accent-purple-400 rounded-full bg-white/10 cursor-pointer"
                            />
                            <span class="text-[9px] font-mono text-white/70 w-12 text-right tabular-nums">{move || format!("{:.2}", state.params.get().lightness)}</span>
                        </div>

                    </div>
                </div>
            })}

            // ===== BOTTOM CONTROLS BAR =====
            <div class="absolute bottom-0 left-0 right-0 z-30 pb-2 sm:pb-3 px-2 pointer-events-none">
                <div class="max-w-2xl mx-auto pointer-events-auto space-y-1.5">

                    // Zones row (compact, Fortnite-style)
                    <div class="flex justify-center gap-0.5 sm:gap-1 bg-black/40 backdrop-blur-lg rounded-2xl px-2 py-1.5 border border-white/[0.03] mx-auto w-fit">
                        {zones.iter().map(|&(zone, emoji, name)| {
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
                                >{emoji}</button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>

                    // Formulas row (compact)
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
                            <span class="hidden sm:inline">New World</span>
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
                            <span class="ml-1 hidden sm:inline">{move || if state.params.get().fly_mode { "Fly" } else { "Walk" }}</span>
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
                            <span class="ml-1 hidden sm:inline">{move || if state.params.get().speed > 25.0 { "Sprint" } else { "Walk" }}</span>
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

                    // Controls hint (very subtle)
                    <div class="text-center text-[6px] text-white/10 font-mono tracking-widest uppercase">
                        Click | Wasd | Qe | Space | Shift
                    </div>
                </div>
            </div>
        </div>
    }
}
