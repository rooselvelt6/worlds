use crate::engine::terrain::Zone;

#[derive(Clone)]
pub struct CreatureInstance {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub rot: f64,
    pub creature_type: u8,
    pub speed: f64,
    pub wander_target: Option<(f64, f64)>,
    pub wander_timer: f64,
}

#[derive(Clone)]
pub struct CreatureData {
    pub cx: i32,
    pub cz: i32,
    pub creatures: Vec<CreatureInstance>,
}

pub fn compute_chunk_creatures(params: &crate::state::WorldParams, cx: i32, cz: i32) -> CreatureData {
    let mut rng: u64 = (params.seed as u64).wrapping_mul(6364136223846793005)
        .wrapping_add(cx as u64 * 924839).wrapping_add(cz as u64 * 729384);
    let zone = crate::engine::terrain::get_zone(params, cx as f64 * 24.0 + 12.0, cz as f64 * 24.0 + 12.0);
    let is_underwater = matches!(zone, Zone::CoralReef | Zone::KelpForest | Zone::RockyReef | Zone::SandyPlain | Zone::DeepOcean);
    let creature_types = creature_types_for_zone(zone);
    if creature_types.is_empty() { return CreatureData { cx, cz, creatures: vec![] }; }

    let mut creatures = Vec::new();
    let count = if is_underwater { ((rng >> 16) & 0x7) + 3 } else { ((rng >> 16) & 0x3) + 1 };
    for i in 0..count {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let lx = ((rng >> 16) & 0xFFFF) as f64 / 65536.0 * 24.0;
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let lz = ((rng >> 16) & 0xFFFF) as f64 / 65536.0 * 24.0;
        let wx = cx as f64 * 24.0 + lx;
        let wz = cz as f64 * 24.0 + lz;
        let h = crate::engine::terrain::get_height(params, wx, wz);
        if is_underwater {
            let depth = params.water_level - h;
            if h > params.water_level - 0.1 || depth > 3.0 { continue; }
        } else {
            if h < params.water_level { continue; }
        }

        let ct = creature_types[i as usize % creature_types.len()];
        creatures.push(CreatureInstance {
            id: format!("c{}_{}_{}", cx, cz, i),
            x: wx, y: if is_underwater { params.water_level - 0.5 - (rng >> 8) as f64 * 0.1 } else { h },
            z: wz, rot: 0.0,
            creature_type: ct, speed: 1.0 + (rng & 3) as f64,
            wander_target: None, wander_timer: 0.0,
        });
    }
    CreatureData { cx, cz, creatures }
}

fn creature_types_for_zone(zone: Zone) -> Vec<u8> {
    match zone {
        Zone::Forest => vec![0, 8],
        Zone::Plains => vec![0, 9],
        Zone::Desert => vec![6, 9],
        Zone::Tundra => vec![2, 7],
        Zone::Jungle => vec![2, 1],
        Zone::Volcanic => vec![5],
        Zone::Crystal => vec![3],
        Zone::Cave => vec![4],
        Zone::Fungus => vec![2],
        Zone::Abyss => vec![4],
        Zone::Storm => vec![5],
        Zone::Aurora => vec![3],
        Zone::Magma => vec![5],
        Zone::CoralReef => vec![10, 10, 10, 11],
        Zone::KelpForest => vec![10, 10, 12],
        Zone::RockyReef => vec![10, 11],
        Zone::SandyPlain => vec![11],
        Zone::DeepOcean => vec![12, 12, 11],
        _ => vec![8],
    }
}

