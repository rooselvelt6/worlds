
use crate::engine::terrain;
use crate::engine::terrain::{Zone, BLK_AIR, BLK_DIRT, BLK_SNOW, BLK_STONE};
use crate::engine::structures;
use crate::math::{hsl_to_rgb, rgb_to_hsl};
use crate::state::WorldParams;

pub const CHUNK_SIZE: f64 = 24.0;
pub const BLOCK_RES: u32 = 24;
const BLOCK_SIZE: f64 = CHUNK_SIZE / BLOCK_RES as f64;
const BLOCK_HALF: f32 = (BLOCK_SIZE * 0.5) as f32;
const UNDERGROUND_LAYERS: i32 = 32;

pub const ATLAS_COLS: u32 = 6;
pub const ATLAS_ROWS: u32 = 4;

pub struct MeshPool {
    pub blocks: Vec<u8>,
    pub zones: Vec<Zone>,
    pub heights: Vec<f64>,
    pub surface_iy: Vec<usize>,
    pub corner_heights: Vec<f64>,
    pub corner_normals: Vec<[f32; 3]>,
}

impl MeshPool {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            zones: Vec::new(),
            heights: Vec::new(),
            surface_iy: Vec::new(),
            corner_heights: Vec::new(),
            corner_normals: Vec::new(),
        }
    }

    pub fn clear_all(&mut self) {
        self.blocks.clear();
        self.zones.clear();
        self.heights.clear();
        self.surface_iy.clear();
        self.corner_heights.clear();
        self.corner_normals.clear();
    }
}

#[derive(Clone)]
pub struct ChunkData {
    pub cx: i32,
    pub cz: i32,
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub colors: Vec<f32>,
    pub uvs: Vec<f32>,
    pub indices: Vec<u32>,
    pub block_types: Vec<u8>,
}

pub fn block_atlas_tile(block_type: u8) -> u32 {
    match block_type {
        terrain::BLK_GRASS => 0,
        terrain::BLK_DIRT => 1,
        terrain::BLK_STONE => 2,
        terrain::BLK_SAND => 3,
        terrain::BLK_SNOW => 4,
        terrain::BLK_GRAVEL => 5,
        terrain::BLK_CLAY => 6,
        terrain::BLK_COAL_ORE => 7,
        terrain::BLK_IRON_ORE => 8,
        terrain::BLK_GOLD_ORE => 9,
        terrain::BLK_DIAMOND_ORE => 10,
        terrain::BLK_LAVA => 11,
        terrain::BLK_PACKED_ICE => 12,
        terrain::BLK_OBSIDIAN => 13,
        terrain::BLK_MOSS => 14,
        terrain::BLK_GLOW_SHROOM => 15,
        terrain::BLK_MAGMA_BLOCK => 16,
        terrain::BLK_SOUL_SAND => 17,
        terrain::BLK_BASALT => 18,
        _ => 0,
    }
}

fn tile_uv(tile: u32) -> (f32, f32, f32, f32) {
    let cols = ATLAS_COLS as f32;
    let rows = ATLAS_ROWS as f32;
    let tx = (tile % ATLAS_COLS) as f32;
    let ty = (tile / ATLAS_COLS) as f32;
    (tx / cols, ty / rows, (tx + 1.0) / cols, (ty + 1.0) / rows)
}

