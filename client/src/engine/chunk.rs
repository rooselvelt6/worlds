use std::collections::HashSet;
use crate::engine::terrain;
use crate::engine::terrain::{Zone, BLK_AIR};
use crate::math::{hsl_to_rgb, rgb_to_hsl};
use crate::state::WorldParams;

pub const CHUNK_SIZE: f64 = 24.0;
pub const BLOCK_RES: u32 = 24;
const BLOCK_SIZE: f64 = CHUNK_SIZE / BLOCK_RES as f64;
const BLOCK_HALF: f32 = (BLOCK_SIZE * 0.5) as f32;
const UNDERGROUND_LAYERS: i32 = 8;

#[derive(Clone)]
pub struct ChunkData {
    pub cx: i32,
    pub cz: i32,
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub colors: Vec<f32>,
    pub indices: Vec<u32>,
}

impl ChunkData {
    pub fn key(&self) -> (i32, i32) {
        (self.cx, self.cz)
    }
}

fn apply_mutation(params: &WorldParams, cx: i32, cz: i32) -> WorldParams {
    if params.mutation <= 0.0 {
        return *params;
    }
    let mut p = *params;
    let h = ((params.seed as i64).wrapping_mul(374761393)
        .wrapping_add(cx as i64 * 668265263)
        .wrapping_add(cz as i64 * 1274126177)) as f64;
    let norm = (h.sin() * 43758.5453).fract().abs();
    let offset = (norm - 0.5) * 2.0 * params.mutation;
    p.scale *= 1.0 + offset * 0.1;
    p.amplitude *= 1.0 + offset * 0.15;
    p
}

fn get_height_map(params: &WorldParams, cx: i32, cz: i32) -> Vec<f64> {
    let ox = cx as f64 * CHUNK_SIZE;
    let oz = cz as f64 * CHUNK_SIZE;
    let step = CHUNK_SIZE / BLOCK_RES as f64;
    let n = BLOCK_RES as usize;
    let mut heights = vec![0.0_f64; n * n];
    for iz in 0..n {
        for ix in 0..n {
            let wx = ox + ix as f64 * step + BLOCK_SIZE * 0.5;
            let wz = oz + iz as f64 * step + BLOCK_SIZE * 0.5;
            let mut h = terrain::get_height(params, wx, wz);
            terrain::zone_effects(params, wx, wz, &mut h);
            heights[iz * n + ix] = h;
        }
    }
    heights
}



fn push_face(
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    v: [[f32; 3]; 4], nx: f32, ny: f32, nz: f32,
    r: f32, g: f32, b: f32,
) {
    let nv = (pos.len() / 3) as u32;
    for &vert in &v {
        pos.push(vert[0]); pos.push(vert[1]); pos.push(vert[2]);
        norms.push(nx); norms.push(ny); norms.push(nz);
        cols.push(r); cols.push(g); cols.push(b);
    }
    idx.push(nv); idx.push(nv + 1); idx.push(nv + 2);
    idx.push(nv); idx.push(nv + 2); idx.push(nv + 3);
}

fn emit_face(
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>,
    wx: f32, wy: f32, wz: f32, dir: u8, r: f32, g: f32, b: f32,
) {
    let hs = BLOCK_HALF;
    let verts = match dir {
        // +X
        0 => [[wx + hs, wy - hs, wz - hs], [wx + hs, wy + hs, wz - hs],
              [wx + hs, wy + hs, wz + hs], [wx + hs, wy - hs, wz + hs]],
        // -X
        1 => [[wx - hs, wy - hs, wz + hs], [wx - hs, wy + hs, wz + hs],
              [wx - hs, wy + hs, wz - hs], [wx - hs, wy - hs, wz - hs]],
        // +Y (top)
        2 => [[wx - hs, wy + hs, wz - hs], [wx + hs, wy + hs, wz - hs],
              [wx + hs, wy + hs, wz + hs], [wx - hs, wy + hs, wz + hs]],
        // -Y (bottom)
        3 => [[wx - hs, wy - hs, wz + hs], [wx + hs, wy - hs, wz + hs],
              [wx + hs, wy - hs, wz - hs], [wx - hs, wy - hs, wz - hs]],
        // +Z
        4 => [[wx - hs, wy - hs, wz + hs], [wx + hs, wy - hs, wz + hs],
              [wx + hs, wy + hs, wz + hs], [wx - hs, wy + hs, wz + hs]],
        // -Z
        5 => [[wx + hs, wy - hs, wz - hs], [wx - hs, wy - hs, wz - hs],
              [wx - hs, wy + hs, wz - hs], [wx + hs, wy + hs, wz - hs]],
        _ => return,
    };
    let norm = match dir {
        0 => (1.0, 0.0, 0.0),
        1 => (-1.0, 0.0, 0.0),
        2 => (0.0, 1.0, 0.0),
        3 => (0.0, -1.0, 0.0),
        4 => (0.0, 0.0, 1.0),
        5 => (0.0, 0.0, -1.0),
        _ => (0.0, 0.0, 0.0),
    };
    push_face(pos, norms, idx, cols, verts, norm.0, norm.1, norm.2, r, g, b);
}

