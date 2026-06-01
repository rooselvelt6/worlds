use crate::engine::terrain::Zone;
use crate::state::WorldParams;
use std::collections::{BinaryHeap, HashMap};

pub const STATE_IDLE: u8 = 0;
pub const STATE_WANDER: u8 = 1;
pub const STATE_FLEE: u8 = 2;
pub const STATE_FOLLOW: u8 = 3;
pub const STATE_EAT: u8 = 4;

pub const ANIM_IDLE: u8 = 0;
pub const ANIM_WALK: u8 = 1;
pub const ANIM_RUN: u8 = 2;
pub const ANIM_ATTACK: u8 = 3;

const SEG: u32 = 6;
const SEG2: u32 = 4;

#[derive(Clone)]
pub struct CreatureInstance {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub rot: f64,
    pub creature_type: u8,
    pub speed: f64,
    pub state: u8,
    pub state_timer: f64,
    pub path: Vec<(f64, f64)>,
    pub path_index: usize,
    pub tamed: bool,
    pub hunger: f64,
    pub wander_target: Option<(f64, f64)>,
    pub wander_timer: f64,
    pub anim_state: u8,
    pub anim_time: f64,
    pub mounted: bool,
    pub rescue_reward: u8,
}

#[derive(Clone)]
pub struct CreatureData {
    pub cx: i32,
    pub cz: i32,
    pub creatures: Vec<CreatureInstance>,
}

pub fn compute_chunk_creatures(params: &crate::state::WorldParams, cx: i32, cz: i32) -> CreatureData {
    compute_chunk_creatures_with_time(params, cx, cz, -1.0)
}

pub fn compute_chunk_creatures_with_time(params: &crate::state::WorldParams, cx: i32, cz: i32, day_time: f64) -> CreatureData {
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

        // Conditional spawn by time of day
        if day_time >= 0.0 {
            let is_night = day_time.sin() < 0.0;
            let is_dusk = (day_time.sin() - 0.1).abs() < 0.3;
            match ct {
                4 | 15 => if !is_night && !is_dusk { continue; }
                13 | 14 => if is_night { continue; }
                _ => {}
            }
        }

        let y_pos = if ct == 13 || ct == 14 || ct == 15 {
            let base_y = if is_underwater { params.water_level } else { h };
            base_y + match ct {
                13 => 0.8 + (rng >> 4) as f64 * 0.5,
                14 => 4.0 + (rng >> 4) as f64 * 3.0,
                15 => 0.3 + (rng >> 4) as f64 * 0.5,
                _ => 0.0,
            }
        } else if is_underwater {
            params.water_level - 0.5 - (rng >> 8) as f64 * 0.1
        } else {
            h
        };
        let speed = 1.0 + (rng & 3) as f64;
        let rescue = if ct == 0 || ct == 6 || ct == 8 { ((rng >> 8) & 0xFF) as u8 } else { 0 };
        creatures.push(CreatureInstance {
            id: format!("c{}_{}_{}", cx, cz, i),
            x: wx, y: y_pos,
            z: wz, rot: 0.0,
            creature_type: ct, speed,
            state: STATE_IDLE,
            state_timer: 1.0 + ((rng >> 8) & 0xFF) as f64 * 0.05,
            path: vec![], path_index: 0,
            tamed: false,
            hunger: 80.0 + ((rng >> 16) & 0x1F) as f64,
            wander_target: None, wander_timer: 0.0,
            anim_state: ANIM_IDLE, anim_time: 0.0,
            mounted: false,
            rescue_reward: rescue,
        });
    }
    CreatureData { cx, cz, creatures }
}

fn creature_types_for_zone(zone: Zone) -> Vec<u8> {
    match zone {
        Zone::Forest => vec![0, 8, 13, 13, 14],
        Zone::Plains => vec![0, 9, 13, 14],
        Zone::Desert => vec![6, 9, 14],
        Zone::Tundra => vec![2, 7, 14],
        Zone::Jungle => vec![2, 1, 13, 13, 13, 14],
        Zone::Volcanic => vec![5, 15],
        Zone::Crystal => vec![3, 15],
        Zone::Cave => vec![4, 15, 15],
        Zone::Fungus => vec![2, 15, 15],
        Zone::Abyss => vec![4, 15],
        Zone::Storm => vec![5, 14],
        Zone::Aurora => vec![3, 15],
        Zone::Magma => vec![5],
        Zone::CoralReef => vec![10, 10, 10, 11],
        Zone::KelpForest => vec![10, 10, 12],
        Zone::RockyReef => vec![10, 11],
        Zone::SandyPlain => vec![11],
        Zone::DeepOcean => vec![12, 12, 11],
        _ => vec![8, 13],
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
        13 => ([1.00, 0.60, 0.10], 0.18),
        14 => ([0.30, 0.30, 0.30], 0.40),
        15 => ([0.80, 0.90, 0.20], 0.15),
        _ => ([0.50, 0.50, 0.50], 0.3),
    }
}

// ── Organic mesh helpers ──

fn push_ellipsoid_impl(
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    cx: f32, cy: f32, cz: f32, rx: f32, ry: f32, rz: f32,
    r: f32, g: f32, b: f32, base_idx: &mut u32,
    seg_lat: u32, seg_lon: u32,
    rot_y: f32,
) {
    let start = *base_idx;
    let (sin_rot, cos_rot) = rot_y.sin_cos();
    for j in 0..=seg_lat {
        let theta = j as f32 * std::f32::consts::PI / seg_lat as f32;
        let st = theta.sin();
        let ct = theta.cos();
        for i in 0..=seg_lon {
            let phi = i as f32 * 2.0 * std::f32::consts::PI / seg_lon as f32;
            let sp = phi.sin();
            let cp = phi.cos();
            let nx = st * cp;
            let ny = ct;
            let nz = st * sp;
            let dx = nx * rx;
            let dy = ny * ry;
            let dz = nz * rz;
            let rdx = dx * cos_rot - dz * sin_rot;
            let rdz = dx * sin_rot + dz * cos_rot;
            pos.push(cx + rdx);
            pos.push(cy + dy);
            pos.push(cz + rdz);
            let rnx = nx * cos_rot - nz * sin_rot;
            let rnz = nx * sin_rot + nz * cos_rot;
            norms.push(rnx); norms.push(ny); norms.push(rnz);
            cols.push(r); cols.push(g); cols.push(b);
        }
    }
    for j in 0..seg_lat {
        for i in 0..seg_lon {
            let a = start + j * (seg_lon + 1) + i;
            let b = a + seg_lon + 1;
            idx.push(a); idx.push(b); idx.push(a + 1);
            idx.push(b); idx.push(b + 1); idx.push(a + 1);
        }
    }
    *base_idx += (seg_lat + 1) * (seg_lon + 1);
}

