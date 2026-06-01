use crate::engine::creatures::{push_cylinder_rot, push_ellipsoid_rot};
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
    Plaza,
    Muralla,
    DungeonEntrance,
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
    let zone_name = zone.as_str();
    if let Some(d) = crate::engine::modding::ModContext::with(|ctx| {
        if let Zone::Custom(id) = zone {
            ctx.get_custom_biome(id).map(|b| b.struct_density)
        } else {
            ctx.get_biome_struct_density(zone_name)
        }
    }) { return d; }
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
        Zone::Custom(_) => 0.3,
        _ => 0.0,
    }
}

fn strut_type_from_name(name: &str) -> Option<StructType> {
    match name {
        "hut" => Some(StructType::Hut), "tower" => Some(StructType::Tower),
        "ruins" => Some(StructType::Ruins), "arch" => Some(StructType::Arch),
        "pillar" => Some(StructType::Pillar), "dome" => Some(StructType::Dome),
        "pyramid" => Some(StructType::Pyramid), "crystal_spire" => Some(StructType::CrystalSpire),
        "mushroom_hut" => Some(StructType::MushroomHut), "obelisk" => Some(StructType::Obelisk),
        "plaza" => Some(StructType::Plaza), "muralla" => Some(StructType::Muralla),
        "dungeon_entrance" => Some(StructType::DungeonEntrance),
        _ => None,
    }
}

fn types_for_zone(zone: Zone) -> Vec<StructType> {
    let zone_name = zone.as_str();
    let mod_types = crate::engine::modding::ModContext::with(|ctx| {
        let names: Option<&Vec<String>> = if let Zone::Custom(id) = zone {
            ctx.get_custom_biome(id).map(|b| &b.struct_types)
        } else {
            ctx.get_biome_struct_types(zone_name)
        };
        names.and_then(|n| {
            let types: Vec<StructType> = n.iter().filter_map(|s| strut_type_from_name(s)).collect();
            if types.is_empty() { None } else { Some(types) }
        })
    });
    if let Some(types) = mod_types { return types; }

    match zone {
        Zone::Forest => vec![StructType::Hut, StructType::Ruins, StructType::Tower, StructType::Plaza],
        Zone::Plains => vec![StructType::Hut, StructType::Tower, StructType::Obelisk, StructType::Plaza],
        Zone::Desert => vec![StructType::Pyramid, StructType::Ruins, StructType::Arch, StructType::Plaza],
        Zone::Tundra => vec![StructType::Dome, StructType::Hut, StructType::Pillar, StructType::DungeonEntrance],
        Zone::Jungle => vec![StructType::Pyramid, StructType::Ruins, StructType::Arch, StructType::DungeonEntrance],
        Zone::Volcanic => vec![StructType::Pillar, StructType::Ruins, StructType::Muralla],
        Zone::Crystal => vec![StructType::CrystalSpire, StructType::Arch, StructType::Plaza],
        Zone::Fungus => vec![StructType::MushroomHut, StructType::Arch],
        Zone::Lava => vec![StructType::Pillar, StructType::Obelisk, StructType::Muralla],
        Zone::Abyss => vec![StructType::Pillar, StructType::DungeonEntrance],
        Zone::Storm => vec![StructType::Tower, StructType::Obelisk, StructType::Muralla],
        Zone::Aurora => vec![StructType::CrystalSpire, StructType::Arch, StructType::Plaza],
        Zone::Magma => vec![StructType::Pillar, StructType::Obelisk, StructType::Muralla],
        Zone::Custom(_) => vec![StructType::Hut, StructType::Tower],
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
        StructType::Plaza => 2.0 + r * 2.0,
        StructType::Muralla => 1.5 + r * 1.5,
        StructType::DungeonEntrance => 1.0 + r * 1.0,
    };
    let scale = match zone {
        Zone::Jungle => 1.3,
        Zone::Tundra => 0.7,
        _ => 1.0,
    };
    (base * scale) as f32
}

