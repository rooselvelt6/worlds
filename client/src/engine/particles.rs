use crate::engine::bridge;
use crate::engine::terrain::Zone;

const PARTICLE_COUNT: u32 = 200;

pub fn spawn_zone_particles(zone: Zone, px: f64, pz: f64, water_level: f64) {
    let (colors, spread, speed, y_offset) = match zone {
        Zone::Forest => (vec![0.2, 0.8, 0.2], 15.0, 0.5, 5.0),
        Zone::Plains => (vec![0.6, 0.9, 0.3], 20.0, 0.3, 2.0),
        Zone::Desert => (vec![0.9, 0.7, 0.3], 18.0, 0.4, 1.0),
        Zone::Tundra => (vec![0.9, 0.95, 1.0], 20.0, 0.6, 8.0),
        Zone::Jungle => (vec![0.1, 0.5, 0.1], 15.0, 0.3, 6.0),
        Zone::Volcanic => (vec![1.0, 0.4, 0.1], 12.0, 0.8, 10.0),
        Zone::Ocean => (vec![0.2, 0.5, 0.9], 10.0, 0.2, 0.5),
        Zone::Crystal => (vec![0.6, 0.3, 1.0], 10.0, 0.3, 4.0),
        Zone::Cave => (vec![0.3, 0.3, 0.3], 8.0, 0.2, 2.0),
        Zone::Lava => (vec![1.0, 0.2, 0.0], 12.0, 0.7, 8.0),
        Zone::Fungus => (vec![0.8, 0.2, 0.8], 12.0, 0.4, 5.0),
        Zone::Abyss => (vec![0.05, 0.05, 0.1], 6.0, 0.1, 1.0),
        Zone::Storm => (vec![0.5, 0.5, 0.6], 20.0, 0.9, 10.0),
        Zone::Aurora => (vec![0.2, 0.9, 0.7], 18.0, 0.3, 8.0),
        Zone::Magma => (vec![0.9, 0.3, 0.05], 14.0, 0.6, 7.0),
        Zone::CoralReef => (vec![0.4, 0.7, 0.9], 8.0, 0.15, 0.3),
        Zone::KelpForest => (vec![0.2, 0.6, 0.3], 10.0, 0.15, 0.3),
        Zone::SandyPlain => (vec![0.6, 0.5, 0.3], 6.0, 0.1, 0.2),
        Zone::RockyReef => (vec![0.3, 0.4, 0.5], 8.0, 0.12, 0.3),
        Zone::DeepOcean => (vec![0.02, 0.05, 0.15], 4.0, 0.05, 0.2),
    };

    let mut positions = Vec::with_capacity((PARTICLE_COUNT * 3) as usize);
    let mut colors_arr = Vec::with_capacity((PARTICLE_COUNT * 3) as usize);

    for i in 0..PARTICLE_COUNT {
        let angle = (i as f64 / PARTICLE_COUNT as f64) * std::f64::consts::TAU;
        let radius = (i as f64 / PARTICLE_COUNT as f64).sqrt() * spread;
        let x = px + angle.cos() * radius;
        let z = pz + angle.sin() * radius;
        let y = y_offset + (angle * 3.0).sin().abs() * 3.0;
        positions.push(x as f32);
        positions.push(y as f32);
        positions.push(z as f32);

        let jitter: f32 = ((i as f64 * 127.1).sin() * 0.3 + 0.7) as f32;
        colors_arr.push(colors[0] * jitter);
        colors_arr.push(colors[1] * jitter);
        colors_arr.push(colors[2] * jitter);
    }

    let pos_arr = js_sys::Float32Array::from(&positions[..]);
    let col_arr = js_sys::Float32Array::from(&colors_arr[..]);
    bridge::spawn_particles("ambient", &pos_arr, &col_arr, PARTICLE_COUNT);
}

pub fn remove_ambient_particles() {
    bridge::remove_particles("ambient");
}