fn height_bilerp(heights: &[f64], n: usize, fx: f64, fz: f64) -> f64 {
    let ix = fx.floor() as i32;
    let iz = fz.floor() as i32;
    let tx = (fx - ix as f64).clamp(0.0, 1.0);
    let tz = (fz - iz as f64).clamp(0.0, 1.0);
    let ix0 = ix.max(0).min(n as i32 - 1) as usize;
    let ix1 = (ix + 1).max(0).min(n as i32 - 1) as usize;
    let iz0 = iz.max(0).min(n as i32 - 1) as usize;
    let iz1 = (iz + 1).max(0).min(n as i32 - 1) as usize;
    let h00 = heights[iz0 * n + ix0];
    let h10 = heights[iz0 * n + ix1];
    let h01 = heights[iz1 * n + ix0];
    let h11 = heights[iz1 * n + ix1];
    let h0 = h00 * (1.0 - tx) + h10 * tx;
    let h1 = h01 * (1.0 - tx) + h11 * tx;
    h0 * (1.0 - tz) + h1 * tz
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

fn get_height_map(params: &WorldParams, cx: i32, cz: i32, heights: &mut Vec<f64>) {
    let ox = cx as f64 * CHUNK_SIZE;
    let oz = cz as f64 * CHUNK_SIZE;
    let step = CHUNK_SIZE / BLOCK_RES as f64;
    let n = BLOCK_RES as usize;
    heights.clear();
    heights.resize(n * n, 0.0_f64);
    for iz in 0..n {
        for ix in 0..n {
            let wx = ox + ix as f64 * step + BLOCK_SIZE * 0.5;
            let wz = oz + iz as f64 * step + BLOCK_SIZE * 0.5;
            let mut h = terrain::get_height(params, wx, wz);
            terrain::zone_effects(params, wx, wz, &mut h);
            heights[iz * n + ix] = h;
        }
    }
}



fn push_face(
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>, uvs: &mut Vec<f32>,
    v: [[f32; 3]; 4], nx: f32, ny: f32, nz: f32,
    r: f32, g: f32, b: f32,
    uv_rect: (f32, f32, f32, f32),
) {
    let nv = (pos.len() / 3) as u32;
    let (u0, v0, u1, v1) = uv_rect;
    let uv_verts = [[u0, v0], [u1, v0], [u1, v1], [u0, v1]];
    for (i, &vert) in v.iter().enumerate() {
        pos.push(vert[0]); pos.push(vert[1]); pos.push(vert[2]);
        norms.push(nx); norms.push(ny); norms.push(nz);
        cols.push(r); cols.push(g); cols.push(b);
        uvs.push(uv_verts[i][0]); uvs.push(uv_verts[i][1]);
    }
    idx.push(nv); idx.push(nv + 1); idx.push(nv + 2);
    idx.push(nv); idx.push(nv + 2); idx.push(nv + 3);
}

fn emit_face(
    pos: &mut Vec<f32>, norms: &mut Vec<f32>, idx: &mut Vec<u32>, cols: &mut Vec<f32>, uvs: &mut Vec<f32>,
    wx: f32, wy: f32, wz: f32, dir: u8, r: f32, g: f32, b: f32, block_type: u8,
    heights: Option<&[f64]>, n: usize,
) {
    let hs = BLOCK_HALF;
    let tile = block_atlas_tile(block_type);
    let uv_rect = if dir == 2 && block_type == terrain::BLK_GRASS {
        tile_uv(0)
    } else if dir == 3 {
        tile_uv(block_atlas_tile(terrain::BLK_DIRT))
    } else {
        tile_uv(tile)
    };
    let verts = match dir {
        // +X
        0 => [[wx + hs, wy - hs, wz - hs], [wx + hs, wy + hs, wz - hs],
              [wx + hs, wy + hs, wz + hs], [wx + hs, wy - hs, wz + hs]],
        // -X
        1 => [[wx - hs, wy - hs, wz + hs], [wx - hs, wy + hs, wz + hs],
              [wx - hs, wy + hs, wz - hs], [wx - hs, wy - hs, wz - hs]],
        // +Y (top) — smooth surface
        2 => {
            if let Some(h) = heights {
                let nf = n as f64;
                let fx = (wx as f64 / BLOCK_SIZE).fract().abs() * nf;
                let fz = (wz as f64 / BLOCK_SIZE).fract().abs() * nf;
                let h00 = height_bilerp(h, n, fx - 0.5, fz - 0.5) as f32;
                let h10 = height_bilerp(h, n, fx + 0.5, fz - 0.5) as f32;
                let h11 = height_bilerp(h, n, fx + 0.5, fz + 0.5) as f32;
                let h01 = height_bilerp(h, n, fx - 0.5, fz + 0.5) as f32;
                let top = wy + hs;
                [[wx - hs, top + (h00 - top).clamp(-hs, hs), wz - hs],
                 [wx + hs, top + (h10 - top).clamp(-hs, hs), wz - hs],
                 [wx + hs, top + (h11 - top).clamp(-hs, hs), wz + hs],
                 [wx - hs, top + (h01 - top).clamp(-hs, hs), wz + hs]]
            } else {
                [[wx - hs, wy + hs, wz - hs], [wx + hs, wy + hs, wz - hs],
                 [wx + hs, wy + hs, wz + hs], [wx - hs, wy + hs, wz + hs]]
            }
        },
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
    push_face(pos, norms, idx, cols, uvs, verts, norm.0, norm.1, norm.2, r, g, b, uv_rect);
}

fn is_air(params: &WorldParams, wx: f64, wy: f64, wz: f64) -> bool {
    let h = terrain::get_height(params, wx, wz);
    let mut h2 = h;
    terrain::zone_effects(params, wx, wz, &mut h2);
    wy > h2
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
                            blocks[idx] = terrain::BLK_LAVA;
                        } else if pool >= 0.7 {
                            blocks[idx] = terrain::BLK_AIR;
                        }
                    }
                }
            }

            // ── Deep lava tubes in Volcanic/Lava/Magma (great depth, any surface_h) ──
            if matches!(zone, Zone::Volcanic | Zone::Lava | Zone::Magma) {
                let tube_noise = crate::math::perlin_noise_3d(wx * 0.02, surface_h * 0.02, wz * 0.02);
                if tube_noise > 0.6 {
                    for iy in 0..grid_ny {
                        let wy = wy_min + iy as f64 + BLOCK_SIZE * 0.5;
                        let depth = surface_h - wy;
                        if depth < 8.0 || depth > 20.0 { continue; }
                        let idx = iy * n * n + iz * n + ix;
                        if blocks[idx] == terrain::BLK_AIR { continue; }
                        let tube_radius = 1.0 + (tube_noise - 0.6) / 0.4 * 2.0;
                        let dist_from_tube_center = (wy - (surface_h - 14.0)).abs();
                        if dist_from_tube_center < tube_radius {
                            if dist_from_tube_center < tube_radius * 0.4 {
                                blocks[idx] = terrain::BLK_LAVA;
                            } else {
                                blocks[idx] = terrain::BLK_AIR;
                            }
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
                // Crystal geodes — small clusters of packed ice deep underground
                let geode_seed = feature_hash(cx, cz, seed.wrapping_add(0xDEF), (ix * n + iz) as u32);
                if geode_seed > 0.99 {
                    for iy in 0..grid_ny {
                        let wy = wy_min + iy as f64 + BLOCK_SIZE * 0.5;
                        let depth = surface_h - wy;
                        if depth < 5.0 || depth > 14.0 { continue; }
                        let idx = iy * n * n + iz * n + ix;
                        if blocks[idx] == terrain::BLK_AIR { continue; }
                        let layer = (wy * 3.0 + 100.0).sin() * (wx * 3.0 + wz * 2.0).cos() * 0.5 + 0.5;
                        if layer > 0.6 {
                            blocks[idx] = terrain::BLK_PACKED_ICE;
                        }
                    }
                }
            }

            // ── Fungus caverns: large open chambers with glow shrooms ──
            if zone == Zone::Fungus && surface_h > 4.0 {
                let cavern_noise = crate::math::perlin_noise_3d(wx * 0.01, surface_h * 0.01, wz * 0.01);
                if cavern_noise > 0.55 {
                    for iy in 0..grid_ny {
                        let wy = wy_min + iy as f64 + BLOCK_SIZE * 0.5;
                        let depth = surface_h - wy;
                        if depth < 3.0 || depth > 15.0 { continue; }
                        let idx = iy * n * n + iz * n + ix;
                        if blocks[idx] == terrain::BLK_AIR { continue; }
                        // Carve a large chamber
                        let czn = crate::math::perlin_noise_3d(wx * 0.04, wy * 0.04, wz * 0.04);
                        if czn > 0.3 {
                            let floor_dist = (wy - (surface_h - depth.max(3.0))).abs();
                            if floor_dist < 0.5 {
                                // Chamber floor — glow shrooms
                                blocks[idx] = terrain::BLK_GLOW_SHROOM;
                            } else {
                                blocks[idx] = terrain::BLK_AIR;
                            }
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
                                if dy == 0 || dy == 2 || dx == 0 || dx == 4 || dz == 0 || dz == 4 {
                                    if blocks[idx2] == terrain::BLK_STONE || blocks[idx2] == terrain::BLK_DIRT {
                                        blocks[idx2] = terrain::BLK_STONE;
                                    }
                                } else {
                                    blocks[idx2] = terrain::BLK_AIR;
                                }
                            }
                        }
                    }
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

            // ── Structure dungeons: rooms below large structures (Plaza, Pyramid, Tower, Dome) ──
            {
                use structures::StructType;
                let struct_data = structures::compute_chunk_structures(params, cx, cz);
                for inst in &struct_data.instances {
                    let is_large = matches!(inst.struct_type,
                        StructType::Plaza | StructType::Pyramid |
                        StructType::Tower | StructType::Dome
                    );
                    if !is_large { continue; }
                    let sx = ((inst.x - ox as f32) / BLOCK_SIZE as f32) as isize;
                    let sz = ((inst.z - oz as f32) / BLOCK_SIZE as f32) as isize;
                    if sx < 0 || sx >= n as isize || sz < 0 || sz >= n as isize { continue; }
                    let room_iy_floor = ((inst.y as f64) - 10.0 - wy_min) as usize;
                    let room_iy_ceil = ((inst.y as f64) - 4.0 - wy_min) as usize;
                    if room_iy_ceil >= grid_ny { continue; }
                    let rs = 3_usize;
                    let sx_min = (sx as usize).max(rs) - rs;
                    let sx_max = (sx as usize + rs).min(n - 1);
                    let sz_min = (sz as usize).max(rs) - rs;
                    let sz_max = (sz as usize + rs).min(n - 1);
                    for dz in sz_min..=sz_max {
                        for dx in sx_min..=sx_max {
                            let is_wall = dx == sx_min || dx == sx_max || dz == sz_min || dz == sz_max;
                            for iy in room_iy_floor..=room_iy_ceil {
                                if iy >= grid_ny { continue; }
                                let idx2 = iy * n * n + dz * n + dx;
                                let wy2 = wy_min + iy as f64 + BLOCK_SIZE * 0.5;
                                if wy2 > surface_h - 0.5 { continue; }
                                if is_wall || iy == room_iy_floor || iy == room_iy_ceil {
                                    if blocks[idx2] == terrain::BLK_STONE || blocks[idx2] == terrain::BLK_DIRT {
                                        let moss_seed = feature_hash(cx, cz, seed.wrapping_add(0x123), (iy * n * n + dz * n + dx) as u32);
                                        blocks[idx2] = if moss_seed > 0.94 { terrain::BLK_MOSS } else { terrain::BLK_STONE };
                                    }
                                } else {
                                    blocks[idx2] = terrain::BLK_AIR;
                                    // Glow shrooms on walls
                                    if is_wall {
                                        let shroom_seed = feature_hash(cx, cz, seed.wrapping_add(0x456), (iy * n * n + dz * n + dx) as u32);
                                        if shroom_seed > 0.97 {
                                            blocks[idx2] = terrain::BLK_GLOW_SHROOM;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Treasure at center
                    let cx_idx = room_iy_floor + 1;
                    let cz_idx = sz_min + (sz_max - sz_min) / 2;
                    let ci_idx = sx_min + (sx_max - sx_min) / 2;
                    if cx_idx < grid_ny {
                        let center_idx = cx_idx * n * n + cz_idx * n + ci_idx;
                        let center_seed = feature_hash(cx, cz, seed ^ 0x789, (cx_idx * n * n + cz_idx * n + ci_idx) as u32);
                        if center_idx < blocks.len() {
                            blocks[center_idx] = if center_seed > 0.998 { terrain::BLK_DIAMOND_ORE }
                                else if center_seed > 0.99 { terrain::BLK_GOLD_ORE }
                                else { terrain::BLK_IRON_ORE };
                        }
                    }
                }
            }
        }
    }
}

/// Compute lighting at a block position by checking nearby light-emitting blocks.
/// surface_h is the terrain surface height above this column.
fn block_light_level(wx: f64, wy: f64, wz: f64, surface_h: f64,
    blocks: &[u8], n: usize, grid_ny: usize, wy_min: f64, ox: f64, oz: f64) -> f32
{
    let depth_below_surface = surface_h - wy;
    let surface_light = if depth_below_surface <= 0.5 { 1.0 }
        else { (1.0 - (depth_below_surface - 0.5) / 6.0).clamp(0.1, 1.0) };

    let mut nearby_light = 0.0_f32;
    let check_radius = 4.0;
    let steps = 5;
    let step_sz = check_radius / steps as f64;

    let base_ix = ((wx - ox) / BLOCK_SIZE) as i32;
    let base_iy = ((wy - wy_min) / BLOCK_SIZE) as i32;
    let base_iz = ((wz - oz) / BLOCK_SIZE) as i32;

    for diy in -steps..=steps {
        let iy = base_iy + diy;
        if iy < 0 || iy >= grid_ny as i32 { continue; }
        let dy = diy as f64 * step_sz;
        for dix in -steps..=steps {
            let ix = base_ix + dix;
            if ix < 0 || ix >= n as i32 { continue; }
            let dx = dix as f64 * step_sz;
            for diz in -steps..=steps {
                let iz = base_iz + diz;
                if iz < 0 || iz >= n as i32 { continue; }
                let dz = diz as f64 * step_sz;

                let dist2 = (dx*dx + dy*dy + dz*dz) as f32;
                if dist2 > (check_radius * check_radius) as f32 { continue; }

                let block_idx = iy as usize * n * n + iz as usize * n + ix as usize;
                let bt = blocks[block_idx];
                if terrain::block_emits_light(bt) {
                    let light_strength = 1.0 - (dist2.sqrt() / check_radius as f32).clamp(0.0, 1.0);
                    nearby_light = nearby_light.max(light_strength * 0.8);
                }
            }
        }
    }

    let total_light = surface_light.max(nearby_light as f64);
    total_light.clamp(0.1, 1.0) as f32
}

pub fn compute_chunk_data(params: &WorldParams, cx: i32, cz: i32, pool: &mut MeshPool) -> ChunkData {
    compute_chunk_data_lod(params, cx, cz, 0, pool)
}

fn underground_layers(lod: u32) -> i32 {
    match lod {
        0 => UNDERGROUND_LAYERS,
        1 => 8,
        _ => 2,
    }
}

pub fn compute_chunk_data_lod(params: &WorldParams, cx: i32, cz: i32, lod: u32, pool: &mut MeshPool) -> ChunkData {
    let mutated = apply_mutation(params, cx, cz);
    let p = &mutated;

    let ox = cx as f64 * CHUNK_SIZE;
    let oz = cz as f64 * CHUNK_SIZE;
    let n = BLOCK_RES as usize;
    let step = BLOCK_SIZE;
    let ug_layers = underground_layers(lod);

    get_height_map(p, cx, cz, &mut pool.heights);
    let heights = &pool.heights;
    let max_h = heights.iter().cloned().fold(0.0_f64, f64::max);

    let wy_min = -(ug_layers as f64);
    let wy_max = CHUNK_SIZE;
    let grid_ny = (wy_max - wy_min) as usize;

    pool.blocks.clear();
    pool.blocks.resize(n * grid_ny * n, 0u8);
    let blocks = &mut pool.blocks;
    pool.zones.clear();
    pool.zones.resize(n * n, Zone::Forest);
    let zones = &mut pool.zones;

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
                blocks[idx] = bt;
            }
        }
    }

    // Apply underground features only for LOD 0 (full detail)
    if lod == 0 {
        apply_underground_features(p, cx, cz, heights, zones, blocks, n, grid_ny, wy_min);
    }

    // Find surface block index for each column (topmost non-air block)
    pool.surface_iy.clear();
    pool.surface_iy.resize(n * n, 0usize);
    let surface_iy = &mut pool.surface_iy;
    for iz in 0..n {
        for ix in 0..n {
            let col_offset = iz * n + ix;
            for iy in (0..grid_ny).rev() {
                if blocks[iy * n * n + col_offset] != BLK_AIR {
                    surface_iy[col_offset] = iy;
                    break;
                }
            }
        }
    }

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let sample_step = match lod {
        0 => 1,
        1 => 2,
        _ => 4,
    };

    // Second pass: mesh visible block faces (sides + bottom only, top handled by surface mesh)
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

                let light = if terrain::block_emits_light(blocks[idx]) {
                    1.0_f32
                } else if sample_step > 1 {
                    0.7_f32
                } else if lod == 0 {
                    block_light_level(wx, wy, wz, surface_h, &blocks, n, grid_ny, wy_min, ox, oz)
                } else {
                    0.7_f32
                };
                let rl = (r * light).clamp(0.0, 1.0);
                let gl = (g * light).clamp(0.0, 1.0);
                let bl = (b * light).clamp(0.0, 1.0);

                let wx_f = wx as f32;
                let wy_f = wy as f32;
                let wz_f = wz as f32;
                let bt = blocks[idx];

                // +X
                if ix + sample_step < n {
                    if blocks[iy * n * n + iz * n + (ix + sample_step)] == 0 {
                        emit_face(&mut positions, &mut normals, &mut indices, &mut colors, &mut uvs, wx_f, wy_f, wz_f, 0, rl, gl, bl, bt, None, 0);
                    }
                } else if is_air(p, wx + BLOCK_SIZE * sample_step as f64, wy, wz) {
                    emit_face(&mut positions, &mut normals, &mut indices, &mut colors, &mut uvs, wx_f, wy_f, wz_f, 0, rl, gl, bl, bt, None, 0);
                }
                // -X
                if ix >= sample_step {
                    if blocks[iy * n * n + iz * n + (ix - sample_step)] == 0 {
                        emit_face(&mut positions, &mut normals, &mut indices, &mut colors, &mut uvs, wx_f, wy_f, wz_f, 1, rl, gl, bl, bt, None, 0);
                    }
                } else if is_air(p, wx - BLOCK_SIZE * sample_step as f64, wy, wz) {
                    emit_face(&mut positions, &mut normals, &mut indices, &mut colors, &mut uvs, wx_f, wy_f, wz_f, 1, rl, gl, bl, bt, None, 0);
                }
                // +Y (top) — only emit for underground blocks (not surface)
                if iy != surface_iy[iz * n + ix] && iy + sample_step < grid_ny {
                    if blocks[(iy + sample_step) * n * n + iz * n + ix] == 0 {
                        emit_face(&mut positions, &mut normals, &mut indices, &mut colors, &mut uvs, wx_f, wy_f, wz_f, 2, rl, gl, bl, bt, None, 0);
                    }
                }
                // -Y (bottom)
                if iy >= sample_step {
                    if blocks[(iy - sample_step) * n * n + iz * n + ix] == 0 {
                        emit_face(&mut positions, &mut normals, &mut indices, &mut colors, &mut uvs, wx_f, wy_f, wz_f, 3, rl, gl, bl, bt, None, 0);
                    }
                }
                // +Z
                if iz + sample_step < n {
                    if blocks[iy * n * n + (iz + sample_step) * n + ix] == 0 {
                        emit_face(&mut positions, &mut normals, &mut indices, &mut colors, &mut uvs, wx_f, wy_f, wz_f, 4, rl, gl, bl, bt, None, 0);
                    }
                } else if is_air(p, wx, wy, wz + BLOCK_SIZE * sample_step as f64) {
                    emit_face(&mut positions, &mut normals, &mut indices, &mut colors, &mut uvs, wx_f, wy_f, wz_f, 4, rl, gl, bl, bt, None, 0);
                }
                // -Z
                if iz >= sample_step {
                    if blocks[iy * n * n + (iz - sample_step) * n + ix] == 0 {
                        emit_face(&mut positions, &mut normals, &mut indices, &mut colors, &mut uvs, wx_f, wy_f, wz_f, 5, rl, gl, bl, bt, None, 0);
                    }
                } else if is_air(p, wx, wy, wz - BLOCK_SIZE * sample_step as f64) {
                    emit_face(&mut positions, &mut normals, &mut indices, &mut colors, &mut uvs, wx_f, wy_f, wz_f, 5, rl, gl, bl, bt, None, 0);
                }

                iy += sample_step;
            }
            ix += sample_step;
        }
        iz += sample_step;
    }

    // Smooth normals of the flat-shaded subsurface (average at shared positions)
    if !positions.is_empty() {
        use std::collections::HashMap;
        let nv = positions.len() / 3;
        let mut nmap: HashMap<u64, (f32, f32, f32, u32)> = HashMap::new();
        for i in 0..nv {
            let px = (positions[i * 3] as f64 * 2.0).round() as i64;
            let py = (positions[i * 3 + 1] as f64 * 2.0).round() as i64;
            let pz = (positions[i * 3 + 2] as f64 * 2.0).round() as i64;
            let key = px.wrapping_mul(374761393).wrapping_add(py.wrapping_mul(668265263)).wrapping_add(pz.wrapping_mul(1274126177)) as u64;
            let e = nmap.entry(key).or_insert((0.0, 0.0, 0.0, 0));
            e.0 += normals[i * 3]; e.1 += normals[i * 3 + 1]; e.2 += normals[i * 3 + 2]; e.3 += 1;
        }
        for i in 0..nv {
            let px = (positions[i * 3] as f64 * 2.0).round() as i64;
            let py = (positions[i * 3 + 1] as f64 * 2.0).round() as i64;
            let pz = (positions[i * 3 + 2] as f64 * 2.0).round() as i64;
            let key = px.wrapping_mul(374761393).wrapping_add(py.wrapping_mul(668265263)).wrapping_add(pz.wrapping_mul(1274126177)) as u64;
            if let Some(&(nx, ny, nz, _)) = nmap.get(&key) {
                let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.001);
                normals[i * 3] = nx / len; normals[i * 3 + 1] = ny / len; normals[i * 3 + 2] = nz / len;
            }
        }
    }

    // Generate smooth terrain surface mesh from heightmap
    // Uses averaged per-corner normals for smooth shading (R1)
    {
        let cn = n + 1;
        pool.corner_heights.clear();
        pool.corner_heights.resize(cn * cn, 0.0_f64);
        let corner_heights = &mut pool.corner_heights;
        for iz in 0..=n {
            for ix in 0..=n {
                let wx = ox + ix as f64 * step;
                let wz = oz + iz as f64 * step;
                let mut h = terrain::get_height(p, wx, wz);
                terrain::zone_effects(p, wx, wz, &mut h);
                corner_heights[iz * cn + ix] = h;
            }
        }

        // First pass: accumulate normals at each corner (iz, ix)
        let corner_normals = &mut pool.corner_normals;
        corner_normals.clear();
        corner_normals.resize(cn * cn, [0.0_f32; 3]);
        for iz in 0..n {
            for ix in 0..n {
                let h00 = corner_heights[iz * cn + ix] as f32;
                let h10 = corner_heights[iz * cn + (ix + 1)] as f32;
                let h01 = corner_heights[(iz + 1) * cn + ix] as f32;
                let h11 = corner_heights[(iz + 1) * cn + (ix + 1)] as f32;

                let x0 = (ox + ix as f64 * step) as f32;
                let z0 = (oz + iz as f64 * step) as f32;
                let x1 = (ox + (ix + 1) as f64 * step) as f32;
                let z1 = (oz + (iz + 1) as f64 * step) as f32;

                // Tri 1: (ix,iz) -> (ix+1,iz+1) -> (ix+1,iz)
                let p0 = [x0, h00, z0]; let p1 = [x1, h11, z1]; let p2 = [x1, h10, z0];
                let e1x = p1[0]-p0[0]; let e1y = p1[1]-p0[1]; let e1z = p1[2]-p0[2];
                let e2x = p2[0]-p0[0]; let e2y = p2[1]-p0[1]; let e2z = p2[2]-p0[2];
                let nx = e1y*e2z - e1z*e2y; let ny = e1z*e2x - e1x*e2z; let nz = e1x*e2y - e1y*e2x;

                let c00 = iz * cn + ix;
                let c10 = iz * cn + (ix + 1);
                let c01 = (iz + 1) * cn + ix;
                let c11 = (iz + 1) * cn + (ix + 1);

                corner_normals[c00][0] += nx; corner_normals[c00][1] += ny; corner_normals[c00][2] += nz;
                corner_normals[c11][0] += nx; corner_normals[c11][1] += ny; corner_normals[c11][2] += nz;
                corner_normals[c10][0] += nx; corner_normals[c10][1] += ny; corner_normals[c10][2] += nz;

                // Tri 2: (ix,iz) -> (ix,iz+1) -> (ix+1,iz+1)
                let p3 = [x0, h01, z1];
                let e3x = p3[0]-p0[0]; let e3y = p3[1]-p0[1]; let e3z = p3[2]-p0[2];
                let e4x = p1[0]-p0[0]; let e4y = p1[1]-p0[1]; let e4z = p1[2]-p0[2];
                let nx2 = e3y*e4z - e3z*e4y; let ny2 = e3z*e4x - e3x*e4z; let nz2 = e3x*e4y - e3y*e4x;

                corner_normals[c00][0] += nx2; corner_normals[c00][1] += ny2; corner_normals[c00][2] += nz2;
                corner_normals[c01][0] += nx2; corner_normals[c01][1] += ny2; corner_normals[c01][2] += nz2;
                corner_normals[c11][0] += nx2; corner_normals[c11][1] += ny2; corner_normals[c11][2] += nz2;
            }
        }

        // Normalize corner normals
        for cn_ref in &mut *corner_normals {
            let len = (cn_ref[0]*cn_ref[0] + cn_ref[1]*cn_ref[1] + cn_ref[2]*cn_ref[2]).sqrt().max(0.001);
            cn_ref[0] /= len; cn_ref[1] /= len; cn_ref[2] /= len;
        }

        // Second pass: output mesh vertices with smoothed normals
        for iz in 0..n {
            for ix in 0..n {
                let col_offset = iz * n + ix;
                let s_iy = surface_iy[col_offset];
                let bt = blocks[s_iy * n * n + col_offset];

                let wz_center = oz + iz as f64 * step + BLOCK_SIZE * 0.5;
                let wx_center = ox + ix as f64 * step + BLOCK_SIZE * 0.5;
                let zone = zones[col_offset];
                let surface_h = heights[col_offset];

                let h00 = corner_heights[iz * cn + ix] as f32;
                let h10 = corner_heights[iz * cn + (ix + 1)] as f32;
                let h01 = corner_heights[(iz + 1) * cn + ix] as f32;
                let h11 = corner_heights[(iz + 1) * cn + (ix + 1)] as f32;

                let dzdx = ((h10 - h00) + (h11 - h01)) / (2.0 * step as f32);
                let dzdy = ((h01 - h00) + (h11 - h10)) / (2.0 * step as f32);
                let slope = (dzdx * dzdx + dzdy * dzdy).sqrt();

                let effective_bt = if surface_h > max_h * 0.85 && !matches!(zone, Zone::Desert | Zone::Tundra) {
                    BLK_SNOW
                } else if slope > 0.6 {
                    BLK_STONE
                } else if slope > 0.3 && !matches!(zone, Zone::Desert | Zone::SandyPlain) {
                    BLK_DIRT
                } else {
                    bt
                };

                let tile = block_atlas_tile(effective_bt);
                let (u0, v0, u1, v1) = tile_uv(tile);

                let c = terrain::block_color(effective_bt, p, wx_center, surface_h, wz_center, zone, surface_h, max_h);
                let (r, g, b) = {
                    let (h, s, l) = rgb_to_hsl(c[0], c[1], c[2]);
                    hsl_to_rgb(
                        (h + params.hue_shift as f32 / 360.0) % 1.0,
                        (s * params.saturation as f32).clamp(0.0, 1.0),
                        (l * params.lightness as f32).clamp(0.0, 1.0),
                    )
                };

                let light = if terrain::block_emits_light(effective_bt) {
                    1.0_f32
                } else if lod == 0 {
                    block_light_level(wx_center, surface_h, wz_center, surface_h, &blocks, n, grid_ny, wy_min, ox, oz)
                } else {
                    0.7_f32
                };
                let rl = (r * light).clamp(0.0, 1.0);
                let gl = (g * light).clamp(0.0, 1.0);
                let bl = (b * light).clamp(0.0, 1.0);

                let x0 = (ox + ix as f64 * step) as f32;
                let z0 = (oz + iz as f64 * step) as f32;
                let x1 = (ox + (ix + 1) as f64 * step) as f32;
                let z1 = (oz + (iz + 1) as f64 * step) as f32;

                let p0 = [x0, h00, z0]; let p1 = [x1, h11, z1]; let p2 = [x1, h10, z0]; let p3 = [x0, h01, z1];

                let n00 = corner_normals[iz * cn + ix];
                let n10 = corner_normals[iz * cn + (ix + 1)];
                let n01 = corner_normals[(iz + 1) * cn + ix];
                let n11 = corner_normals[(iz + 1) * cn + (ix + 1)];

                let nv = (positions.len() / 3) as u32;

                for &(v, n) in &[(p0, n00), (p1, n11), (p2, n10)] {
                    positions.push(v[0]); positions.push(v[1]); positions.push(v[2]);
                    normals.push(n[0]); normals.push(n[1]); normals.push(n[2]);
                    colors.push(rl); colors.push(gl); colors.push(bl);
                }
                uvs.push(u0); uvs.push(v0);
                uvs.push(u1); uvs.push(v1);
                uvs.push(u1); uvs.push(v0);
                indices.push(nv); indices.push(nv + 1); indices.push(nv + 2);

                for &(v, n) in &[(p0, n00), (p3, n01), (p1, n11)] {
                    positions.push(v[0]); positions.push(v[1]); positions.push(v[2]);
                    normals.push(n[0]); normals.push(n[1]); normals.push(n[2]);
                    colors.push(rl); colors.push(gl); colors.push(bl);
                }
                uvs.push(u0); uvs.push(v0);
                uvs.push(u0); uvs.push(v1);
                uvs.push(u1); uvs.push(v1);
                indices.push(nv + 3); indices.push(nv + 4); indices.push(nv + 5);
            }
        }
    }

    ChunkData { cx, cz, positions, normals, colors, uvs, indices, block_types: vec![] }
}