fn struct_color(st: StructType, cv: u8) -> [f32; 3] {
    let varied = cv as f32 / 5.0;
    match st {
        StructType::Hut => [0.45 + varied * 0.1, 0.25 + varied * 0.1, 0.12 + varied * 0.05],
        StructType::Tower => [0.45 + varied * 0.1, 0.45 + varied * 0.1, 0.45 + varied * 0.1],
        StructType::Ruins => [0.30 + varied * 0.1, 0.30 + varied * 0.1, 0.30 + varied * 0.1],
        StructType::Arch => [0.55 + varied * 0.1, 0.50 + varied * 0.1, 0.45 + varied * 0.1],
        StructType::Pillar => [0.45 + varied * 0.1, 0.45 + varied * 0.1, 0.45 + varied * 0.1],
        StructType::Dome => [0.75 + varied * 0.1, 0.75 + varied * 0.1, 0.75 + varied * 0.1],
        StructType::Pyramid => [0.65 + varied * 0.1, 0.55 + varied * 0.1, 0.35 + varied * 0.1],
        StructType::CrystalSpire => [0.35 + varied * 0.1, 0.45 + varied * 0.1, 0.95 + varied * 0.05],
        StructType::MushroomHut => [0.45 + varied * 0.1, 0.25 + varied * 0.1, 0.15 + varied * 0.05],
        StructType::Obelisk => [0.25 + varied * 0.1, 0.25 + varied * 0.1, 0.25 + varied * 0.1],
        StructType::Plaza => [0.55 + varied * 0.1, 0.50 + varied * 0.1, 0.40 + varied * 0.1],
        StructType::Muralla => [0.40 + varied * 0.1, 0.35 + varied * 0.1, 0.30 + varied * 0.1],
        StructType::DungeonEntrance => [0.30 + varied * 0.1, 0.28 + varied * 0.1, 0.25 + varied * 0.1],
    }
}

