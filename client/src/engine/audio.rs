use crate::engine::terrain::Zone;
use std::cell::RefCell;
use js_sys::Float32Array;
use web_sys::{
    AudioContext, GainNode, OscillatorNode, OscillatorType,
    PannerNode, ConvolverNode,
};

thread_local! {
    static CTX: RefCell<Option<AudioContext>> = const { RefCell::new(None) };
    static MASTER: RefCell<Option<GainNode>> = const { RefCell::new(None) };
    static AMBIENT: RefCell<Option<(OscillatorNode, GainNode)>> = const { RefCell::new(None) };
    static WEATHER_FADE: RefCell<f64> = const { RefCell::new(0.0) };
    static TICK_TIMER: RefCell<f64> = const { RefCell::new(0.0) };
    static FOOTSTEP_TIMER: RefCell<f64> = const { RefCell::new(0.0) };
    static REVERB: RefCell<Option<(ConvolverNode, GainNode)>> = const { RefCell::new(None) };
    static SPATIAL: RefCell<Vec<(PannerNode, GainNode, f64)>> = const { RefCell::new(Vec::new()) };
    static MUSIC_BASS: RefCell<Option<(OscillatorNode, GainNode)>> = const { RefCell::new(None) };
    static MUSIC_PAD: RefCell<Option<(OscillatorNode, GainNode)>> = const { RefCell::new(None) };
    static PREV_ZONE: RefCell<Zone> = const { RefCell::new(Zone::Plains) };
    static MUSIC_FADE_TIMER: RefCell<f64> = const { RefCell::new(0.0) };
    static WF_SOUND_IDX: RefCell<usize> = const { RefCell::new(0) };
    static WF_SOUND_TIMER: RefCell<f64> = const { RefCell::new(0.0) };
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
    start_music();
}

fn zone_root_note(zone: Zone) -> f32 {
    match zone {
        Zone::Forest | Zone::Jungle => 261.63,
        Zone::Plains => 293.66,
        Zone::Desert => 220.00,
        Zone::Tundra => 196.00,
        Zone::Ocean | Zone::CoralReef | Zone::KelpForest | Zone::RockyReef | Zone::SandyPlain | Zone::DeepOcean => 349.23,
        Zone::Volcanic | Zone::Lava | Zone::Magma => 130.81,
        Zone::Crystal | Zone::Aurora => 523.25,
        Zone::Cave | Zone::Abyss => 110.00,
        Zone::Fungus => 392.00,
        Zone::Storm => 155.56,
        Zone::Custom(_) => 261.63,
    }
}

fn start_music() {
    CTX.with(|ctx_cell| {
        let ctx_binding = ctx_cell.borrow();
        let ctx = match *ctx_binding { Some(ref c) => c, None => return };
        MASTER.with(|master_cell| {
            let master_binding = master_cell.borrow();
            let master = match *master_binding { Some(ref m) => m, None => return };
            if let Ok(osc) = ctx.create_oscillator() {
                osc.frequency().set_value(130.81);
                osc.set_type(OscillatorType::Triangle);
                if let Ok(gain) = ctx.create_gain() {
                    gain.gain().set_value(0.0);
                    let _ = gain.connect_with_audio_node(master);
                    let _ = osc.connect_with_audio_node(&gain);
                    let _ = osc.start();
                    MUSIC_BASS.with(|b| *b.borrow_mut() = Some((osc, gain)));
                }
            }
            if let Ok(osc) = ctx.create_oscillator() {
                osc.frequency().set_value(392.0);
                osc.set_type(OscillatorType::Sine);
                if let Ok(gain) = ctx.create_gain() {
                    gain.gain().set_value(0.0);
                    let _ = gain.connect_with_audio_node(master);
                    let _ = osc.connect_with_audio_node(&gain);
                    let _ = osc.start();
                    MUSIC_PAD.with(|p| *p.borrow_mut() = Some((osc, gain)));
                }
            }
        });
    });
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
        Zone::Custom(_) => (200.0, 0.04),
    }
}

fn play_nature_sound(zone: Zone, weather_power: f64) {
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
    play_tone_inner(freq, vol, 0.3 + (rand_hash() % 100) as f32 * 0.01);
}

fn rand_hash() -> u32 {
    let t = web_sys::window().and_then(|w| w.performance()).map(|p| p.now() as u32).unwrap_or(0);
    t.wrapping_mul(1103515245).wrapping_add(12345) & 0x7FFF
}

pub fn play_tone(freq: f32, duration: f32) {
    play_tone_inner(freq, 0.08, duration);
}

