use crate::engine::bridge;
use crate::engine::terrain;
use crate::state::WorldParams;

const SCAN_INTERVAL: f64 = 4.0;
const FOAM_COUNT: usize = 60;

pub struct FoamSystem {
    positions: Vec<f64>,
    lifetimes: Vec<f64>,
    max_lifetimes: Vec<f64>,
    timer: f64,
    spawned: bool,
}

impl FoamSystem {
    pub fn new() -> Self {
        Self {
            positions: vec![0.0; FOAM_COUNT * 3],
            lifetimes: vec![0.0; FOAM_COUNT],
            max_lifetimes: vec![2.0; FOAM_COUNT],
            timer: 0.0,
            spawned: false,
        }
    }

    pub fn ensure_spawned(&mut self) {
        if !self.spawned {
            bridge::create_particles("foam", FOAM_COUNT as u32, 0.9, 0.95, 1.0, 0.4);
            let arr = js_sys::Float32Array::from(&vec![0.0f32; FOAM_COUNT * 3][..]);
            bridge::update_particles("foam", &arr);
            self.spawned = true;
        }
    }

    pub fn remove(&self) {
        if self.spawned {
            bridge::remove_mesh("foam");
        }
    }

    pub fn update(&mut self, delta: f64, params: &WorldParams, px: f64, pz: f64, water_level: f64) {
        self.ensure_spawned();
        self.timer += delta;

        let mut needs_rescan = false;
        if self.timer >= SCAN_INTERVAL {
            self.timer = 0.0;
            needs_rescan = true;
        }

        let mut rng: u64 = (params.seed as u64).wrapping_mul(6364136223846793005);
        for i in 0..FOAM_COUNT {
            self.lifetimes[i] -= delta;
            if self.lifetimes[i] <= 0.0 {
                let i3 = i * 3;
                if needs_rescan {
                    // Find shore positions: sample terrain in a ring around player
                    let angle = (i as f64 / FOAM_COUNT as f64) * std::f64::consts::TAU;
                    let radius = 8.0 + (i as f64 * 0.7).fract() * 12.0;
                    let sx = px + angle.cos() * radius;
                    let sz = pz + angle.sin() * radius;
                    let h = terrain::get_height(params, sx, sz);
                    // Only place foam where terrain is near water level
                    if h >= water_level - 0.5 && h <= water_level + 0.3 {
                        self.positions[i3] = sx;
                        self.positions[i3 + 1] = water_level + 0.05;
                        self.positions[i3 + 2] = sz;
                        self.lifetimes[i] = 1.5 + (i as f64 * 0.3).fract() * 1.0;
                        self.max_lifetimes[i] = self.lifetimes[i];
                    } else {
                        self.positions[i3] = px + (i as f64 - FOAM_COUNT as f64 * 0.5) * 0.1;
                        self.positions[i3 + 1] = -10.0;
                        self.positions[i3 + 2] = pz;
                        self.lifetimes[i] = 0.0;
                    }
                } else {
                    // Drift foam along shore
                    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let r1 = ((rng >> 16) & 0x7FFF) as f64 / 32767.0;
                    self.positions[i3] += (r1 - 0.5) * 0.5;
                    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let r2 = ((rng >> 16) & 0x7FFF) as f64 / 32767.0;
                    self.positions[i3 + 2] += (r2 - 0.5) * 0.5;
                    let h = terrain::get_height(params, self.positions[i3], self.positions[i3 + 2]);
                    if h > water_level {
                        self.lifetimes[i] = 0.0; // fade out on land
                    }
                }
            }
        }

        let flat: Vec<f32> = self.positions.iter().map(|&v| v as f32).collect();
        let arr = js_sys::Float32Array::from(&flat[..]);
        bridge::update_particles("foam", &arr);
    }
}