fn emit_struct(
    st: StructType, x: f32, y: f32, z: f32, scale: f32, rot: f32, cv: u8,
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    base_idx: &mut u32,
) {
    let s = scale;
    let c = struct_color(st, cv);
    let seg = 6;
    match st {
        StructType::Hut => {
            let hw = s * 0.5; let hh = s * 0.3; let hd = s * 0.5;
            push_cylinder_rot(pos, norms, idx, cols, x, y + hh, z, hw, hh, hd, c[0], c[1], c[2], base_idx, seg, rot);
            let rh = s * 0.2;
            push_ellipsoid_rot(pos, norms, idx, cols, x, y + hh * 2.0 + rh, z, hw * 0.9, rh, hd * 0.9, 0.6, 0.3, 0.15, base_idx, seg, seg, rot);
        }
        StructType::Tower => {
            let hw = s * 0.22; let hh = s * 0.6; let hd = s * 0.22;
            push_cylinder_rot(pos, norms, idx, cols, x, y + hh, z, hw, hh, hd, c[0], c[1], c[2], base_idx, seg, rot);
            let th = s * 0.18;
            push_ellipsoid_rot(pos, norms, idx, cols, x, y + hh * 2.0 + th, z, hw * 0.5, th, hd * 0.5, 0.5, 0.3, 0.15, base_idx, seg, seg, rot);
        }
        StructType::Ruins => {
            let hw = s * 0.4; let hh = s * 0.3; let hd = s * 0.4;
            push_cylinder_rot(pos, norms, idx, cols, x + s * 0.1, y + hh, z - s * 0.05, hw, hh, hd, c[0], c[1], c[2], base_idx, 5, rot);
            push_cylinder_rot(pos, norms, idx, cols, x - s * 0.15, y + hh * 0.6, z + s * 0.1, hw * 0.6, hh * 0.6, hd * 0.6, c[0] * 0.8, c[1] * 0.8, c[2] * 0.8, base_idx, 5, rot);
        }
        StructType::Arch => {
            let pw = s * 0.1; let ph = s * 0.5; let pd = s * 0.15;
            push_cylinder_rot(pos, norms, idx, cols, x - s * 0.3, y + ph, z, pw, ph, pd, c[0], c[1], c[2], base_idx, 6, rot);
            push_cylinder_rot(pos, norms, idx, cols, x + s * 0.3, y + ph, z, pw, ph, pd, c[0], c[1], c[2], base_idx, 6, rot);
            push_ellipsoid_rot(pos, norms, idx, cols, x, y + s, z, s * 0.35, s * 0.06, pd, c[0], c[1], c[2], base_idx, 5, 5, rot);
        }
        StructType::Pillar => {
            let pw = s * 0.15; let ph = s * 0.6; let pd = s * 0.15;
            push_cylinder_rot(pos, norms, idx, cols, x, y + ph, z, pw, ph, pd, c[0], c[1], c[2], base_idx, 8, rot);
        }
        StructType::Dome => {
            let bw = s * 0.45; let bh = s * 0.15; let bd = s * 0.45;
            push_cylinder_rot(pos, norms, idx, cols, x, y + bh, z, bw, bh, bd, c[0], c[1], c[2], base_idx, 8, rot);
            let dw = s * 0.4; let dh = s * 0.25; let dd = s * 0.4;
            push_ellipsoid_rot(pos, norms, idx, cols, x, y + bh * 2.0 + dh, z, dw, dh, dd, c[0] * 0.9, c[1] * 0.9, c[2] * 0.9, base_idx, seg, seg, rot);
        }
        StructType::Pyramid => {
            let bw = s * 0.4; let bh = s * 0.15; let bd = s * 0.4;
            push_cylinder_rot(pos, norms, idx, cols, x, y + bh, z, bw, bh, bd, c[0], c[1], c[2], base_idx, 6, rot);
            let mw = s * 0.28; let mh = s * 0.2; let md = s * 0.28;
            push_cylinder_rot(pos, norms, idx, cols, x, y + bh * 2.0 + mh, z, mw, mh, md, c[0], c[1], c[2], base_idx, 6, rot);
            let tw = s * 0.14; let th = s * 0.25; let td = s * 0.14;
            push_ellipsoid_rot(pos, norms, idx, cols, x, y + bh * 2.0 + mh * 2.0 + th, z, tw, th, td, c[0], c[1], c[2], base_idx, seg, seg, rot);
        }
        StructType::CrystalSpire => {
            let sw = s * 0.07; let sh = s * 0.7; let sd = s * 0.07;
            push_cylinder_rot(pos, norms, idx, cols, x, y + sh, z, sw, sh, sd, c[0], c[1], c[2], base_idx, 6, rot);
            let bw = s * 0.18; let bh = s * 0.05; let bd = s * 0.18;
            push_ellipsoid_rot(pos, norms, idx, cols, x, y + sh * 2.0 - bh, z, bw, bh, bd, c[0] * 1.1, c[1] * 1.1, c[2] * 1.1, base_idx, 5, 5, rot);
        }
        StructType::MushroomHut => {
            let sw = s * 0.07; let sh = s * 0.3; let sd = s * 0.07;
            push_cylinder_rot(pos, norms, idx, cols, x, y + sh, z, sw, sh, sd, c[0], c[1], c[2], base_idx, 6, rot);
            let cw = s * 0.5; let ch = s * 0.15; let cd = s * 0.5;
            push_ellipsoid_rot(pos, norms, idx, cols, x, y + sh * 2.0 + ch, z, cw, ch, cd, 0.75, 0.2, 0.2, base_idx, seg, seg, rot);
        }
        StructType::Obelisk => {
            let ow = s * 0.09; let oh = s * 0.7; let od = s * 0.09;
            push_cylinder_rot(pos, norms, idx, cols, x, y + oh, z, ow, oh, od, c[0], c[1], c[2], base_idx, 6, rot);
            let tw = s * 0.03; let th = s * 0.15; let td = s * 0.03;
            push_ellipsoid_rot(pos, norms, idx, cols, x, y + oh * 2.0 + th, z, tw, th, td, c[0], c[1], c[2], base_idx, 5, 5, rot);
        }
        StructType::Plaza => {
            let pw = s * 0.6; let ph = s * 0.08; let pd = s * 0.6;
            push_cylinder_rot(pos, norms, idx, cols, x, y + ph, z, pw, ph, pd, c[0], c[1], c[2], base_idx, 8, rot);
            let rw = pw * 0.5; let rh = ph * 0.4; let rd = pd * 0.5;
            push_ellipsoid_rot(pos, norms, idx, cols, x, y + ph * 2.0 + rh, z, rw, rh, rd, c[0] * 1.1, c[1] * 1.1, c[2] * 0.9, base_idx, 6, 6, rot);
            let cw = s * 0.07; let ch = s * 0.2; let cd = s * 0.07;
            push_cylinder_rot(pos, norms, idx, cols, x, y + ph * 2.0 + ch, z, cw, ch, cd, c[0] * 1.2, c[1] * 1.2, c[2] * 1.2, base_idx, 6, rot);
        }
        StructType::Muralla => {
            let hw = s * 0.4; let hh = s * 0.25; let hd = s * 0.06;
            push_cylinder_rot(pos, norms, idx, cols, x, y + hh, z, hw, hh, hd, c[0], c[1], c[2], base_idx, 6, rot);
            let ch = s * 0.08; let cw = s * 0.08;
            let num_cren = 3;
            for i in 0..num_cren {
                let cx2 = x - hw + (i as f32 + 0.5) * (hw * 2.0 / num_cren as f32);
                push_cylinder_rot(pos, norms, idx, cols, cx2, y + hh * 2.0 + ch, z, cw, ch, hd, c[0] * 0.9, c[1] * 0.9, c[2] * 0.9, base_idx, 5, rot);
            }
        }
        StructType::DungeonEntrance => {
            let pw = s * 0.15; let ph = s * 0.35; let pd = s * 0.15;
            push_cylinder_rot(pos, norms, idx, cols, x - s * 0.2, y + ph, z, pw, ph, pd, c[0], c[1], c[2], base_idx, 6, rot);
            push_cylinder_rot(pos, norms, idx, cols, x + s * 0.2, y + ph, z, pw, ph, pd, c[0], c[1], c[2], base_idx, 6, rot);
            let ah = s * 0.06;
            push_ellipsoid_rot(pos, norms, idx, cols, x, y + s * 0.85, z, s * 0.18, ah, pd, c[0], c[1], c[2], base_idx, 5, 5, rot);
            for i in 0..3 {
                let step_y = y - (i as f32) * s * 0.06;
                let step_z = z + (i as f32 + 1.0) * s * 0.1;
                push_cylinder_rot(pos, norms, idx, cols, x, step_y, step_z, s * 0.1, s * 0.03, s * 0.05, c[0] * 0.8, c[1] * 0.8, c[2] * 0.8, base_idx, 4, rot);
            }
            push_ellipsoid_rot(pos, norms, idx, cols, x, y + ph * 0.6, z + pd * 0.5, pw * 0.8, ph * 0.6, pd * 0.3, 0.05, 0.05, 0.08, base_idx, 5, 5, rot);
        }
    }
}

