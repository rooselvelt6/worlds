use crate::math::*;
use crate::state::WorldParams;
use serde::{Deserialize, Serialize};

pub fn get_height(params: &WorldParams, wx: f64, wz: f64) -> f64 {
    let scale = params.scale;
    let amplitude = params.amplitude;
    let water_level = params.water_level;

    let nx = wx * scale;
    let nz = wz * scale;

    let base = fbm(nx, nz, params.octaves);
    let mut h = (base * amplitude + water_level).max(0.0);

    if params.canyons {
        let canyon = (wx * 0.04).sin() * (wz * 0.04).cos()
            + (wx * 0.06 + wz * 0.08).sin() * 0.5;
        if canyon < -0.2 {
            let depth = (-canyon - 0.2) * 12.0;
            h = (h - depth).max(params.water_level - 6.0);
        }
    }

    match params.zone {
        Zone::Ocean => h += water_level,
        Zone::Volcanic => h = h * 1.5 + 2.0,
        Zone::Crystal => h *= 0.5,
        Zone::Cave => h = 3.0 + (wx * 0.5).sin() * 2.0,
        Zone::Fungus => h = h * 0.6 + 1.0 + (wx * 0.3).sin() * (wz * 0.3).cos() * 1.5,
        Zone::Abyss => h = h * 0.2 + water_level * 0.3,
        Zone::Storm => h = h * 1.8 + 1.0,
        Zone::Aurora => h = h * 0.5 + 0.5,
        Zone::Magma => h = h * 2.0 + 3.0 + (wx * 0.2).sin().abs() * 2.0,
        _ => {}
    }

    h
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Zone {
    Forest, Plains, Desert, Tundra, Jungle,
    Volcanic, Ocean, Crystal, Cave, Lava,
    Fungus, Abyss, Storm, Aurora, Magma,
    CoralReef, KelpForest, SandyPlain, RockyReef, DeepOcean,
}

impl Zone {
    pub fn from_str(s: &str) -> Self {
        match s {
            "plains" => Zone::Plains, "desert" => Zone::Desert,
            "tundra" => Zone::Tundra, "jungle" => Zone::Jungle,
            "volcanic" => Zone::Volcanic, "ocean" => Zone::Ocean,
            "crystal" => Zone::Crystal, "cave" => Zone::Cave,
            "lava" => Zone::Lava, "fungus" => Zone::Fungus,
            "abyss" => Zone::Abyss, "storm" => Zone::Storm,
            "aurora" => Zone::Aurora, "magma" => Zone::Magma,
            "coral_reef" => Zone::CoralReef, "kelp_forest" => Zone::KelpForest,
            "sandy_plain" => Zone::SandyPlain, "rocky_reef" => Zone::RockyReef,
            "deep_ocean" => Zone::DeepOcean,
            _ => Zone::Forest,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Zone::Forest => "forest", Zone::Plains => "plains",
            Zone::Desert => "desert", Zone::Tundra => "tundra",
            Zone::Jungle => "jungle", Zone::Volcanic => "volcanic",
            Zone::Ocean => "ocean", Zone::Crystal => "crystal",
            Zone::Cave => "cave", Zone::Lava => "lava",
            Zone::Fungus => "fungus", Zone::Abyss => "abyss",
            Zone::Storm => "storm", Zone::Aurora => "aurora",
            Zone::Magma => "magma",
            Zone::CoralReef => "coral_reef", Zone::KelpForest => "kelp_forest",
            Zone::SandyPlain => "sandy_plain", Zone::RockyReef => "rocky_reef",
            Zone::DeepOcean => "deep_ocean",
        }
    }
}

pub fn get_zone(params: &WorldParams, wx: f64, wz: f64) -> Zone {
    if params.zone != Zone::Forest {
        return params.zone;
    }
    let h = get_height(params, wx, wz);
    let water = params.water_level;
    if h <= water {
        let depth = water - h;
        let n = fbm(wx * 0.008, wz * 0.008, 3);
        let n2 = fbm(wx * 0.012 + 100.0, wz * 0.012, 2);
        if depth < 0.8 && n > 0.2 && n2 > -0.1 {
            Zone::CoralReef
        } else if depth < 2.0 && n.abs() < 0.25 && n2 < 0.2 {
            Zone::KelpForest
        } else if depth > 4.0 || n < -0.3 {
            Zone::DeepOcean
        } else if n > 0.0 {
            Zone::RockyReef
        } else {
            Zone::SandyPlain
        }
    } else {
        let t = fbm(wx * 0.008, wz * 0.008, 2);
        let h2 = fbm(wx * 0.008 + 50.0, wz * 0.008, 2);
        if t < -0.35 { Zone::Tundra }
        else if t > 0.45 {
            if h2 < -0.25 { Zone::Desert } else { Zone::Volcanic }
        } else if h2 > 0.45 {
            if h2 > 0.6 { Zone::Lava } else { Zone::Jungle }
        } else if h2 < -0.35 { Zone::Plains }
        else if h2 < 0.0 { Zone::Ocean }
        else { Zone::Forest }
    }
}

pub fn get_zone_color(zone: Zone) -> [f32; 3] {
    match zone {
        Zone::Forest => [0.176, 0.353, 0.153],
        Zone::Plains => [0.486, 0.702, 0.259],
        Zone::Desert => [0.831, 0.659, 0.294],
        Zone::Tundra => [0.784, 0.902, 0.941],
        Zone::Jungle => [0.106, 0.302, 0.106],
        Zone::Volcanic => [0.361, 0.251, 0.200],
        Zone::Ocean => [0.118, 0.377, 0.565],
        Zone::Crystal => [0.545, 0.361, 0.965],
        Zone::Cave => [0.290, 0.290, 0.290],
        Zone::Lava => [1.0, 0.267, 0.0],
        Zone::Fungus => [0.600, 0.200, 0.600],
        Zone::Abyss => [0.050, 0.050, 0.100],
        Zone::Storm => [0.300, 0.350, 0.450],
        Zone::Aurora => [0.200, 0.800, 0.600],
        Zone::Magma => [0.800, 0.300, 0.050],
        Zone::CoralReef => [0.800, 0.400, 0.400],
        Zone::KelpForest => [0.200, 0.500, 0.300],
        Zone::SandyPlain => [0.700, 0.600, 0.400],
        Zone::RockyReef => [0.400, 0.350, 0.300],
        Zone::DeepOcean => [0.020, 0.050, 0.150],
    }
}

fn mix_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn gradient(stops: &[[f32; 3]], t: f32) -> [f32; 3] {
    let n = stops.len() - 1;
    let tt = (t * n as f32).clamp(0.0, n as f32 - 0.001);
    let i = tt as usize;
    let frac = tt - i as f32;
    mix_color(stops[i], stops[i + 1], frac)
}

pub fn get_terrain_color(h: f64, max_h: f64) -> [f32; 3] {
    let t = (h / max_h.max(0.1)).clamp(0.0, 1.0) as f32;
    gradient(&[[0.05,0.15,0.30],[0.10,0.35,0.25],[0.25,0.55,0.20],[0.55,0.50,0.25],[0.85,0.80,0.70],[1.0,1.0,1.0]], t)
}

pub fn zone_effects(params: &WorldParams, wx: f64, wz: f64, h: &mut f64) {
    match params.zone {
        Zone::Crystal => {
            let crystal = (wx * 0.8).sin() * (wz * 0.8).cos();
            if crystal.abs() > 0.7 {
                *h += 3.0 + crystal * 2.0;
            }
        }
        Zone::Cave => {
            let cave_n = fbm(wx * 0.04 + params.seed as f64 * 0.01, wz * 0.04, 3);
            let canyon = (wx * 0.08).sin() * (wz * 0.08).cos() + (wx * 0.12 + wz * 0.15).sin() * 0.5;
            if canyon < -0.3 || cave_n < -0.2 {
                let depth = (-canyon).max(0.0) * 3.0 + (-cave_n).max(0.0) * 2.0;
                *h = (*h - depth).max(params.water_level - 4.0);
            }
            let pillar = (wx * 0.2).sin() * (wz * 0.2).cos();
            if pillar > 0.6 && *h > params.water_level {
                *h += (pillar - 0.6) * 3.0;
            }
        }
        Zone::Fungus => {
            let spore = (wx * 1.5).sin() * (wz * 1.5).cos();
            if spore.abs() > 0.6 {
                *h += 2.0 + spore * 1.5;
            }
        }
        Zone::Abyss => {
            let pillar = (wx * 0.3).sin() * (wz * 0.3).cos();
            if pillar.abs() > 0.8 {
                *h += 4.0;
            }
            *h = h.min(4.0);
        }
        Zone::Storm => {
            let spike = (wx * 2.0).sin().abs() * (wz * 2.0).cos().abs();
            *h += spike * 2.0;
        }
        Zone::Aurora => {
            let wave = (wx * 0.5).sin() + (wz * 0.7).cos();
            *h += wave * 0.5;
        }
        Zone::Magma => {
            let fissure = (wx * 0.4).sin() * (wz * 0.4).cos();
            if fissure.abs() > 0.5 {
                *h += 2.0;
            }
        }
        _ => {}
    }
}
