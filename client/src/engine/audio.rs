use crate::engine::terrain;
use crate::engine::terrain::Zone;
use crate::state::WorldParams;
use std::cell::RefCell;
use js_sys::Float32Array;
use web_sys::{
    AudioContext, GainNode, OscillatorNode, OscillatorType,
    PannerNode, ConvolverNode, DistanceModelType,
};

thread_local! {
    static CTX: RefCell<Option<AudioContext>> = const { RefCell::new(None) };
    static MASTER: RefCell<Option<GainNode>> = const { RefCell::new(None) };
    static AMBIENT: RefCell<Option<(OscillatorNode, GainNode)>> = const { RefCell::new(None) };
    static WEATHER_FADE: RefCell<f64> = const { RefCell::new(0.0) };
    static WIND_FADE: RefCell<f64> = const { RefCell::new(0.0) };
    static WIND_PITCH: RefCell<f64> = const { RefCell::new(1.0) };
    static TICK_TIMER: RefCell<f64> = const { RefCell::new(0.0) };
    static FAUNA_TIMER: RefCell<f64> = const { RefCell::new(0.0) };
    static FOOTSTEP_TIMER: RefCell<f64> = const { RefCell::new(0.0) };
    static REVERB: RefCell<Option<(ConvolverNode, GainNode)>> = const { RefCell::new(None) };
    static CURRENT_REVERB_ZONE: RefCell<Option<Zone>> = const { RefCell::new(None) };
    static SPATIAL: RefCell<Vec<(PannerNode, GainNode, f64)>> = const { RefCell::new(Vec::new()) };
    static MUSIC_BASS: RefCell<Option<(OscillatorNode, GainNode)>> = const { RefCell::new(None) };
    static MUSIC_PAD: RefCell<Option<(OscillatorNode, GainNode)>> = const { RefCell::new(None) };
    static PREV_ZONE: RefCell<Zone> = const { RefCell::new(Zone::Plains) };
    static MUSIC_FADE_TIMER: RefCell<f64> = const { RefCell::new(0.0) };
    static WF_SOUND_IDX: RefCell<usize> = const { RefCell::new(0) };
    static WF_SOUND_TIMER: RefCell<f64> = const { RefCell::new(0.0) };
    static WATER_MURMUR: RefCell<Option<(OscillatorNode, GainNode, OscillatorNode, GainNode)>> = const { RefCell::new(None) };
    static WIND_NOISE: RefCell<Option<(OscillatorNode, GainNode)>> = const { RefCell::new(None) };
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

fn rand_hash() -> u32 {
    let t = web_sys::window().and_then(|w| w.performance()).map(|p| p.now() as u32).unwrap_or(0);
    t.wrapping_mul(1103515245).wrapping_add(12345) & 0x7FFF
}

fn seeded_rand(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    *seed as f32 / 2147483648.0
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

// ── R8.1: HRTF avanzado ────────────────────────────────────────────

pub fn set_listener_position(x: f32, y: f32, z: f32) {
    CTX.with(|ctx_cell| {
        let ctx_binding = ctx_cell.borrow();
        if let Some(ref ctx) = *ctx_binding {
            ctx.listener().set_position(x as f64, y as f64, z as f64);
        }
    });
}

pub fn set_listener_orientation(yaw: f64, pitch: f64) {
    CTX.with(|ctx_cell| {
        let ctx_binding = ctx_cell.borrow();
        if let Some(ref ctx) = *ctx_binding {
            let (sy, cy) = yaw.sin_cos();
            let (sp, cp) = pitch.sin_cos();
            let fx = sy * cp;
            let fy = sp;
            let fz = cy * cp;
            ctx.listener().set_orientation(fx, fy, fz, 0.0, 1.0, 0.0);
        }
    });
}

fn configure_panner(panner: &PannerNode) {
    let _ = panner.set_distance_model(DistanceModelType::Inverse);
    panner.set_ref_distance(8.0);
    panner.set_max_distance(60.0);
    panner.set_rolloff_factor(1.2);
    panner.set_cone_inner_angle(360.0);
    panner.set_cone_outer_angle(360.0);
    panner.set_cone_outer_gain(0.0);
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
                        configure_panner(&panner);
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

pub fn play_directional_tone(freq: f32, duration: f32, x: f32, y: f32, z: f32,
    inner_angle: f64, outer_angle: f64, outer_gain: f64, volume: f32)
{
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
                    if let Ok(panner) = PannerNode::new(ctx) {
                        configure_panner(&panner);
                        panner.set_position(x as f64, y as f64, z as f64);
                        panner.set_cone_inner_angle(inner_angle);
                        panner.set_cone_outer_angle(outer_angle);
                        panner.set_cone_outer_gain(outer_gain);
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

// ── R8.2: IR por zona ──────────────────────────────────────────────

fn zone_reverb_params(zone: Zone) -> Option<(f64, f64, f64)> {
    match zone {
        Zone::Cave | Zone::Abyss => Some((1.5, 4.0, 0.0)),
        Zone::Forest | Zone::Jungle => Some((0.6, 2.0, 800.0)),
        Zone::Volcanic | Zone::Lava | Zone::Magma => Some((1.0, 3.0, 0.0)),
        Zone::Crystal | Zone::Aurora => Some((0.7, 2.5, 2000.0)),
        Zone::Fungus => Some((0.5, 2.0, 400.0)),
        Zone::Storm => Some((0.3, 1.5, 0.0)),
        _ => None,
    }
}

fn set_zone_reverb(zone: Zone) {
    CURRENT_REVERB_ZONE.with(|current| {
        if *current.borrow() == Some(zone) { return; }
        *current.borrow_mut() = Some(zone);
    });

    let params = match zone_reverb_params(zone) {
        Some(p) => p,
        None => {
            CTX.with(|ctx_cell| {
                let ctx_binding = ctx_cell.borrow();
                let _ctx = match *ctx_binding { Some(ref c) => c, None => return };
                MASTER.with(|master_cell| {
                    let master_binding = master_cell.borrow();
                    let master = match *master_binding { Some(ref m) => m, None => return };
                    REVERB.with(|r| {
                        if let Some((_, ref wet)) = *r.borrow() {
                            let _ = master.disconnect_with_audio_node(wet);
                        }
                        *r.borrow_mut() = None;
                    });
                });
            });
            return;
        }
    };
    let (duration, decay, lowpass_cutoff) = params;

    CTX.with(|ctx_cell| {
        let ctx_binding = ctx_cell.borrow();
        let ctx = match *ctx_binding { Some(ref c) => c, None => return };
        MASTER.with(|master_cell| {
            let master_binding = master_cell.borrow();
            let master = match *master_binding { Some(ref m) => m, None => return };
            REVERB.with(|r| {
                let mut reverb = r.borrow_mut();
                if let Ok(conv) = ConvolverNode::new(ctx) {
                    conv.set_normalize(true);
                    if let Some(buf) = create_impulse_response(ctx, duration, decay, lowpass_cutoff, matches!(zone, Zone::Volcanic | Zone::Lava | Zone::Magma)) {
                        conv.set_buffer(Some(&buf));
                        if let Ok(wet) = ctx.create_gain() {
                            let wet_amount = match zone {
                                Zone::Cave | Zone::Abyss => 0.35,
                                Zone::Forest | Zone::Jungle => 0.25,
                                Zone::Volcanic | Zone::Lava | Zone::Magma => 0.3,
                                Zone::Crystal | Zone::Aurora => 0.2,
                                Zone::Fungus => 0.2,
                                Zone::Storm => 0.15,
                                _ => 0.0,
                            };
                            wet.gain().set_value(wet_amount);
                            let _ = wet.connect_with_audio_node(&conv);
                            let _ = conv.connect_with_audio_node(&ctx.destination());
                            let _ = master.connect_with_audio_node(&wet);
                            *reverb = Some((conv, wet));
                        }
                    }
                }
            });
        });
    });
}

fn create_impulse_response(ctx: &AudioContext, duration: f64, decay: f64, lowpass_cutoff: f64, metallic: bool) -> Option<web_sys::AudioBuffer> {
    let sample_rate = ctx.sample_rate() as u32;
    let length = (sample_rate as f64 * duration) as u32;
    if length == 0 { return None; }
    let buf = ctx.create_buffer(2, length, sample_rate as f32).ok()?;
    let mut seed = 12345u32;
    for ch in 0..2i32 {
        let mut samples = vec![0.0f32; length as usize];
        for i in 0..length as usize {
            let t = i as f64 / sample_rate as f64;
            let noise = seeded_rand(&mut seed) as f64 * 2.0 - 1.0;
            let mut sample = noise * (-t * decay).exp();
            if lowpass_cutoff > 0.0 {
                let rc = 1.0 / (lowpass_cutoff * 2.0 * std::f64::consts::PI);
                let dt = 1.0 / sample_rate as f64;
                let alpha = dt / (rc + dt);
                if i > 0 {
                    let prev = samples[i - 1] as f64;
                    sample = prev + alpha * (sample - prev);
                }
            }
            if metallic {
                let ringing = (t * 2000.0 * std::f64::consts::PI * 2.0).sin() * (-t * 10.0).exp();
                sample = sample * 0.5 + ringing * 0.5;
            }
            samples[i] = (sample * 0.7) as f32;
        }
        let arr = unsafe { Float32Array::view(&samples) };
        let _ = buf.copy_to_channel_with_f32_array(&arr, ch);
    }
    Some(buf)
}

// ── R8.5: Water presence ───────────────────────────────────────────

fn start_water_murmur(params: &WorldParams, player_x: f64, player_z: f64) {
    let river_dist = nearest_river_distance(params, player_x, player_z);
    let is_near_river = river_dist < 30.0;

    WATER_MURMUR.with(|wm| {
        let has_murmur = wm.borrow().is_some();
        if is_near_river && !has_murmur {
            CTX.with(|ctx_cell| {
                let ctx_binding = ctx_cell.borrow();
                let ctx = match *ctx_binding { Some(ref c) => c, None => return };
                MASTER.with(|master_cell| {
                    let master_binding = master_cell.borrow();
                    let master = match *master_binding { Some(ref m) => m, None => return };
                    if let Ok(osc1) = ctx.create_oscillator() {
                        osc1.set_type(OscillatorType::Sine);
                        osc1.frequency().set_value((60.0 + seeded_rand(&mut 42) as f64 * 20.0) as f32);
                        if let Ok(gain1) = ctx.create_gain() {
                            gain1.gain().set_value(0.0);
                            let _ = gain1.connect_with_audio_node(master);
                            let _ = osc1.connect_with_audio_node(&gain1);
                            let _ = osc1.start();
                            if let Ok(osc2) = ctx.create_oscillator() {
                                osc2.set_type(OscillatorType::Sine);
                                osc2.frequency().set_value((90.0 + seeded_rand(&mut 99) as f64 * 30.0) as f32);
                                if let Ok(gain2) = ctx.create_gain() {
                                    gain2.gain().set_value(0.0);
                                    let _ = gain2.connect_with_audio_node(master);
                                    let _ = osc2.connect_with_audio_node(&gain2);
                                    let _ = osc2.start();
                                    *wm.borrow_mut() = Some((osc1, gain1, osc2, gain2));
                                }
                            }
                        }
                    }
                });
            });
        } else if !is_near_river && has_murmur {
            if let Some((osc1, gain1, osc2, gain2)) = wm.borrow_mut().take() {
                gain1.gain().linear_ramp_to_value_at_time(0.0, CTX.with(|c| {
                    c.borrow().as_ref().map(|ctx| ctx.current_time() + 0.5).unwrap_or(0.0)
                })).ok();
                gain2.gain().linear_ramp_to_value_at_time(0.0, CTX.with(|c| {
                    c.borrow().as_ref().map(|ctx| ctx.current_time() + 0.5).unwrap_or(0.0)
                })).ok();
                let _ = osc1.stop_with_when(CTX.with(|c| {
                    c.borrow().as_ref().map(|ctx| ctx.current_time() + 0.5).unwrap_or(0.0)
                }));
                let _ = osc2.stop_with_when(CTX.with(|c| {
                    c.borrow().as_ref().map(|ctx| ctx.current_time() + 0.5).unwrap_or(0.0)
                }));
            }
        }

        if is_near_river {
            if let Some((ref osc1, ref gain1, ref osc2, ref gain2)) = *wm.borrow() {
                let intensity = ((1.0 - (river_dist / 30.0).min(1.0)) * 0.015) as f32;
                let now = CTX.with(|c| c.borrow().as_ref().map(|ctx| ctx.current_time() + 0.3).unwrap_or(0.0));
                gain1.gain().linear_ramp_to_value_at_time(intensity, now).ok();
                gain2.gain().linear_ramp_to_value_at_time(intensity * 0.6, now).ok();
                let wobble = (now * 0.7).sin() as f32 * 10.0;
                osc1.frequency().linear_ramp_to_value_at_time(60.0f32 + wobble, now).ok();
                osc2.frequency().linear_ramp_to_value_at_time(90.0f32 + wobble * 0.5, now).ok();
            }
        }
    });
}

fn nearest_river_distance(params: &WorldParams, wx: f64, wz: f64) -> f64 {
    let sample_step = 4.0;
    let search_radius = 24.0;
    let mut min_dist = search_radius;
    let mut rx = (wx - search_radius / sample_step).floor() * sample_step;
    while rx <= wx + search_radius {
        let mut rz = (wz - search_radius / sample_step).floor() * sample_step;
        while rz <= wz + search_radius {
            if terrain::is_river(params, rx, rz) {
                let dx = rx - wx;
                let dz = rz - wz;
                let dist = (dx * dx + dz * dz).sqrt();
                if dist < min_dist { min_dist = dist; }
            }
            rz += sample_step;
        }
        rx += sample_step;
    }
    min_dist
}

// ── R8.3: Wind audio mejorado ──────────────────────────────────────

fn start_wind_noise() {
    WIND_NOISE.with(|wn| {
        if wn.borrow().is_some() { return; }
        CTX.with(|ctx_cell| {
            let ctx_binding = ctx_cell.borrow();
            let ctx = match *ctx_binding { Some(ref c) => c, None => return };
            MASTER.with(|master_cell| {
                let master_binding = master_cell.borrow();
                let master = match *master_binding { Some(ref m) => m, None => return };
                if let Ok(osc) = ctx.create_oscillator() {
                    osc.set_type(OscillatorType::Sawtooth);
                    osc.frequency().set_value(80.0);
                    if let Ok(gain) = ctx.create_gain() {
                        gain.gain().set_value(0.0);
                        let _ = gain.connect_with_audio_node(master);
                        let _ = osc.connect_with_audio_node(&gain);
                        let _ = osc.start();
                        *wn.borrow_mut() = Some((osc, gain));
                    }
                }
            });
        });
    });
}

fn update_wind(weather_power: f64, player_y: f32, _wind_dir: f64, wind_strength: f64) {
    start_wind_noise();

    let height_factor = ((player_y as f64 - 10.0) / 90.0).clamp(0.0, 1.0);
    let target_fade = (weather_power * 0.15 + wind_strength * 0.05 + height_factor * 0.03).min(0.08);
    let target_pitch = 60.0 + weather_power * 40.0 + height_factor * 50.0;

    WIND_FADE.with(|wf| {
        let mut fade = *wf.borrow();
        fade += (target_fade - fade) * 0.05;
        *wf.borrow_mut() = fade;
    });

    WIND_PITCH.with(|wp| {
        let mut pitch = *wp.borrow();
        pitch += (target_pitch - pitch) * 0.05;
        *wp.borrow_mut() = pitch;
    });

    WIND_NOISE.with(|wn| {
        if let Some((ref osc, ref gain)) = *wn.borrow() {
            let wind_vol = WIND_FADE.with(|f| *f.borrow()) as f32;
            gain.gain().linear_ramp_to_value_at_time(wind_vol, CTX.with(|c| {
                c.borrow().as_ref().map(|ctx| ctx.current_time() + 0.2).unwrap_or(0.0)
            })).ok();
            let now = CTX.with(|c| c.borrow().as_ref().map(|ctx| ctx.current_time() + 0.3).unwrap_or(0.0));
            let pitch = WIND_PITCH.with(|p| *p.borrow());
            osc.frequency().linear_ramp_to_value_at_time((pitch + (now * 2.0).sin() * 5.0) as f32, now).ok();
            osc.detune().linear_ramp_to_value_at_time(((now * 1.3).sin() * 20.0) as f32, now).ok();
        }
    });

    WEATHER_FADE.with(|wf| {
        let mut fade = *wf.borrow();
        let old_target = (weather_power * 0.15).min(0.08);
        fade += (old_target - fade) * 0.05;
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
}

// ── R8.4: Fauna sounds mejorados ───────────────────────────────────

fn play_fauna_sound(zone: Zone, weather_power: f64, time_of_day: f32, player_x: f64, player_y: f64, player_z: f64) {
    let day_factor = (time_of_day * 0.5).sin().max(0.0);
    let night_factor = 1.0 - day_factor;
    let dawn_dusk = (time_of_day * 0.5).sin().abs();

    let sounds = match zone {
        Zone::Forest | Zone::Jungle => {
            if dawn_dusk > 0.8 {
                vec![
                    (1100.0 + (rand_hash() % 600) as f32, 0.035, 0.2, OscillatorType::Sine),
                    (1500.0 + (rand_hash() % 400) as f32, 0.03, 0.15, OscillatorType::Sine),
                    (800.0 + (rand_hash() % 800) as f32, 0.025, 0.15, OscillatorType::Sine),
                ]
            } else if day_factor > 0.5 {
                vec![
                    (1200.0 + (rand_hash() % 1200) as f32, 0.025, 0.15, OscillatorType::Sine),
                    (1800.0 + (rand_hash() % 800) as f32, 0.02, 0.12, OscillatorType::Sine),
                    (900.0 + (rand_hash() % 600) as f32, 0.03, 0.2, OscillatorType::Triangle),
                ]
            } else if night_factor > 0.5 {
                vec![
                    (600.0 + (rand_hash() % 400) as f32, 0.02, 0.25, OscillatorType::Sawtooth),
                    (400.0 + (rand_hash() % 200) as f32, 0.015, 0.3, OscillatorType::Triangle),
                ]
            } else {
                vec![
                    (800.0 + (rand_hash() % 800) as f32, 0.025, 0.15, OscillatorType::Sine),
                ]
            }
        }
        Zone::Plains => {
            if day_factor > 0.5 {
                vec![
                    (500.0 + (rand_hash() % 300) as f32, 0.02, 0.15, OscillatorType::Sine),
                    (700.0 + (rand_hash() % 200) as f32, 0.015, 0.1, OscillatorType::Triangle),
                ]
            } else {
                vec![(300.0 + (rand_hash() % 200) as f32, 0.015, 0.2, OscillatorType::Triangle)]
            }
        }
        Zone::Tundra => {
            vec![
                (1000.0 + (rand_hash() % 200) as f32, 0.015, 0.2, OscillatorType::Sine),
                (600.0 + (rand_hash() % 100) as f32, 0.01, 0.3, OscillatorType::Triangle),
            ]
        }
        Zone::Desert => {
            if night_factor > 0.5 {
                vec![(300.0 + (rand_hash() % 100) as f32, 0.015, 0.15, OscillatorType::Sawtooth)]
            } else {
                vec![]
            }
        }
        Zone::Crystal | Zone::Aurora => {
            vec![
                (1200.0 + (rand_hash() % 600) as f32, 0.02, 0.2, OscillatorType::Sine),
                (2000.0 + (rand_hash() % 1000) as f32, 0.015, 0.15, OscillatorType::Sine),
            ]
        }
        Zone::Fungus => {
            vec![
                (400.0 + (rand_hash() % 200) as f32, 0.02, 0.2, OscillatorType::Sawtooth),
                (250.0 + (rand_hash() % 100) as f32, 0.015, 0.3, OscillatorType::Triangle),
            ]
        }
        Zone::Cave | Zone::Abyss => {
            if rand_hash() % 3 == 0 {
                vec![(100.0 + (rand_hash() % 50) as f32, 0.015, 0.4, OscillatorType::Sawtooth)]
            } else { vec![] }
        }
        Zone::Volcanic | Zone::Lava | Zone::Magma => {
            if rand_hash() % 2 == 0 {
                vec![(200.0 + (rand_hash() % 150) as f32, 0.03, 0.2, OscillatorType::Sawtooth)]
            } else { vec![] }
        }
        _ => vec![],
    };

    if sounds.is_empty() { return; }
    let vol = if weather_power > 0.3 { 1.0 - weather_power as f32 * 0.5 } else { 1.0 };

    for (freq, vol_base, dur, _osc_type) in sounds {
        let spawn_dist = 5.0 + (rand_hash() % 10) as f32;
        let angle = (rand_hash() as f32 / 32768.0) * std::f32::consts::TAU;
        let sx = player_x as f32 + angle.cos() * spawn_dist;
        let sz = player_z as f32 + angle.sin() * spawn_dist;
        let sy = player_y as f32 + (rand_hash() % 5) as f32;
        play_directional_tone(freq, dur, sx, sy, sz, 60.0, 120.0, 0.3, vol_base * vol);
    }
}

// ── Music ─────────────────────────────────────────────────────────

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

// ── Main update ────────────────────────────────────────────────────

pub fn update(zone: Zone, _formula_seed: u32, walking: bool, speed: f64,
    weather_power: f64, surface_type: u8, player_y: f32, time_of_day: f32,
    wind_dir: f64, wind_strength: f64, params: &WorldParams,
    player_x: f64, player_z: f64)
{
    cleanup_spatial();

    let (amb_freq, amb_vol) = zone_ambient(zone);
    let weather_muffle = (1.0 - weather_power * 0.3) as f32;
    set_ambient_freq(amb_freq, (amb_vol * weather_muffle).max(0.01));

    // R8.2: Reverb por zona en lugar de solo cueva
    set_zone_reverb(zone);

    update_music(zone, player_y, speed, time_of_day);

    // R8.3: Wind mejorado
    update_wind(weather_power, player_y, wind_dir, wind_strength);

    // R8.4: Fauna sounds
    FAUNA_TIMER.with(|t| {
        let mut timer = *t.borrow();
        timer += 1.0 / 60.0;
        let fauna_interval = match zone {
            Zone::Forest | Zone::Jungle => 1.5,
            Zone::Plains => 2.5,
            Zone::Tundra => 3.0,
            Zone::Cave | Zone::Abyss => 4.0,
            _ => 2.0,
        };
        if timer > fauna_interval {
            timer = 0.0;
            play_fauna_sound(zone, weather_power, time_of_day, player_x, player_y as f64, player_z);
        }
        *t.borrow_mut() = timer;
    });

    // Keep legacy nature sound for zones not covered by fauna
    TICK_TIMER.with(|t| {
        let mut timer = *t.borrow();
        timer += 1.0 / 60.0;
        if timer > 2.0 {
            timer = 0.0;
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

    // R8.5: Water presence
    start_water_murmur(params, player_x, player_z);
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
    WIND_NOISE.with(|wn| {
        if let Some((osc, gain)) = wn.borrow_mut().take() {
            gain.gain().set_value(0.0);
            let _ = osc.stop();
        }
    });
    WATER_MURMUR.with(|wm| {
        if let Some((osc1, gain1, osc2, gain2)) = wm.borrow_mut().take() {
            gain1.gain().set_value(0.0);
            let _ = osc1.stop();
            gain2.gain().set_value(0.0);
            let _ = osc2.stop();
        }
    });
    SPATIAL.with(|s| *s.borrow_mut() = Vec::new());
    REVERB.with(|r| *r.borrow_mut() = None);
    CURRENT_REVERB_ZONE.with(|z| *z.borrow_mut() = None);
}
