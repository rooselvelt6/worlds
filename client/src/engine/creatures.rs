use crate::engine::terrain::Zone;

#[derive(Clone)]
pub struct CreatureInstance {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub rot: f64,
    pub creature_type: u8,
    pub speed: f64,
    pub wander_target: Option<(f64, f64)>,
    pub wander_timer: f64,
}

#[derive(Clone)]
pub struct CreatureData {
    pub cx: i32,
    pub cz: i32,
    pub creatures: Vec<CreatureInstance>,
}

pub fn compute_chunk_creatures(params: &crate::state::WorldParams, cx: i32, cz: i32) -> CreatureData {
    let mut rng: u64 = (params.seed as u64).wrapping_mul(6364136223846793005)
        .wrapping_add(cx as u64 * 924839).wrapping_add(cz as u64 * 729384);
    let zone = crate::engine::terrain::get_zone(params, cx as f64 * 24.0 + 12.0, cz as f64 * 24.0 + 12.0);
    let creature_types = creature_types_for_zone(zone);
    if creature_types.is_empty() { return CreatureData { cx, cz, creatures: vec![] }; }

    let mut creatures = Vec::new();
    let count = ((rng >> 16) & 0x3) + 1;
    for i in 0..count {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let lx = ((rng >> 16) & 0xFFFF) as f64 / 65536.0 * 24.0;
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let lz = ((rng >> 16) & 0xFFFF) as f64 / 65536.0 * 24.0;
        let wx = cx as f64 * 24.0 + lx;
        let wz = cz as f64 * 24.0 + lz;
        let h = crate::engine::terrain::get_height(params, wx, wz);
        if h < params.water_level { continue; }

        let ct = creature_types[i as usize % creature_types.len()];
        creatures.push(CreatureInstance {
            id: format!("c{}_{}_{}", cx, cz, i),
            x: wx, y: h, z: wz, rot: 0.0,
            creature_type: ct, speed: 2.0 + (rng & 3) as f64,
            wander_target: None, wander_timer: 0.0,
        });
    }
    CreatureData { cx, cz, creatures }
}

fn creature_types_for_zone(zone: Zone) -> Vec<u8> {
    match zone {
        Zone::Forest => vec![0, 8],
        Zone::Plains => vec![0, 9],
        Zone::Desert => vec![6, 9],
        Zone::Tundra => vec![2, 7],
        Zone::Jungle => vec![2, 1],
        Zone::Volcanic => vec![5],
        Zone::Crystal => vec![3],
        Zone::Cave => vec![4],
        Zone::Fungus => vec![2],
        Zone::Abyss => vec![4],
        Zone::Storm => vec![5],
        Zone::Aurora => vec![3],
        Zone::Magma => vec![5],
        _ => vec![8],
    }
}
