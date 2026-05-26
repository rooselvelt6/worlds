use crate::engine::terrain::Zone;
use std::cell::RefCell;
use web_sys::{AudioContext, GainNode, OscillatorNode, OscillatorType};

thread_local! {
    static CTX: RefCell<Option<AudioContext>> = const { RefCell::new(None) };
    static MASTER: RefCell<Option<GainNode>> = const { RefCell::new(None) };
    static AMBIENT: RefCell<Option<(OscillatorNode, GainNode)>> = const { RefCell::new(None) };
    static WEATHER_FADE: RefCell<f64> = const { RefCell::new(0.0) };
    static TICK_TIMER: RefCell<f64> = const { RefCell::new(0.0) };
    static FOOTSTEP_TIMER: RefCell<f64> = const { RefCell::new(0.0) };
}

pub fn init() {
    if let Ok(ctx) = AudioContext::new() {
        if let Ok(gain) = ctx.create_gain() {
            gain.gain().set_value(0.12);
            let _ = gain.connect_with_audio_node(&ctx.destination());
            CTX.with(|c| *c.borrow_mut() = Some(ctx));
            MASTER.with(|m| *m.borrow_mut() = Some(gain));
        }
    }
    start_ambient();
}

fn start_ambient() {
    CTX.with(|ctx_cell| {
        let ctx_binding = ctx_cell.borrow();
        let ctx = match *ctx_binding { Some(ref c) => c, None => return };
        MASTER.with(|master_cell| {
            let master_binding = master_cell.borrow();
            let master = match *master_binding { Some(ref m) => m, None => return };
            if let Ok(osc) = ctx.create_oscillator() {
                osc.frequency().set_value(200.0);
                osc.set_type(OscillatorType::Sine);
                if let Ok(gain) = ctx.create_gain() {
                    gain.gain().set_value(0.04);
                    let _ = gain.connect_with_audio_node(master);
                    let _ = osc.connect_with_audio_node(&gain);
                    let _ = osc.start();
                    AMBIENT.with(|a| *a.borrow_mut() = Some((osc, gain)));
                }
            }
        });
    });
}

fn set_ambient_freq(freq: f32, volume: f32) {
    AMBIENT.with(|a| {
        if let Some((ref osc, ref gain)) = *a.borrow() {
            osc.frequency().set_value(freq);
            gain.gain().linear_ramp_to_value_at_time(volume, CTX.with(|c| {
                c.borrow().as_ref().map(|ctx| ctx.current_time() + 0.5).unwrap_or(0.0)
            })).ok();
        }
    });
}

fn zone_ambient(zone: Zone) -> (f32, f32) {
    match zone {
        Zone::Forest | Zone::Jungle => (260.0, 0.06),
        Zone::Plains => (220.0, 0.04),
        Zone::Desert => (180.0, 0.03),
        Zone::Tundra => (140.0, 0.05),
        Zone::Ocean | Zone::CoralReef | Zone::KelpForest | Zone::RockyReef | Zone::SandyPlain | Zone::DeepOcean => (160.0, 0.05),
        Zone::Volcanic | Zone::Lava | Zone::Magma => (90.0, 0.07),
        Zone::Crystal | Zone::Aurora => (500.0, 0.04),
        Zone::Cave | Zone::Abyss => (70.0, 0.04),
        Zone::Fungus => (280.0, 0.05),
        Zone::Storm => (100.0, 0.08),
    }
}

fn play_nature_sound(zone: Zone, weather_power: f64) {
    CTX.with(|ctx_cell| {
        let ctx_binding = ctx_cell.borrow();
        let ctx = match *ctx_binding { Some(ref c) => c, None => return };
        MASTER.with(|master_cell| {
            let master_binding = master_cell.borrow();
            let master = match *master_binding { Some(ref m) => m, None => return };
            let (freq, vol) = match zone {
                Zone::Forest | Zone::Jungle => (800.0 + (rand_hash() % 400) as f32, 0.03),
                Zone::Plains => (500.0 + (rand_hash() % 300) as f32, 0.02),
                Zone::Tundra => (1000.0 + (rand_hash() % 200) as f32, 0.02),
                Zone::Desert => (300.0 + (rand_hash() % 100) as f32, 0.015),
                Zone::Crystal | Zone::Aurora => (1200.0 + (rand_hash() % 600) as f32, 0.025),
                Zone::Fungus => (400.0 + (rand_hash() % 200) as f32, 0.02),
                Zone::Cave | Zone::Abyss => (100.0 + (rand_hash() % 50) as f32, 0.015),
                Zone::Volcanic | Zone::Lava | Zone::Magma => (200.0 + (rand_hash() % 150) as f32, 0.03),
                _ => return,
            };
            let vol = if weather_power > 0.3 { vol * (1.0 - weather_power as f32 * 0.5) } else { vol };
            if let Ok(osc) = ctx.create_oscillator() {
                osc.frequency().set_value(freq);
                osc.set_type(OscillatorType::Triangle);
                if let Ok(gain) = ctx.create_gain() {
                    gain.gain().set_value(vol);
                    let _ = gain.connect_with_audio_node(master);
                    let _ = osc.connect_with_audio_node(&gain);
                    let _ = osc.start();
                    let _ = osc.stop_with_when(ctx.current_time() + 0.3 + (rand_hash() % 100) as f64 * 0.01);
                }
            }
        });
    });
}

