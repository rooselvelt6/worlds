use crate::engine::bridge;
use crate::engine::terrain::{self, Zone};
use crate::state::WorldParams;

pub struct ParticleSystem {
    key: String,
    count: usize,
    positions: Vec<f64>,
    velocities: Vec<[f64; 3]>,
    lifetimes: Vec<f64>,
    max_lifetime: Vec<f64>,
}

fn simple_rng(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*seed >> 16) & 0x7FFF) as f64 / 32767.0
}

pub fn particle_color_size(zone: Zone) -> (f64, f64, f64, f64) {
    match zone {
        Zone::Forest | Zone::Plains | Zone::Jungle | Zone::Storm => (0.7, 0.8, 1.0, 0.15),
        Zone::Tundra => (0.9, 0.95, 1.0, 0.12),
        Zone::Desert => (0.85, 0.75, 0.55, 0.2),
        Zone::Volcanic | Zone::Lava | Zone::Magma => (1.0, 0.6, 0.2, 0.25),
        Zone::Crystal | Zone::Aurora => (0.6, 0.8, 1.0, 0.1),
        Zone::Ocean | Zone::CoralReef | Zone::KelpForest | Zone::DeepOcean | Zone::RockyReef | Zone::SandyPlain => (0.5, 0.7, 0.9, 0.15),
        Zone::Cave | Zone::Abyss => (0.4, 0.4, 0.45, 0.08),
        Zone::Fungus => (0.6, 0.3, 0.6, 0.12),
    }
}

pub fn particle_count(zone: Zone) -> usize {
    match zone {
        Zone::Forest | Zone::Plains => 300,
        Zone::Jungle | Zone::Storm => 600,
        Zone::Tundra => 400,
        Zone::Desert => 200,
        Zone::Volcanic | Zone::Lava | Zone::Magma => 250,
        Zone::Crystal | Zone::Aurora => 150,
        Zone::Ocean | Zone::CoralReef | Zone::KelpForest | Zone::DeepOcean | Zone::RockyReef | Zone::SandyPlain => 200,
        Zone::Cave | Zone::Abyss => 100,
        Zone::Fungus => 150,
    }
}

impl ParticleSystem {
    pub fn new(key: &str, count: usize, r: f64, g: f64, b: f64, size: f64) -> Self {
        let count = count.max(1);
        let pos = vec![0.0; count * 3];
        let vel = vec![[0.0; 3]; count];
        let lt = vec![0.0; count];
        let mlt = vec![0.0; count];
        let p: Vec<f32> = pos.iter().map(|&v| v as f32).collect();
        let arr = js_sys::Float32Array::from(&p[..]);
        bridge::create_particles(key, count as u32, r, g, b, size);
        bridge::update_particles(key, &arr);
        Self {
            key: key.to_string(),
            count,
            positions: pos,
            velocities: vel,
            lifetimes: lt,
            max_lifetime: mlt,
        }
    }

    pub fn remove(&self) {
        bridge::remove_mesh(&self.key);
    }