pub fn generate_struct_mesh(params: &WorldParams, cx: i32, cz: i32) -> Option<(Vec<f32>, Vec<f32>, Vec<u32>, Vec<f32>)> {
    let data = compute_chunk_structures(params, cx, cz);
    if data.instances.is_empty() { return None; }
    let mut pos = Vec::new();
    let mut norms = Vec::new();
    let mut idx = Vec::new();
    let mut cols = Vec::new();
    let mut base_idx = 0u32;
    for inst in &data.instances {
        emit_struct(inst.struct_type, inst.x, inst.y, inst.z, inst.scale, inst.rotation, inst.color_variation,
            &mut pos, &mut norms, &mut idx, &mut cols, &mut base_idx);
    }
    Some((pos, norms, idx, cols))
}

#[derive(Clone)]
pub struct RoadSegment {
    pub x1: f32, pub y1: f32, pub z1: f32,
    pub x2: f32, pub y2: f32, pub z2: f32,
}

fn push_road_quad(
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32, width: f32,
    r: f32, g: f32, b: f32, base_idx: &mut u32,
) {
    let dx = x2 - x1;
    let dz = z2 - z1;
    let len = (dx * dx + dz * dz).sqrt();
    if len < 0.01 { return; }
    let nx = -dz / len * width * 0.5;
    let nz = dx / len * width * 0.5;
    let thickness = 0.06;
    let verts: [[f32; 3]; 8] = [
        [x1 + nx, y1 + thickness, z1 + nz],
        [x2 + nx, y2 + thickness, z2 + nz],
        [x2 - nx, y2 + thickness, z2 - nz],
        [x1 - nx, y1 + thickness, z1 - nz],
        [x1 + nx, y1, z1 + nz],
        [x2 + nx, y2, z2 + nz],
        [x2 - nx, y2, z2 - nz],
        [x1 - nx, y1, z1 - nz],
    ];
    let norms_data: [[f32; 3]; 8] = [
        [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0],
    ];
    let nv = pos.len() as u32 / 3;
    for &v in &verts { pos.push(v[0]); pos.push(v[1]); pos.push(v[2]); }
    for &n in &norms_data { norms.push(n[0]); norms.push(n[1]); norms.push(n[2]); }
    for _ in 0..8 { cols.push(r); cols.push(g); cols.push(b); }
    // Top quad: 0-1-2, 0-2-3; bottom: 4-6-5, 4-7-6; sides: 0-4-1, 1-4-5, etc.
    let ipat: [u32; 36] = [
        0, 1, 2, 0, 2, 3,
        4, 6, 5, 4, 7, 6,
        0, 4, 1, 1, 4, 5,
        1, 5, 2, 2, 5, 6,
        2, 6, 3, 3, 6, 7,
        3, 7, 0, 0, 7, 4,
    ];
    for &i in &ipat { idx.push(nv + i); }
    *base_idx = nv + 8;
}

