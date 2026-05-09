use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaType {
    FBM, Perlin, Simplex, Voronoi, Mandelbrot,
    Sierpinski, Julia, Tetrahedron, Cube, Sphere,
    Menger, Vortex, Ice, Wave, Spiral, Hexagonal,
    RidgedMF, DomainWarp, Hybrid,
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    DPad,
    Joystick,
}

#[derive(Clone, Debug, Copy)]
pub struct WorldParams {
    pub seed: u32,
    pub scale: f64,
    pub octaves: u32,
    pub amplitude: f64,
    pub water_level: f64,
    pub render_distance: u32,
    pub zone: crate::engine::terrain::Zone,
    pub formula: FormulaType,
    pub speed: f64,
    pub mouse_sensitivity: f64,
    pub fly_mode: bool,
    pub control_mode: ControlMode,
    pub hue_shift: f64,
    pub saturation: f64,
    pub lightness: f64,
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
            speed: 18.0,
            mouse_sensitivity: 1.0,
            fly_mode: false,
            control_mode: ControlMode::DPad,
            hue_shift: 0.0,
            saturation: 1.0,
            lightness: 1.0,
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
