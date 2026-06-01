use crate::engine::creatures::push_ellipsoid_rot;
use crate::engine::terrain::{self, Zone};
use crate::state::WorldParams;

#[derive(Clone)]
pub struct MineralDeposit {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub mineral_type: u8,
    pub size: f32,
}

#[derive(Clone)]
pub struct MineralData {
    pub cx: i32,
    pub cz: i32,
    pub deposits: Vec<MineralDeposit>,
}

const MAX_MINERALS: usize = 30;

pub fn compute_chunk_minerals(params: &WorldParams, cx: i32, cz: i32) -> MineralData {
    let ox = cx as f64 * crate::engine::chunk::CHUNK_SIZE;
    let oz = cz as f64 * crate::engine::chunk::CHUNK_SIZE;
    let seed = params.seed.wrapping_mul(2654435761)
        .wrapping_add((cx as u32).wrapping_mul(374761393))
        .wrapping_add((cz as u32).wrapping_mul(668265263));

    let zone = terrain::get_zone(params, ox + 12.0, oz + 12.0);
    let density = mineral_density(zone);
    let count = ((MAX_MINERALS as f64 * density) as usize).min(MAX_MINERALS);

    let mut deposits = Vec::with_capacity(count);
    let mut rng = seed;

    for _ in 0..count {
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let rx = (rng as f64 / u32::MAX as f64) * crate::engine::chunk::CHUNK_SIZE;
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let rz = (rng as f64 / u32::MAX as f64) * crate::engine::chunk::CHUNK_SIZE;

        let wx = ox + rx;
        let wz = oz + rz;
        let h = terrain::get_height(params, wx, wz);
        let water = params.water_level;

        // Minerals mostly underground or on cliff faces
        let above_ground = h > water + 0.5;
        let underground = h <= water - 0.5;

        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let r = rng as f64 / u32::MAX as f64;

        if underground || (above_ground && r < 0.3) {
            let mineral_type = mineral_for_zone(zone, r);
            let placement_y = if underground {
                h as f32 + (r as f32 * 2.0 - 1.0) * 0.5
            } else {
                h as f32
            };
            let size = 0.1 + r as f32 * 0.3;

            deposits.push(MineralDeposit {
                x: wx as f32,
                y: placement_y,
                z: wz as f32,
                mineral_type,
                size,
            });
        }
    }

    MineralData { cx, cz, deposits }
}

fn mineral_density(zone: Zone) -> f64 {
    match zone {
        Zone::Cave => 0.8,
        Zone::Abyss => 0.6,
        Zone::Crystal => 0.7,
        Zone::Volcanic | Zone::Lava | Zone::Magma => 0.5,
        Zone::Fungus => 0.4,
        Zone::Storm => 0.3,
        _ => 0.15,
    }
}

fn mineral_for_zone(zone: Zone, r: f64) -> u8 {
    match zone {
        Zone::Cave => if r < 0.3 { 0 } else if r < 0.6 { 1 } else { 2 },
        Zone::Crystal => if r < 0.4 { 3 } else { 4 },
        Zone::Volcanic | Zone::Lava | Zone::Magma => if r < 0.5 { 5 } else { 6 },
        Zone::Abyss => if r < 0.5 { 2 } else { 6 },
        Zone::Fungus => if r < 0.5 { 7 } else { 0 },
        _ => (r * 8.0) as u8 % 8,
    }
}

fn mineral_color(mt: u8) -> [f32; 3] {
    match mt {
        0 => [0.55, 0.25, 0.15], // iron
        1 => [0.75, 0.45, 0.15], // copper
        2 => [0.15, 0.15, 0.15], // coal
        3 => [0.15, 0.25, 0.85], // sapphire
        4 => [0.15, 0.75, 0.25], // emerald
        5 => [0.85, 0.15, 0.15], // ruby
        6 => [0.85, 0.75, 0.15], // gold
        7 => [0.65, 0.15, 0.65], // amethyst
        _ => [0.50, 0.50, 0.50],
    }
}

pub fn generate_mineral_mesh(params: &WorldParams, cx: i32, cz: i32) -> Option<(Vec<f32>, Vec<f32>, Vec<u32>, Vec<f32>)> {
    let data = compute_chunk_minerals(params, cx, cz);
    if data.deposits.is_empty() { return None; }
    let mut pos = Vec::new();
    let mut norms = Vec::new();
    let mut idx = Vec::new();
    let mut cols = Vec::new();
    let mut base_idx = 0u32;
    for d in &data.deposits {
        let c = mineral_color(d.mineral_type);
        let s = d.size * 0.5;
        push_ellipsoid_rot(&mut pos, &mut norms, &mut idx, &mut cols, d.x, d.y + s, d.z, s, s, s, c[0], c[1], c[2], &mut base_idx, 4, 5, 0.0);
    }
    Some((pos, norms, idx, cols))
}