pub fn generate_road_mesh(params: &WorldParams, cx: i32, cz: i32) -> Option<(Vec<f32>, Vec<f32>, Vec<u32>, Vec<f32>)> {
    let ox = cx as f64 * crate::engine::chunk::CHUNK_SIZE;
    let oz = cz as f64 * crate::engine::chunk::CHUNK_SIZE;
    let min_dist = 8.0;
    let max_dist = 30.0;
    let mut all_instances = Vec::new();

    // Collect structures from 3x3 chunk area
    for dx in -1..=1 {
        for dz in -1..=1 {
            let data = compute_chunk_structures(params, cx + dx, cz + dz);
            all_instances.extend(data.instances);
        }
    }
    if all_instances.len() < 2 { return None; }

    // Build roads: connect nearest-neighbor pairs within range
    let mut used = vec![false; all_instances.len()];
    let mut segments = Vec::new();
    for i in 0..all_instances.len() {
        if used[i] { continue; }
        let a = &all_instances[i];
        let mut best = max_dist;
        let mut best_j = None;
        for j in (i + 1)..all_instances.len() {
            if used[j] { continue; }
            let b = &all_instances[j];
            let dx = a.x as f64 - b.x as f64;
            let dz = a.z as f64 - b.z as f64;
            let dist = (dx * dx + dz * dz).sqrt();
            if dist >= min_dist && dist < best {
                best = dist;
                best_j = Some(j);
            }
        }
        if let Some(j) = best_j {
            let b = &all_instances[j];
            let mid_x = (a.x + b.x) * 0.5;
            let mid_z = (a.z + b.z) * 0.5;
            // Only keep roads whose midpoint is in this chunk
            if mid_x >= ox as f32 && mid_x < (ox + crate::engine::chunk::CHUNK_SIZE) as f32 &&
               mid_z >= oz as f32 && mid_z < (oz + crate::engine::chunk::CHUNK_SIZE) as f32
            {
                segments.push(RoadSegment {
                    x1: a.x, y1: a.y, z1: a.z,
                    x2: b.x, y2: b.y, z2: b.z,
                });
            }
            used[i] = true;
            used[j] = true;
        }
    }
    if segments.is_empty() { return None; }

    let mut pos = Vec::new();
    let mut norms = Vec::new();
    let mut idx = Vec::new();
    let mut cols = Vec::new();
    let mut base_idx = 0u32;
    let road_color = [0.55, 0.45, 0.3];
    let bridge_color = [0.45, 0.35, 0.25];
    let water_level = params.water_level as f32;
    for seg in &segments {
        let mid_x = (seg.x1 + seg.x2) * 0.5;
        let mid_z = (seg.z1 + seg.z2) * 0.5;
        // Check if this road segment crosses a river
        if terrain::is_river(params, mid_x as f64, mid_z as f64) {
            // Bridge deck
            let dx = seg.x2 - seg.x1;
            let dz = seg.z2 - seg.z1;
            let len = (dx * dx + dz * dz).sqrt().max(0.01);
            let nx = -dz / len;
            let nz = dx / len;
            let bridge_y = water_level + 0.5;
            push_cylinder_rot(&mut pos, &mut norms, &mut idx, &mut cols,
                mid_x, bridge_y + 0.04, mid_z,
                0.5, 0.04, 0.5,
                bridge_color[0], bridge_color[1], bridge_color[2], &mut base_idx, 8, 0.0);
            // Railings
            let rail_h = 0.25;
            let rail_w = 0.03;
            let r_offset = 0.45;
            push_cylinder_rot(&mut pos, &mut norms, &mut idx, &mut cols,
                mid_x + nx * r_offset, bridge_y + 0.08 + rail_h, mid_z + nz * r_offset,
                rail_w, rail_h, rail_w,
                bridge_color[0] * 0.8, bridge_color[1] * 0.8, bridge_color[2] * 0.8, &mut base_idx, 5, 0.0);
            push_cylinder_rot(&mut pos, &mut norms, &mut idx, &mut cols,
                mid_x - nx * r_offset, bridge_y + 0.08 + rail_h, mid_z - nz * r_offset,
                rail_w, rail_h, rail_w,
                bridge_color[0] * 0.8, bridge_color[1] * 0.8, bridge_color[2] * 0.8, &mut base_idx, 5, 0.0);
            // Bridge supports at ends
            let sup_h = (bridge_y - seg.y1).max(0.1);
            push_cylinder_rot(&mut pos, &mut norms, &mut idx, &mut cols,
                seg.x1, seg.y1 + sup_h * 0.5, seg.z1,
                0.05, sup_h * 0.5, 0.05,
                bridge_color[0], bridge_color[1], bridge_color[2], &mut base_idx, 5, 0.0);
            let sup_h2 = (bridge_y - seg.y2).max(0.1);
            push_cylinder_rot(&mut pos, &mut norms, &mut idx, &mut cols,
                seg.x2, seg.y2 + sup_h2 * 0.5, seg.z2,
                0.05, sup_h2 * 0.5, 0.05,
                bridge_color[0], bridge_color[1], bridge_color[2], &mut base_idx, 5, 0.0);
        } else {
                    push_road_quad(
                        &mut pos, &mut norms, &mut idx, &mut cols,
                        seg.x1, seg.y1 + 0.1, seg.z1,
                        seg.x2, seg.y2 + 0.1, seg.z2,
                        0.8, road_color[0], road_color[1], road_color[2], &mut base_idx,
                    );
                }
            }
            Some((pos, norms, idx, cols))
        }

