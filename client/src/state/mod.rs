use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormulaType {
    FBM, Perlin, Simplex, Voronoi, Mandelbrot,
    Sierpinski, Julia, Tetrahedron, Cube, Sphere,
    Menger, Vortex, Ice, Wave, Spiral, Hexagonal,
    RidgedMF, DomainWarp, Hybrid,
    Plasma, Cellular, Strange, Worley, Marble,
    Terrazas, Erosion, Thermal,
}

impl FormulaType {
    pub fn all() -> Vec<Self> {
        vec![
            Self::FBM, Self::Perlin, Self::Simplex, Self::Voronoi,
            Self::Mandelbrot, Self::Sierpinski, Self::Julia,
            Self::Tetrahedron, Self::Cube, Self::Sphere,
            Self::Menger, Self::Vortex, Self::Ice, Self::Wave,
            Self::Spiral, Self::Hexagonal, Self::RidgedMF,
            Self::DomainWarp, Self::Hybrid,
            Self::Plasma, Self::Cellular, Self::Strange,
            Self::Worley, Self::Marble, Self::Terrazas,
            Self::Erosion, Self::Thermal,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::FBM => "FBM",
            Self::Perlin => "Perlin",
            Self::Simplex => "Simplex",
            Self::Voronoi => "Voronoi",
            Self::Mandelbrot => "Mandelbrot",
            Self::Sierpinski => "Sierpinski",
            Self::Julia => "Julia",
            Self::Tetrahedron => "Tetrahedron",
            Self::Cube => "Cube",
            Self::Sphere => "Sphere",
            Self::Menger => "Menger",
            Self::Vortex => "Vortex",
            Self::Ice => "Ice",
            Self::Wave => "Wave",
            Self::Spiral => "Spiral",
            Self::Hexagonal => "Hexagonal",
            Self::RidgedMF => "Ridged MF",
            Self::DomainWarp => "Domain Warp",
            Self::Hybrid => "Hybrid",
            Self::Plasma => "Plasma",
            Self::Cellular => "Cellular",
            Self::Strange => "Strange",
            Self::Worley => "Worley",
            Self::Marble => "Marble",
            Self::Terrazas => "Terrazas",
            Self::Erosion => "Erosion",
            Self::Thermal => "Thermal",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::FBM => "mountain",
            Self::Perlin => "waves",
            Self::Simplex => "hexagon",
            Self::Voronoi => "grid-3x3",
            Self::Mandelbrot => "infinity",
            Self::Sierpinski => "triangle",
            Self::Julia => "sparkles",
            Self::Tetrahedron => "diamond",
            Self::Cube => "box",
            Self::Sphere => "circle-dot",
            Self::Menger => "boxes",
            Self::Vortex => "wind",
            Self::Ice => "snowflake",
            Self::Wave => "waves",
            Self::Spiral => "shell",
            Self::Hexagonal => "hexagon",
            Self::RidgedMF => "zap",
            Self::DomainWarp => "rotate-3d",
            Self::Hybrid => "shuffle",
            Self::Plasma => "zap",
            Self::Cellular => "grid",
            Self::Strange => "infinity",
            Self::Worley => "circle",
            Self::Marble => "swirl",
            Self::Terrazas => "layers",
            Self::Erosion => "droplets",
            Self::Thermal => "flame",
        }
    }

