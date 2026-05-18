use crate::engine::bridge;
use crate::engine::terrain::Zone;

static mut PREV_ZONE: Option<Zone> = None;
static mut PREV_WEATHER: Option<Weather> = None;
static mut PREV_FORMULA: Option<u32> = None;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Weather {
    Clear,
    Rain,
    Snow,
    Storm,
    Dust,
    Ash,
}

pub fn init() {
    bridge::audio_init();
}

pub fn update(zone: Zone, formula_seed: u32, walking: bool, speed: f64) {
    let prev = unsafe { PREV_ZONE };

    // Zone changed -> play effect + switch ambient
    if prev.map(|z| z != zone).unwrap_or(true) {
        unsafe { PREV_ZONE = Some(zone) };
        bridge::audio_play_ambient(zone.as_str());
        bridge::audio_play_effect("zone");

        // Update weather based on zone
        let weather = weather_for_zone(zone);
        if unsafe { PREV_WEATHER.map(|w| w != weather).unwrap_or(true) } {
            unsafe { PREV_WEATHER = Some(weather) };
            match weather {
                Weather::Clear => bridge::clear_weather(),
                _ => bridge::set_weather(weather.name(), weather_intensity(zone)),
            }
        }
    }

    // Formula changed
    let f_seed = formula_seed;
    if unsafe { PREV_FORMULA.map(|p| p != f_seed).unwrap_or(true) } {
        unsafe { PREV_FORMULA = Some(f_seed) };
        bridge::audio_play_effect("formula");
    }

    // Footsteps
    if walking && speed > 0.5 {
        bridge::audio_play_footstep((speed / 30.0).min(1.0) as f32);
    }
}

pub fn stop() {
    bridge::audio_stop_ambient();
    bridge::clear_weather();
    unsafe {
        PREV_ZONE = None;
        PREV_WEATHER = None;
        PREV_FORMULA = None;
    }
}

fn weather_for_zone(zone: Zone) -> Weather {
    match zone {
        Zone::Forest | Zone::Jungle => Weather::Rain,
        Zone::Tundra => Weather::Snow,
        Zone::Storm => Weather::Storm,
        Zone::Desert => Weather::Dust,
        Zone::Volcanic | Zone::Lava | Zone::Magma => Weather::Ash,
        Zone::Ocean => Weather::Rain,
        _ => Weather::Clear,
    }
}

fn weather_intensity(zone: Zone) -> f32 {
    match zone {
        Zone::Storm => 1.0,
        Zone::Tundra => 0.7,
        Zone::Ocean => 0.6,
        Zone::Forest | Zone::Jungle => 0.4,
        Zone::Desert => 0.5,
        Zone::Volcanic | Zone::Lava | Zone::Magma => 0.6,
        _ => 0.3,
    }
}

impl Weather {
    pub fn name(&self) -> &'static str {
        match self {
            Weather::Clear => "none",
            Weather::Rain => "rain",
            Weather::Snow => "snow",
            Weather::Storm => "storm",
            Weather::Dust => "dust",
            Weather::Ash => "ash",
        }
    }
}