fn push_ellipsoid(
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    cx: f32, cy: f32, cz: f32, rx: f32, ry: f32, rz: f32,
    r: f32, g: f32, b: f32, base_idx: &mut u32,
    seg_lat: u32, seg_lon: u32,
) {
    push_ellipsoid_impl(pos, norms, idx, cols, cx, cy, cz, rx, ry, rz, r, g, b, base_idx, seg_lat, seg_lon, 0.0);
}

pub(crate) fn push_ellipsoid_rot(
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    cx: f32, cy: f32, cz: f32, rx: f32, ry: f32, rz: f32,
    r: f32, g: f32, b: f32, base_idx: &mut u32,
    seg_lat: u32, seg_lon: u32,
    rot_y: f32,
) {
    push_ellipsoid_impl(pos, norms, idx, cols, cx, cy, cz, rx, ry, rz, r, g, b, base_idx, seg_lat, seg_lon, rot_y);
}

fn push_ellipsoid_pos_impl(
    pos: &mut Vec<f32>,
    cx: f32, cy: f32, cz: f32, rx: f32, ry: f32, rz: f32,
    seg_lat: u32, seg_lon: u32,
    rot_y: f32,
) {
    let (sin_rot, cos_rot) = rot_y.sin_cos();
    for j in 0..=seg_lat {
        let theta = j as f32 * std::f32::consts::PI / seg_lat as f32;
        let st = theta.sin();
        let ct = theta.cos();
        for i in 0..=seg_lon {
            let phi = i as f32 * 2.0 * std::f32::consts::PI / seg_lon as f32;
            let sp = phi.sin();
            let cp = phi.cos();
            let nx = st * cp;
            let nz = st * sp;
            let dx = nx * rx;
            let dz = nz * rz;
            let rdx = dx * cos_rot - dz * sin_rot;
            let rdz = dx * sin_rot + dz * cos_rot;
            pos.push(cx + rdx);
            pos.push(cy + ct * ry);
            pos.push(cz + rdz);
        }
    }
}

fn push_ellipsoid_pos(
    pos: &mut Vec<f32>,
    cx: f32, cy: f32, cz: f32, rx: f32, ry: f32, rz: f32,
    seg_lat: u32, seg_lon: u32,
) {
    push_ellipsoid_pos_impl(pos, cx, cy, cz, rx, ry, rz, seg_lat, seg_lon, 0.0);
}

fn push_cylinder_impl(
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    cx: f32, cy: f32, cz: f32, rx: f32, ry: f32, rz: f32,
    r: f32, g: f32, b: f32, base_idx: &mut u32,
    seg: u32,
    rot_y: f32,
) {
    let start = *base_idx;
    let (sin_rot, cos_rot) = rot_y.sin_cos();
    // Side vertices (bottom ring then top ring)
    for i in 0..=seg {
        let phi = i as f32 * 2.0 * std::f32::consts::PI / seg as f32;
        let sp = phi.sin();
        let cp = phi.cos();
        let dx = cp * rx;
        let dz = sp * rz;
        let rdx = dx * cos_rot - dz * sin_rot;
        let rdz = dx * sin_rot + dz * cos_rot;
        let rnx = cp * cos_rot - sp * sin_rot;
        let rnz = cp * sin_rot + sp * cos_rot;
        // Bottom
        pos.push(cx + rdx); pos.push(cy - ry); pos.push(cz + rdz);
        norms.push(rnx); norms.push(0.0); norms.push(rnz);
        cols.push(r); cols.push(g); cols.push(b);
        // Top
        pos.push(cx + rdx); pos.push(cy + ry); pos.push(cz + rdz);
        norms.push(rnx); norms.push(0.0); norms.push(rnz);
        cols.push(r); cols.push(g); cols.push(b);
    }
    // Bottom cap center
    pos.push(cx); pos.push(cy - ry); pos.push(cz);
    norms.push(0.0); norms.push(-1.0); norms.push(0.0);
    cols.push(r); cols.push(g); cols.push(b);
    // Top cap center
    pos.push(cx); pos.push(cy + ry); pos.push(cz);
    norms.push(0.0); norms.push(1.0); norms.push(0.0);
    cols.push(r); cols.push(g); cols.push(b);

    let side = start;
    let bot_cap = start + (seg + 1) * 2;
    let top_cap = bot_cap + 1;
    // Side quads
    for i in 0..seg {
        let a = side + i * 2;
        let b = a + 2;
        idx.push(a); idx.push(b); idx.push(a + 1);
        idx.push(b); idx.push(b + 1); idx.push(a + 1);
    }
    // Bottom cap triangles
    for i in 0..seg {
        let a = side + i * 2;
        let na = (i + 1) % seg;
        let b = side + na * 2;
        idx.push(bot_cap); idx.push(b); idx.push(a);
    }
    // Top cap triangles
    for i in 0..seg {
        let a = side + i * 2 + 1;
        let na = (i + 1) % seg;
        let b = side + na * 2 + 1;
        idx.push(top_cap); idx.push(a); idx.push(b);
    }
    *base_idx += (seg + 1) * 2 + 2;
}

fn push_cylinder(
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    cx: f32, cy: f32, cz: f32, rx: f32, ry: f32, rz: f32,
    r: f32, g: f32, b: f32, base_idx: &mut u32,
    seg: u32,
) {
    push_cylinder_impl(pos, norms, idx, cols, cx, cy, cz, rx, ry, rz, r, g, b, base_idx, seg, 0.0);
}