fn rand_hash() -> u32 {
    let t = web_sys::window().and_then(|w| w.performance()).map(|p| p.now() as u32).unwrap_or(0);
    t.wrapping_mul(1103515245).wrapping_add(12345) & 0x7FFF
}

pub fn play_tone(freq: f32, duration: f32) {
    CTX.with(|ctx_cell| {
        let ctx_binding = ctx_cell.borrow();
        let ctx = match *ctx_binding { Some(ref c) => c, None => return };
        MASTER.with(|master_cell| {
            let master_binding = master_cell.borrow();
            let master = match *master_binding { Some(ref m) => m, None => return };
            if let Ok(osc) = ctx.create_oscillator() {
                osc.frequency().set_value(freq);
                if let Ok(gain) = ctx.create_gain() {
                    gain.gain().set_value(0.08);
                    let _ = gain.connect_with_audio_node(master);
                    let _ = osc.connect_with_audio_node(&gain);
                    let _ = osc.start();
                    let _ = osc.stop_with_when(ctx.current_time() + duration as f64);
                }
            }
        });
    });
}

fn footstep_sound(surface_type: u8, weather_power: f64) {
    let (freq, vol, dur) = match surface_type {
        0 => (400.0, 0.04, 0.04),   // dirt
        1 => (600.0, 0.05, 0.03),   // stone
        2 => (300.0, 0.03, 0.05),   // grass
        3 => (800.0, 0.03, 0.04),   // sand
        4 => (200.0, 0.02, 0.06),   // water
        5 => (500.0, 0.06, 0.03),   // snow
        _ => (500.0, 0.04, 0.04),
    };
    let vol = vol * (1.0 - weather_power as f32 * 0.3);
    CTX.with(|ctx_cell| {
        let ctx_binding = ctx_cell.borrow();
        let ctx = match *ctx_binding { Some(ref c) => c, None => return };
        MASTER.with(|master_cell| {
            let master_binding = master_cell.borrow();
            let master = match *master_binding { Some(ref m) => m, None => return };
            if let Ok(osc) = ctx.create_oscillator() {
                osc.frequency().set_value(freq);
                osc.set_type(OscillatorType::Triangle);
                if let Ok(gain) = ctx.create_gain() {
                    gain.gain().set_value(vol);
                    let _ = gain.connect_with_audio_node(master);
                    let _ = osc.connect_with_audio_node(&gain);
                    let _ = osc.start();
                    let _ = osc.stop_with_when(ctx.current_time() + dur as f64);
                }
            }
        });
    });
}

pub fn update(zone: Zone, _formula_seed: u32, walking: bool, _speed: f64, weather_power: f64, surface_type: u8) {
    let (amb_freq, amb_vol) = zone_ambient(zone);
    let weather_muffle = (1.0 - weather_power * 0.3) as f32;
    set_ambient_freq(amb_freq, (amb_vol * weather_muffle).max(0.01));

    // Rain gradation: add noise-like oscillation as weather increases
    WEATHER_FADE.with(|wf| {
        let mut fade = *wf.borrow();
        let target = (weather_power * 0.15).min(0.08);
        fade += (target - fade) * 0.05;
        *wf.borrow_mut() = fade;
        if fade > 0.01 {
            CTX.with(|ctx_cell| {
                let ctx_binding = ctx_cell.borrow();
                if let Some(ref ctx) = *ctx_binding {
                    let t = ctx.current_time();
                    let beat = (t * 12.0 * (1.0 + weather_power * 4.0)).fract();
                    if beat < 0.02 {
                        play_tone(1200.0 + (weather_power * 800.0) as f32, 0.01);
                    }
                }
            });
        }
    });

    TICK_TIMER.with(|t| {
        let mut timer = *t.borrow();
        timer += 1.0 / 60.0;
        if timer > 2.0 {
            timer = 0.0;
            play_nature_sound(zone, weather_power);
        }
        *t.borrow_mut() = timer;
    });

    if walking {
        FOOTSTEP_TIMER.with(|ft| {
            let mut timer = *ft.borrow();
            timer += 1.0 / 60.0;
            let interval = match surface_type {
                4 => 0.35,
                _ => 0.2,
            };
            if timer > interval {
                timer = 0.0;
                footstep_sound(surface_type, weather_power);
            }
            *ft.borrow_mut() = timer;
        });
    }
}

pub fn play_effect(effect: &str) {
    match effect {
        "portal_open" => {
            play_tone(220.0, 0.3);
            play_tone(330.0, 0.3);
            play_tone(440.0, 0.5);
        }
        "portal_hum" => {
            play_tone(110.0, 0.8);
        }
        "thunder" => {
            play_tone(60.0, 0.6);
            play_tone(45.0, 0.4);
        }
        "heal" => {
            play_tone(520.0, 0.15);
            play_tone(660.0, 0.15);
            play_tone(780.0, 0.2);
        }
        "power_up" => {
            play_tone(400.0, 0.1);
            play_tone(600.0, 0.1);
            play_tone(900.0, 0.15);
        }
        "craft" => {
            play_tone(700.0, 0.08);
            play_tone(900.0, 0.1);
        }
        _ => {}
    }
}

pub fn stop() {
    AMBIENT.with(|a| {
        if let Some((osc, gain)) = a.borrow_mut().take() {
            gain.gain().set_value(0.0);
            let _ = osc.stop();
        }
    });
}