fn is_air(params: &WorldParams, wx: f64, wy: f64, wz: f64) -> bool {
    let h = terrain::get_height(params, wx, wz);
    let mut h2 = h;
    terrain::zone_effects(params, wx, wz, &mut h2);
    wy > h2
}

fn block_key(cx: i32, cz: i32, ix: usize, iy: usize, iz: usize) -> (i32, i32, i32) {
    (cx * 24 + ix as i32, -UNDERGROUND_LAYERS + iy as i32, cz * 24 + iz as i32)
}

fn feature_hash(cx: i32, cz: i32, seed: u32, index: u32) -> f64 {
    let h = (seed as i64).wrapping_mul(374761393)
        .wrapping_add((cx as i64).wrapping_mul(668265263))
        .wrapping_add((cz as i64).wrapping_mul(1274126177))
        .wrapping_add(index as i64 * 1013904243);
    (h as f64 * 0.000000001).fract().abs()
}

fn apply_underground_features(
    params: &WorldParams, cx: i32, cz: i32,
    heights: &[f64], zones: &[Zone],
    blocks: &mut [u8], n: usize, grid_ny: usize, wy_min: f64,
) {
    let seed = params.seed;
    let ox = cx as f64 * CHUNK_SIZE;
    let oz = cz as f64 * CHUNK_SIZE;
    let step = BLOCK_SIZE;

    for iz in 0..n {
        for ix in 0..n {
            let wz = oz + iz as f64 * step + BLOCK_SIZE * 0.5;
            let wx = ox + ix as f64 * step + BLOCK_SIZE * 0.5;
            let zone = zones[iz * n + ix];
            let surface_h = heights[iz * n + ix];

            // ── Lava lakes in Volcanic/Magma zones (depth > 6) ──
            if (zone == Zone::Volcanic || zone == Zone::Magma) && surface_h > 6.0 {
                let depth_to_check = 6.0_f64.max(surface_h - 10.0);
                for iy in 0..grid_ny {
                    let wy = wy_min + iy as f64 + BLOCK_SIZE * 0.5;
                    if wy > surface_h - depth_to_check { break; }
                    let idx = iy * n * n + iz * n + ix;
                    if blocks[idx] == terrain::BLK_AIR { continue; }
                    let lava_noise = crate::math::fbm_3d(wx * 0.03, wy * 0.03, wz * 0.03, 2);
                    if lava_noise > 0.55 {
                        let depth = surface_h - wy;
                        let pool = (lava_noise - 0.55) / 0.45;
                        if depth < 12.0 && pool > 0.3 && pool < 0.7 {
                            // Lava pool center — make lava
                            blocks[idx] = terrain::BLK_LAVA;
                        } else if pool >= 0.7 {
                            // Carve air above lava
                            blocks[idx] = terrain::BLK_AIR;
                        }
                    }
                }
            }

            // ── Giant crystals in Crystal zone (depth > 3) ──
            if zone == Zone::Crystal && surface_h > 3.0 {
                let crystal_seed = feature_hash(cx, cz, seed, (ix * n + iz) as u32);
                if crystal_seed > 0.97 {
                    let crystal_h = 3 + (crystal_seed * 6.0) as usize;
                    for iy in 0..grid_ny {
                        let wy = wy_min + iy as f64 + BLOCK_SIZE * 0.5;
                        if wy > surface_h - 3.0 { break; }
                        let idx = iy * n * n + iz * n + ix;
                        if blocks[idx] == terrain::BLK_AIR { continue; }
                        let dist_from_top = (surface_h - 3.0 - wy) as usize;
                        if dist_from_top < crystal_h {
                            blocks[idx] = terrain::BLK_DIAMOND_ORE;
                        }
                    }
                }
            }

            // ── Dungeon rooms (rare, any zone, depth > 8) ──
            let dungeon_seed = feature_hash(cx, cz, seed ^ 0xABCD, (ix / 5 * n + iz / 5) as u32);
            if dungeon_seed > 0.998 && surface_h > 8.0 {
                let room_ox = (ix / 5) * 5;
                let room_oz = (iz / 5) * 5;
                let room_iy = 4 + (dungeon_seed * 6.0) as usize;
                if room_iy < grid_ny && room_ox < n && room_oz < n {
                    for dz in 0..5 {
                        for dx in 0..5 {
                            let rx = room_ox + dx;
                            let rz = room_oz + dz;
                            if rx >= n || rz >= n { continue; }
                            for dy in 0..3 {
                                let iy = room_iy + dy;
                                if iy >= grid_ny { continue; }
                                let idx2 = iy * n * n + rz * n + rx;
                                let wy2 = wy_min + iy as f64 + BLOCK_SIZE * 0.5;
                                if wy2 > surface_h - 0.5 { continue; }
                                // Carve room interior
                                if dy == 0 || dy == 2 || dx == 0 || dx == 4 || dz == 0 || dz == 4 {
                                    // Walls — only replace stone/dirt, not ores
                                    if blocks[idx2] == terrain::BLK_STONE || blocks[idx2] == terrain::BLK_DIRT {
                                        blocks[idx2] = terrain::BLK_STONE;
                                    }
                                } else {
                                    // Interior — air
                                    blocks[idx2] = terrain::BLK_AIR;
                                }
                            }
                        }
                    }
                    // Place a treasure block on the floor center
                    let center_idx = (room_iy + 1) * n * n + (room_oz + 2) * n + (room_ox + 2);
                    if center_idx < blocks.len() {
                        blocks[center_idx] = if dungeon_seed > 0.9995 {
                            terrain::BLK_DIAMOND_ORE
                        } else if dungeon_seed > 0.999 {
                            terrain::BLK_GOLD_ORE
                        } else {
                            terrain::BLK_IRON_ORE
                        };
                    }
                }
            }
        }
    }
}