pub(crate) fn push_cylinder_rot(
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    cx: f32, cy: f32, cz: f32, rx: f32, ry: f32, rz: f32,
    r: f32, g: f32, b: f32, base_idx: &mut u32,
    seg: u32,
    rot_y: f32,
) {
    push_cylinder_impl(pos, norms, idx, cols, cx, cy, cz, rx, ry, rz, r, g, b, base_idx, seg, rot_y);
}

fn push_cylinder_pos_impl(
    pos: &mut Vec<f32>,
    cx: f32, cy: f32, cz: f32, rx: f32, ry: f32, rz: f32,
    seg: u32,
    rot_y: f32,
) {
    let (sin_rot, cos_rot) = rot_y.sin_cos();
    for i in 0..=seg {
        let phi = i as f32 * 2.0 * std::f32::consts::PI / seg as f32;
        let sp = phi.sin();
        let cp = phi.cos();
        let dx = cp * rx;
        let dz = sp * rz;
        let rdx = dx * cos_rot - dz * sin_rot;
        let rdz = dx * sin_rot + dz * cos_rot;
        pos.push(cx + rdx); pos.push(cy - ry); pos.push(cz + rdz);
        pos.push(cx + rdx); pos.push(cy + ry); pos.push(cz + rdz);
    }
    pos.push(cx); pos.push(cy - ry); pos.push(cz);
    pos.push(cx); pos.push(cy + ry); pos.push(cz);
}

fn push_cylinder_pos(
    pos: &mut Vec<f32>,
    cx: f32, cy: f32, cz: f32, rx: f32, ry: f32, rz: f32,
    seg: u32,
) {
    push_cylinder_pos_impl(pos, cx, cy, cz, rx, ry, rz, seg, 0.0);
}

// ── Creature mesh generation ──

