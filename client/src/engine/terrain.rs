use crate::math::*;
use crate::state::{FormulaType, WorldParams};

pub fn get_height(params: &WorldParams, wx: f64, wz: f64) -> f64 {
    let scale = params.scale;
    let octaves = params.octaves;
    let amplitude = params.amplitude;
    let water_level = params.water_level;

    let nx = wx * scale;
    let nz = wz * scale;

    let base = match params.formula {
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
        FormulaType::Wave => wave(nx, nz),
        FormulaType::Spiral => spiral(nx, nz),
        FormulaType::Hexagonal => hexagonal(nx, nz),
        FormulaType::RidgedMF => ridged_fbm(nx, nz, octaves.min(4)),
        FormulaType::DomainWarp => domain_warp_strength(nx, nz, params.param_b * 4.0),
        FormulaType::Hybrid => hybrid_terrain(nx, nz),
    };

    let height = match params.formula {
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
        _ => base * amplitude + water_level,
    };

    let mut h = height.max(0.0);

    match params.zone {
        Zone::Ocean => h += water_level,
        Zone::Volcanic => h = h * 1.5 + 2.0,
        Zone::Crystal => h *= 0.5,
        Zone::Cave => h = 3.0 + (wx * 0.5).sin() * 2.0,
        _ => {}
    }

    crystal_effect(params, wx, wz, &mut h);
    cave_effect(params, wx, wz, &mut h);

    h
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Zone {
    Forest, Plains, Desert, Tundra, Jungle,
    Volcanic, Ocean, Crystal, Cave, Lava,
}

impl Zone {
    pub fn from_str(s: &str) -> Self {
        match s {
            "plains" => Zone::Plains, "desert" => Zone::Desert,
            "tundra" => Zone::Tundra, "jungle" => Zone::Jungle,
            "volcanic" => Zone::Volcanic, "ocean" => Zone::Ocean,
            "crystal" => Zone::Crystal, "cave" => Zone::Cave,
            "lava" => Zone::Lava, _ => Zone::Forest,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Zone::Forest => "forest", Zone::Plains => "plains",
            Zone::Desert => "desert", Zone::Tundra => "tundra",
            Zone::Jungle => "jungle", Zone::Volcanic => "volcanic",
            Zone::Ocean => "ocean", Zone::Crystal => "crystal",
            Zone::Cave => "cave", Zone::Lava => "lava",
        }
    }
}

pub fn get_zone(params: &WorldParams, wx: f64, wz: f64) -> Zone {
    if params.zone != Zone::Forest {
        return params.zone;
    }
    let t = fbm(wx * 0.008, wz * 0.008, 2);
    let h = fbm(wx * 0.008 + 50.0, wz * 0.008, 2);
    if t < -0.35 { Zone::Tundra }
    else if t > 0.45 {
        if h < -0.25 { Zone::Desert } else { Zone::Volcanic }
    } else if h > 0.45 {
        if h > 0.6 { Zone::Lava } else { Zone::Jungle }
    } else if h < -0.35 { Zone::Plains }
    else if h < 0.0 { Zone::Ocean }
    else { Zone::Forest }
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
    }
}

pub fn crystal_effect(params: &WorldParams, wx: f64, wz: f64, h: &mut f64) {
    if params.zone == Zone::Crystal {
        let crystal = (wx * 0.8).sin() * (wz * 0.8).cos();
        if crystal.abs() > 0.7 {
            *h += 3.0 + crystal * 2.0;
        }
    }
}

pub fn cave_effect(params: &WorldParams, wx: f64, wz: f64, h: &mut f64) {
    if params.zone == Zone::Cave {
        let cave = (wx * 12.9898 + wz * 78.233 + params.seed as f64).sin() * 43758.5453;
        let n = (cave - cave.floor()) * 2.0 - 1.0;
        if n > 0.3 {
            *h = 2.0 + n * 3.0;
        }
    }
}
