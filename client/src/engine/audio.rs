use crate::engine::terrain::Zone;
use std::cell::RefCell;
use web_sys::{AudioContext, GainNode, OscillatorNode, OscillatorType};

thread_local! {
    static CTX: RefCell<Option<AudioContext>> = const { RefCell::new(None) };
    static MASTER: RefCell<Option<GainNode>> = const { RefCell::new(None) };
    static AMBIENT: RefCell<Option<(OscillatorNode, GainNode)>> = const { RefCell::new(None) };
    static TICK_TIMER: RefCell<f64> = const { RefCell::new(0.0) };
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

fn play_nature_sound(zone: Zone) {
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

pub fn update(zone: Zone, _formula_seed: u32, walking: bool, _speed: f64) {
    set_ambient_freq(zone_ambient(zone).0, zone_ambient(zone).1);

    TICK_TIMER.with(|t| {
        let mut timer = *t.borrow();
        timer += 1.0 / 60.0;
        if timer > 2.0 {
            timer = 0.0;
            play_nature_sound(zone);
        }
        *t.borrow_mut() = timer;
    });

    if walking {
        CTX.with(|ctx_cell| {
            let ctx_binding = ctx_cell.borrow();
            let ctx = match *ctx_binding { Some(ref c) => c, None => return };
            let now = ctx.current_time();
            let beat = (now * 5.0).fract();
            if beat < 0.05 {
                play_tone(600.0, 0.03);
            }
        });
    }
}

pub fn play_effect(_effect: &str) {}

pub fn stop() {
    AMBIENT.with(|a| {
        if let Some((osc, gain)) = a.borrow_mut().take() {
            gain.gain().set_value(0.0);
            let _ = osc.stop();
        }
    });
}