fn play_tone_inner(freq: f32, volume: f32, duration: f32) {
    CTX.with(|ctx_cell| {
        let ctx_binding = ctx_cell.borrow();
        let ctx = match *ctx_binding { Some(ref c) => c, None => return };
        MASTER.with(|master_cell| {
            let master_binding = master_cell.borrow();
            let master = match *master_binding { Some(ref m) => m, None => return };
            if let Ok(osc) = ctx.create_oscillator() {
                osc.frequency().set_value(freq);
                if let Ok(gain) = ctx.create_gain() {
                    gain.gain().set_value(volume);
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
        0 => (400.0, 0.04, 0.04),
        1 => (600.0, 0.05, 0.03),
        2 => (300.0, 0.03, 0.05),
        3 => (800.0, 0.03, 0.04),
        4 => (200.0, 0.02, 0.06),
        5 => (500.0, 0.06, 0.03),
        _ => (500.0, 0.04, 0.04),
    };
    let vol = vol * (1.0 - weather_power as f32 * 0.3);
    play_tone_inner(freq, vol, dur);
}

pub fn set_listener_position(x: f32, y: f32, z: f32) {
    CTX.with(|ctx_cell| {
        let ctx_binding = ctx_cell.borrow();
        if let Some(ref ctx) = *ctx_binding {
            ctx.listener().set_position(x as f64, y as f64, z as f64);
        }
    });
}

pub fn play_spatial_tone(freq: f32, duration: f32, x: f32, y: f32, z: f32) {
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
                    if let Ok(panner) = PannerNode::new(ctx) {
                        panner.set_position(x as f64, y as f64, z as f64);
                        let _ = gain.connect_with_audio_node(&panner);
                        let _ = panner.connect_with_audio_node(master);
                        let _ = osc.connect_with_audio_node(&gain);
                        let expiry = ctx.current_time() + duration as f64;
                        let _ = osc.start();
                        let _ = osc.stop_with_when(expiry);
                        SPATIAL.with(|s| s.borrow_mut().push((panner, gain, expiry)));
                    }
                }
            }
        });
    });
}

fn cleanup_spatial() {
    SPATIAL.with(|s| {
        let now = CTX.with(|c| c.borrow().as_ref().map(|ctx| ctx.current_time()).unwrap_or(0.0));
        s.borrow_mut().retain(|(_, _, expiry)| *expiry > now);
    });
}

pub fn set_cave_reverb(active: bool) {
    CTX.with(|ctx_cell| {
        let ctx_binding = ctx_cell.borrow();
        let ctx = match *ctx_binding { Some(ref c) => c, None => return };
        MASTER.with(|master_cell| {
            let master_binding = master_cell.borrow();
            let master = match *master_binding { Some(ref m) => m, None => return };
            REVERB.with(|r| {
                let mut reverb = r.borrow_mut();
                if active && reverb.is_none() {
                    if let Ok(conv) = ConvolverNode::new(ctx) {
                        conv.set_normalize(true);
                        if let Some(buf) = create_cave_impulse(ctx) {
                            conv.set_buffer(Some(&buf));
                            if let Ok(wet) = ctx.create_gain() {
                                wet.gain().set_value(0.4);
                                let _ = wet.connect_with_audio_node(&conv);
                                let _ = conv.connect_with_audio_node(&ctx.destination());
                                let _ = master.connect_with_audio_node(&wet);
                                *reverb = Some((conv, wet));
                            }
                        }
                    }
                } else if !active {
                    if let Some((_, ref wet)) = *reverb {
                        let _ = master.disconnect_with_audio_node(wet);
                    }
                    *reverb = None;
                }
            });
        });
    });
}

fn create_cave_impulse(ctx: &AudioContext) -> Option<web_sys::AudioBuffer> {
    let sample_rate = ctx.sample_rate() as u32;
    let length = (sample_rate as f64 * 1.2) as u32;
    let buf = ctx.create_buffer(2, length, sample_rate as f32).ok()?;
    for ch in 0..2i32 {
        let mut samples = vec![0.0f32; length as usize];
        for i in 0..length as usize {
            let t = i as f64 / sample_rate as f64;
            let decay = (-t * 4.0).exp();
            let noise = (rand_hash() as f64 / 16384.0 - 1.0) * 0.7;
            samples[i] = (noise * decay) as f32;
        }
        let arr = unsafe { Float32Array::view(&samples) };
        let _ = buf.copy_to_channel_with_f32_array(&arr, ch);
    }
    Some(buf)
}