pub fn generate_blueprint_mesh(params: &WorldParams, cx: i32, cz: i32) -> Option<(Vec<f32>, Vec<f32>, Vec<u32>, Vec<f32>)> {
    use crate::engine::modding::ModContext;
    let has_blueprints = ModContext::with(|ctx| ctx.active && !ctx.blueprints.is_empty());
    if !has_blueprints { return None; }

    let ox = cx as f64 * crate::engine::chunk::CHUNK_SIZE;
    let oz = cz as f64 * crate::engine::chunk::CHUNK_SIZE;
    let seed = params.seed.wrapping_mul(2654435761)
        .wrapping_add((cx as u32).wrapping_mul(374761393))
        .wrapping_add((cz as u32).wrapping_mul(668265263));
    let water = params.water_level;

    let count = if ModContext::with(|ctx| ctx.blueprints.len()) > 0 { 2 } else { 0 };
    let mut placed = false;
    let mut pos = Vec::new();
    let mut norms = Vec::new();
    let mut idx = Vec::new();
    let mut cols = Vec::new();
    let mut base_idx = 0u32;
    let mut rng = seed;

    ModContext::with(|ctx| {
        let bp_names: Vec<String> = ctx.blueprints.keys().cloned().collect();
        if bp_names.is_empty() { return; }

        for _ in 0..count {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            let rx = (rng as f64 / u32::MAX as f64) * crate::engine::chunk::CHUNK_SIZE;
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            let rz = (rng as f64 / u32::MAX as f64) * crate::engine::chunk::CHUNK_SIZE;

            let wx = ox + rx;
            let wz = oz + rz;
            let h = terrain::get_height(params, wx, wz);
            if h <= water + 0.5 { continue; }

            let mut flat = true;
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let sh = terrain::get_height(params, wx + dx as f64, wz + dy as f64);
                    if (sh - h).abs() > 1.0 { flat = false; }
                }
            }
            if !flat { continue; }

            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            let idx2 = (rng as f64 / u32::MAX as f64 * bp_names.len() as f64) as usize;
            let name = &bp_names[idx2.min(bp_names.len() - 1)];
            if let Some(bp) = ctx.get_blueprint(name) {
                for block in &bp.blocks {
                    push_ellipsoid_rot(&mut pos, &mut norms, &mut idx, &mut cols,
                        wx as f32 + block.x,
                        h as f32 + block.y,
                        wz as f32 + block.z,
                        block.w * 0.5, block.h * 0.5, block.d * 0.5,
                        block.color[0], block.color[1], block.color[2],
                        &mut base_idx, 4, 4, 0.0);
                }
                placed = true;
            }
        }
    });

    if placed { Some((pos, norms, idx, cols)) } else { None }
}