fn emit_creature(
    ct: u8, x: f32, y: f32, z: f32,
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    base_idx: &mut u32,
) {
    let (color, size) = creature_color_size(ct);
    let s = size * 0.5;
    match ct {
        13 => {
            // Butterfly: 4 thin wings + body
            let ws = size * 0.35;
            let wt = 0.008;
            let by = y + size * 0.12;
            // Upper-left wing
            push_ellipsoid(pos, norms, idx, cols, x - ws * 0.35, by, z + wt, ws * 0.4, ws * 0.25, wt, color[0]*1.2, color[1]*0.7, color[2]*0.2, base_idx, SEG2, SEG2);
            // Upper-right wing
            push_ellipsoid(pos, norms, idx, cols, x + ws * 0.35, by, z - wt, ws * 0.4, ws * 0.25, wt, color[0]*0.9, color[1]*0.6, color[2]*0.3, base_idx, SEG2, SEG2);
            // Lower-left wing
            push_ellipsoid(pos, norms, idx, cols, x - ws * 0.2, by - ws * 0.1, z + wt, ws * 0.25, ws * 0.15, wt, color[0]*0.8, color[1]*0.5, color[2]*0.5, base_idx, SEG2, SEG2);
            // Lower-right wing
            push_ellipsoid(pos, norms, idx, cols, x + ws * 0.2, by - ws * 0.1, z - wt, ws * 0.25, ws * 0.15, wt, color[0]*1.1, color[1]*0.8, color[2]*0.1, base_idx, SEG2, SEG2);
            // Body
            push_ellipsoid(pos, norms, idx, cols, x, by, z, 0.015, 0.035, 0.04, 0.2, 0.15, 0.1, base_idx, 4, 4);
            return;
        }
        14 => {
            // Bird: ellipsoid body + swept wings
            let by = y + size * 0.18;
            let ws = size * 0.3;
            // Body
            push_ellipsoid(pos, norms, idx, cols, x, by + size * 0.05, z, 0.04, 0.05, 0.08, color[0], color[1], color[2], base_idx, SEG2, SEG2);
            // Left wing
            push_ellipsoid(pos, norms, idx, cols, x - ws * 0.4, by + size * 0.08, z, ws * 0.5, 0.015, 0.04, color[0], color[1], color[2], base_idx, SEG2, 4);
            // Right wing
            push_ellipsoid(pos, norms, idx, cols, x + ws * 0.4, by + size * 0.08, z, ws * 0.5, 0.015, 0.04, color[0], color[1], color[2], base_idx, SEG2, 4);
            // Head
            push_ellipsoid(pos, norms, idx, cols, x, by + size * 0.15, z + 0.05, 0.025, 0.025, 0.025, color[0], color[1], color[2], base_idx, 4, 4);
            return;
        }
        15 => {
            // Firefly: glowing ellipsoid
            let gs = 0.03;
            push_ellipsoid(pos, norms, idx, cols, x, y + gs, z, gs, gs*0.4, gs, color[0], color[1], color[2], base_idx, 4, 4);
            push_ellipsoid(pos, norms, idx, cols, x, y + gs * 0.8, z, gs*0.3, gs*0.3, gs*0.3, 1.0, 1.0, 0.6, base_idx, 4, 4);
            return;
        }
        _ => {}
    }
    let body_h = match ct {
        0 => size * 0.45,
        3 => size * 0.55,
        6 => size * 0.65,
        9 => size * 0.42,
        _ => size * 0.32,
    };
    let body_w = match ct {
        0 => size * 0.18,
        6 => size * 0.07,
        7 => size * 0.28,
        10 => size * 0.12,
        11 => size * 0.22,
        _ => size * 0.16,
    };
    let body_d = match ct {
        0 => size * 0.22,
        6 => size * 0.07,
        7 => size * 0.22,
        10 => size * 0.28,
        _ => size * 0.16,
    };
    // Body
    push_ellipsoid(pos, norms, idx, cols, x, y + body_h, z, body_w, body_h, body_d, color[0], color[1], color[2], base_idx, SEG, SEG);
    match ct {
        0 | 1 | 2 | 3 | 4 | 7 | 8 | 9 => {
            // Head
            let hh = s * 0.28;
            push_ellipsoid(pos, norms, idx, cols, x, y + body_h * 1.8 + hh, z, hh*0.9, hh, hh*0.9, color[0], color[1], color[2], base_idx, SEG2, SEG2);
            // Neck
            push_cylinder(pos, norms, idx, cols, x, y + body_h * 1.3, z, body_w*0.4, body_h*0.3, body_d*0.4, color[0]*0.9, color[1]*0.9, color[2]*0.9, base_idx, 4);
            // Four legs
            let lw = body_w * 0.25;
            let lh = body_h * 0.55;
            let ld = body_d * 0.25;
            let leg_color = [color[0] * 0.85, color[1] * 0.85, color[2] * 0.85];
            let leg_off = body_w * 0.5;
            let leg_off_z = body_d * 0.5;
            push_cylinder(pos, norms, idx, cols, x - leg_off, y + lh, z + leg_off_z, lw, lh, ld, leg_color[0], leg_color[1], leg_color[2], base_idx, 4);
            push_cylinder(pos, norms, idx, cols, x + leg_off, y + lh, z + leg_off_z, lw, lh, ld, leg_color[0], leg_color[1], leg_color[2], base_idx, 4);
            push_cylinder(pos, norms, idx, cols, x - leg_off, y + lh, z - leg_off_z, lw, lh, ld, leg_color[0], leg_color[1], leg_color[2], base_idx, 4);
            push_cylinder(pos, norms, idx, cols, x + leg_off, y + lh, z - leg_off_z, lw, lh, ld, leg_color[0], leg_color[1], leg_color[2], base_idx, 4);
            // Tail for fox (8)
            if ct == 8 {
                let tl = s * 0.4;
                push_ellipsoid(pos, norms, idx, cols, x, y + body_h * 0.5, z - body_d - tl*0.5, 0.03, 0.03, tl*0.5, color[0]*1.1, color[1]*0.9, color[2]*0.7, base_idx, 4, 4);
            }
            // Antlers for deer (0)
            if ct == 0 {
                let ah = s * 0.25;
                push_cylinder(pos, norms, idx, cols, x - 0.04, y + body_h * 1.8 + hh + ah*0.5, z - 0.02, 0.008, ah, 0.008, 0.5, 0.35, 0.15, base_idx, 4);
                push_cylinder(pos, norms, idx, cols, x + 0.04, y + body_h * 1.8 + hh + ah*0.5, z - 0.02, 0.008, ah, 0.008, 0.5, 0.35, 0.15, base_idx, 4);
            }
            // Wings for bat (4)
            if ct == 4 {
                let ws = s * 0.4;
                push_ellipsoid(pos, norms, idx, cols, x - ws*0.4, y + body_h*0.8, z, ws*0.5, 0.015, s*0.15, 0.25, 0.15, 0.25, base_idx, SEG2, 4);
                push_ellipsoid(pos, norms, idx, cols, x + ws*0.4, y + body_h*0.8, z, ws*0.5, 0.015, s*0.15, 0.25, 0.15, 0.25, base_idx, SEG2, 4);
            }
        }
        5 => {
            // Fire elemental: brighter inner core
            push_ellipsoid(pos, norms, idx, cols, x, y + body_h, z, body_w*0.4, body_h*0.7, body_w*0.4, 1.0, 0.8, 0.3, base_idx, SEG, SEG);
        }
        6 => {
            // Snake: body segments along Z
            let segs = 5;
            for si in 0..segs {
                let t = si as f32 / (segs - 1) as f32;
                let sz = z + (t - 0.5) * s * 0.8;
                let sy = y + body_h + (t - 0.5).sin() * 0.03;
                let sr = body_w * (1.0 - 0.3 * t);
                push_ellipsoid(pos, norms, idx, cols, x, sy, sz, sr, sr, body_d*0.3, color[0], color[1], color[2], base_idx, SEG2, SEG2);
            }
            // Head
            push_ellipsoid(pos, norms, idx, cols, x, y + body_h + 0.02, z + s*0.45, body_w*0.6, body_w*0.6, body_d*0.15, color[0], color[1], color[2], base_idx, 4, 4);
        }
        10 => {
            // Fish: streamlined body + tail
            push_ellipsoid(pos, norms, idx, cols, x, y + body_h, z, body_w, body_h, body_d, color[0], color[1], color[2], base_idx, SEG, SEG);
            // Tail fin
            let tf = s * 0.25;
            push_ellipsoid(pos, norms, idx, cols, x, y + body_h*0.5, z - body_d - tf*0.4, tf*0.3, tf*0.4, tf*0.3, color[0], color[1], color[2], base_idx, 4, 4);
            // Dorsal fin
            push_ellipsoid(pos, norms, idx, cols, x, y + body_h*1.3, z, 0.01, body_h*0.3, body_d*0.3, color[0]*0.8, color[1]*0.8, color[2]*0.8, base_idx, 4, 4);
        }
        11 => {
            // Crab: flat body + legs
            push_ellipsoid(pos, norms, idx, cols, x, y + body_h, z, body_w, body_h*0.5, body_d, color[0], color[1], color[2], base_idx, SEG, SEG);
            // 6 legs
            for li in 0..3 {
                let la = li as f32 * 0.4 - 0.4;
                let lx = (la * 0.5).sin() * body_w * 0.6;
                let lz = body_d * 0.5 + li as f32 * body_d * 0.15;
                let ll = s * 0.2;
                push_cylinder(pos, norms, idx, cols, x - body_w*0.3 + lx, y + ll, z + lz, 0.015, ll, 0.015, color[0]*0.9, color[1]*0.9, color[2]*0.9, base_idx, 4);
                push_cylinder(pos, norms, idx, cols, x + body_w*0.3 - lx, y + ll, z + lz, 0.015, ll, 0.015, color[0]*0.9, color[1]*0.9, color[2]*0.9, base_idx, 4);
            }
            // Claws
            push_ellipsoid(pos, norms, idx, cols, x - body_w*0.8, y + body_h*0.3, z + body_d*0.6, 0.035, 0.02, 0.02, color[0]*0.7, color[1]*0.2, color[2]*0.1, base_idx, 4, 4);
            push_ellipsoid(pos, norms, idx, cols, x + body_w*0.8, y + body_h*0.3, z + body_d*0.6, 0.035, 0.02, 0.02, color[0]*0.7, color[1]*0.2, color[2]*0.1, base_idx, 4, 4);
        }
        12 => {
            // Jellyfish: dome + tentacles
            push_ellipsoid(pos, norms, idx, cols, x, y + body_h, z, body_w*0.8, body_h*0.4, body_w*0.8, color[0], color[1], color[2], base_idx, SEG, SEG);
            // Dome top
            push_ellipsoid(pos, norms, idx, cols, x, y + body_h*1.2, z, body_w*0.6, body_h*0.3, body_w*0.6, 0.9, 0.5, 0.8, base_idx, SEG, SEG);
            // Tentacles
            for ti in 0..5 {
                let ta = ti as f32 * 1.256;
                let tx = x + ta.cos() * body_w * 0.5;
                let tz = z + ta.sin() * body_w * 0.5;
                let tt = s * 0.3;
                push_cylinder(pos, norms, idx, cols, tx, y + tt*0.3, tz, 0.012, tt, 0.012, 0.9, 0.4, 0.7, base_idx, 4);
            }
        }
        _ => {}
    }
}

