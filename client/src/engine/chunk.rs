
use crate::engine::terrain;
use crate::engine::terrain::{Zone, BLK_AIR, BLK_DIRT, BLK_SNOW, BLK_STONE};
use crate::engine::structures;
use crate::math::{hsl_to_rgb, rgb_to_hsl};
use crate::state::WorldParams;

pub const CHUNK_SIZE: f64 = 24.0;
pub const BLOCK_RES: u32 = 24;
const BLOCK_SIZE: f64 = CHUNK_SIZE / BLOCK_RES as f64;
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
    pub sdf_grid: Vec<f64>,
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
            sdf_grid: Vec::new(),
        }
    }

    pub fn clear_all(&mut self) {
        self.blocks.clear();
        self.zones.clear();
        self.heights.clear();
        self.surface_iy.clear();
        self.corner_heights.clear();
        self.corner_normals.clear();
        self.sdf_grid.clear();
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

// Which two corners each edge (0-11) connects.
const EDGE_VERTS: [[usize; 2]; 12] = [
    [0, 1], [1, 2], [3, 2], [0, 3],
    [4, 5], [5, 6], [7, 6], [4, 7],
    [0, 4], [1, 5], [2, 6], [3, 7],
];

// From http://paulbourke.net/geometry/polygonise/
pub const EDGE_TABLE: [u16; 256] = [
    0x000, 0x109, 0x203, 0x30a, 0x406, 0x50f, 0x605, 0x70c,
    0x80c, 0x905, 0xa0f, 0xb06, 0xc0a, 0xd03, 0xe09, 0xf00,
    0x190, 0x099, 0x393, 0x29a, 0x596, 0x49f, 0x795, 0x69c,
    0x99c, 0x895, 0xb9f, 0xa96, 0xd9a, 0xc93, 0xf99, 0xe90,
    0x230, 0x339, 0x033, 0x13a, 0x636, 0x73f, 0x435, 0x53c,
    0xa3c, 0xb35, 0x83f, 0x936, 0xe3a, 0xf33, 0xc39, 0xd30,
    0x3a0, 0x2a9, 0x1a3, 0x0aa, 0x7a6, 0x6af, 0x5a5, 0x4ac,
    0xbac, 0xaa5, 0x9af, 0x8a6, 0xfaa, 0xea3, 0xda9, 0xca0,
    0x460, 0x569, 0x663, 0x76a, 0x066, 0x16f, 0x265, 0x36c,
    0xc6c, 0xd65, 0xe6f, 0xf66, 0x86a, 0x963, 0xa69, 0xb60,
    0x5f0, 0x4f9, 0x7f3, 0x6fa, 0x1f6, 0x0ff, 0x3f5, 0x2fc,
    0xdfc, 0xcf5, 0xfff, 0xef6, 0x9fa, 0x8f3, 0xbf9, 0xaf0,
    0x650, 0x759, 0x453, 0x55a, 0x256, 0x35f, 0x055, 0x15c,
    0xe5c, 0xf55, 0xc5f, 0xd56, 0xa5a, 0xb53, 0x859, 0x950,
    0x7c0, 0x6c9, 0x5c3, 0x4ca, 0x3c6, 0x2cf, 0x1c5, 0x0cc,
    0xfcc, 0xec5, 0xdcf, 0xcc6, 0xbca, 0xac3, 0x9c9, 0x8c0,
    0x8c0, 0x9c9, 0xac3, 0xbca, 0xcc6, 0xdcf, 0xec5, 0xfcc,
    0x0cc, 0x1c5, 0x2cf, 0x3c6, 0x4ca, 0x5c3, 0x6c9, 0x7c0,
    0x950, 0x859, 0xb53, 0xa5a, 0xd56, 0xc5f, 0xf55, 0xe5c,
    0x15c, 0x055, 0x35f, 0x256, 0x55a, 0x453, 0x759, 0x650,
    0xaf0, 0xbf9, 0x8f3, 0x9fa, 0xef6, 0xfff, 0xcf5, 0xdfc,
    0x2fc, 0x3f5, 0x0ff, 0x1f6, 0x6fa, 0x7f3, 0x4f9, 0x5f0,
    0xb60, 0xa69, 0x963, 0x86a, 0xf66, 0xe6f, 0xd65, 0xc6c,
    0x36c, 0x265, 0x16f, 0x066, 0x76a, 0x663, 0x569, 0x460,
    0xca0, 0xda9, 0xea3, 0xfaa, 0x8a6, 0x9af, 0xaa5, 0xbac,
    0x4ac, 0x5a5, 0x6af, 0x7a6, 0x0aa, 0x1a3, 0x2a9, 0x3a0,
    0xd30, 0xc39, 0xf33, 0xe3a, 0x936, 0x83f, 0xb35, 0xa3c,
    0x53c, 0x435, 0x73f, 0x636, 0x13a, 0x033, 0x339, 0x230,
    0xe90, 0xf99, 0xc93, 0xd9a, 0xa96, 0xb9f, 0x895, 0x99c,
    0x69c, 0x795, 0x49f, 0x596, 0x29a, 0x393, 0x099, 0x190,
    0xf00, 0xe09, 0xd03, 0xc0a, 0xb06, 0xa0f, 0x905, 0x80c,
    0x70c, 0x605, 0x50f, 0x406, 0x30a, 0x203, 0x109, 0x000,
];

pub const TRI_TABLE: [[i8; 16]; 256] = [
    [ -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   8,   3,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   1,   9,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   8,   3,   9,   8,   1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   2,  10,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   8,   3,   1,   2,  10,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   2,  10,   0,   2,   9,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  2,   8,   3,   2,  10,   8,  10,   9,   8,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,  11,   2,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,  11,   2,   8,  11,   0,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   9,   0,   2,   3,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,  11,   2,   1,   9,  11,   9,   8,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,  10,   1,  11,  10,   3,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,  10,   1,   0,   8,  10,   8,  11,  10,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,   9,   0,   3,  11,   9,  11,  10,   9,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   8,  10,  10,   8,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  4,   7,   8,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  4,   3,   0,   7,   3,   4,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   1,   9,   8,   4,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  4,   1,   9,   4,   7,   1,   7,   3,   1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   2,  10,   8,   4,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,   4,   7,   3,   0,   4,   1,   2,  10,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   2,  10,   9,   0,   2,   8,   4,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  2,  10,   9,   2,   9,   7,   2,   7,   3,   7,   9,   4,  -1,  -1,  -1,  -1],
    [  8,   4,   7,   3,  11,   2,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 11,   4,   7,  11,   2,   4,   2,   0,   4,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   0,   1,   8,   4,   7,   2,   3,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  4,   7,  11,   9,   4,  11,   9,  11,   2,   9,   2,   1,  -1,  -1,  -1,  -1],
    [  3,  10,   1,   3,  11,  10,   7,   8,   4,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,  11,  10,   1,   4,  11,   1,   0,   4,   7,  11,   4,  -1,  -1,  -1,  -1],
    [  4,   7,   8,   9,   0,  11,   9,  11,  10,  11,   0,   3,  -1,  -1,  -1,  -1],
    [  4,   7,  11,   4,  11,   9,   9,  11,  10,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   5,   4,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   5,   4,   0,   8,   3,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   5,   4,   1,   5,   0,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  8,   5,   4,   8,   3,   5,   3,   1,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   2,  10,   9,   5,   4,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,   0,   8,   1,   2,  10,   4,   9,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  5,   2,  10,   5,   4,   2,   4,   0,   2,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  2,  10,   5,   3,   2,   5,   3,   5,   4,   3,   4,   8,  -1,  -1,  -1,  -1],
    [  9,   5,   4,   2,   3,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,  11,   2,   0,   8,  11,   4,   9,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   5,   4,   0,   1,   5,   2,   3,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  2,   1,   5,   2,   5,   8,   2,   8,  11,   4,   8,   5,  -1,  -1,  -1,  -1],
    [ 10,   3,  11,  10,   1,   3,   9,   5,   4,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  4,   9,   5,   0,   8,   1,   8,  10,   1,   8,  11,  10,  -1,  -1,  -1,  -1],
    [  5,   4,   0,   5,   0,  11,   5,  11,  10,  11,   0,   3,  -1,  -1,  -1,  -1],
    [  5,   4,   8,   5,   8,  10,  10,   8,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   7,   8,   5,   7,   9,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   3,   0,   9,   5,   3,   5,   7,   3,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   7,   8,   0,   1,   7,   1,   5,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   5,   3,   3,   5,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   7,   8,   9,   5,   7,  10,   1,   2,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 10,   1,   2,   9,   5,   0,   5,   3,   0,   5,   7,   3,  -1,  -1,  -1,  -1],
    [  8,   0,   2,   8,   2,   5,   8,   5,   7,  10,   5,   2,  -1,  -1,  -1,  -1],
    [  2,  10,   5,   2,   5,   3,   3,   5,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  7,   9,   5,   7,   8,   9,   3,  11,   2,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   5,   7,   9,   7,   2,   9,   2,   0,   2,   7,  11,  -1,  -1,  -1,  -1],
    [  2,   3,  11,   0,   1,   8,   1,   7,   8,   1,   5,   7,  -1,  -1,  -1,  -1],
    [ 11,   2,   1,  11,   1,   7,   7,   1,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   5,   8,   8,   5,   7,  10,   1,   3,  10,   3,  11,  -1,  -1,  -1,  -1],
    [  5,   7,   0,   5,   0,   9,   7,  11,   0,   1,   0,  10,  11,  10,   0,  -1],
    [ 11,  10,   0,  11,   0,   3,  10,   5,   0,   8,   0,   7,   5,   7,   0,  -1],
    [ 11,  10,   5,   7,  11,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 10,   6,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   8,   3,   5,  10,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   0,   1,   5,  10,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   8,   3,   1,   9,   8,   5,  10,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   6,   5,   2,   6,   1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   6,   5,   1,   2,   6,   3,   0,   8,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   6,   5,   9,   0,   6,   0,   2,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  5,   9,   8,   5,   8,   2,   5,   2,   6,   3,   2,   8,  -1,  -1,  -1,  -1],
    [  2,   3,  11,  10,   6,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 11,   0,   8,  11,   2,   0,  10,   6,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   1,   9,   2,   3,  11,   5,  10,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  5,  10,   6,   1,   9,   2,   9,  11,   2,   9,   8,  11,  -1,  -1,  -1,  -1],
    [  6,   3,  11,   6,   5,   3,   5,   1,   3,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   8,  11,   0,  11,   5,   0,   5,   1,   5,  11,   6,  -1,  -1,  -1,  -1],
    [  3,  11,   6,   0,   3,   6,   0,   6,   5,   0,   5,   9,  -1,  -1,  -1,  -1],
    [  6,   5,   9,   6,   9,  11,  11,   9,   8,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  5,  10,   6,   4,   7,   8,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  4,   3,   0,   4,   7,   3,   6,   5,  10,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   9,   0,   5,  10,   6,   8,   4,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 10,   6,   5,   1,   9,   7,   1,   7,   3,   7,   9,   4,  -1,  -1,  -1,  -1],
    [  6,   1,   2,   6,   5,   1,   4,   7,   8,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   2,   5,   5,   2,   6,   3,   0,   4,   3,   4,   7,  -1,  -1,  -1,  -1],
    [  8,   4,   7,   9,   0,   5,   0,   6,   5,   0,   2,   6,  -1,  -1,  -1,  -1],
    [  7,   3,   9,   7,   9,   4,   3,   2,   9,   5,   9,   6,   2,   6,   9,  -1],
    [  3,  11,   2,   7,   8,   4,  10,   6,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  5,  10,   6,   4,   7,   2,   4,   2,   0,   2,   7,  11,  -1,  -1,  -1,  -1],
    [  0,   1,   9,   4,   7,   8,   2,   3,  11,   5,  10,   6,  -1,  -1,  -1,  -1],
    [  9,   2,   1,   9,  11,   2,   9,   4,  11,   7,  11,   4,   5,  10,   6,  -1],
    [  8,   4,   7,   3,  11,   5,   3,   5,   1,   5,  11,   6,  -1,  -1,  -1,  -1],
    [  5,   1,  11,   5,  11,   6,   1,   0,  11,   7,  11,   4,   0,   4,  11,  -1],
    [  0,   5,   9,   0,   6,   5,   0,   3,   6,  11,   6,   3,   8,   4,   7,  -1],
    [  6,   5,   9,   6,   9,  11,   4,   7,   9,   7,  11,   9,  -1,  -1,  -1,  -1],
    [ 10,   4,   9,   6,   4,  10,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  4,  10,   6,   4,   9,  10,   0,   8,   3,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 10,   0,   1,  10,   6,   0,   6,   4,   0,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  8,   3,   1,   8,   1,   6,   8,   6,   4,   6,   1,  10,  -1,  -1,  -1,  -1],
    [  1,   4,   9,   1,   2,   4,   2,   6,   4,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,   0,   8,   1,   2,   9,   2,   4,   9,   2,   6,   4,  -1,  -1,  -1,  -1],
    [  0,   2,   4,   4,   2,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  8,   3,   2,   8,   2,   4,   4,   2,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 10,   4,   9,  10,   6,   4,  11,   2,   3,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   8,   2,   2,   8,  11,   4,   9,  10,   4,  10,   6,  -1,  -1,  -1,  -1],
    [  3,  11,   2,   0,   1,   6,   0,   6,   4,   6,   1,  10,  -1,  -1,  -1,  -1],
    [  6,   4,   1,   6,   1,  10,   4,   8,   1,   2,   1,  11,   8,  11,   1,  -1],
    [  9,   6,   4,   9,   3,   6,   9,   1,   3,  11,   6,   3,  -1,  -1,  -1,  -1],
    [  8,  11,   1,   8,   1,   0,  11,   6,   1,   9,   1,   4,   6,   4,   1,  -1],
    [  3,  11,   6,   3,   6,   0,   0,   6,   4,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  6,   4,   8,  11,   6,   8,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  7,  10,   6,   7,   8,  10,   8,   9,  10,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   7,   3,   0,  10,   7,   0,   9,  10,   6,   7,  10,  -1,  -1,  -1,  -1],
    [ 10,   6,   7,   1,  10,   7,   1,   7,   8,   1,   8,   0,  -1,  -1,  -1,  -1],
    [ 10,   6,   7,  10,   7,   1,   1,   7,   3,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   2,   6,   1,   6,   8,   1,   8,   9,   8,   6,   7,  -1,  -1,  -1,  -1],
    [  2,   6,   9,   2,   9,   1,   6,   7,   9,   0,   9,   3,   7,   3,   9,  -1],
    [  7,   8,   0,   7,   0,   6,   6,   0,   2,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  7,   3,   2,   6,   7,   2,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  2,   3,  11,  10,   6,   8,  10,   8,   9,   8,   6,   7,  -1,  -1,  -1,  -1],
    [  2,   0,   7,   2,   7,  11,   0,   9,   7,   6,   7,  10,   9,  10,   7,  -1],
    [  1,   8,   0,   1,   7,   8,   1,  10,   7,   6,   7,  10,   2,   3,  11,  -1],
    [ 11,   2,   1,  11,   1,   7,  10,   6,   1,   6,   7,   1,  -1,  -1,  -1,  -1],
    [  8,   9,   6,   8,   6,   7,   9,   1,   6,  11,   6,   3,   1,   3,   6,  -1],
    [  0,   9,   1,  11,   6,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  7,   8,   0,   7,   0,   6,   3,  11,   0,  11,   6,   0,  -1,  -1,  -1,  -1],
    [  7,  11,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  7,   6,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,   0,   8,  11,   7,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   1,   9,  11,   7,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  8,   1,   9,   8,   3,   1,  11,   7,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 10,   1,   2,   6,  11,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   2,  10,   3,   0,   8,   6,  11,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  2,   9,   0,   2,  10,   9,   6,  11,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  6,  11,   7,   2,  10,   3,  10,   8,   3,  10,   9,   8,  -1,  -1,  -1,  -1],
    [  7,   2,   3,   6,   2,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  7,   0,   8,   7,   6,   0,   6,   2,   0,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  2,   7,   6,   2,   3,   7,   0,   1,   9,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   6,   2,   1,   8,   6,   1,   9,   8,   8,   7,   6,  -1,  -1,  -1,  -1],
    [ 10,   7,   6,  10,   1,   7,   1,   3,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 10,   7,   6,   1,   7,  10,   1,   8,   7,   1,   0,   8,  -1,  -1,  -1,  -1],
    [  0,   3,   7,   0,   7,  10,   0,  10,   9,   6,  10,   7,  -1,  -1,  -1,  -1],
    [  7,   6,  10,   7,  10,   8,   8,  10,   9,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  6,   8,   4,  11,   8,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,   6,  11,   3,   0,   6,   0,   4,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  8,   6,  11,   8,   4,   6,   9,   0,   1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   4,   6,   9,   6,   3,   9,   3,   1,  11,   3,   6,  -1,  -1,  -1,  -1],
    [  6,   8,   4,   6,  11,   8,   2,  10,   1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   2,  10,   3,   0,  11,   0,   6,  11,   0,   4,   6,  -1,  -1,  -1,  -1],
    [  4,  11,   8,   4,   6,  11,   0,   2,   9,   2,  10,   9,  -1,  -1,  -1,  -1],
    [ 10,   9,   3,  10,   3,   2,   9,   4,   3,  11,   3,   6,   4,   6,   3,  -1],
    [  8,   2,   3,   8,   4,   2,   4,   6,   2,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   4,   2,   4,   6,   2,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   9,   0,   2,   3,   4,   2,   4,   6,   4,   3,   8,  -1,  -1,  -1,  -1],
    [  1,   9,   4,   1,   4,   2,   2,   4,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  8,   1,   3,   8,   6,   1,   8,   4,   6,   6,  10,   1,  -1,  -1,  -1,  -1],
    [ 10,   1,   0,  10,   0,   6,   6,   0,   4,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  4,   6,   3,   4,   3,   8,   6,  10,   3,   0,   3,   9,  10,   9,   3,  -1],
    [ 10,   9,   4,   6,  10,   4,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  4,   9,   5,   7,   6,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   8,   3,   4,   9,   5,  11,   7,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  5,   0,   1,   5,   4,   0,   7,   6,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 11,   7,   6,   8,   3,   4,   3,   5,   4,   3,   1,   5,  -1,  -1,  -1,  -1],
    [  9,   5,   4,  10,   1,   2,   7,   6,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  6,  11,   7,   1,   2,  10,   0,   8,   3,   4,   9,   5,  -1,  -1,  -1,  -1],
    [  7,   6,  11,   5,   4,  10,   4,   2,  10,   4,   0,   2,  -1,  -1,  -1,  -1],
    [  3,   4,   8,   3,   5,   4,   3,   2,   5,  10,   5,   2,  11,   7,   6,  -1],
    [  7,   2,   3,   7,   6,   2,   5,   4,   9,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   5,   4,   0,   8,   6,   0,   6,   2,   6,   8,   7,  -1,  -1,  -1,  -1],
    [  3,   6,   2,   3,   7,   6,   1,   5,   0,   5,   4,   0,  -1,  -1,  -1,  -1],
    [  6,   2,   8,   6,   8,   7,   2,   1,   8,   4,   8,   5,   1,   5,   8,  -1],
    [  9,   5,   4,  10,   1,   6,   1,   7,   6,   1,   3,   7,  -1,  -1,  -1,  -1],
    [  1,   6,  10,   1,   7,   6,   1,   0,   7,   8,   7,   0,   9,   5,   4,  -1],
    [  4,   0,  10,   4,  10,   5,   0,   3,  10,   6,  10,   7,   3,   7,  10,  -1],
    [  7,   6,  10,   7,  10,   8,   5,   4,  10,   4,   8,  10,  -1,  -1,  -1,  -1],
    [  6,   9,   5,   6,  11,   9,  11,   8,   9,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,   6,  11,   0,   6,   3,   0,   5,   6,   0,   9,   5,  -1,  -1,  -1,  -1],
    [  0,  11,   8,   0,   5,  11,   0,   1,   5,   5,   6,  11,  -1,  -1,  -1,  -1],
    [  6,  11,   3,   6,   3,   5,   5,   3,   1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   2,  10,   9,   5,  11,   9,  11,   8,  11,   5,   6,  -1,  -1,  -1,  -1],
    [  0,  11,   3,   0,   6,  11,   0,   9,   6,   5,   6,   9,   1,   2,  10,  -1],
    [ 11,   8,   5,  11,   5,   6,   8,   0,   5,  10,   5,   2,   0,   2,   5,  -1],
    [  6,  11,   3,   6,   3,   5,   2,  10,   3,  10,   5,   3,  -1,  -1,  -1,  -1],
    [  5,   8,   9,   5,   2,   8,   5,   6,   2,   3,   8,   2,  -1,  -1,  -1,  -1],
    [  9,   5,   6,   9,   6,   0,   0,   6,   2,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   5,   8,   1,   8,   0,   5,   6,   8,   3,   8,   2,   6,   2,   8,  -1],
    [  1,   5,   6,   2,   1,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   3,   6,   1,   6,  10,   3,   8,   6,   5,   6,   9,   8,   9,   6,  -1],
    [ 10,   1,   0,  10,   0,   6,   9,   5,   0,   5,   6,   0,  -1,  -1,  -1,  -1],
    [  0,   3,   8,   5,   6,  10,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 10,   5,   6,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 11,   5,  10,   7,   5,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 11,   5,  10,  11,   7,   5,   8,   3,   0,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  5,  11,   7,   5,  10,  11,   1,   9,   0,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 10,   7,   5,  10,  11,   7,   9,   8,   1,   8,   3,   1,  -1,  -1,  -1,  -1],
    [ 11,   1,   2,  11,   7,   1,   7,   5,   1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   8,   3,   1,   2,   7,   1,   7,   5,   7,   2,  11,  -1,  -1,  -1,  -1],
    [  9,   7,   5,   9,   2,   7,   9,   0,   2,   2,  11,   7,  -1,  -1,  -1,  -1],
    [  7,   5,   2,   7,   2,  11,   5,   9,   2,   3,   2,   8,   9,   8,   2,  -1],
    [  2,   5,  10,   2,   3,   5,   3,   7,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  8,   2,   0,   8,   5,   2,   8,   7,   5,  10,   2,   5,  -1,  -1,  -1,  -1],
    [  9,   0,   1,   5,  10,   3,   5,   3,   7,   3,  10,   2,  -1,  -1,  -1,  -1],
    [  9,   8,   2,   9,   2,   1,   8,   7,   2,  10,   2,   5,   7,   5,   2,  -1],
    [  1,   3,   5,   3,   7,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   8,   7,   0,   7,   1,   1,   7,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   0,   3,   9,   3,   5,   5,   3,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,   8,   7,   5,   9,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  5,   8,   4,   5,  10,   8,  10,  11,   8,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  5,   0,   4,   5,  11,   0,   5,  10,  11,  11,   3,   0,  -1,  -1,  -1,  -1],
    [  0,   1,   9,   8,   4,  10,   8,  10,  11,  10,   4,   5,  -1,  -1,  -1,  -1],
    [ 10,  11,   4,  10,   4,   5,  11,   3,   4,   9,   4,   1,   3,   1,   4,  -1],
    [  2,   5,   1,   2,   8,   5,   2,  11,   8,   4,   5,   8,  -1,  -1,  -1,  -1],
    [  0,   4,  11,   0,  11,   3,   4,   5,  11,   2,  11,   1,   5,   1,  11,  -1],
    [  0,   2,   5,   0,   5,   9,   2,  11,   5,   4,   5,   8,  11,   8,   5,  -1],
    [  9,   4,   5,   2,  11,   3,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  2,   5,  10,   3,   5,   2,   3,   4,   5,   3,   8,   4,  -1,  -1,  -1,  -1],
    [  5,  10,   2,   5,   2,   4,   4,   2,   0,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,  10,   2,   3,   5,  10,   3,   8,   5,   4,   5,   8,   0,   1,   9,  -1],
    [  5,  10,   2,   5,   2,   4,   1,   9,   2,   9,   4,   2,  -1,  -1,  -1,  -1],
    [  8,   4,   5,   8,   5,   3,   3,   5,   1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   4,   5,   1,   0,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  8,   4,   5,   8,   5,   3,   9,   0,   5,   0,   3,   5,  -1,  -1,  -1,  -1],
    [  9,   4,   5,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  4,  11,   7,   4,   9,  11,   9,  10,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   8,   3,   4,   9,   7,   9,  11,   7,   9,  10,  11,  -1,  -1,  -1,  -1],
    [  1,  10,  11,   1,  11,   4,   1,   4,   0,   7,   4,  11,  -1,  -1,  -1,  -1],
    [  3,   1,   4,   3,   4,   8,   1,  10,   4,   7,   4,  11,  10,  11,   4,  -1],
    [  4,  11,   7,   9,  11,   4,   9,   2,  11,   9,   1,   2,  -1,  -1,  -1,  -1],
    [  9,   7,   4,   9,  11,   7,   9,   1,  11,   2,  11,   1,   0,   8,   3,  -1],
    [ 11,   7,   4,  11,   4,   2,   2,   4,   0,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ 11,   7,   4,  11,   4,   2,   8,   3,   4,   3,   2,   4,  -1,  -1,  -1,  -1],
    [  2,   9,  10,   2,   7,   9,   2,   3,   7,   7,   4,   9,  -1,  -1,  -1,  -1],
    [  9,  10,   7,   9,   7,   4,  10,   2,   7,   8,   7,   0,   2,   0,   7,  -1],
    [  3,   7,  10,   3,  10,   2,   7,   4,  10,   1,  10,   0,   4,   0,  10,  -1],
    [  1,  10,   2,   8,   7,   4,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  4,   9,   1,   4,   1,   7,   7,   1,   3,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  4,   9,   1,   4,   1,   7,   0,   8,   1,   8,   7,   1,  -1,  -1,  -1,  -1],
    [  4,   0,   3,   7,   4,   3,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  4,   8,   7,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,  10,   8,  10,  11,   8,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,   0,   9,   3,   9,  11,  11,   9,  10,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   1,  10,   0,  10,   8,   8,  10,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,   1,  10,  11,   3,  10,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   2,  11,   1,  11,   9,   9,  11,   8,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,   0,   9,   3,   9,  11,   1,   2,   9,   2,  11,   9,  -1,  -1,  -1,  -1],
    [  0,   2,  11,   8,   0,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  3,   2,  11,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  2,   3,   8,   2,   8,  10,  10,   8,   9,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  9,  10,   2,   0,   9,   2,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  2,   3,   8,   2,   8,  10,   0,   1,   8,   1,  10,   8,  -1,  -1,  -1,  -1],
    [  1,  10,   2,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  1,   3,   8,   9,   1,   8,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   9,   1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [  0,   3,   8,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
    [ -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1,  -1],
];

/// Extract isosurface from SDF grid using marching cubes.
/// SDF layout: `sdf[i + j*nx + k*nx*ny]` for corner (i,j,k).
/// Output appended to positions/normals/indices.
pub fn extract_isosurface(
    sdf: &[f64],
    nx: usize, ny: usize, nz: usize,
    ox: f64, oy: f64, oz: f64,
    cell_size: f64,
    positions: &mut Vec<f32>,
    normals: &mut Vec<f32>,
    indices: &mut Vec<u32>,
) {
    let mut edge_verts = [[0.0_f32; 3]; 12];

    for k in 0..nz - 1 {
        for j in 0..ny - 1 {
            for i in 0..nx - 1 {
                let sdf_c = [
                    sdf[i     + j      * nx + k      * nx * ny],
                    sdf[i + 1 + j      * nx + k      * nx * ny],
                    sdf[i + 1 + (j + 1) * nx + k      * nx * ny],
                    sdf[i     + (j + 1) * nx + k      * nx * ny],
                    sdf[i     + j      * nx + (k + 1) * nx * ny],
                    sdf[i + 1 + j      * nx + (k + 1) * nx * ny],
                    sdf[i + 1 + (j + 1) * nx + (k + 1) * nx * ny],
                    sdf[i     + (j + 1) * nx + (k + 1) * nx * ny],
                ];

                let mut cube_index = 0u16;
                for ci in 0..8 {
                    if sdf_c[ci] > 0.0 {
                        cube_index |= 1 << ci;
                    }
                }

                if cube_index == 0 || cube_index == 255 {
                    continue;
                }

                let edge_mask = EDGE_TABLE[cube_index as usize];
                if edge_mask == 0 {
                    continue;
                }

                let wp = |di: usize, dj: usize, dk: usize| -> [f64; 3] {
                    [ox + (i + di) as f64 * cell_size,
                     oy + (j + dj) as f64 * cell_size,
                     oz + (k + dk) as f64 * cell_size]
                };
                let corners = [
                    wp(0,0,0), wp(1,0,0), wp(1,1,0), wp(0,1,0),
                    wp(0,0,1), wp(1,0,1), wp(1,1,1), wp(0,1,1),
                ];

                for e in 0..12 {
                    if (edge_mask >> e) & 1 != 0 {
                        let [a, b] = EDGE_VERTS[e];
                        let va = corners[a];
                        let vb = corners[b];
                        let t = (sdf_c[a] / (sdf_c[a] - sdf_c[b])).clamp(0.0, 1.0);
                        edge_verts[e] = [
                            (va[0] + t * (vb[0] - va[0])) as f32,
                            (va[1] + t * (vb[1] - va[1])) as f32,
                            (va[2] + t * (vb[2] - va[2])) as f32,
                        ];
                    }
                }

                let tri_row = &TRI_TABLE[cube_index as usize];
                let mut ti = 0;
                while ti < 16 && tri_row[ti] != -1 {
                    let a = tri_row[ti] as usize;
                    let b = tri_row[ti + 1] as usize;
                    let c = tri_row[ti + 2] as usize;
                    ti += 3;

                    let v0 = edge_verts[a];
                    let v1 = edge_verts[b];
                    let v2 = edge_verts[c];

                    let e1x = v1[0] - v0[0];
                    let e1y = v1[1] - v0[1];
                    let e1z = v1[2] - v0[2];
                    let e2x = v2[0] - v0[0];
                    let e2y = v2[1] - v0[1];
                    let e2z = v2[2] - v0[2];
                    let nx = e1y * e2z - e1z * e2y;
                    let ny = e1z * e2x - e1x * e2z;
                    let nz = e1x * e2y - e1y * e2x;
                    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.001);
                    let nx = nx / len;
                    let ny = ny / len;
                    let nz = nz / len;

                    let nv = (positions.len() / 3) as u32;

                    for &v in &[v0, v1, v2] {
                        positions.push(v[0]); positions.push(v[1]); positions.push(v[2]);
                        normals.push(nx); normals.push(ny); normals.push(nz);
                    }
                    indices.push(nv); indices.push(nv + 1); indices.push(nv + 2);
                }
            }
        }
    }
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

    if sample_step == 1 {
        // ── Fill SDF grid ──
        let nx = n + 1;
        let ny = grid_ny + 1;
        let nz = n + 1;

        pool.sdf_grid.clear();
        pool.sdf_grid.resize(nx * ny * nz, 0.0);
        let sdf = &mut pool.sdf_grid;

        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let wx = ox + i as f64 * step;
                    let wy = wy_min + j as f64 * step;
                    let wz = oz + k as f64 * step;
                    sdf[i + j * nx + k * nx * ny] = terrain::sdf_sample(p, wx, wy, wz);
                }
            }
        }

        // ── Extract isosurface ──
        extract_isosurface(sdf, nx, ny, nz, ox, wy_min, oz, step,
            &mut positions, &mut normals, &mut indices);

        // ── Vertex colors from zone rock + depth ──
        let nv = positions.len() / 3;
        colors.reserve(nv * 3);
        for vi in 0..nv {
            let vx = positions[vi * 3] as f64;
            let vy = positions[vi * 3 + 1] as f64;
            let vz = positions[vi * 3 + 2] as f64;

            let fx = ((vx - ox) / step).clamp(0.0, n as f64);
            let fz = ((vz - oz) / step).clamp(0.0, n as f64);
            let ix = fx.floor() as i32;
            let iz = fz.floor() as i32;
            let tx = fx - ix as f64;
            let tz = fz - iz as f64;
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
            let surface_h = h0 * (1.0 - tz) + h1 * tz;

            let zone = zones[((iz0 + iz1) / 2) * n + (ix0 + ix1) / 2];

            let depth = surface_h - vy;
            let rock = terrain::zone_rock_color(zone);
            let darken = (1.0 - (depth / 20.0).clamp(0.0, 0.75)).max(0.25) as f32;
            let r = rock[0] * darken;
            let g = rock[1] * darken;
            let b = rock[2] * darken;
            let (h, s, l) = rgb_to_hsl(r, g, b);
            let (r2, g2, b2) = hsl_to_rgb(
                (h + params.hue_shift as f32 / 360.0) % 1.0,
                (s * params.saturation as f32).clamp(0.0, 1.0),
                (l * params.lightness as f32).clamp(0.0, 1.0),
            );
            colors.push(r2); colors.push(g2); colors.push(b2);
        }

        // ── Vertex UVs (zeroes — MC uses vertex colors, not textures) ──
        uvs.reserve(nv * 2);
        for _ in 0..nv {
            uvs.push(0.0); uvs.push(0.0);
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