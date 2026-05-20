use crate::engine::terrain::{self, Zone};
use crate::state::WorldParams;

#[derive(Clone, Copy, PartialEq)]
pub enum VegType {
    Tree,
    Bush,
    Rock,
    Cactus,
    Mushroom,
    Crystal,
    DeadTree,
    Flower,
    Coral,
    Kelp,
}

#[derive(Clone)]
pub struct VegInstance {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub size: f32,
    pub veg_type: VegType,
    pub variation: u8,
}

#[derive(Clone)]
pub struct VegData {
    pub cx: i32,
    pub cz: i32,
    pub instances: Vec<VegInstance>,
}

const MAX_VEG: usize = 120;

pub fn compute_chunk_vegetation(params: &WorldParams, cx: i32, cz: i32) -> VegData {
    let ox = cx as f64 * crate::engine::chunk::CHUNK_SIZE;
    let oz = cz as f64 * crate::engine::chunk::CHUNK_SIZE;
    let seed = params.seed.wrapping_mul(2654435761).wrapping_add((cx as u32).wrapping_mul(374761393)).wrapping_add((cz as u32).wrapping_mul(668265263));

    let zone = terrain::get_zone(params, ox + 12.0, oz + 12.0);
    let is_underwater_zone = matches!(zone, Zone::CoralReef | Zone::KelpForest | Zone::RockyReef | Zone::SandyPlain | Zone::DeepOcean);
    let density = veg_density(zone);
    let count = ((MAX_VEG as f64 * density) as usize).min(MAX_VEG);

    let mut instances = Vec::with_capacity(count);
    let mut rng = seed;

    for i in 0..count {
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let rx = (rng as f64 / u32::MAX as f64) * crate::engine::chunk::CHUNK_SIZE;
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let rz = (rng as f64 / u32::MAX as f64) * crate::engine::chunk::CHUNK_SIZE;

        let wx = ox + rx;
        let wz = oz + rz;
        let h = terrain::get_height(params, wx, wz);
        let water = params.water_level;

        if !is_underwater_zone {
            // Above-water vegetation: skip underwater spots
            if h <= water + 0.3 { continue; }
            let hx = terrain::get_height(params, wx + 1.0, wz);
            let hz = terrain::get_height(params, wx, wz + 1.0);
            let slope = ((hx - h).abs() + (hz - h).abs()) * 0.5;
            if slope > 1.5 { continue; }
        } else {
            // Underwater vegetation: skip above-water or too deep
            if h > water - 0.1 { continue; }
            let depth = water - h;
            if depth > 3.0 { continue; }
        }

        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let r = rng as f64 / u32::MAX as f64;

        let veg_type = veg_for_zone(zone, r);
        let size = veg_size(veg_type, zone, r);
        let variation = (rng.wrapping_mul(7) as u8) % 8;

        instances.push(VegInstance {
            x: wx as f32,
            y: h as f32,
            z: wz as f32,
            size,
            veg_type,
            variation,
        });
    }

    VegData { cx, cz, instances }
}

fn veg_density(zone: Zone) -> f64 {
    match zone {
        Zone::Forest | Zone::Jungle => 0.7,
        Zone::Plains => 0.4,
        Zone::Desert => 0.2,
        Zone::Tundra => 0.15,
        Zone::Crystal => 0.3,
        Zone::Fungus => 0.5,
        Zone::Volcanic | Zone::Lava | Zone::Magma => 0.1,
        Zone::Cave => 0.05,
        Zone::Abyss => 0.02,
        Zone::Storm => 0.1,
        Zone::Aurora => 0.2,
        Zone::Ocean => 0.0,
        Zone::CoralReef => 0.7,
        Zone::KelpForest => 0.5,
        Zone::SandyPlain => 0.0,
        Zone::RockyReef => 0.3,
        Zone::DeepOcean => 0.0,
    }
}

fn veg_for_zone(zone: Zone, r: f64) -> VegType {
    match zone {
        Zone::Forest | Zone::Jungle => {
            if r < 0.4 { VegType::Tree }
            else if r < 0.6 { VegType::Bush }
            else if r < 0.8 { VegType::Flower }
            else { VegType::Rock }
        }
        Zone::Plains => {
            if r < 0.3 { VegType::Bush }
            else if r < 0.6 { VegType::Flower }
            else { VegType::Rock }
        }
        Zone::Desert => {
            if r < 0.5 { VegType::Cactus }
            else { VegType::Rock }
        }
        Zone::Tundra => {
            if r < 0.4 { VegType::Tree }
            else { VegType::Rock }
        }
        Zone::Crystal => {
            if r < 0.6 { VegType::Crystal }
            else { VegType::Rock }
        }
        Zone::Fungus => {
            if r < 0.5 { VegType::Mushroom }
            else if r < 0.7 { VegType::Tree }
            else { VegType::Flower }
        }
        Zone::Volcanic | Zone::Lava | Zone::Magma => {
            if r < 0.4 { VegType::DeadTree }
            else { VegType::Rock }
        }
        Zone::Storm => {
            if r < 0.3 { VegType::DeadTree }
            else { VegType::Rock }
        }
        Zone::Aurora => {
            if r < 0.5 { VegType::Crystal }
            else { VegType::Rock }
        }
        Zone::CoralReef => {
            if r < 0.6 { VegType::Coral }
            else { VegType::Rock }
        }
        Zone::KelpForest => {
            if r < 0.5 { VegType::Kelp }
            else if r < 0.7 { VegType::Coral }
            else { VegType::Rock }
        }
        Zone::RockyReef => {
            if r < 0.4 { VegType::Coral }
            else { VegType::Rock }
        }
        _ => VegType::Rock,
    }
}

fn veg_size(veg_type: VegType, zone: Zone, r: f64) -> f32 {
    let base = match veg_type {
        VegType::Tree => 1.5 + r * 2.0,
        VegType::Bush => 0.5 + r * 0.8,
        VegType::Rock => 0.3 + r * 0.8,
        VegType::Cactus => 1.0 + r * 2.5,
        VegType::Mushroom => 0.6 + r * 1.2,
        VegType::Crystal => 0.5 + r * 2.0,
        VegType::DeadTree => 1.0 + r * 1.5,
        VegType::Flower => 0.1 + r * 0.2,
        VegType::Coral => 0.3 + r * 0.8,
        VegType::Kelp => 2.0 + r * 3.0,
    };
    let scale = match zone {
        Zone::Jungle => 1.5,
        Zone::Tundra => 0.6,
        Zone::Desert => 0.7,
        Zone::Volcanic | Zone::Lava | Zone::Magma => 0.8,
        Zone::CoralReef => 0.8,
        Zone::KelpForest => 1.2,
        _ => 1.0,
    };
    (base * scale) as f32
}
