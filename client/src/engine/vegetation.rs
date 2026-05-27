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
    Seaweed,
    Anemone,
    Sponge,
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

pub fn compute_chunk_vegetation(params: &WorldParams, cx: i32, cz: i32, season: u8) -> VegData {
    let ox = cx as f64 * crate::engine::chunk::CHUNK_SIZE;
    let oz = cz as f64 * crate::engine::chunk::CHUNK_SIZE;
    let seed = params.seed.wrapping_mul(2654435761).wrapping_add((cx as u32).wrapping_mul(374761393)).wrapping_add((cz as u32).wrapping_mul(668265263));

    let zone = terrain::get_zone(params, ox + 12.0, oz + 12.0);
    let is_underwater_zone = matches!(zone, Zone::CoralReef | Zone::KelpForest | Zone::RockyReef | Zone::SandyPlain | Zone::DeepOcean);
    let density = veg_density(zone);
    let count = ((MAX_VEG as f64 * density) as usize).min(MAX_VEG);

    let mut instances = Vec::with_capacity(count);
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
        let base_size = veg_size(veg_type, zone, r);
        let variation = (rng.wrapping_mul(7) as u8) % 8;
        let size = growth_size(base_size, variation, season);

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
    let zone_name = zone.as_str();
    if let Some(d) = crate::engine::modding::ModContext::with(|ctx| {
        if let Zone::Custom(id) = zone {
            ctx.get_custom_biome(id).map(|b| b.vegetation_density)
        } else {
            ctx.get_biome_veg_density(zone_name)
        }
    }) { return d; }
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
        Zone::CoralReef => 0.8,
        Zone::KelpForest => 0.6,
        Zone::SandyPlain => 0.4,
        Zone::RockyReef => 0.5,
        Zone::DeepOcean => 0.2,
        Zone::Custom(_) => 0.5,
    }
}

fn veg_for_zone(zone: Zone, r: f64) -> VegType {
    // Check mod context for custom vegetation types
    let zone_name = zone.as_str();
    let mod_veg = crate::engine::modding::ModContext::with(|ctx| {
        let types = if let Zone::Custom(id) = zone {
            ctx.get_custom_biome(id).map(|b| &b.vegetation_types)
        } else {
            ctx.get_biome_veg_types(zone_name)
        };
        types.and_then(|tv| {
            let mut cum = 0.0f64;
            for t in tv {
                cum += t.weight;
                if r < cum {
                    return crate::engine::modding::biome::veg_type_name_to_id(&t.r#type)
                        .map(|id| match id {
                            0 => VegType::Tree, 1 => VegType::Bush, 2 => VegType::Rock,
                            3 => VegType::Cactus, 4 => VegType::Mushroom, 5 => VegType::Crystal,
                            6 => VegType::DeadTree, 7 => VegType::Flower, 8 => VegType::Coral,
                            9 => VegType::Kelp, 10 => VegType::Seaweed, 11 => VegType::Anemone,
                            12 => VegType::Sponge,
                            _ => VegType::Rock,
                        });
                }
            }
            None
        })
    });
    if let Some(vt) = mod_veg { return vt; }

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
            if r < 0.4 { VegType::Coral }
            else if r < 0.55 { VegType::Anemone }
            else if r < 0.7 { VegType::Kelp }
            else { VegType::Rock }
        }
        Zone::KelpForest => {
            if r < 0.45 { VegType::Kelp }
            else if r < 0.55 { VegType::Anemone }
            else if r < 0.7 { VegType::Coral }
            else { VegType::Rock }
        }
        Zone::RockyReef => {
            if r < 0.3 { VegType::Coral }
            else if r < 0.45 { VegType::Anemone }
            else if r < 0.6 { VegType::Sponge }
            else { VegType::Rock }
        }
        Zone::SandyPlain => {
            if r < 0.5 { VegType::Seaweed }
            else if r < 0.7 { VegType::Anemone }
            else { VegType::Rock }
        }
        Zone::DeepOcean => {
            if r < 0.3 { VegType::Kelp }
            else if r < 0.5 { VegType::Sponge }
            else { VegType::Rock }
        }
        _ => VegType::Rock,
    }
}