fn emit_creature_positions(
    ct: u8, x: f32, y: f32, z: f32,
    pos: &mut Vec<f32>,
    anim_state: u8, anim_time: f32,
) {
    let (_color, size) = creature_color_size(ct);
    let s = size * 0.5;
    let (leg_amp, leg_speed) = match anim_state {
        ANIM_RUN => (0.15, 8.0),
        ANIM_WALK => (0.08, 4.5),
        ANIM_ATTACK => (0.04, 2.0),
        _ => (0.01, 0.5),
    };
    let t = anim_time * leg_speed;
    match ct {
        13 => {
            // Butterfly
            let ws = size * 0.35;
            let wt = 0.008;
            let by = y + size * 0.12;
            let wing_angle = t.sin() * 0.15;
            push_ellipsoid_pos(pos, x - ws * 0.35 + wing_angle*0.3, by + wing_angle*0.1, z + wt, ws * 0.4, ws * 0.25, wt, SEG2, SEG2);
            push_ellipsoid_pos(pos, x + ws * 0.35 - wing_angle*0.3, by - wing_angle*0.1, z - wt, ws * 0.4, ws * 0.25, wt, SEG2, SEG2);
            push_ellipsoid_pos(pos, x - ws * 0.2 + wing_angle*0.2, by - ws * 0.1 + wing_angle*0.05, z + wt, ws * 0.25, ws * 0.15, wt, SEG2, SEG2);
            push_ellipsoid_pos(pos, x + ws * 0.2 - wing_angle*0.2, by - ws * 0.1 - wing_angle*0.05, z - wt, ws * 0.25, ws * 0.15, wt, SEG2, SEG2);
            push_ellipsoid_pos(pos, x, by, z, 0.015, 0.035, 0.04, 4, 4);
            return;
        }
        14 => {
            // Bird
            let by = y + size * 0.18;
            let ws = size * 0.3;
            let wing_flap = t.sin() * 0.04;
            push_ellipsoid_pos(pos, x, by + size * 0.05, z, 0.04, 0.05, 0.08, SEG2, SEG2);
            push_ellipsoid_pos(pos, x - ws * 0.4 - wing_flap*0.5, by + size * 0.08 + wing_flap*0.3, z, ws * 0.5, 0.015, 0.04, SEG2, 4);
            push_ellipsoid_pos(pos, x + ws * 0.4 + wing_flap*0.5, by + size * 0.08 - wing_flap*0.3, z, ws * 0.5, 0.015, 0.04, SEG2, 4);
            push_ellipsoid_pos(pos, x, by + size * 0.15, z + 0.05, 0.025, 0.025, 0.025, 4, 4);
            return;
        }
        15 => {
            // Firefly
            let gs = 0.03;
            let glow = t.sin() * 0.008;
            push_ellipsoid_pos(pos, x, y + gs + glow, z, gs, gs*0.4, gs, 4, 4);
            push_ellipsoid_pos(pos, x, y + gs*0.8 + glow, z, gs*0.3, gs*0.3, gs*0.3, 4, 4);
            return;
        }
        _ => {}
    }
    let body_h = match ct {
        0 => size * 0.45,
        3 => size * 0.55,
        6 => size * 0.65,
        9 => size * 0.42,
        _ => size * 0.32,
    };
    let body_w = match ct {
        0 => size * 0.18,
        6 => size * 0.07,
        7 => size * 0.28,
        10 => size * 0.12,
        11 => size * 0.22,
        _ => size * 0.16,
    };
    let body_d = match ct {
        0 => size * 0.22,
        6 => size * 0.07,
        7 => size * 0.22,
        10 => size * 0.28,
        _ => size * 0.16,
    };
    let body_sway = t.sin() * leg_amp * 0.3;
    let body_y = y + body_h + body_sway;
    // Body
    push_ellipsoid_pos(pos, x, body_y, z, body_w, body_h, body_d, SEG, SEG);
    match ct {
        0 | 1 | 2 | 3 | 4 | 7 | 8 | 9 => {
            let hh = s * 0.28;
            let head_nod = t.sin() * 0.02 * leg_amp * 5.0;
            push_ellipsoid_pos(pos, x + head_nod, y + body_h * 1.8 + hh, z, hh*0.9, hh, hh*0.9, SEG2, SEG2);
            push_cylinder_pos(pos, x, y + body_h * 1.3, z, body_w*0.4, body_h*0.3, body_d*0.4, 4);
            let lw = body_w * 0.25;
            let lh = body_h * 0.55;
            let ld = body_d * 0.25;
            let leg_off = body_w * 0.5;
            let leg_off_z = body_d * 0.5;
            let fl_z = leg_off_z + t.sin() * leg_amp;
            let fr_z = leg_off_z - t.sin() * leg_amp;
            let bl_z = -leg_off_z - t.sin() * leg_amp;
            let br_z = -leg_off_z + t.sin() * leg_amp;
            push_cylinder_pos(pos, x - leg_off, y + lh, z + fl_z, lw, lh, ld, 4);
            push_cylinder_pos(pos, x + leg_off, y + lh, z + fr_z, lw, lh, ld, 4);
            push_cylinder_pos(pos, x - leg_off, y + lh, z + bl_z, lw, lh, ld, 4);
            push_cylinder_pos(pos, x + leg_off, y + lh, z + br_z, lw, lh, ld, 4);
            if ct == 8 {
                let tl = s * 0.4;
                let tail_wag = t.sin() * 0.02;
                push_ellipsoid_pos(pos, x + tail_wag, y + body_h * 0.5, z - body_d - tl*0.5, 0.03, 0.03, tl*0.5, 4, 4);
            }
            if ct == 0 {
                let ah = s * 0.25;
                push_cylinder_pos(pos, x - 0.04, y + body_h * 1.8 + hh + ah*0.5, z - 0.02, 0.008, ah, 0.008, 4);
                push_cylinder_pos(pos, x + 0.04, y + body_h * 1.8 + hh + ah*0.5, z - 0.02, 0.008, ah, 0.008, 4);
            }
            if ct == 4 {
                let ws = s * 0.4;
                let wf = t.sin() * 0.03;
                push_ellipsoid_pos(pos, x - ws*0.4 - wf, y + body_h*0.8 + wf*0.3, z, ws*0.5, 0.015, s*0.15, SEG2, 4);
                push_ellipsoid_pos(pos, x + ws*0.4 + wf, y + body_h*0.8 - wf*0.3, z, ws*0.5, 0.015, s*0.15, SEG2, 4);
            }
        }
        5 => {
            // Fire elemental pulse
            let pulse = t.sin() * 0.03;
            push_ellipsoid_pos(pos, x, y + body_h + pulse, z, body_w*0.4, body_h*0.7, body_w*0.4, SEG, SEG);
        }
        6 => {
            // Snake slither
            let segs = 5;
            for si in 0..segs {
                let st = si as f32 / (segs - 1) as f32;
                let sz = z + (st - 0.5) * s * 0.8;
                let sw = (t + st * 2.0).sin() * 0.04;
                let sy = y + body_h + (st - 0.5).sin() * 0.03;
                let sr = body_w * (1.0 - 0.3 * st);
                push_ellipsoid_pos(pos, x + sw, sy, sz, sr, sr, body_d*0.3, SEG2, SEG2);
            }
            push_ellipsoid_pos(pos, x + (t + 1.0).sin()*0.02, y + body_h + 0.02, z + s*0.45, body_w*0.6, body_w*0.6, body_d*0.15, 4, 4);
        }
        10 => {
            // Fish swim
            let swim = t.sin() * 0.04;
            push_ellipsoid_pos(pos, x + swim*0.3, y + body_h, z, body_w, body_h, body_d, SEG, SEG);
            let tf = s * 0.25;
            let tail_swish = t.sin() * 0.03;
            push_ellipsoid_pos(pos, x + tail_swish, y + body_h*0.5, z - body_d - tf*0.4, tf*0.3, tf*0.4, tf*0.3, 4, 4);
            push_ellipsoid_pos(pos, x, y + body_h*1.3, z, 0.01, body_h*0.3, body_d*0.3, 4, 4);
        }
        11 => {
            // Crab
            push_ellipsoid_pos(pos, x, y + body_h, z, body_w, body_h*0.5, body_d, SEG, SEG);
            for li in 0..3 {
                let la = li as f32 * 0.4 - 0.4;
                let lx = (la * 0.5).sin() * body_w * 0.6;
                let lz = body_d * 0.5 + li as f32 * body_d * 0.15;
                let ll = s * 0.2;
                let leg_sway = (t + li as f32).sin() * 0.015;
                push_cylinder_pos(pos, x - body_w*0.3 + lx + leg_sway, y + ll, z + lz, 0.015, ll, 0.015, 4);
                push_cylinder_pos(pos, x + body_w*0.3 - lx - leg_sway, y + ll, z + lz, 0.015, ll, 0.015, 4);
            }
            push_ellipsoid_pos(pos, x - body_w*0.8, y + body_h*0.3, z + body_d*0.6, 0.035, 0.02, 0.02, 4, 4);
            push_ellipsoid_pos(pos, x + body_w*0.8, y + body_h*0.3, z + body_d*0.6, 0.035, 0.02, 0.02, 4, 4);
        }
        12 => {
            // Jellyfish pulse
            let pulse = t.sin() * 0.02;
            push_ellipsoid_pos(pos, x, y + body_h + pulse, z, body_w*0.8, body_h*0.4, body_w*0.8, SEG, SEG);
            push_ellipsoid_pos(pos, x, y + body_h*1.2 + pulse, z, body_w*0.6, body_h*0.3, body_w*0.6, SEG, SEG);
            for ti in 0..5 {
                let ta = ti as f32 * 1.256;
                let tx = x + ta.cos() * body_w * 0.5;
                let tz = z + ta.sin() * body_w * 0.5;
                let tt = s * 0.3;
                let tentacle_sway = (t + ti as f32 * 1.3).sin() * 0.03;
                push_cylinder_pos(pos, tx + tentacle_sway, y + tt*0.3, tz + tentacle_sway*0.5, 0.012, tt, 0.012, 4);
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

pub fn generate_creature_mesh_from_data(data: &CreatureData) -> Option<(Vec<f32>, Vec<f32>, Vec<u32>, Vec<f32>)> {
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

pub fn creature_animated_positions(params: &crate::state::WorldParams, cx: i32, cz: i32, time: f64) -> Option<Vec<f32>> {
    let data = compute_chunk_creatures(params, cx, cz);
    if data.creatures.is_empty() { return None; }
    let mut pos = Vec::new();
    for (i, c) in data.creatures.iter().enumerate() {
        let phase = (cx.wrapping_mul(739).wrapping_add(cz.wrapping_mul(431)) as f64 * 0.1 + i as f64 * 1.7).fract() * std::f64::consts::TAU;
        let bob = (time * 1.8 + phase).sin() * 0.04;
        let y = c.y as f32 + bob as f32;
        emit_creature_positions(c.creature_type, c.x as f32, y, c.z as f32, &mut pos, ANIM_IDLE, time as f32);
    }
    Some(pos)
}

// ── Walkability check for pathfinding ──
fn is_walkable(
    params: &WorldParams,
    wx: f64, wz: f64,
    is_underwater: bool,
    veg_chunks: &HashMap<(i32, i32), crate::engine::vegetation::VegData>,
) -> bool {
    let h = crate::engine::terrain::get_height(params, wx, wz);
    if is_underwater {
        if h > params.water_level - 0.1 { return false; }
        let depth = params.water_level - h;
        if depth > 8.0 { return false; }
    } else {
        if h < params.water_level - 0.5 { return false; }
    }
    let step = 0.5;
    let h1 = crate::engine::terrain::get_height(params, wx + step, wz);
    if (h1 - h).abs() > 1.2 { return false; }
    let h2 = crate::engine::terrain::get_height(params, wx - step, wz);
    if (h2 - h).abs() > 1.2 { return false; }
    let h3 = crate::engine::terrain::get_height(params, wx, wz + step);
    if (h3 - h).abs() > 1.2 { return false; }
    let h4 = crate::engine::terrain::get_height(params, wx, wz - step);
    if (h4 - h).abs() > 1.2 { return false; }
    if crate::engine::collides_with_veg(wx, wz, veg_chunks) { return false; }
    true
}

#[derive(Clone, PartialEq)]
struct ANode {
    gx: i32, gz: i32,
    f: f64,
    g: f64,
}
impl Eq for ANode {}
impl std::cmp::Ord for ANode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f.partial_cmp(&self.f).unwrap_or(std::cmp::Ordering::Equal)
    }
}
impl std::cmp::PartialOrd for ANode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.f.partial_cmp(&self.f)
    }
}

fn pathfind_a_star(
    params: &WorldParams,
    start_x: f64, start_z: f64,
    goal_x: f64, goal_z: f64,
    is_underwater: bool,
    veg_chunks: &HashMap<(i32, i32), crate::engine::vegetation::VegData>,
) -> Vec<(f64, f64)> {
    const CELL: f64 = 1.0;
    let to_gx = |wx: f64| -> i32 { (wx / CELL).floor() as i32 };
    let to_wx = |gx: i32| -> f64 { gx as f64 * CELL + CELL * 0.5 };

    let start = (to_gx(start_x), to_gx(start_z));
    let goal = (to_gx(goal_x), to_gx(goal_z));
    if !is_walkable(params, goal_x, goal_z, is_underwater, veg_chunks) {
        return vec![];
    }

    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<i64, (i32, i32)> = HashMap::new();
    let mut g_score: HashMap<i64, f64> = HashMap::new();

    let key = |gx: i32, gz: i32| -> i64 { (gx as i64) << 32 | (gz as i64 & 0xFFFFFFFF) };

    let h = |gx: i32, gz: i32| -> f64 {
        let dx = (gx - goal.0) as f64;
        let dz = (gz - goal.1) as f64;
        (dx * dx + dz * dz).sqrt() * 1.001
    };

    g_score.insert(key(start.0, start.1), 0.0);
    open.push(ANode { gx: start.0, gz: start.1, f: h(start.0, start.1), g: 0.0 });

    const MAX_ITER: u32 = 2000;
    const SEARCH_RADIUS: i32 = 32;

    let dirs = [(-1,0),(1,0),(0,-1),(0,1),(-1,-1),(-1,1),(1,-1),(1,1)];

    for _ in 0..MAX_ITER {
        let current = match open.pop() {
            Some(n) => n,
            None => break,
        };
        if current.gx == goal.0 && current.gz == goal.1 {
            let mut path = Vec::new();
            let mut cur = (current.gx, current.gz);
            while let Some(&prev) = came_from.get(&key(cur.0, cur.1)) {
                path.push((to_wx(cur.0), to_wx(cur.1)));
                cur = prev;
            }
            path.reverse();
            return path;
        }
        let cur_g = current.g;
        for &(dx, dz) in &dirs {
            let nx = current.gx + dx;
            let nz = current.gz + dz;
            if (nx - start.0).abs() > SEARCH_RADIUS || (nz - start.1).abs() > SEARCH_RADIUS {
                continue;
            }
            if !is_walkable(params, to_wx(nx), to_wx(nz), is_underwater, veg_chunks) {
                continue;
            }
            let move_cost = if dx != 0 && dz != 0 { 1.414 } else { 1.0 };
            let tent_g = cur_g + move_cost;
            let nk = key(nx, nz);
            if tent_g < *g_score.get(&nk).unwrap_or(&1e9) {
                g_score.insert(nk, tent_g);
                came_from.insert(nk, (current.gx, current.gz));
                open.push(ANode { gx: nx, gz: nz, f: tent_g + h(nx, nz), g: tent_g });
            }
        }
    }
    vec![]
}

fn creature_phase(id: &str) -> f64 {
    id.as_bytes().iter().fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64)) as f64 * 0.001
}

