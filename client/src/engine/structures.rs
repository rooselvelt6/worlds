use crate::engine::terrain::{self, Zone};
use crate::state::WorldParams;

#[derive(Clone, Copy, PartialEq)]
pub enum StructType {
    Hut,
    Tower,
    Ruins,
    Arch,
    Pillar,
    Dome,
    Pyramid,
    CrystalSpire,
    MushroomHut,
    Obelisk,
}

#[derive(Clone)]
pub struct StructInstance {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation: f32,
    pub scale: f32,
    pub struct_type: StructType,
    pub color_variation: u8,
}

#[derive(Clone)]
pub struct StructData {
    pub cx: i32,
    pub cz: i32,
    pub instances: Vec<StructInstance>,
}

const MAX_STRUCTS: usize = 3;

pub fn compute_chunk_structures(params: &WorldParams, cx: i32, cz: i32) -> StructData {
    let ox = cx as f64 * crate::engine::chunk::CHUNK_SIZE;
    let oz = cz as f64 * crate::engine::chunk::CHUNK_SIZE;
    let seed = params.seed.wrapping_mul(2654435761)
        .wrapping_add((cx as u32).wrapping_mul(374761393))
        .wrapping_add((cz as u32).wrapping_mul(668265263));

    let zone = terrain::get_zone(params, ox + 12.0, oz + 12.0);
    let density = struct_density(zone);
    let count = ((MAX_STRUCTS as f64 * density) as usize).min(MAX_STRUCTS);

    let mut instances = Vec::with_capacity(count);
    let mut rng = seed;
    let struct_types = types_for_zone(zone);

    if struct_types.is_empty() {
        return StructData { cx, cz, instances };
    }

    for _ in 0..count {
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let rx = (rng as f64 / u32::MAX as f64) * crate::engine::chunk::CHUNK_SIZE;
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let rz = (rng as f64 / u32::MAX as f64) * crate::engine::chunk::CHUNK_SIZE;

        let wx = ox + rx;
        let wz = oz + rz;
        let h = terrain::get_height(params, wx, wz);
        let water = params.water_level;

        if h <= water + 0.5 { continue; }

        // Check flat area (5x5 grid)
        let mut flat = true;
        for dy in -2..=2 {
            for dx in -2..=2 {
                let sh = terrain::get_height(params, wx + dx as f64, wz + dy as f64);
                if (sh - h).abs() > 1.0 { flat = false; }
            }
        }
        if !flat { continue; }

        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let r = rng as f64 / u32::MAX as f64;

        let idx = (r * struct_types.len() as f64) as usize;
        let struct_type = struct_types[idx.min(struct_types.len() - 1)];
        let scale = struct_size(struct_type, zone, r);
        let rotation = (r * std::f64::consts::TAU) as f32;
        let color_variation = (rng.wrapping_mul(13) as u8) % 6;

        instances.push(StructInstance {
            x: wx as f32,
            y: h as f32,
            z: wz as f32,
            rotation,
            scale,
            struct_type,
            color_variation,
        });
    }

    StructData { cx, cz, instances }
}

fn struct_density(zone: Zone) -> f64 {
    match zone {
        Zone::Plains | Zone::Forest => 0.5,
        Zone::Desert | Zone::Tundra => 0.4,
        Zone::Jungle => 0.6,
        Zone::Crystal => 0.3,
        Zone::Fungus => 0.4,
        Zone::Volcanic | Zone::Lava | Zone::Magma => 0.2,
        Zone::Storm => 0.2,
        Zone::Aurora => 0.2,
        Zone::Abyss => 0.1,
        _ => 0.0,
    }
}

fn types_for_zone(zone: Zone) -> Vec<StructType> {
    match zone {
        Zone::Forest => vec![StructType::Hut, StructType::Ruins, StructType::Tower],
        Zone::Plains => vec![StructType::Hut, StructType::Tower, StructType::Obelisk],
        Zone::Desert => vec![StructType::Pyramid, StructType::Ruins, StructType::Arch],
        Zone::Tundra => vec![StructType::Dome, StructType::Hut, StructType::Pillar],
        Zone::Jungle => vec![StructType::Pyramid, StructType::Ruins, StructType::Arch],
        Zone::Volcanic => vec![StructType::Pillar, StructType::Ruins],
        Zone::Crystal => vec![StructType::CrystalSpire, StructType::Arch],
        Zone::Fungus => vec![StructType::MushroomHut, StructType::Arch],
        Zone::Lava => vec![StructType::Pillar, StructType::Obelisk],
        Zone::Abyss => vec![StructType::Pillar],
        Zone::Storm => vec![StructType::Tower, StructType::Obelisk],
        Zone::Aurora => vec![StructType::CrystalSpire, StructType::Arch],
        Zone::Magma => vec![StructType::Pillar, StructType::Obelisk],
        _ => vec![],
    }
}

fn struct_size(struct_type: StructType, zone: Zone, r: f64) -> f32 {
    let base = match struct_type {
        StructType::Hut => 1.0 + r * 0.8,
        StructType::Tower => 0.8 + r * 1.2,
        StructType::Ruins => 1.0 + r * 1.0,
        StructType::Arch => 0.6 + r * 1.0,
        StructType::Pillar => 0.5 + r * 0.8,
        StructType::Dome => 0.8 + r * 0.7,
        StructType::Pyramid => 0.6 + r * 1.2,
        StructType::CrystalSpire => 0.5 + r * 1.5,
        StructType::MushroomHut => 0.7 + r * 1.0,
        StructType::Obelisk => 0.4 + r * 0.8,
    };
    let scale = match zone {
        Zone::Jungle => 1.3,
        Zone::Tundra => 0.7,
        _ => 1.0,
    };
    (base * scale) as f32
}
