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
        let h = crate::engine::terrain::get_height(params, wx, wz).max(params.water_level + 1.0);
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
