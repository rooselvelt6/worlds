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

fn veg_color(veg_type: VegType) -> [f32; 3] {
    match veg_type {
        VegType::Tree => [0.4, 0.25, 0.1],
        VegType::Bush => [0.2, 0.5, 0.1],
        VegType::Rock => [0.45, 0.38, 0.32],
        VegType::Cactus => [0.2, 0.5, 0.2],
        VegType::Mushroom => [0.85, 0.2, 0.2],
        VegType::Crystal => [0.5, 0.3, 0.85],
        VegType::DeadTree => [0.3, 0.2, 0.1],
        VegType::Flower => [1.0, 0.35, 0.6],
        VegType::Coral => [0.9, 0.4, 0.4],
        VegType::Kelp => [0.1, 0.5, 0.2],
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
    veg_type: VegType, x: f32, y: f32, z: f32, size: f32,
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    base_idx: &mut u32,
) {
    let c = veg_color(veg_type);
    match veg_type {
        VegType::Tree => {
            let th = size * 0.6;
            let tw = size * 0.06;
            let fh = size * 0.5;
            let fw = size * 0.3;
            push_box(pos, norms, idx, cols, x, y + th * 0.5, z, tw, th * 0.5, tw, 0.4, 0.25, 0.1, base_idx);
            push_box(pos, norms, idx, cols, x, y + th + fh * 0.5, z, fw, fh * 0.5, fw, 0.15, 0.45, 0.08, base_idx);
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
        VegType::DeadTree => {
            let th = size * 0.7;
            let tw = size * 0.05;
            push_box(pos, norms, idx, cols, x, y + th * 0.5, z, tw, th * 0.5, tw, c[0], c[1], c[2], base_idx);
            // branch
            push_box(pos, norms, idx, cols, x + size * 0.1, y + th * 0.6, z + size * 0.05, tw * 0.5, size * 0.15, tw * 0.5, c[0], c[1], c[2], base_idx);
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
    }
}

pub fn generate_veg_mesh(params: &WorldParams, cx: i32, cz: i32) -> Option<(Vec<f32>, Vec<f32>, Vec<u32>, Vec<f32>)> {
    let veg = compute_chunk_vegetation(params, cx, cz);
    if veg.instances.is_empty() { return None; }
    let mut pos = Vec::new();
    let mut norms = Vec::new();
    let mut idx = Vec::new();
    let mut cols = Vec::new();
    let mut base_idx = 0u32;
    for inst in &veg.instances {
        emit_veg(inst.veg_type, inst.x, inst.y, inst.z, inst.size, &mut pos, &mut norms, &mut idx, &mut cols, &mut base_idx);
    }
    Some((pos, norms, idx, cols))
}