    fn reset_particle(&mut self, i: usize, zone: Zone, params: &WorldParams, px: f64, py: f64, pz: f64, _water_level: f64) {
        let mut rng: u64 = (params.seed as u64).wrapping_mul(6364136223846793005)
            .wrapping_add(i as u64 * 374761393);
        let i3 = i * 3;
        let spread = 15.0;
        match zone {
            Zone::Forest | Zone::Plains | Zone::Jungle | Zone::Storm => {
                self.positions[i3] = px + (simple_rng(&mut rng) - 0.5) * spread * 2.0;
                self.positions[i3 + 1] = py + 10.0 + simple_rng(&mut rng) * 5.0;
                self.positions[i3 + 2] = pz + (simple_rng(&mut rng) - 0.5) * spread * 2.0;
                self.velocities[i] = [(simple_rng(&mut rng) - 0.5) * 0.5, -8.0 - simple_rng(&mut rng) * 3.0, (simple_rng(&mut rng) - 0.5) * 0.5];
                self.max_lifetime[i] = 2.0 + simple_rng(&mut rng);
                self.lifetimes[i] = self.max_lifetime[i];
            }
            Zone::Tundra => {
                self.positions[i3] = px + (simple_rng(&mut rng) - 0.5) * spread * 2.0;
                self.positions[i3 + 1] = py + 8.0 + simple_rng(&mut rng) * 6.0;
                self.positions[i3 + 2] = pz + (simple_rng(&mut rng) - 0.5) * spread * 2.0;
                self.velocities[i] = [(simple_rng(&mut rng) - 0.5) * 0.3, -1.5 - simple_rng(&mut rng), (simple_rng(&mut rng) - 0.5) * 0.3];
                self.max_lifetime[i] = 5.0 + simple_rng(&mut rng) * 3.0;
                self.lifetimes[i] = self.max_lifetime[i];
            }
            Zone::Desert => {
                self.positions[i3] = px + (simple_rng(&mut rng) - 0.5) * spread * 2.0;
                self.positions[i3 + 1] = py + simple_rng(&mut rng) * 2.0;
                self.positions[i3 + 2] = pz + (simple_rng(&mut rng) - 0.5) * spread * 2.0;
                self.velocities[i] = [1.0 + simple_rng(&mut rng) * 2.0, (simple_rng(&mut rng) - 0.5) * 0.3, (simple_rng(&mut rng) - 0.5) * 0.5];
                self.max_lifetime[i] = 3.0 + simple_rng(&mut rng) * 2.0;
                self.lifetimes[i] = self.max_lifetime[i];
            }
            Zone::Volcanic | Zone::Lava | Zone::Magma => {
                self.positions[i3] = px + (simple_rng(&mut rng) - 0.5) * spread;
                self.positions[i3 + 1] = py + simple_rng(&mut rng);
                self.positions[i3 + 2] = pz + (simple_rng(&mut rng) - 0.5) * spread;
                self.velocities[i] = [(simple_rng(&mut rng) - 0.5), 2.0 + simple_rng(&mut rng) * 3.0, (simple_rng(&mut rng) - 0.5)];
                self.max_lifetime[i] = 2.0 + simple_rng(&mut rng);
                self.lifetimes[i] = self.max_lifetime[i];
            }
            Zone::Crystal | Zone::Aurora => {
                self.positions[i3] = px + (simple_rng(&mut rng) - 0.5) * spread * 2.0;
                self.positions[i3 + 1] = py + 1.0 + simple_rng(&mut rng) * 5.0;
                self.positions[i3 + 2] = pz + (simple_rng(&mut rng) - 0.5) * spread * 2.0;
                self.velocities[i] = [(simple_rng(&mut rng) - 0.5) * 0.2, (simple_rng(&mut rng) - 0.5) * 0.15, (simple_rng(&mut rng) - 0.5) * 0.2];
                self.max_lifetime[i] = 6.0 + simple_rng(&mut rng) * 4.0;
                self.lifetimes[i] = self.max_lifetime[i];
            }
            _ => {
                self.positions[i3] = px + (simple_rng(&mut rng) - 0.5) * spread * 2.0;
                self.positions[i3 + 1] = py + 10.0 + simple_rng(&mut rng) * 5.0;
                self.positions[i3 + 2] = pz + (simple_rng(&mut rng) - 0.5) * spread * 2.0;
                self.velocities[i] = [0.0, -5.0, 0.0];
                self.max_lifetime[i] = 3.0;
                self.lifetimes[i] = 3.0;
            }
        }
    }

    pub fn update(&mut self, delta: f64, zone: Zone, params: &WorldParams, px: f64, _py: f64, pz: f64, water_level: f64) {
        let ground_y = terrain::get_height(params, px, pz);
        for i in 0..self.count {
            let i3 = i * 3;
            self.lifetimes[i] -= delta;
            if self.lifetimes[i] <= 0.0 {
                self.reset_particle(i, zone, params, px, ground_y, pz, water_level);
                continue;
            }
            let age_frac = 1.0 - (self.lifetimes[i] / self.max_lifetime[i].max(0.001));
            let vx = self.velocities[i][0];
            let vy = self.velocities[i][1];
            let vz = self.velocities[i][2];
            self.positions[i3] += vx * delta;
            self.positions[i3 + 1] += vy * delta;
            self.positions[i3 + 2] += vz * delta;

            let wp = terrain::get_height(params, self.positions[i3], self.positions[i3 + 2]);
            let min_y = if zone == Zone::Tundra || zone == Zone::Crystal || zone == Zone::Aurora {
                wp - 2.0
            } else if matches!(zone, Zone::Ocean | Zone::CoralReef | Zone::KelpForest | Zone::DeepOcean | Zone::RockyReef | Zone::SandyPlain) {
                water_level - 2.0
            } else {
                wp - 0.5
            };
            if self.positions[i3 + 1] < min_y {
                self.reset_particle(i, zone, params, px, ground_y, pz, water_level);
            }
            if matches!(zone, Zone::Volcanic | Zone::Lava | Zone::Magma) && age_frac > 0.8 {
                self.positions[i3 + 1] -= vy * delta * 0.5;
            }
        }
        let flat: Vec<f32> = self.positions.iter().map(|&v| v as f32).collect();
        let arr = js_sys::Float32Array::from(&flat[..]);
        bridge::update_particles(&self.key, &arr);
    }
}