pub fn compute_chunk_data(params: &WorldParams, cx: i32, cz: i32, mined: &HashSet<(i32,i32,i32)>) -> ChunkData {
    compute_chunk_data_lod(params, cx, cz, mined, 0)
}

fn underground_layers(lod: u32) -> i32 {
    match lod {
        0 => UNDERGROUND_LAYERS,
        1 => 3,
        _ => 1,
    }
}

pub fn compute_chunk_data_lod(params: &WorldParams, cx: i32, cz: i32, mined: &HashSet<(i32,i32,i32)>, lod: u32) -> ChunkData {
    let mutated = apply_mutation(params, cx, cz);
    let p = &mutated;

    let ox = cx as f64 * CHUNK_SIZE;
    let oz = cz as f64 * CHUNK_SIZE;
    let n = BLOCK_RES as usize;
    let step = BLOCK_SIZE;
    let ug_layers = underground_layers(lod);

    let heights = get_height_map(p, cx, cz);
    let max_h = heights.iter().cloned().fold(0.0_f64, f64::max);

    let wy_min = -(ug_layers as f64);
    let wy_max = CHUNK_SIZE;
    let grid_ny = (wy_max - wy_min) as usize;

    let mut blocks = vec![0u8; n * grid_ny * n];
    let mut zones = vec![Zone::Forest; n * n];

    // First pass: determine which blocks are solid
    for iz in 0..n {
        for ix in 0..n {
            let wz = oz + iz as f64 * step + BLOCK_SIZE * 0.5;
            let wx = ox + ix as f64 * step + BLOCK_SIZE * 0.5;
            let surface_h = heights[iz * n + ix];
            let zone = terrain::get_zone(p, wx, wz);
            zones[iz * n + ix] = zone;

            for iy in 0..grid_ny {
                let wy = wy_min + iy as f64 + BLOCK_SIZE * 0.5;
                let idx = iy * n * n + iz * n + ix;
                let bt = terrain::get_block_type(p, wx, wy, wz, surface_h, zone);
                if bt != BLK_AIR && mined.contains(&block_key(cx, cz, ix, iy, iz)) {
                    blocks[idx] = BLK_AIR;
                } else {
                    blocks[idx] = bt;
                }
            }
        }
    }

    // Apply underground features only for LOD 0 (full detail)
    if lod == 0 {
        apply_underground_features(p, cx, cz, &heights, &zones, &mut blocks, n, grid_ny, wy_min);
    }

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let sample_step = match lod {
        0 => 1,
        1 => 2,
        _ => 4,
    };

    // Second pass: mesh visible faces with face culling and LOD sampling
    let mut iz: usize = 0;
    while iz < n {
        let mut ix: usize = 0;
        while ix < n {
            let wz = ox + iz as f64 * step + BLOCK_SIZE * 0.5;
            let wx = ox + ix as f64 * step + BLOCK_SIZE * 0.5;
            let surface_h = heights[iz * n + ix];
            let zone = zones[iz * n + ix];

            let mut iy: usize = 0;
            while iy < grid_ny {
                let wy = wy_min + iy as f64 + BLOCK_SIZE * 0.5;
                let idx = iy * n * n + iz * n + ix;
                if blocks[idx] == 0 { iy += 1; continue; }

                let c = terrain::block_color(blocks[idx], p, wx, wy, wz, zone, surface_h, max_h);
                let (r, g, b) = {
                    let (h, s, l) = rgb_to_hsl(c[0], c[1], c[2]);
                    hsl_to_rgb(
                        (h + params.hue_shift as f32 / 360.0) % 1.0,
                        (s * params.saturation as f32).clamp(0.0, 1.0),
                        (l * params.lightness as f32).clamp(0.0, 1.0),
                    )
                };

                let wx_f = wx as f32;
                let wy_f = wy as f32;
                let wz_f = wz as f32;

                // +X
                if ix + sample_step < n {
                    if blocks[iy * n * n + iz * n + (ix + sample_step)] == 0 {
                        emit_face(&mut positions, &mut normals, &mut indices, &mut colors, wx_f, wy_f, wz_f, 0, r, g, b);
                    }
                } else if is_air(p, wx + BLOCK_SIZE * sample_step as f64, wy, wz) {
                    emit_face(&mut positions, &mut normals, &mut indices, &mut colors, wx_f, wy_f, wz_f, 0, r, g, b);
                }
                // -X
                if ix >= sample_step {
                    if blocks[iy * n * n + iz * n + (ix - sample_step)] == 0 {
                        emit_face(&mut positions, &mut normals, &mut indices, &mut colors, wx_f, wy_f, wz_f, 1, r, g, b);
                    }
                } else if is_air(p, wx - BLOCK_SIZE * sample_step as f64, wy, wz) {
                    emit_face(&mut positions, &mut normals, &mut indices, &mut colors, wx_f, wy_f, wz_f, 1, r, g, b);
                }
                // +Y (top)
                if iy + sample_step < grid_ny {
                    if blocks[(iy + sample_step) * n * n + iz * n + ix] == 0 {
                        emit_face(&mut positions, &mut normals, &mut indices, &mut colors, wx_f, wy_f, wz_f, 2, r, g, b);
                    }
                }
                // -Y (bottom)
                if iy >= sample_step {
                    if blocks[(iy - sample_step) * n * n + iz * n + ix] == 0 {
                        emit_face(&mut positions, &mut normals, &mut indices, &mut colors, wx_f, wy_f, wz_f, 3, r, g, b);
                    }
                }
                // +Z
                if iz + sample_step < n {
                    if blocks[iy * n * n + (iz + sample_step) * n + ix] == 0 {
                        emit_face(&mut positions, &mut normals, &mut indices, &mut colors, wx_f, wy_f, wz_f, 4, r, g, b);
                    }
                } else if is_air(p, wx, wy, wz + BLOCK_SIZE * sample_step as f64) {
                    emit_face(&mut positions, &mut normals, &mut indices, &mut colors, wx_f, wy_f, wz_f, 4, r, g, b);
                }
                // -Z
                if iz >= sample_step {
                    if blocks[iy * n * n + (iz - sample_step) * n + ix] == 0 {
                        emit_face(&mut positions, &mut normals, &mut indices, &mut colors, wx_f, wy_f, wz_f, 5, r, g, b);
                    }
                } else if is_air(p, wx, wy, wz - BLOCK_SIZE * sample_step as f64) {
                    emit_face(&mut positions, &mut normals, &mut indices, &mut colors, wx_f, wy_f, wz_f, 5, r, g, b);
                }

                iy += sample_step;
            }
            ix += sample_step;
        }
        iz += sample_step;
    }

    ChunkData { cx, cz, positions, normals, colors, indices }
}