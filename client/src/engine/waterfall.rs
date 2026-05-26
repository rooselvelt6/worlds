use crate::engine::bridge;
use crate::engine::terrain::get_height;
use crate::state::WorldParams;

const SCAN_INTERVAL: f64 = 3.0;
const SCAN_RANGE: f64 = 40.0;
const SCAN_STEP: f64 = 8.0;
const CLIFF_MIN: f64 = 2.5;
const PARTICLES_PER_FALL: usize = 40;
const FALL_SPEED: f64 = 12.0;
const SPREAD: f64 = 1.5;

pub struct WaterfallSystem {
    locations: Vec<WaterfallLoc>,
    scan_timer: f64,
}

struct WaterfallLoc {
    x: f64, y: f64, z: f64,
    key: String,
    px: Vec<f64>,
    py: Vec<f64>,
    pz: Vec<f64>,
    vy: Vec<f64>,
    lt: Vec<f64>,
    max_lt: Vec<f64>,
}

impl WaterfallSystem {
    pub fn new() -> Self {
        Self {
            locations: Vec::new(),
            scan_timer: 0.0,
        }
    }

    pub fn remove_all(&mut self) {
        for loc in self.locations.drain(..) {
            bridge::remove_mesh(&loc.key);
        }
    }

    fn spawn_location(params: &WorldParams, loc_x: f64, loc_y: f64, loc_z: f64) -> WaterfallLoc {
        let key = format!("wf_{:.0}_{:.0}", loc_x, loc_z);
        let count = PARTICLES_PER_FALL;
        let mut px = vec![0.0; count];
        let mut py = vec![0.0; count];
        let mut pz = vec![0.0; count];
        let mut vy = vec![0.0; count];
        let mut lt = vec![0.0; count];
        let mut max_lt = vec![0.0; count];

        let mut seed = (params.seed as u64).wrapping_mul(6364136223846793005)
            .wrapping_add((loc_x as u64 * 1000).wrapping_mul(374761393));
        for i in 0..count {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let r1 = ((seed >> 16) & 0x7FFF) as f64 / 32767.0;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let r2 = ((seed >> 16) & 0x7FFF) as f64 / 32767.0;
            px[i] = loc_x + (r1 - 0.5) * SPREAD;
            py[i] = loc_y - r2 * 0.5;
            pz[i] = loc_z + (r1 - 0.5) * SPREAD;
            vy[i] = -FALL_SPEED - r2 * 3.0;
            max_lt[i] = 1.0 + r1 * 0.5;
            lt[i] = max_lt[i];
        }

        let flat_all: Vec<f32> = (0..count).flat_map(|i| {
            vec![px[i] as f32, py[i] as f32, pz[i] as f32]
        }).collect();
        let arr = js_sys::Float32Array::from(&flat_all[..]);
        bridge::create_particles(&key, count as u32, 0.6, 0.75, 0.9, 0.3);
        bridge::update_particles(&key, &arr);

        WaterfallLoc { x: loc_x, y: loc_y, z: loc_z, key, px, py, pz, vy, lt, max_lt }
    }

    pub fn update(&mut self, delta: f64, params: &WorldParams, px: f64, py: f64, pz: f64) {
        self.scan_timer += delta;
        if self.scan_timer >= SCAN_INTERVAL {
            self.scan_timer = 0.0;
            self.rescan(params, px, py, pz);
        }

        let water = params.water_level;
        for loc in &mut self.locations {
            let count = PARTICLES_PER_FALL;
            for i in 0..count {
                loc.lt[i] -= delta;
                if loc.lt[i] <= 0.0 {
                    let mut seed = (params.seed as u64).wrapping_mul(6364136223846793005)
                        .wrapping_add(i as u64 * 374761393);
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let r1 = ((seed >> 16) & 0x7FFF) as f64 / 32767.0;
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let r2 = ((seed >> 16) & 0x7FFF) as f64 / 32767.0;
                    loc.px[i] = loc.x + (r1 - 0.5) * SPREAD;
                    loc.py[i] = loc.y;
                    loc.pz[i] = loc.z + (r1 - 0.5) * SPREAD;
                    loc.vy[i] = -FALL_SPEED - r2 * 3.0;
                    loc.max_lt[i] = 1.0 + r1 * 0.5;
                    loc.lt[i] = loc.max_lt[i];
                } else {
                    loc.py[i] += loc.vy[i] * delta;
                    if loc.py[i] < water - 0.3 {
                        loc.lt[i] = 0.0;
                    }
                }
            }
            let flat: Vec<f32> = (0..count).flat_map(|i| {
                vec![loc.px[i] as f32, loc.py[i] as f32, loc.pz[i] as f32]
            }).collect();
            let arr = js_sys::Float32Array::from(&flat[..]);
            bridge::update_particles(&loc.key, &arr);
        }
    }

    fn rescan(&mut self, params: &WorldParams, px: f64, py: f64, pz: f64) {
        // Remove existing locations
        self.remove_all();

        let water = params.water_level;
        let mut found: Vec<(f64, f64, f64)> = Vec::new();

        let mut wx = (px - SCAN_RANGE / SCAN_STEP).floor() * SCAN_STEP;
        while wx < px + SCAN_RANGE {
            let mut wz = (pz - SCAN_RANGE / SCAN_STEP).floor() * SCAN_STEP;
            while wz < pz + SCAN_RANGE {
                let h = get_height(params, wx, wz);
                if h > water + 0.5 {
                    // Check 4 cardinal directions for steep drop into water
                    let dirs = [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)];
                    for &(ddx, ddz) in &dirs {
                        let nx = wx + ddx * 3.0;
                        let nz = wz + ddz * 3.0;
                        let nh = get_height(params, nx, nz);
                        if nh <= water && h - nh >= CLIFF_MIN {
                            // Place waterfall at cliff edge
                            let fx = wx;
                            let fy = h.max(water + 1.0);
                            let fz = wz;
                            let dist2 = (fx - px).powi(2) + (fy - py).powi(2) + (fz - pz).powi(2);
                            if dist2 < (SCAN_RANGE * 0.8).powi(2) {
                                found.push((fx, fy, fz));
                            }
                            break;
                        }
                    }
                }
                wz += SCAN_STEP;
            }
            wx += SCAN_STEP;
        }

        // Limit to 12 waterfalls max
        let limit = found.len().min(12);
        for i in 0..limit {
            let (fx, fy, fz) = found[i];
            self.locations.push(Self::spawn_location(params, fx, fy, fz));
        }
    }
}
