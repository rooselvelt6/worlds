use crate::engine::creatures::{push_cylinder_rot, push_ellipsoid_rot};
use crate::engine::terrain;

#[derive(Clone)]
pub struct PortalData {
    pub portals: Vec<PortalInstance>,
}

#[derive(Clone)]
pub struct PortalInstance {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub target_seed: u32,
    pub radius: f64,
}

pub fn compute_portals(params: &crate::state::WorldParams) -> PortalData {
    let mut rng: u64 = params.seed as u64;
    let mut portals = Vec::new();
    let count = ((rng >> 8) & 0x3) + 1;
    for i in 0..count {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let angle = ((rng >> 16) & 0xFFFF) as f64 / 65536.0 * std::f64::consts::TAU;
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let dist = 80.0 + ((rng >> 16) & 0xFFFF) as f64 / 65536.0 * 200.0;
        let wx = angle.cos() * dist;
        let wz = angle.sin() * dist;
        let h = terrain::get_height(params, wx, wz).max(params.water_level + 1.0);
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let target_seed = ((rng >> 16) & 0xFFFF) as u32 % 9999 + 1;
        portals.push(PortalInstance {
            id: format!("portal_{}", i),
            x: wx, y: h, z: wz,
            target_seed,
            radius: 3.0,
        });
    }
    PortalData { portals }
}

pub fn generate_portal_mesh(params: &crate::state::WorldParams, cx: i32, cz: i32) -> Option<(Vec<f32>, Vec<f32>, Vec<u32>, Vec<f32>, u32, f32)> {
    let data = compute_portals(params);
    let chunk_ox = cx as f64 * 24.0;
    let chunk_oz = cz as f64 * 24.0;
    let in_chunk: Vec<_> = data.portals.iter().filter(|p| {
        p.x >= chunk_ox && p.x < chunk_ox + 24.0 && p.z >= chunk_oz && p.z < chunk_oz + 24.0
    }).collect();
    if in_chunk.is_empty() { return None; }
    let target_seed = in_chunk[0].target_seed;
    let radius = in_chunk[0].radius as f32;
    let mut pos = Vec::new();
    let mut norms = Vec::new();
    let mut idx = Vec::new();
    let mut cols = Vec::new();
    let mut base_idx = 0u32;
    for p in &in_chunk {
        let r = p.radius as f32 * 0.5;
        let segs = 8;
        for si in 0..segs {
            let a = si as f32 / segs as f32 * std::f64::consts::TAU as f32;
            let nx = a.cos();
            let nz = a.sin();
            let rx = p.x as f32 + nx * r;
            let rz = p.z as f32 + nz * r;
            push_cylinder_rot(&mut pos, &mut norms, &mut idx, &mut cols, rx, p.y as f32 + 1.5, rz, 0.08, 0.5, 0.08, 0.2, 0.4, 1.0, &mut base_idx, 6, 0.0);
        }
        push_ellipsoid_rot(&mut pos, &mut norms, &mut idx, &mut cols, p.x as f32, p.y as f32 + 1.5, p.z as f32, r * 0.15, 0.5, r * 0.15, 0.6, 0.8, 1.0, &mut base_idx, 5, 5, 0.0);
    }
    Some((pos, norms, idx, cols, target_seed, radius))
}
