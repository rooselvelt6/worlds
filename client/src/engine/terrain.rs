use crate::math::*;
use crate::state::{FormulaType, WorldParams};
use serde::{Deserialize, Serialize};

fn eval_formula(formula: FormulaType, nx: f64, nz: f64, params: &WorldParams) -> f64 {
    let octaves = params.octaves;
    match formula {
        FormulaType::FBM => fbm(nx, nz, octaves),
        FormulaType::Perlin => perlin_noise(nx, nz),
        FormulaType::Simplex => simplex_noise(nx, nz),
        FormulaType::Voronoi => voronoi(nx, nz),
        FormulaType::Mandelbrot => mandelbrot(nx, nz),
        FormulaType::Sierpinski => sierpinski_triangle(nx, nz),
        FormulaType::Julia => juliaset(nx, nz, params.param_a * 2.0 - 0.5),
        FormulaType::Tetrahedron => tetrahedron(nx, nz),
        FormulaType::Cube => cube_fractal(nx, nz),
        FormulaType::Sphere => sphere_field(nx, nz),
        FormulaType::Menger => menger_sponge(nx, nz),
        FormulaType::Vortex => vortex(nx, nz),
        FormulaType::Ice => ice(nx, nz),
        FormulaType::Wave => wave_param(nx, nz, params.param_a * 2.0 + 0.2),
        FormulaType::Spiral => spiral_param(nx, nz, params.param_b * 3.0 + 0.5),
        FormulaType::Hexagonal => hexagonal(nx, nz),
        FormulaType::RidgedMF => ridged_fbm(nx, nz, octaves.min(4)),
        FormulaType::DomainWarp => domain_warp_strength(nx, nz, params.param_b * 4.0),
        FormulaType::Hybrid => hybrid_terrain(nx, nz),
        FormulaType::Plasma => plasma(nx, nz),
        FormulaType::Cellular => cellular(nx, nz),
        FormulaType::Strange => strange_attractor(nx, nz, params.param_a, params.param_b),
        FormulaType::Worley => worley(nx, nz),
        FormulaType::Marble => marble(nx, nz),
        FormulaType::Terrazas => terrazas(nx, nz, params.param_a * 5.0 + 2.0),
        FormulaType::Erosion => erosion(nx, nz),
        FormulaType::Thermal => thermal(nx, nz),
    }
}

fn formula_to_height(formula: FormulaType, base: f64, amplitude: f64, water_level: f64) -> f64 {
    match formula {
        FormulaType::Mandelbrot => base * amplitude * 0.3,
        FormulaType::Sierpinski => base * amplitude * 2.0,
        FormulaType::Voronoi => base * amplitude * 0.2 + water_level,
        FormulaType::Cube | FormulaType::Sphere | FormulaType::Menger => base * amplitude * 2.0 + water_level,
        FormulaType::Julia => base * amplitude * 0.4 + water_level,
        FormulaType::Tetrahedron => base * amplitude * 1.5 + water_level,
        FormulaType::Vortex => base * amplitude * 0.6 + water_level,
        FormulaType::Ice => base * amplitude * 1.2 + water_level,
        FormulaType::Wave | FormulaType::Spiral | FormulaType::Hexagonal => base * amplitude * 0.8 + water_level,
        FormulaType::RidgedMF => base * amplitude * 1.5 + water_level,
        FormulaType::DomainWarp => base * amplitude * 1.2 + water_level,
        FormulaType::Hybrid => base * amplitude + water_level,
        FormulaType::Plasma => base * amplitude * 0.8 + water_level,
        FormulaType::Cellular => base * amplitude * 1.0 + water_level,
        FormulaType::Strange => base * amplitude * 1.2 + water_level,
        FormulaType::Worley => base * amplitude * 0.3 + water_level,
        FormulaType::Marble => base * amplitude * 0.6 + water_level,
        FormulaType::Terrazas => base * amplitude * 1.5 + water_level,
        FormulaType::Erosion => base * amplitude * 1.0 + water_level,
        FormulaType::Thermal => base * amplitude * 0.8 + water_level,
        _ => base * amplitude + water_level,
    }
}