pub fn veg_color_season(veg_type: VegType, season: u8) -> [f32; 3] {
    match veg_type {
        VegType::Tree => [0.4, 0.25, 0.1],
        VegType::Bush => bush_color(season),
        VegType::Rock => [0.45, 0.38, 0.32],
        VegType::Cactus => [0.2, 0.5, 0.2],
        VegType::Mushroom => [0.85, 0.2, 0.2],
        VegType::Crystal => [0.5, 0.3, 0.85],
        VegType::DeadTree => [0.3, 0.2, 0.1],
        VegType::Flower => flower_color(season),
        VegType::Coral => [0.9, 0.4, 0.4],
        VegType::Kelp => [0.1, 0.5, 0.2],
        VegType::Seaweed => [0.08, 0.45, 0.18],
        VegType::Anemone => [0.95, 0.3, 0.5],
        VegType::Sponge => [0.7, 0.6, 0.2],
    }
}

fn bush_color(season: u8) -> [f32; 3] {
    match season {
        0 => [0.3, 0.65, 0.15], // spring: bright green
        1 => [0.2, 0.5, 0.1],   // summer: green
        2 => [0.55, 0.3, 0.08], // autumn: orange/brown
        _ => [0.35, 0.35, 0.30], // winter: gray
    }
}

fn flower_color(season: u8) -> [f32; 3] {
    match season {
        0 => [1.0, 0.5, 0.7],   // spring: pink
        1 => [1.0, 0.35, 0.6],  // summer: red/pink
        2 => [0.9, 0.4, 0.2],   // autumn: orange
        _ => [0.7, 0.7, 0.7],   // winter: gray
    }
}

pub fn tree_canopy_color(season: u8) -> [f32; 3] {
    match season {
        0 => [0.25, 0.60, 0.15], // spring: fresh green
        1 => [0.15, 0.45, 0.08], // summer: deep green
        2 => [0.55, 0.25, 0.05], // autumn: orange/brown
        _ => [0.35, 0.35, 0.30], // winter: bare/gray
    }
}