// ── Creature AI state machine update ──
pub fn update_creature_ai(
    params: &WorldParams,
    data: &mut CreatureData,
    time: f64,
    player_x: f64, player_z: f64,
    veg_chunks: &HashMap<(i32, i32), crate::engine::vegetation::VegData>,
    _day_time: f64,
    delta: f64,
) -> Option<Vec<f32>> {
    if data.creatures.is_empty() { return None; }

    let zone = crate::engine::terrain::get_zone(params, data.cx as f64 * 24.0 + 12.0, data.cz as f64 * 24.0 + 12.0);
    let is_underwater = matches!(zone, Zone::CoralReef | Zone::KelpForest | Zone::RockyReef | Zone::SandyPlain | Zone::DeepOcean);

    let mut positions = Vec::new();

    for creature in &mut data.creatures {
        if creature.mounted {
            let h = crate::engine::terrain::get_height(params, creature.x, creature.z);
            creature.y = h;
            emit_creature_positions(creature.creature_type, creature.x as f32, creature.y as f32, creature.z as f32, &mut positions, ANIM_IDLE, 0.0);
            continue;
        }

        creature.hunger = (creature.hunger - delta * 0.3).max(0.0);
        creature.state_timer -= delta;
        creature.anim_time += delta;

        let dx = creature.x - player_x;
        let dz = creature.z - player_z;
        let dist_to_player = (dx * dx + dz * dz).sqrt();

        if !creature.tamed && dist_to_player < 5.0 {
            creature.state = STATE_FLEE;
            creature.state_timer = 1.5;
        }

        creature.anim_state = match creature.state {
            STATE_IDLE | STATE_EAT => ANIM_IDLE,
            STATE_WANDER => ANIM_WALK,
            STATE_FLEE => ANIM_RUN,
            STATE_FOLLOW => ANIM_RUN,
            _ => ANIM_IDLE,
        };

        match creature.state {
            STATE_IDLE => {
                if creature.state_timer <= 0.0 {
                    let ph = creature_phase(&creature.id);
                    let angle = ph + time * 0.1;
                    let radius = 3.0 + (ph * 7.0).fract() * 5.0;
                    let tx = creature.x + angle.cos() * radius;
                    let tz = creature.z + angle.sin() * radius;
                    creature.path = pathfind_a_star(params, creature.x, creature.z, tx, tz, is_underwater, veg_chunks);
                    creature.path_index = 0;
                    creature.state = STATE_WANDER;
                    creature.state_timer = 4.0 + (ph * 3.0).fract() * 4.0;
                }
            }
            STATE_WANDER => {
                if creature.path_index < creature.path.len() {
                    let target = creature.path[creature.path_index];
                    let ddx = target.0 - creature.x;
                    let ddz = target.1 - creature.z;
                    let d = (ddx * ddx + ddz * ddz).sqrt();
                    if d < 0.3 {
                        creature.path_index += 1;
                    } else {
                        let spd = creature.speed * 2.0 * delta;
                        creature.x += (ddx / d) * spd;
                        creature.z += (ddz / d) * spd;
                        creature.rot = ddx.atan2(ddz);
                    }
                } else {
                    creature.state = STATE_IDLE;
                    creature.state_timer = 2.0 + creature_phase(&creature.id).fract() * 4.0;
                }
                if creature.state_timer <= 0.0 {
                    creature.state = STATE_IDLE;
                    creature.state_timer = 2.0 + (time * 0.1).fract() * 3.0;
                }
            }
            STATE_FLEE => {
                let flee_angle = dx.atan2(dz);
                let spd = creature.speed * 4.0 * delta;
                creature.x += flee_angle.cos() * spd;
                creature.z += flee_angle.sin() * spd;
                creature.rot = flee_angle + std::f64::consts::PI;
                if dist_to_player > 12.0 || creature.state_timer <= 0.0 {
                    creature.state = STATE_WANDER;
                    creature.path.clear();
                    creature.path_index = 0;
                }
            }
            STATE_FOLLOW => {
                if dist_to_player > 2.5 {
                    if creature.path.is_empty() || creature.path_index >= creature.path.len() {
                        creature.path = pathfind_a_star(params, creature.x, creature.z, player_x, player_z, is_underwater, veg_chunks);
                        creature.path_index = 0;
                    }
                    if creature.path_index < creature.path.len() {
                        let target = creature.path[creature.path_index];
                        let ddx = target.0 - creature.x;
                        let ddz = target.1 - creature.z;
                        let d = (ddx * ddx + ddz * ddz).sqrt();
                        if d < 0.3 {
                            creature.path_index += 1;
                        } else {
                            let spd = creature.speed * 3.0 * delta;
                            creature.x += (ddx / d) * spd;
                            creature.z += (ddz / d) * spd;
                            creature.rot = ddx.atan2(ddz);
                        }
                    }
                }
                if dist_to_player > 30.0 {
                    creature.state = STATE_IDLE;
                    creature.tamed = false;
                }
            }
            STATE_EAT => {
                if creature.state_timer <= 0.0 {
                    creature.hunger = (creature.hunger + 30.0).min(100.0);
                    creature.state = STATE_IDLE;
                    creature.state_timer = 3.0;
                }
            }
            _ => {}
        }

        // Clamp to valid terrain
        let h = crate::engine::terrain::get_height(params, creature.x, creature.z);
        let is_flying = creature.creature_type == 13 || creature.creature_type == 14 || creature.creature_type == 15;
        let y_pos = if is_flying {
            let base_y = if is_underwater { params.water_level } else { h };
            base_y + match creature.creature_type {
                13 => 0.8 + 0.5,
                14 => 4.0 + 3.0,
                15 => 0.3 + 0.5,
                _ => 0.0,
            }
        } else if is_underwater {
            params.water_level - 0.5
        } else {
            h
        };
        creature.y = y_pos;

        let phase = creature_phase(&creature.id);
        let (bob_amp, bob_speed) = match creature.state {
            STATE_FLEE => (0.10, 4.5),
            STATE_FOLLOW => (0.06, 3.0),
            STATE_WANDER => (0.04, 2.5),
            STATE_EAT => (0.02, 1.0),
            _ => (0.02, 1.2),
        };
        let bob = (time * bob_speed + phase * std::f64::consts::TAU).sin() * bob_amp;
        let emit_y = creature.y as f32 + bob as f32;
        emit_creature_positions(creature.creature_type, creature.x as f32, emit_y, creature.z as f32, &mut positions, creature.anim_state, creature.anim_time as f32);
    }

    if positions.is_empty() { None } else { Some(positions) }
}

pub fn creature_name(ct: u8) -> &'static str {
    match ct {
        0 => "Ciervo",
        1 => "Mono",
        2 => "Ave",
        3 => "Cristalino",
        4 => "Murciélago",
        5 => "Elemental de fuego",
        6 => "Serpiente",
        7 => "Oso polar",
        8 => "Zorro",
        9 => "Suricata",
        10 => "Pez",
        11 => "Cangrejo",
        12 => "Medusa",
        13 => "Mariposa",
        14 => "Pájaro",
        15 => "Luciérnaga",
        _ => "Desconocido",
    }
}