fn update_music(zone: Zone, player_y: f32, speed: f64, time_of_day: f32) {
    PREV_ZONE.with(|pz| {
        let prev = *pz.borrow();
        if prev != zone {
            *pz.borrow_mut() = zone;
            MUSIC_FADE_TIMER.with(|t| *t.borrow_mut() = 0.0);
        }
    });

    let day_phase = (time_of_day * 0.5).sin().max(0.0);
    let root = zone_root_note(zone);
    let height_factor = ((player_y - 10.0) / 90.0).clamp(0.0, 1.0);
    let speed_wobble = (speed as f32 * 0.5).min(0.03);

    let bass_vol = 0.025 * day_phase;
    let pad_vol = 0.018 * (0.2 + 0.8 * height_factor) * (0.5 + 0.5 * day_phase);

    MUSIC_BASS.with(|b| {
        if let Some((ref osc, ref gain)) = *b.borrow() {
            let bass_freq = root * 0.5 + speed_wobble;
            osc.frequency().linear_ramp_to_value_at_time(bass_freq, CTX.with(|c| {
                c.borrow().as_ref().map(|ctx| ctx.current_time() + 0.3).unwrap_or(0.0)
            })).ok();
            gain.gain().linear_ramp_to_value_at_time(bass_vol, CTX.with(|c| {
                c.borrow().as_ref().map(|ctx| ctx.current_time() + 0.5).unwrap_or(0.0)
            })).ok();
        }
    });

    MUSIC_PAD.with(|p| {
        if let Some((ref osc, ref gain)) = *p.borrow() {
            let pad_freq = root * 1.5 + speed_wobble * 2.0;
            osc.frequency().linear_ramp_to_value_at_time(pad_freq, CTX.with(|c| {
                c.borrow().as_ref().map(|ctx| ctx.current_time() + 0.3).unwrap_or(0.0)
            })).ok();
            gain.gain().linear_ramp_to_value_at_time(pad_vol, CTX.with(|c| {
                c.borrow().as_ref().map(|ctx| ctx.current_time() + 0.5).unwrap_or(0.0)
            })).ok();
        }
    });
}

pub fn update(zone: Zone, _formula_seed: u32, walking: bool, speed: f64, weather_power: f64, surface_type: u8, player_y: f32, time_of_day: f32) {
    cleanup_spatial();

    let (amb_freq, amb_vol) = zone_ambient(zone);
    let weather_muffle = (1.0 - weather_power * 0.3) as f32;
    set_ambient_freq(amb_freq, (amb_vol * weather_muffle).max(0.01));

    let is_cave = matches!(zone, Zone::Cave | Zone::Abyss);
    set_cave_reverb(is_cave);

    update_music(zone, player_y, speed, time_of_day);

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

pub fn play_spatial_effect(effect: &str, x: f32, y: f32, z: f32) {
    match effect {
        "portal_open" => {
            play_spatial_tone(220.0, 0.3, x, y, z);
            play_spatial_tone(330.0, 0.3, x, y, z);
            play_spatial_tone(440.0, 0.5, x, y, z);
        }
        "portal_hum" => {
            play_spatial_tone(110.0, 0.8, x, y, z);
        }
        "thunder" => {
            play_spatial_tone(60.0, 0.6, x, y, z);
            play_spatial_tone(45.0, 0.4, x, y, z);
        }
        _ => {
            play_spatial_tone(400.0, 0.2, x, y, z);
        }
    }
}

pub fn update_waterfall_sounds(delta: f64, locations: &[(f64, f64, f64)]) {
    if locations.is_empty() { return; }
    WF_SOUND_TIMER.with(|t| {
        let mut timer = *t.borrow();
        timer += delta;
        if timer > 1.5 {
            timer = 0.0;
            WF_SOUND_IDX.with(|idx| {
                let mut i = *idx.borrow();
                let (x, y, z) = locations[i % locations.len()];
                let freq = 80.0 + (i as f32 * 37.0).fract() * 60.0;
                play_spatial_tone(freq, 1.0, x as f32, y as f32, z as f32);
                i = (i + 1) % locations.len();
                *idx.borrow_mut() = i;
            });
        }
        *t.borrow_mut() = timer;
    });
}

pub fn stop() {
    AMBIENT.with(|a| {
        if let Some((osc, gain)) = a.borrow_mut().take() {
            gain.gain().set_value(0.0);
            let _ = osc.stop();
        }
    });
    MUSIC_BASS.with(|b| {
        if let Some((osc, gain)) = b.borrow_mut().take() {
            gain.gain().set_value(0.0);
            let _ = osc.stop();
        }
    });
    MUSIC_PAD.with(|p| {
        if let Some((osc, gain)) = p.borrow_mut().take() {
            gain.gain().set_value(0.0);
            let _ = osc.stop();
        }
    });
    SPATIAL.with(|s| *s.borrow_mut() = Vec::new());
    REVERB.with(|r| *r.borrow_mut() = None);
}