    pub fn color_hex(&self) -> &'static str {
        match self {
            Self::FBM => "#22d3ee",
            Self::Perlin => "#34d399",
            Self::Simplex => "#a78bfa",
            Self::Voronoi => "#f472b6",
            Self::Mandelbrot => "#fb923c",
            Self::Sierpinski => "#f87171",
            Self::Julia => "#e879f9",
            Self::Tetrahedron => "#2dd4bf",
            Self::Cube => "#fbbf24",
            Self::Sphere => "#60a5fa",
            Self::Menger => "#a3e635",
            Self::Vortex => "#818cf8",
            Self::Ice => "#67e8f9",
            Self::Wave => "#38bdf8",
            Self::Spiral => "#c084fc",
            Self::Hexagonal => "#facc15",
            Self::RidgedMF => "#fb7185",
            Self::DomainWarp => "#34d399",
            Self::Hybrid => "#e879f9",
            Self::Plasma => "#f43f5e",
            Self::Cellular => "#14b8a6",
            Self::Strange => "#8b5cf6",
            Self::Worley => "#f97316",
            Self::Marble => "#e2e8f0",
            Self::Terrazas => "#d97706",
            Self::Erosion => "#06b6d4",
            Self::Thermal => "#ef4444",
        }
    }

    pub fn param_a_label(&self) -> &'static str {
        match self {
            Self::Julia => "Julia C",
            Self::DomainWarp => "Warp",
            Self::Vortex => "Twist",
            Self::Wave => "Freq",
            Self::Spiral => "Spiral",
            Self::Ice => "Mix",
            Self::RidgedMF => "Rough",
            Self::Hybrid => "Blend",
            Self::Plasma => "Freq",
            Self::Strange => "Chaos",
            Self::Marble => "Vein",
            Self::Terrazas => "Steps",
            _ => "Param A",
        }
    }

    pub fn param_b_label(&self) -> &'static str {
        match self {
            Self::DomainWarp => "Strength",
            Self::Spiral => "Turns",
            Self::Hexagonal => "Size",
            Self::Voronoi => "Jitter",
            Self::Cube => "Edge",
            Self::Strange => "Attract",
            Self::Erosion => "Iter",
            _ => "Param B",
        }
    }

    pub fn formula_expr(&self, scale: f64, octaves: u32) -> String {
        match self {
            Self::FBM => format!("FBM(x·{:.3}, z·{:.3}, {})", scale, scale, octaves),
            Self::Perlin => format!("Perlin(x·{:.3}, z·{:.3})", scale, scale),
            Self::Simplex => format!("Simplex(x·{:.3}, z·{:.3})", scale, scale),
            Self::Voronoi => format!("Voronoi(x·{:.3}, z·{:.3})", scale, scale),
            Self::Mandelbrot => format!("Mandelbrot(x·{:.3}, z·{:.3})", scale, scale),
            Self::Sierpinski => format!("Sierpinski(x·{:.3}, z·{:.3})", scale, scale),
            Self::Julia => format!("Julia(x·{:.3}, z·{:.3})", scale, scale),
            Self::Tetrahedron => format!("Tetra(x·{:.3}, z·{:.3})", scale, scale),
            Self::Cube => format!("Cube(x·{:.3}, z·{:.3})", scale, scale),
            Self::Sphere => format!("Sphere(x·{:.3}, z·{:.3})", scale, scale),
            Self::Menger => format!("Menger(x·{:.3}, z·{:.3})", scale, scale),
            Self::Vortex => format!("Vortex(x·{:.3}, z·{:.3})", scale, scale),
            Self::Ice => format!("Ice(x·{:.3}, z·{:.3})", scale, scale),
            Self::Wave => format!("Wave(x·{:.3}, z·{:.3})", scale, scale),
            Self::Spiral => format!("Spiral(x·{:.3}, z·{:.3})", scale, scale),
            Self::Hexagonal => format!("Hex(x·{:.3}, z·{:.3})", scale, scale),
            Self::RidgedMF => format!("RidgedMF(x·{:.3}, z·{:.3}, {})", scale, scale, octaves.min(4)),
            Self::DomainWarp => format!("Warp(x·{:.3}, z·{:.3})", scale, scale),
            Self::Hybrid => format!("Hybrid(x·{:.3}, z·{:.3})", scale, scale),
            Self::Plasma => format!("Plasma(x·{:.3}, z·{:.3})", scale, scale),
            Self::Cellular => format!("Cellular(x·{:.3}, z·{:.3})", scale, scale),
            Self::Strange => format!("Strange(x·{:.3}, z·{:.3})", scale, scale),
            Self::Worley => format!("Worley(x·{:.3}, z·{:.3})", scale, scale),
            Self::Marble => format!("Marble(x·{:.3}, z·{:.3})", scale, scale),
            Self::Terrazas => format!("Terrazas(x·{:.3}, z·{:.3})", scale, scale),
            Self::Erosion => format!("Erosion(x·{:.3}, z·{:.3})", scale, scale),
            Self::Thermal => format!("Thermal(x·{:.3}, z·{:.3})", scale, scale),
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::FBM => "🏔️",
            Self::Perlin => "🌊",
            Self::Simplex => "🫧",
            Self::Voronoi => "🧬",
            Self::Mandelbrot => "♾️",
            Self::Sierpinski => "🔺",
            Self::Julia => "✨",
            Self::Tetrahedron => "💎",
            Self::Cube => "📦",
            Self::Sphere => "⚪",
            Self::Menger => "🧽",
            Self::Vortex => "🌪️",
            Self::Ice => "❄️",
            Self::Wave => "🌊",
            Self::Spiral => "🌀",
            Self::Hexagonal => "🐝",
            Self::RidgedMF => "⚡",
            Self::DomainWarp => "🌀",
            Self::Hybrid => "🔀",
            Self::Plasma => "⚡",
            Self::Cellular => "🧫",
            Self::Strange => "🫧",
            Self::Worley => "🔮",
            Self::Marble => "💎",
            Self::Terrazas => "🏛️",
            Self::Erosion => "🏜️",
            Self::Thermal => "🌡️",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlMode {
    DPad,
    Joystick,
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct WorldParams {
    pub seed: u32,
    pub scale: f64,
    pub octaves: u32,
    pub amplitude: f64,
    pub water_level: f64,
    pub render_distance: u32,
    pub zone: crate::engine::terrain::Zone,
    pub formula: FormulaType,
    pub formula_b: FormulaType,
    pub blend_a: f64,
    pub mutation: f64,
    pub speed: f64,
    pub mouse_sensitivity: f64,
    pub fly_mode: bool,
    pub control_mode: ControlMode,
    pub hue_shift: f64,
    pub saturation: f64,
    pub lightness: f64,
    pub param_a: f64,
    pub param_b: f64,
    pub volume: f64,
}

impl Default for WorldParams {
    fn default() -> Self {
        Self {
            seed: 42,
            scale: 0.025,
            octaves: 5,
            amplitude: 1.8,
            water_level: 0.6,
            render_distance: 2,
            zone: crate::engine::terrain::Zone::Forest,
            formula: FormulaType::FBM,
            formula_b: FormulaType::FBM,
            blend_a: 0.0,
            mutation: 0.0,
            speed: 18.0,
            mouse_sensitivity: 1.0,
            fly_mode: false,
            control_mode: ControlMode::DPad,
            hue_shift: 0.0,
            saturation: 1.0,
            lightness: 1.0,
            param_a: 0.5,
            param_b: 0.5,
            volume: 0.3,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub params: RwSignal<WorldParams>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            params: RwSignal::new(WorldParams::default()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub slot_name: String,
    pub params: WorldParams,
    pub pos: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub waypoints: Vec<(f64, f64, f64, String)>,
    pub discovered_biomes: Vec<String>,
    pub time_of_day: f64,
    pub fly_mode: bool,
    pub observer_mode: bool,
    pub created_at: f64,
}

impl SaveData {
    pub fn new(slot_name: &str, params: &WorldParams, pos: [f64; 3], yaw: f64, pitch: f64,
               waypoints: &[(f64, f64, f64, String)], discovered: &[String],
               time_of_day: f64, fly_mode: bool, observer_mode: bool) -> Self {
        Self {
            slot_name: slot_name.to_string(),
            params: *params,
            pos,
            yaw,
            pitch,
            waypoints: waypoints.to_vec(),
            discovered_biomes: discovered.to_vec(),
            time_of_day,
            fly_mode,
            observer_mode,
            created_at: js_sys::Date::now(),
        }
    }
}