pub fn get_height(params: &WorldParams, wx: f64, wz: f64) -> f64 {
    let scale = params.scale;
    let amplitude = params.amplitude;
    let water_level = params.water_level;

    let nx = wx * scale;
    let nz = wz * scale;

    let base_a = eval_formula(params.formula, nx, nz, params);
    let height_a = formula_to_height(params.formula, base_a, amplitude, water_level);

    let base_b = eval_formula(params.formula_b, nx, nz, params);
    let height_b = formula_to_height(params.formula_b, base_b, amplitude, water_level);

    let blend = params.blend_a.clamp(0.0, 1.0);
    let _base = if blend <= 0.0 { base_a } else if blend >= 1.0 { base_b } else {
        base_a * (1.0 - blend) + base_b * blend
    };
    let height = if blend <= 0.0 { height_a } else if blend >= 1.0 { height_b } else {
        height_a * (1.0 - blend) + height_b * blend
    };

    let mut h = height.max(0.0);

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

pub fn get_formula_color(formula: FormulaType, h: f64, max_h: f64) -> [f32; 3] {
    let t = (h / max_h.max(0.1)).clamp(0.0, 1.0) as f32;
    formula_color_map(formula, t)
}

pub fn get_blended_formula_color(formula_a: FormulaType, formula_b: FormulaType, blend: f64, h: f64, max_h: f64) -> [f32; 3] {
    let t = (h / max_h.max(0.1)).clamp(0.0, 1.0) as f32;
    let blend = blend.clamp(0.0, 1.0);
    if blend <= 0.0 {
        return formula_color_map(formula_a, t);
    }
    if blend >= 1.0 {
        return formula_color_map(formula_b, t);
    }
    let ca = formula_color_map(formula_a, t);
    let cb = formula_color_map(formula_b, t);
    let b = blend as f32;
    [
        ca[0] * (1.0 - b) + cb[0] * b,
        ca[1] * (1.0 - b) + cb[1] * b,
        ca[2] * (1.0 - b) + cb[2] * b,
    ]
}

fn formula_color_map(formula: FormulaType, t: f32) -> [f32; 3] {
    match formula {
        FormulaType::FBM =>
            gradient(&[[0.05,0.15,0.30],[0.10,0.35,0.25],[0.25,0.55,0.20],[0.55,0.50,0.25],[0.85,0.80,0.70],[1.0,1.0,1.0]], t),
        FormulaType::Perlin =>
            gradient(&[[0.08,0.12,0.28],[0.15,0.30,0.50],[0.25,0.50,0.35],[0.50,0.45,0.25],[0.80,0.75,0.65],[0.95,0.95,0.95]], t),
        FormulaType::Simplex =>
            gradient(&[[0.10,0.05,0.25],[0.20,0.20,0.45],[0.30,0.45,0.55],[0.45,0.60,0.40],[0.75,0.70,0.55],[0.90,0.90,0.85]], t),
        FormulaType::Voronoi =>
            gradient(&[[0.15,0.10,0.20],[0.30,0.20,0.40],[0.50,0.30,0.55],[0.70,0.50,0.50],[0.85,0.75,0.60],[0.95,0.90,0.80]], t),
        FormulaType::Mandelbrot =>
            gradient(&[[0.05,0.00,0.10],[0.20,0.05,0.25],[0.45,0.10,0.40],[0.75,0.25,0.35],[0.95,0.60,0.40],[1.0,0.85,0.65]], t),
        FormulaType::Sierpinski =>
            gradient(&[[0.15,0.05,0.00],[0.35,0.10,0.00],[0.60,0.25,0.05],[0.85,0.50,0.10],[1.0,0.75,0.30],[1.0,0.95,0.60]], t),
        FormulaType::Julia =>
            gradient(&[[0.10,0.00,0.15],[0.30,0.05,0.35],[0.55,0.15,0.50],[0.80,0.30,0.45],[0.95,0.60,0.50],[1.0,0.85,0.75]], t),
        FormulaType::Tetrahedron =>
            gradient(&[[0.05,0.15,0.15],[0.10,0.30,0.35],[0.20,0.50,0.45],[0.40,0.60,0.40],[0.70,0.65,0.45],[0.90,0.85,0.75]], t),
        FormulaType::Cube =>
            gradient(&[[0.10,0.10,0.05],[0.25,0.25,0.10],[0.45,0.40,0.15],[0.65,0.55,0.25],[0.85,0.75,0.45],[1.0,0.95,0.70]], t),
        FormulaType::Sphere =>
            gradient(&[[0.05,0.10,0.20],[0.15,0.25,0.40],[0.25,0.40,0.55],[0.40,0.55,0.50],[0.70,0.70,0.60],[0.90,0.90,0.85]], t),
        FormulaType::Menger =>
            gradient(&[[0.10,0.05,0.10],[0.25,0.15,0.25],[0.40,0.25,0.35],[0.60,0.40,0.35],[0.80,0.65,0.45],[0.95,0.90,0.70]], t),
        FormulaType::Vortex =>
            gradient(&[[0.00,0.10,0.20],[0.10,0.25,0.45],[0.20,0.40,0.50],[0.40,0.55,0.40],[0.65,0.70,0.55],[0.90,0.90,0.80]], t),
        FormulaType::Ice =>
            gradient(&[[0.40,0.50,0.60],[0.55,0.65,0.75],[0.65,0.75,0.85],[0.75,0.85,0.90],[0.85,0.90,0.95],[0.95,0.97,1.0]], t),
        FormulaType::Wave =>
            gradient(&[[0.00,0.15,0.30],[0.05,0.30,0.50],[0.15,0.45,0.55],[0.35,0.55,0.45],[0.60,0.65,0.55],[0.85,0.85,0.80]], t),
        FormulaType::Spiral =>
            gradient(&[[0.15,0.05,0.20],[0.30,0.10,0.40],[0.50,0.20,0.50],[0.70,0.35,0.45],[0.85,0.60,0.50],[0.95,0.85,0.75]], t),
        FormulaType::Hexagonal =>
            gradient(&[[0.10,0.15,0.05],[0.20,0.30,0.10],[0.35,0.45,0.20],[0.55,0.55,0.30],[0.75,0.70,0.45],[0.95,0.90,0.70]], t),
        FormulaType::RidgedMF =>
            gradient(&[[0.20,0.15,0.10],[0.35,0.25,0.15],[0.50,0.35,0.20],[0.65,0.50,0.30],[0.85,0.70,0.50],[1.0,0.95,0.85]], t),
        FormulaType::DomainWarp =>
            gradient(&[[0.05,0.05,0.15],[0.15,0.10,0.30],[0.30,0.20,0.45],[0.50,0.35,0.45],[0.75,0.55,0.50],[0.95,0.85,0.75]], t),
        FormulaType::Hybrid =>
            gradient(&[[0.10,0.05,0.05],[0.25,0.15,0.15],[0.45,0.30,0.20],[0.65,0.50,0.30],[0.85,0.70,0.50],[1.0,0.90,0.75]], t),
        FormulaType::Plasma =>
            gradient(&[[0.30,0.05,0.10],[0.55,0.10,0.20],[0.70,0.20,0.30],[0.85,0.40,0.35],[0.95,0.65,0.45],[1.0,0.90,0.70]], t),
        FormulaType::Cellular =>
            gradient(&[[0.10,0.20,0.15],[0.15,0.35,0.25],[0.20,0.50,0.35],[0.35,0.60,0.40],[0.60,0.70,0.50],[0.85,0.90,0.80]], t),
        FormulaType::Strange =>
            gradient(&[[0.20,0.05,0.25],[0.35,0.10,0.40],[0.50,0.20,0.50],[0.65,0.35,0.45],[0.85,0.55,0.50],[1.0,0.85,0.75]], t),
        FormulaType::Worley =>
            gradient(&[[0.15,0.10,0.05],[0.30,0.20,0.10],[0.50,0.35,0.15],[0.65,0.50,0.25],[0.80,0.70,0.45],[0.95,0.90,0.75]], t),
        FormulaType::Marble =>
            gradient(&[[0.20,0.20,0.22],[0.35,0.35,0.38],[0.50,0.50,0.52],[0.65,0.65,0.67],[0.80,0.80,0.82],[0.95,0.95,0.97]], t),
        FormulaType::Terrazas =>
            gradient(&[[0.15,0.10,0.05],[0.30,0.22,0.10],[0.50,0.40,0.20],[0.70,0.60,0.35],[0.85,0.75,0.55],[1.0,0.95,0.80]], t),
        FormulaType::Erosion =>
            gradient(&[[0.10,0.15,0.20],[0.20,0.30,0.40],[0.30,0.45,0.50],[0.45,0.55,0.45],[0.65,0.65,0.55],[0.85,0.85,0.80]], t),
        FormulaType::Thermal =>
            gradient(&[[0.30,0.10,0.05],[0.50,0.20,0.10],[0.65,0.35,0.20],[0.80,0.55,0.35],[0.95,0.75,0.55],[1.0,0.95,0.85]], t),
    }
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
