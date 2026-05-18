use crate::engine::terrain;
use crate::math::{hsl_to_rgb, rgb_to_hsl};
use crate::state::WorldParams;

pub const CHUNK_SIZE: f64 = 24.0;
pub const RES: u32 = 20;
const NORMAL_DX: f64 = 0.5;

#[derive(Clone)]
pub struct ChunkData {
    pub cx: i32,
    pub cz: i32,
    pub positions: Vec<f32>,
    pub colors: Vec<f32>,
    pub indices: Vec<u16>,
}

impl ChunkData {
    pub fn key(&self) -> (i32, i32) {
        (self.cx, self.cz)
    }
}

pub fn compute_chunk_data(params: &WorldParams, cx: i32, cz: i32) -> ChunkData {
    let ox = cx as f64 * CHUNK_SIZE;
    let oz = cz as f64 * CHUNK_SIZE;
    let step = CHUNK_SIZE / RES as f64;
    let verts_per_side = (RES + 1) as usize;
    let num_verts = verts_per_side * verts_per_side;

    let mut positions = Vec::with_capacity(num_verts * 3);
    let mut normals = Vec::with_capacity(num_verts * 3);
    let mut colors = Vec::with_capacity(num_verts * 3);

    let mut heights = vec![0.0_f64; num_verts];

    for iz in 0..=RES {
        for ix in 0..=RES {
            let idx = iz as usize * verts_per_side + ix as usize;
            let wx = ox + ix as f64 * step;
            let wz = oz + iz as f64 * step;
            let mut h = terrain::get_height(params, wx, wz);
            terrain::zone_effects(params, wx, wz, &mut h);
            heights[idx] = h;
        }
    }

    let max_h = heights.iter().cloned().fold(0.0_f64, f64::max);

    for iz in 0..=RES {
        for ix in 0..=RES {
            let idx = iz as usize * verts_per_side + ix as usize;
            let wx = ox + ix as f64 * step;
            let wz = oz + iz as f64 * step;
            let h = heights[idx] as f32;

            positions.push(wx as f32);
            positions.push(h);
            positions.push(wz as f32);

            let h_l = terrain::get_height(params, wx - NORMAL_DX, wz);
            let h_r = terrain::get_height(params, wx + NORMAL_DX, wz);
            let h_d = terrain::get_height(params, wx, wz - NORMAL_DX);
            let h_u = terrain::get_height(params, wx, wz + NORMAL_DX);
            let dx_h = (h_r - h_l) / (2.0 * NORMAL_DX);
            let dz_h = (h_u - h_d) / (2.0 * NORMAL_DX);
            let len = (dx_h * dx_h + 1.0 + dz_h * dz_h).sqrt() as f32;
            normals.push((-dx_h as f32) / len);
            normals.push(1.0 / len);
            normals.push((-dz_h as f32) / len);

            let mut c = terrain::get_formula_color(params.formula, heights[idx], max_h);
            // Subsurface rock blending for cave/underground areas
            let water = params.water_level as f32;
            if heights[idx] as f32 <= water - 1.0 {
                let rock_t = ((water - 1.0 - heights[idx] as f32) / 4.0).clamp(0.0, 1.0);
                let rock = [0.25, 0.22, 0.20];
                c[0] = c[0] * (1.0 - rock_t) + rock[0] * rock_t;
                c[1] = c[1] * (1.0 - rock_t) + rock[1] * rock_t;
                c[2] = c[2] * (1.0 - rock_t) + rock[2] * rock_t;
            }
            let (h, s, l) = rgb_to_hsl(c[0], c[1], c[2]);
            let (r, g, b) = hsl_to_rgb(
                (h + params.hue_shift as f32 / 360.0) % 1.0,
                (s * params.saturation as f32).clamp(0.0, 1.0),
                (l * params.lightness as f32).clamp(0.0, 1.0),
            );
            colors.push(r);
            colors.push(g);
            colors.push(b);
        }
    }

    let mut indices = Vec::with_capacity((RES * RES * 6) as usize);
    for iz in 0..RES {
        for ix in 0..RES {
            let a = (iz as usize * verts_per_side + ix as usize) as u16;
            let b = (iz as usize * verts_per_side + ix as usize + 1) as u16;
            let c = ((iz as usize + 1) * verts_per_side + ix as usize) as u16;
            let d = ((iz as usize + 1) * verts_per_side + ix as usize + 1) as u16;
            indices.push(a); indices.push(c); indices.push(b);
            indices.push(b); indices.push(c); indices.push(d);
        }
    }

    ChunkData { cx, cz, positions, colors, indices }
}