pub fn growth_size(base_size: f32, variation: u8, season: u8) -> f32 {
    // Simulate growth stages: smaller plants in spring, mature in summer, dying in autumn/winter
    let stage = match season {
        0 => 0.3 + (variation as f32 / 255.0) * 0.4, // spring: growing
        1 => 0.7 + (variation as f32 / 255.0) * 0.3, // summer: mature
        2 => 0.6 + (variation as f32 / 255.0) * 0.3, // autumn: fading
        _ => 0.3 + (variation as f32 / 255.0) * 0.2, // winter: dormant
    };
    base_size * stage
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
        VegType::Seaweed => 0.5 + r * 1.2,
        VegType::Anemone => 0.3 + r * 0.8,
        VegType::Sponge => 0.3 + r * 0.6,
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

fn push_box(
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    cx: f32, cy: f32, cz: f32, hw: f32, hh: f32, hd: f32,
    r: f32, g: f32, b: f32, base_idx: &mut u32,
) {
    let verts: [[f32; 3]; 24] = [
        [ hw, -hh, -hd], [ hw,  hh, -hd], [ hw,  hh,  hd], [ hw, -hh,  hd],
        [-hw, -hh,  hd], [-hw,  hh,  hd], [-hw,  hh, -hd], [-hw, -hh, -hd],
        [-hw,  hh,  hd], [ hw,  hh,  hd], [ hw,  hh, -hd], [-hw,  hh, -hd],
        [-hw, -hh, -hd], [ hw, -hh, -hd], [ hw, -hh,  hd], [-hw, -hh,  hd],
        [-hw, -hh,  hd], [ hw, -hh,  hd], [ hw,  hh,  hd], [-hw,  hh,  hd],
        [ hw, -hh, -hd], [-hw, -hh, -hd], [-hw,  hh, -hd], [ hw,  hh, -hd],
    ];
    let norms_data: [[f32; 3]; 24] = [
        [1.0,0.0,0.0],[1.0,0.0,0.0],[1.0,0.0,0.0],[1.0,0.0,0.0],
        [-1.0,0.0,0.0],[-1.0,0.0,0.0],[-1.0,0.0,0.0],[-1.0,0.0,0.0],
        [0.0,1.0,0.0],[0.0,1.0,0.0],[0.0,1.0,0.0],[0.0,1.0,0.0],
        [0.0,-1.0,0.0],[0.0,-1.0,0.0],[0.0,-1.0,0.0],[0.0,-1.0,0.0],
        [0.0,0.0,1.0],[0.0,0.0,1.0],[0.0,0.0,1.0],[0.0,0.0,1.0],
        [0.0,0.0,-1.0],[0.0,0.0,-1.0],[0.0,0.0,-1.0],[0.0,0.0,-1.0],
    ];
    let nv = pos.len() as u32 / 3;
    for &v in &verts { pos.push(cx + v[0]); pos.push(cy + v[1]); pos.push(cz + v[2]); }
    for &n in &norms_data { norms.push(n[0]); norms.push(n[1]); norms.push(n[2]); }
    for _ in 0..24 { cols.push(r); cols.push(g); cols.push(b); }
    let ibase = nv;
    let ipat: [u32; 36] = [
        0,1,2, 0,2,3, 4,5,6, 4,6,7,
        8,9,10, 8,10,11, 12,13,14, 12,14,15,
        16,17,18, 16,18,19, 20,21,22, 20,22,23,
    ];
    for &i in &ipat { idx.push(ibase + i); }
    *base_idx = nv + 24;
}

fn emit_veg(
    veg_type: VegType, x: f32, y: f32, z: f32, size: f32, season: u8,
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    base_idx: &mut u32,
) {
    let c = veg_color_season(veg_type, season);
    match veg_type {
        VegType::Tree => {
            let th = size * 0.6;
            let tw = size * 0.06;
            let fh = size * 0.5;
            let fw = size * 0.3;
            let canopy = tree_canopy_color(season);
            push_box(pos, norms, idx, cols, x, y + th * 0.5, z, tw, th * 0.5, tw, 0.4, 0.25, 0.1, base_idx);
            push_box(pos, norms, idx, cols, x, y + th + fh * 0.5, z, fw, fh * 0.5, fw, canopy[0], canopy[1], canopy[2], base_idx);
            // Fruit in summer and autumn
            if season == 1 || season == 2 {
                let fruit_color = if season == 1 { [0.9, 0.2, 0.1] } else { [1.0, 0.5, 0.1] };
                let fruit_r = fw * 0.15;
                for fi in 0..4 {
                    let angle = fi as f32 * std::f32::consts::PI * 0.5 + 0.3;
                    let fx = x + angle.cos() * fw * 0.6;
                    let fz = z + angle.sin() * fw * 0.6;
                    let fy = y + th + fh * 0.3 + (fi as f32 * 0.1);
                    push_box(pos, norms, idx, cols, fx, fy, fz, fruit_r, fruit_r, fruit_r, fruit_color[0], fruit_color[1], fruit_color[2], base_idx);
                }
            }
        }
        VegType::DeadTree => {
            let th = size * 0.7;
            let tw = size * 0.05;
            push_box(pos, norms, idx, cols, x, y + th * 0.5, z, tw, th * 0.5, tw, c[0], c[1], c[2], base_idx);
            push_box(pos, norms, idx, cols, x + size * 0.1, y + th * 0.6, z + size * 0.05, tw * 0.5, size * 0.15, tw * 0.5, c[0], c[1], c[2], base_idx);
        }
        VegType::Bush => {
            let s = size * 0.35;
            push_box(pos, norms, idx, cols, x, y + s, z, s, s, s, c[0], c[1], c[2], base_idx);
        }
        VegType::Rock => {
            let s = size * 0.35;
            push_box(pos, norms, idx, cols, x, y + s, z, s * 0.9, s * 0.7, s * 0.8, c[0], c[1], c[2], base_idx);
        }
        VegType::Cactus => {
            let th = size * 0.7;
            let tw = size * 0.05;
            push_box(pos, norms, idx, cols, x, y + th * 0.5, z, tw, th * 0.5, tw, c[0], c[1], c[2], base_idx);
            // arm
            let ah = size * 0.3;
            let aw = size * 0.04;
            push_box(pos, norms, idx, cols, x + size * 0.12, y + th * 0.6 + ah * 0.5, z, aw, ah * 0.5, aw, c[0], c[1], c[2], base_idx);
        }
        VegType::Mushroom => {
            let sh = size * 0.5;
            let sw = size * 0.04;
            push_box(pos, norms, idx, cols, x, y + sh * 0.5, z, sw, sh * 0.5, sw, 0.9, 0.85, 0.75, base_idx);
            let ch = size * 0.35;
            let cw = size * 0.25;
            push_box(pos, norms, idx, cols, x, y + sh + ch * 0.5, z, cw, ch * 0.5, cw, c[0], c[1], c[2], base_idx);
        }
        VegType::Crystal => {
            let s = size * 0.3;
            push_box(pos, norms, idx, cols, x, y + s, z, s * 0.4, s, s * 0.4, c[0], c[1], c[2], base_idx);
        }
        VegType::Flower => {
            let sh = size * 0.4;
            push_box(pos, norms, idx, cols, x, y + sh * 0.5, z, 0.02, sh * 0.5, 0.02, 0.3, 0.6, 0.2, base_idx);
            push_box(pos, norms, idx, cols, x, y + sh + size * 0.08, z, size * 0.08, size * 0.08, size * 0.08, c[0], c[1], c[2], base_idx);
        }
        VegType::Coral => {
            let s = size * 0.25;
            push_box(pos, norms, idx, cols, x, y + s, z, s, s, s, c[0], c[1], c[2], base_idx);
            push_box(pos, norms, idx, cols, x + s * 0.5, y + s * 0.8, z, s * 0.5, s * 0.5, s * 0.5, c[0], c[1], c[2], base_idx);
        }
        VegType::Kelp => {
            let h = size * 0.5;
            let w = size * 0.02;
            push_box(pos, norms, idx, cols, x, y + h, z, w, h, w, c[0], c[1], c[2], base_idx);
        }
        VegType::Seaweed => {
            let h = size * 0.35;
            let w = size * 0.015;
            push_box(pos, norms, idx, cols, x, y + h, z, w, h, w, c[0], c[1], c[2], base_idx);
            push_box(pos, norms, idx, cols, x + w * 2.0, y + h * 0.6, z, w, h * 0.6, w, c[0] * 0.8, c[1] * 1.1, c[2] * 0.8, base_idx);
        }
        VegType::Anemone => {
            let s = size * 0.2;
            push_box(pos, norms, idx, cols, x, y + s * 0.5, z, s * 0.5, s * 0.5, s * 0.5, c[0], c[1], c[2], base_idx);
            // tentacles
            for di in 0..4 {
                let angle = di as f32 * std::f32::consts::PI * 0.5;
                let tx = x + angle.cos() * s * 0.6;
                let tz = z + angle.sin() * s * 0.6;
                push_box(pos, norms, idx, cols, tx, y + s * 0.5 + s * 0.4, tz, s * 0.08, s * 0.4, s * 0.08, c[0] * 0.9, c[1] * 0.8, c[2] * 1.1, base_idx);
            }
        }
        VegType::Sponge => {
            let s = size * 0.2;
            push_box(pos, norms, idx, cols, x, y + s, z, s, s, s, c[0], c[1], c[2], base_idx);
            push_box(pos, norms, idx, cols, x - s * 0.2, y + s * 0.7, z - s * 0.2, s * 0.4, s * 0.4, s * 0.4, c[0] * 0.8, c[1] * 0.8, c[2] * 0.8, base_idx);
        }
    }
}

fn tree_growth_factor(inst: &VegInstance, growth_ticks: u64) -> f32 {
    if inst.veg_type != VegType::Tree { return 1.0; }
    // Derive birth tick from instance position hash
    let h = (inst.x.to_bits() as u64).wrapping_mul(374761393)
        .wrapping_add((inst.z.to_bits() as u64).wrapping_mul(668265263))
        .wrapping_add(inst.variation as u64);
    let birth_tick = (h >> 16) % 20; // 0..19 — tree "plants" at different times
    let age = if growth_ticks > birth_tick { growth_ticks - birth_tick } else { 0 };
    let stage = (age / 3).min(3); // 0=seed, 1=sprout, 2=young, 3=adult
    match stage {
        0 => 0.25,
        1 => 0.45,
        2 => 0.70,
        _ => 1.0,
    }
}

pub fn generate_veg_mesh(params: &WorldParams, cx: i32, cz: i32, season: u8, growth_ticks: u64) -> Option<(Vec<f32>, Vec<f32>, Vec<u32>, Vec<f32>)> {
    let veg = compute_chunk_vegetation(params, cx, cz, season);
    if veg.instances.is_empty() { return None; }
    Some(generate_veg_mesh_from_data(&veg, season, growth_ticks))
}

pub fn generate_veg_mesh_from_data(veg: &VegData, season: u8, growth_ticks: u64) -> (Vec<f32>, Vec<f32>, Vec<u32>, Vec<f32>) {
    let mut pos = Vec::new();
    let mut norms = Vec::new();
    let mut idx = Vec::new();
    let mut cols = Vec::new();
    let mut base_idx = 0u32;
    for inst in &veg.instances {
        let mut gs = inst.size;
        if inst.veg_type == VegType::Tree {
            gs *= tree_growth_factor(inst, growth_ticks);
        }
        emit_veg(inst.veg_type, inst.x, inst.y, inst.z, gs, season, &mut pos, &mut norms, &mut idx, &mut cols, &mut base_idx);
    }
    (pos, norms, idx, cols)
}