fn creature_color_size(ct: u8) -> ([f32; 3], f32) {
    match ct {
        0 => ([0.55, 0.35, 0.15], 0.6),
        1 => ([0.40, 0.25, 0.10], 0.4),
        2 => ([0.20, 0.40, 0.70], 0.35),
        3 => ([0.50, 0.60, 1.00], 0.5),
        4 => ([0.30, 0.15, 0.30], 0.3),
        5 => ([1.00, 0.50, 0.10], 0.4),
        6 => ([0.30, 0.60, 0.20], 0.5),
        7 => ([0.85, 0.82, 0.78], 0.7),
        8 => ([0.70, 0.55, 0.40], 0.3),
        9 => ([0.75, 0.60, 0.40], 0.35),
        10 => ([0.60, 0.60, 0.70], 0.35),
        11 => ([0.80, 0.20, 0.15], 0.3),
        12 => ([0.70, 0.30, 0.70], 0.35),
        _ => ([0.50, 0.50, 0.50], 0.3),
    }
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

fn emit_creature(
    ct: u8, x: f32, y: f32, z: f32,
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    base_idx: &mut u32,
) {
    let (color, size) = creature_color_size(ct);
    let s = size * 0.5;
    let body_h = match ct {
        0 => size * 0.5,  // deer
        3 => size * 0.6,  // crystal
        6 => size * 0.7,  // snake
        9 => size * 0.45, // meerkat
        _ => size * 0.35,
    };
    let body_w = match ct {
        0 => size * 0.2,
        6 => size * 0.08,
        7 => size * 0.3,
        10 => size * 0.15,
        11 => size * 0.25,
        _ => size * 0.18,
    };
    let body_d = match ct {
        0 => size * 0.25,
        6 => size * 0.08,
        7 => size * 0.25,
        10 => size * 0.3,
        _ => size * 0.18,
    };
    // Body
    push_box(pos, norms, idx, cols, x, y + body_h, z, body_w, body_h, body_d, color[0], color[1], color[2], base_idx);
    // Head (most creatures)
    match ct {
        0 | 1 | 2 | 3 | 4 | 7 | 8 | 9 => {
            let hh = s * 0.3;
            push_box(pos, norms, idx, cols, x, y + body_h * 2.0 + hh, z, hh, hh, hh, color[0], color[1], color[2], base_idx);
        }
        5 => {
            // Fire elemental: brighter core
            push_box(pos, norms, idx, cols, x, y + body_h, z, body_w * 0.6, body_h * 0.8, body_w * 0.6, 1.0, 0.8, 0.3, base_idx);
        }
        10 => {
            // Fish tail
            let th = s * 0.2;
            push_box(pos, norms, idx, cols, x, y + body_h * 0.5, z - body_d - th, th * 0.5, th, th * 0.5, color[0], color[1], color[2], base_idx);
        }
        12 => {
            // Jellyfish tentacles
            for ti in 0..3 {
                let tx = x + (ti as f32 - 1.0) * s * 0.2;
                let tz = z;
                let th = s * 0.3;
                push_box(pos, norms, idx, cols, tx, y + th * 0.3, tz, 0.02, th, 0.02, 0.9, 0.5, 0.8, base_idx);
            }
        }
        _ => {}
    }
}

pub fn generate_creature_mesh(params: &crate::state::WorldParams, cx: i32, cz: i32) -> Option<(Vec<f32>, Vec<f32>, Vec<u32>, Vec<f32>)> {
    let data = compute_chunk_creatures(params, cx, cz);
    if data.creatures.is_empty() { return None; }
    let mut pos = Vec::new();
    let mut norms = Vec::new();
    let mut idx = Vec::new();
    let mut cols = Vec::new();
    let mut base_idx = 0u32;
    for c in &data.creatures {
        emit_creature(c.creature_type, c.x as f32, c.y as f32, c.z as f32, &mut pos, &mut norms, &mut idx, &mut cols, &mut base_idx);
    }
    Some((pos, norms, idx, cols))
}

fn push_box_positions(
    pos: &mut Vec<f32>,
    cx: f32, cy: f32, cz: f32, hw: f32, hh: f32, hd: f32,
) {
    let verts: [[f32; 3]; 24] = [
        [ hw, -hh, -hd], [ hw,  hh, -hd], [ hw,  hh,  hd], [ hw, -hh,  hd],
        [-hw, -hh,  hd], [-hw,  hh,  hd], [-hw,  hh, -hd], [-hw, -hh, -hd],
        [-hw,  hh,  hd], [ hw,  hh,  hd], [ hw,  hh, -hd], [-hw,  hh, -hd],
        [-hw, -hh, -hd], [ hw, -hh, -hd], [ hw, -hh,  hd], [-hw, -hh,  hd],
        [-hw, -hh,  hd], [ hw, -hh,  hd], [ hw,  hh,  hd], [-hw,  hh,  hd],
        [ hw, -hh, -hd], [-hw, -hh, -hd], [-hw,  hh, -hd], [ hw,  hh, -hd],
    ];
    for &v in &verts { pos.push(cx + v[0]); pos.push(cy + v[1]); pos.push(cz + v[2]); }
}

fn emit_creature_positions(
    ct: u8, x: f32, y: f32, z: f32,
    pos: &mut Vec<f32>,
) {
    let (_color, size) = creature_color_size(ct);
    let s = size * 0.5;
    let body_h = match ct {
        0 => size * 0.5,
        3 => size * 0.6,
        6 => size * 0.7,
        9 => size * 0.45,
        _ => size * 0.35,
    };
    let body_w = match ct {
        0 => size * 0.2,
        6 => size * 0.08,
        7 => size * 0.3,
        10 => size * 0.15,
        11 => size * 0.25,
        _ => size * 0.18,
    };
    let body_d = match ct {
        0 => size * 0.25,
        6 => size * 0.08,
        7 => size * 0.25,
        10 => size * 0.3,
        _ => size * 0.18,
    };

    // Body
    push_box_positions(pos, x, y + body_h, z, body_w, body_h, body_d);
    // Head or extras
    match ct {
        0 | 1 | 2 | 3 | 4 | 7 | 8 | 9 => {
            let hh = s * 0.3;
            push_box_positions(pos, x, y + body_h * 2.0 + hh, z, hh, hh, hh);
        }
        5 => {
            push_box_positions(pos, x, y + body_h, z, body_w * 0.6, body_h * 0.8, body_w * 0.6);
        }
        10 => {
            let th = s * 0.2;
            push_box_positions(pos, x, y + body_h * 0.5, z - body_d - th, th * 0.5, th, th * 0.5);
        }
        12 => {
            for ti in 0..3 {
                let tx = x + (ti as f32 - 1.0) * s * 0.2;
                let tz = z;
                let th = s * 0.3;
                push_box_positions(pos, tx, y + th * 0.3, tz, 0.02, th, 0.02);
            }
        }
        _ => {}
    }
}

pub fn creature_animated_positions(params: &crate::state::WorldParams, cx: i32, cz: i32, time: f64) -> Option<Vec<f32>> {
    let data = compute_chunk_creatures(params, cx, cz);
    if data.creatures.is_empty() { return None; }
    let mut pos = Vec::new();
    for (i, c) in data.creatures.iter().enumerate() {
        let phase = (cx.wrapping_mul(739).wrapping_add(cz.wrapping_mul(431)) as f64 * 0.1 + i as f64 * 1.7).fract() * std::f64::consts::TAU;
        let bob = (time * 1.8 + phase).sin() * 0.04;
        let y = c.y as f32 + bob as f32;
        emit_creature_positions(c.creature_type, c.x as f32, y, c.z as f32, &mut pos);
    }
    Some(pos)
}
