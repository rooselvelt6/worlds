use std::cell::RefCell;
use std::collections::HashSet;
use crate::math::*;
use crate::state::WorldParams;
use serde::{Deserialize, Serialize};

thread_local! {
    static CRATERS: RefCell<HashSet<(i32, i32)>> = RefCell::new(HashSet::new());
}

pub fn set_craters(craters: &HashSet<(i32, i32)>) {
    CRATERS.with(|c| *c.borrow_mut() = craters.clone());
}

pub fn add_crater(bx: i32, bz: i32) {
    CRATERS.with(|c| { c.borrow_mut().insert((bx, bz)); });
}

pub fn get_craters() -> HashSet<(i32, i32)> {
    CRATERS.with(|c| c.borrow().clone())
}

fn crater_depth(wx: f64, wz: f64) -> f64 {
    let mut depth = 0.0;
    CRATERS.with(|c| {
        let craters = c.borrow();
        let bx = wx.floor() as i32;
        let bz = wz.floor() as i32;
        for dx in -2..=2 {
            for dz in -2..=2 {
                if craters.contains(&(bx + dx, bz + dz)) {
                    let cx = (bx + dx) as f64 + 0.5;
                    let cz = (bz + dz) as f64 + 0.5;
                    let dist = ((wx - cx).powi(2) + (wz - cz).powi(2)).sqrt();
                    let radius = 1.8;
                    if dist < radius {
                        let falloff = 1.0 - (dist / radius);
                        let d = 1.2 * falloff * falloff;
                        if d > depth { depth = d; }
                    }
                }
            }
        }
    });
    depth
}

/// Pure noise-based height without any modifiers (for erosion calculations).
pub fn get_height_base(params: &WorldParams, wx: f64, wz: f64) -> f64 {
    let base = fbm(wx * params.scale, wz * params.scale, params.octaves);
    (base * params.amplitude + params.water_level).max(0.0)
}

pub fn get_height(params: &WorldParams, wx: f64, wz: f64) -> f64 {
    let water_level = params.water_level;

    let mut h = get_height_base(params, wx, wz);

    // R9.1: Plate tectonics - uplift at convergent boundaries, subsidence at divergent
    let plate_uplift = crate::engine::erosion::apply_plate_tectonics(wx, wz, params);
    h += plate_uplift;

    // R9.3: Watershed-based river carving (more natural than pure sin/cos)
    let river_depth = river_carve(params, wx, wz);
    let ws_depth = crate::engine::erosion::apply_watershed_river(wx, wz, params);
    // Blend: use max of both river systems for channel depth
    h -= river_depth.max(ws_depth);

    if params.canyons {
        let canyon = (wx * 0.04).sin() * (wz * 0.04).cos()
            + (wx * 0.06 + wz * 0.08).sin() * 0.5;
        if canyon < -0.2 {
            let depth = (-canyon - 0.2) * 12.0;
            h = (h - depth).max(params.water_level - 6.0);
        }
    }

    // R9.2: Thermal + hydraulic erosion
    let erosion_delta = crate::engine::erosion::apply_erosion(wx, wz, h, params);
    h += erosion_delta;

    // R9.4: Sedimentation in valleys and river mouths
    let sediment = crate::engine::erosion::apply_sedimentation(wx, wz, h, params);
    h += sediment;

    match params.zone {
        Zone::Ocean => h += water_level,
        Zone::Volcanic => h = h * 1.5 + 2.0,
        Zone::Crystal => h *= 0.5,
        Zone::Cave => {
            let cave_noise = (wx * 0.3).sin() * (wz * 0.25).cos() * 3.0;
            let entrance = ((wx * 0.8).sin() * (wz * 0.7).cos()).abs().max(0.3) * 4.0;
            h = 5.0 + cave_noise - entrance.max(0.0).min(2.0);
        }
        Zone::Fungus => h = h * 0.6 + 1.0 + (wx * 0.3).sin() * (wz * 0.3).cos() * 1.5,
        Zone::Abyss => h = h * 0.2 + water_level * 0.3,
        Zone::Storm => h = h * 1.8 + 1.0,
        Zone::Aurora => h = h * 0.5 + 0.5,
        Zone::Magma => h = h * 2.0 + 3.0 + (wx * 0.2).sin().abs() * 2.0,
        Zone::Custom(id) => {
            crate::engine::modding::ModContext::with(|ctx| {
                if let Some(biome) = ctx.get_custom_biome(id) {
                    h = h * biome.height_multiplier + biome.height_offset;
                }
            });
        }
        _ => {}
    }

    // R9.5: Continental shelf profile for underwater areas
    h = crate::engine::erosion::apply_continental_shelf(h, water_level);

    // R10: Terrain destruction — craters from mining/explosions
    let crater_d = crater_depth(wx, wz);
    if crater_d > 0.0 {
        h = (h - crater_d).max(params.water_level - 2.0);
    }

    h
}

// ── Block type constants for voxel terrain ──
pub const BLK_AIR: u8 = 0;
pub const BLK_GRASS: u8 = 1;
pub const BLK_DIRT: u8 = 2;
pub const BLK_STONE: u8 = 3;
pub const BLK_SAND: u8 = 4;
pub const BLK_SNOW: u8 = 5;
pub const BLK_COAL_ORE: u8 = 6;
pub const BLK_IRON_ORE: u8 = 7;
pub const BLK_GOLD_ORE: u8 = 8;
pub const BLK_DIAMOND_ORE: u8 = 9;
pub const BLK_GRAVEL: u8 = 10;
pub const BLK_CLAY: u8 = 11;
pub const BLK_WATER: u8 = 12;
pub const BLK_LAVA: u8 = 13;

// Underground biome blocks (20+ to avoid conflict with build-mode block types 0-8)
pub const BLK_PACKED_ICE: u8 = 20;
pub const BLK_OBSIDIAN: u8 = 21;
pub const BLK_MOSS: u8 = 22;
pub const BLK_GLOW_SHROOM: u8 = 23;
pub const BLK_MAGMA_BLOCK: u8 = 24;
pub const BLK_SOUL_SAND: u8 = 25;
pub const BLK_BASALT: u8 = 26;

/// Returns true if the block type emits light
pub fn block_emits_light(bt: u8) -> bool {
    matches!(bt, BLK_LAVA | BLK_GLOW_SHROOM | BLK_DIAMOND_ORE | BLK_GOLD_ORE)
}

/// Maximum light radius in blocks
pub fn block_light_radius(bt: u8) -> f64 {
    match bt {
        BLK_LAVA => 6.0,
        BLK_GLOW_SHROOM => 4.0,
        _ => 0.0,
    }
}

fn block_hash(wx: f64, wy: f64, wz: f64, seed: u64) -> f64 {
    let h = (seed as i64).wrapping_mul(374761393)
        .wrapping_add((wx.floor() as i64).wrapping_mul(668265263))
        .wrapping_add((wy.floor() as i64).wrapping_mul(1274126177))
        .wrapping_add((wz.floor() as i64).wrapping_mul(1013904243));
    let norm = (h as f64 * 0.000000001).fract().abs();
    norm
}

/// 3D worm tunnel noise: creates connected winding tunnels through the terrain
fn worm_tunnel_noise(wx: f64, wy: f64, wz: f64, seed: u32) -> f64 {
    let s = seed as f64 * 0.001;
    // Curved path through 3D space
    let path_x = wx + (wz * 0.05 + wy * 0.03 + s).sin() * 3.0;
    let path_z = wz + (wx * 0.04 + wy * 0.02 + s * 1.3).cos() * 3.0;
    let path_y = wy + (wx * 0.03 + wz * 0.04 + s * 0.7).sin() * 2.0;

    // Distance from the curved path (worm center)
    let dx = wx - path_x;
    let dy = wy - path_y;
    let dz = wz - path_z;
    let dist_from_path = (dx * dx + dy * dy + dz * dz).sqrt();

    // Varying tunnel radius
    let radius_noise = crate::math::perlin_noise_3d(wx * 0.02, wy * 0.02, wz * 0.02);
    let tunnel_radius = 1.5 + radius_noise * 1.5;

    // Smooth falloff: 1.0 at center, 0.0 at edge
    let t = (1.0 - dist_from_path / tunnel_radius).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t) // smoothstep
}

/// Check if a 3D position is inside a worm tunnel
fn is_worm_tunnel_carved(wx: f64, wy: f64, wz: f64, params: &WorldParams, depth: f64) -> bool {
    if depth < 2.0 { return false; }

    let seed = params.seed;
    // Multiple worm seeds for a connected network
    let worm_seeds = [0u32, 137, 337, 577, 733];
    for &ws in &worm_seeds {
        let worm_strength = worm_tunnel_noise(wx, wy, wz, seed.wrapping_add(ws));
        if worm_strength > 0.6 {
            let carve_strength = (worm_strength - 0.6) / 0.4;
            if carve_strength > 0.3 + (seed as f64 * 0.001).fract() * 0.15 {
                return true;
            }
        }
    }
    false
}

/// Returns true if a block should be filled with water (aquifer)
fn is_aquifer(wx: f64, wy: f64, wz: f64, surface_h: f64, _params: &WorldParams, zone: Zone) -> bool {
    let depth = surface_h - wy;
    if depth < 5.0 || depth > 25.0 { return false; }
    if zone == Zone::Desert || zone == Zone::Volcanic || zone == Zone::Lava || zone == Zone::Magma { return false; }
    // Water pockets in certain depth ranges
    let aquifer_noise = crate::math::perlin_noise_3d(wx * 0.03, wy * 0.03, wz * 0.03);
    aquifer_noise > 0.72 && (wy as i64).wrapping_mul(127) as f64 * 0.001 % 1.0 > 0.3
}

/// Returns the zone-specific underground block type at a given depth
fn deep_block_type(zone: Zone, depth: f64, seed: f64) -> u8 {
    match zone {
        Zone::Tundra | Zone::Aurora => {
            if depth > 8.0 && seed < 0.06 { return BLK_PACKED_ICE; }
        }
        Zone::Volcanic | Zone::Lava | Zone::Magma => {
            if depth > 6.0 && seed < 0.08 { return BLK_MAGMA_BLOCK; }
            if depth > 12.0 && seed < 0.04 { return BLK_OBSIDIAN; }
            if depth > 4.0 && seed < 0.1 { return BLK_BASALT; }
        }
        Zone::Crystal => {
            if depth > 5.0 && seed < 0.05 {
                return if seed < 0.025 { BLK_DIAMOND_ORE } else { BLK_PACKED_ICE };
            }
        }
        Zone::Fungus => {
            if depth > 4.0 && seed < 0.07 { return BLK_GLOW_SHROOM; }
            if depth > 8.0 && seed < 0.08 { return BLK_MOSS; }
        }
        Zone::Jungle | Zone::Forest | Zone::Plains => {
            if depth > 6.0 && seed < 0.05 { return BLK_MOSS; }
        }
        Zone::Abyss => {
            if depth > 3.0 && seed < 0.1 { return BLK_SOUL_SAND; }
            if depth > 8.0 && seed < 0.05 { return BLK_OBSIDIAN; }
        }
        Zone::Cave => {
            if depth > 3.0 && seed < 0.06 { return BLK_MOSS; }
            if depth > 10.0 && seed < 0.03 { return BLK_GLOW_SHROOM; }
        }
        _ => {}
    }
    BLK_STONE
}

pub fn get_block_type(params: &WorldParams, wx: f64, wy: f64, wz: f64, surface_h: f64, zone: Zone) -> u8 {
    if wy > surface_h {
        // River surface water: fill carved channels with shallow water
        if is_river(params, wx, wz) && wy < surface_h + 1.5 {
            return BLK_WATER;
        }
        return BLK_AIR;
    }

    let depth = surface_h - wy;
    let below = wy < surface_h - 0.5;

    if below {
        // 3D cave noise (existing method)
        let cave_noise = crate::math::fbm_3d(wx * 0.04, wy * 0.04, wz * 0.04, 3);
        let cave_threshold = 0.45 + (params.seed as f64 * 0.001).fract() * 0.1;
        if cave_noise > cave_threshold {
            let carve = (cave_noise - cave_threshold) / (1.0 - cave_threshold);
            if carve > 0.3 {
                return BLK_AIR;
            }
        }
        let room_noise = crate::math::perlin_noise_3d(wx * 0.015, wy * 0.015, wz * 0.015);
        if room_noise > 0.65 {
            return BLK_AIR;
        }

        // Worm tunnel carving (connected cave systems)
        if is_worm_tunnel_carved(wx, wy, wz, params, depth) {
            // Check for aquifers first: if worm tunnel intersects an aquifer, fill with water
            if is_aquifer(wx, wy, wz, surface_h, params, zone) {
                return BLK_WATER;
            }
            return BLK_AIR;
        }

        // Fill aquifer blocks with water
        if is_aquifer(wx, wy, wz, surface_h, params, zone) {
            return BLK_WATER;
        }
    }

    if depth < 0.5 {
        return match zone {
            Zone::Desert | Zone::SandyPlain => BLK_SAND,
            Zone::Tundra => BLK_SNOW,
            _ => BLK_GRASS,
        };
    }

    if depth < 2.0 {
        let seed = block_hash(wx, wy, wz, params.seed as u64);
        return if seed < 0.1 { BLK_GRAVEL } else { BLK_DIRT };
    }

    let seed = block_hash(wx, wy, wz, params.seed as u64);

    // Ore distribution at various depths
    if depth > 1.0 && depth < 10.0 && seed < 0.08 {
        return BLK_COAL_ORE;
    }
    if depth > 4.0 && depth < 16.0 && seed < 0.04 {
        return BLK_IRON_ORE;
    }
    if depth > 8.0 && depth < 24.0 && seed < 0.015 {
        return BLK_GOLD_ORE;
    }
    if depth > 14.0 && seed < 0.008 {
        return BLK_DIAMOND_ORE;
    }

    // Deep ore: rare, valuable ores at great depth
    if depth > 20.0 && seed < 0.003 {
        return BLK_DIAMOND_ORE;
    }
    if depth > 16.0 && seed < 0.006 {
        return BLK_GOLD_ORE;
    }

    // Zone-specific deep blocks
    deep_block_type(zone, depth, seed)
}

fn block_name_for_type(block_type: u8) -> &'static str {
    match block_type {
        1 => "grass", 2 => "dirt", 3 => "stone", 4 => "sand", 5 => "snow",
        6 => "coal_ore", 7 => "iron_ore", 8 => "gold_ore", 9 => "diamond_ore",
        10 => "gravel", 11 => "clay", 12 => "water", 13 => "lava",
        20 => "packed_ice", 21 => "obsidian", 22 => "moss", 23 => "glow_shroom",
        24 => "magma_block", 25 => "soul_sand", 26 => "basalt",
        _ => "stone",
    }
}

pub fn block_color(block_type: u8, _params: &WorldParams, wx: f64, wy: f64, wz: f64, zone: Zone, surface_h: f64, max_h: f64) -> [f32; 3] {
    let depth = (surface_h - wy) as f32;
    // Darken with depth but with a higher floor (min 0.25 brightness so caves aren't totally black)
    let darken = (1.0 - (depth / 20.0).clamp(0.0, 0.75)).max(0.25);

    let block_name = block_name_for_type(block_type);
    let palette_color = crate::engine::modding::ModContext::with(|ctx| ctx.get_palette_color(block_name));
    if let Some(pc) = palette_color {
        return if block_emits_light(block_type) { pc } else {
            let d = darken.max(0.5);
            [pc[0] * d, pc[1] * d, pc[2] * d]
        };
    }

    let base = match block_type {
        BLK_GRASS => get_zone_terrain_color(zone, surface_h, max_h, 0.8, wx, wz),
        BLK_DIRT => [0.55, 0.35, 0.18],
        BLK_SAND => [0.85, 0.75, 0.5],
        BLK_SNOW => [0.9, 0.92, 0.95],
        BLK_STONE => zone_rock_color(zone),
        BLK_COAL_ORE => [0.15, 0.15, 0.15],
        BLK_IRON_ORE => [0.7, 0.6, 0.5],
        BLK_GOLD_ORE => [0.9, 0.75, 0.2],
        BLK_DIAMOND_ORE => [0.4, 0.7, 0.9],
        BLK_GRAVEL => [0.5, 0.45, 0.4],
        BLK_CLAY => [0.6, 0.55, 0.5],
        BLK_WATER => [0.2, 0.4, 0.8],
        BLK_LAVA => [1.0, 0.3, 0.05],
        BLK_PACKED_ICE => [0.6, 0.7, 0.85],
        BLK_OBSIDIAN => [0.08, 0.06, 0.12],
        BLK_MOSS => [0.35, 0.45, 0.2],
        BLK_GLOW_SHROOM => [0.5, 0.7, 0.3],
        BLK_MAGMA_BLOCK => [0.7, 0.2, 0.05],
        BLK_SOUL_SAND => [0.25, 0.2, 0.15],
        BLK_BASALT => [0.18, 0.18, 0.2],
        _ => zone_rock_color(zone),
    };

    if block_type == BLK_GRASS || block_type == BLK_WATER {
        return base;
    }

    // Emissive blocks glow
    if block_emits_light(block_type) {
        let glow = match block_type {
            BLK_LAVA => 1.0,
            BLK_GLOW_SHROOM => 0.7,
            _ => 0.3,
        };
        return [base[0].max(glow), base[1].max(glow), base[2].max(glow)];
    }

    if depth > 0.5 {
        let rock = zone_rock_color(zone);
        let rock_t = ((depth - 0.5) / 4.0).clamp(0.0, 1.0);
        let mut c = [
            base[0] * (1.0 - rock_t) + rock[0] * rock_t,
            base[1] * (1.0 - rock_t) + rock[1] * rock_t,
            base[2] * (1.0 - rock_t) + rock[2] * rock_t,
        ];
        c[0] *= darken;
        c[1] *= darken;
        c[2] *= darken;
        c
    } else {
        base
    }
}

fn river_carve(params: &WorldParams, wx: f64, wz: f64) -> f64 {
    let seed = params.seed as f64;
    // Meandering river paths using phase-shifted sin combinations
    // Creates diagonal ribbon patterns that naturally meander
    let s = seed * 0.01;
    let river_a = ((wx * 0.004 + wz * 0.002 + s).sin() + (wx * 0.002 - wz * 0.004 + s * 0.7).cos()) * 0.5;
    let river_b = ((wx * 0.006 - wz * 0.003 + s * 1.3).sin() + (wx * 0.003 + wz * 0.005 + s * 0.5).cos()) * 0.5;

    // River mask: winding ribbon-like paths near zero crossings
    let river_val = (river_a * 0.65 + river_b * 0.35).abs();

    // Only carve where terrain is above water level (rivers above water)
    let base_h = fbm(wx * params.scale, wz * params.scale, params.octaves) * params.amplitude + params.water_level;
    if base_h <= params.water_level + 0.5 { return 0.0; }

    // River width varies with noise for natural meandering
    let width_vary = fbm_2d(wx * 0.005 + 100.0, wz * 0.005, 2) * 0.5 + 0.5;
    let base_width = 0.12 + width_vary * 0.13;

    // Carve where river_val is below threshold (within the river bed)
    if river_val < base_width {
        let t = 1.0 - (river_val / base_width).max(0.0).min(1.0);
        let depth = t * t * (3.0 + width_vary * 3.0);
        // Smooth banks: no carve near edges
        let edge = (river_val / base_width).clamp(0.0, 1.0);
        let smooth = 1.0 - edge * edge * edge;
        depth * smooth
    } else {
        0.0
    }
}

pub fn is_river(params: &WorldParams, wx: f64, wz: f64) -> bool {
    river_carve(params, wx, wz) > 0.3
}

pub fn river_width(params: &WorldParams, wx: f64, wz: f64) -> f64 {
    let seed = params.seed as f64;
    let s = seed * 0.01;
    let river_a = ((wx * 0.004 + wz * 0.002 + s).sin() + (wx * 0.002 - wz * 0.004 + s * 0.7).cos()) * 0.5;
    let river_b = ((wx * 0.006 - wz * 0.003 + s * 1.3).sin() + (wx * 0.003 + wz * 0.005 + s * 0.5).cos()) * 0.5;
    let river_val = (river_a * 0.65 + river_b * 0.35).abs();
    let width_vary = fbm_2d(wx * 0.005 + 100.0, wz * 0.005, 2) * 0.5 + 0.5;
    let base_width = 0.12 + width_vary * 0.13;
    if river_val < base_width { base_width } else { 0.0 }
}

fn fbm_2d(x: f64, y: f64, octaves: u32) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_val = 0.0;
    for _ in 0..octaves {
        value += amplitude * (x * frequency).sin() * (y * frequency).cos();
        max_val += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    value / max_val
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Zone {
    Forest, Plains, Desert, Tundra, Jungle,
    Volcanic, Ocean, Crystal, Cave, Lava,
    Fungus, Abyss, Storm, Aurora, Magma,
    CoralReef, KelpForest, SandyPlain, RockyReef, DeepOcean,
    Custom(u32),
}

impl Zone {
    pub fn from_str(s: &str) -> Self {
        match s {
            "plains" => Zone::Plains, "desert" => Zone::Desert,
            "tundra" => Zone::Tundra, "jungle" => Zone::Jungle,
            "volcanic" => Zone::Volcanic, "ocean" => Zone::Ocean,
            "crystal" => Zone::Crystal, "cave" => Zone::Cave,
            "lava" => Zone::Lava, "fungus" => Zone::Fungus,
            "abyss" => Zone::Abyss, "storm" => Zone::Storm,
            "aurora" => Zone::Aurora, "magma" => Zone::Magma,
            "coral_reef" => Zone::CoralReef, "kelp_forest" => Zone::KelpForest,
            "sandy_plain" => Zone::SandyPlain, "rocky_reef" => Zone::RockyReef,
            "deep_ocean" => Zone::DeepOcean,
            _ => Zone::Forest,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Zone::Forest => "forest", Zone::Plains => "plains",
            Zone::Desert => "desert", Zone::Tundra => "tundra",
            Zone::Jungle => "jungle", Zone::Volcanic => "volcanic",
            Zone::Ocean => "ocean", Zone::Crystal => "crystal",
            Zone::Cave => "cave", Zone::Lava => "lava",
            Zone::Fungus => "fungus", Zone::Abyss => "abyss",
            Zone::Storm => "storm", Zone::Aurora => "aurora",
            Zone::Magma => "magma",
            Zone::CoralReef => "coral_reef", Zone::KelpForest => "kelp_forest",
            Zone::SandyPlain => "sandy_plain", Zone::RockyReef => "rocky_reef",
            Zone::DeepOcean => "deep_ocean",
            Zone::Custom(_) => "custom",
        }
    }

    pub fn name_for_hud(&self) -> String {
        match self {
            Zone::Custom(id) => {
                crate::engine::modding::ModContext::with(|ctx| {
                    ctx.get_custom_biome(*id).map(|b| b.display_name.clone())
                        .unwrap_or_else(|| format!("custom_{}", id))
                })
            }
            _ => self.as_str().to_string(),
        }
    }
}

pub fn get_zone(params: &WorldParams, wx: f64, wz: f64) -> Zone {
    if params.zone != Zone::Forest {
        return params.zone;
    }
    let h = get_height(params, wx, wz);
    let water = params.water_level;
    if h <= water {
        let depth = water - h;
        let n = fbm(wx * 0.008, wz * 0.008, 3);
        let n2 = fbm(wx * 0.012 + 100.0, wz * 0.012, 2);
        if depth < 0.8 && n > 0.2 && n2 > -0.1 {
            Zone::CoralReef
        } else if depth < 2.0 && n.abs() < 0.25 && n2 < 0.2 {
            Zone::KelpForest
        } else if depth > 4.0 || n < -0.3 {
            Zone::DeepOcean
        } else if n > 0.0 {
            Zone::RockyReef
        } else {
            Zone::SandyPlain
        }
    } else {
        let t = fbm(wx * 0.008, wz * 0.008, 2);
        let h2 = fbm(wx * 0.008 + 50.0, wz * 0.008, 2);
        if t < -0.35 { Zone::Tundra }
        else if t > 0.45 {
            if h2 < -0.25 { Zone::Desert } else { Zone::Volcanic }
        } else if h2 > 0.45 {
            if h2 > 0.6 { Zone::Lava } else { Zone::Jungle }
        } else if h2 < -0.35 { Zone::Plains }
        else if h2 < 0.0 { Zone::Ocean }
        else { Zone::Forest }
    }
}

pub fn get_zone_color(zone: Zone) -> [f32; 3] {
    if let Zone::Custom(id) = zone {
        return crate::engine::modding::ModContext::with(|ctx| {
            ctx.get_custom_biome(id)
                .map(|b| b.color)
                .unwrap_or([0.5, 0.5, 0.5])
        });
    }
    // Check mod overrides for built-in biomes
    let zone_name = zone.as_str();
    if let Some(override_color) = crate::engine::modding::ModContext::with(|ctx| {
        ctx.get_biome_color(zone_name)
    }) {
        return override_color;
    }
    match zone {
        Zone::Forest => [0.176, 0.353, 0.153],
        Zone::Plains => [0.486, 0.702, 0.259],
        Zone::Desert => [0.831, 0.659, 0.294],
        Zone::Tundra => [0.784, 0.902, 0.941],
        Zone::Jungle => [0.106, 0.302, 0.106],
        Zone::Volcanic => [0.361, 0.251, 0.200],
        Zone::Ocean => [0.118, 0.377, 0.565],
        Zone::Crystal => [0.545, 0.361, 0.965],
        Zone::Cave => [0.290, 0.290, 0.290],
        Zone::Lava => [1.0, 0.267, 0.0],
        Zone::Fungus => [0.600, 0.200, 0.600],
        Zone::Abyss => [0.050, 0.050, 0.100],
        Zone::Storm => [0.300, 0.350, 0.450],
        Zone::Aurora => [0.200, 0.800, 0.600],
        Zone::Magma => [0.800, 0.300, 0.050],
        Zone::CoralReef => [0.800, 0.400, 0.400],
        Zone::KelpForest => [0.200, 0.500, 0.300],
        Zone::SandyPlain => [0.700, 0.600, 0.400],
        Zone::RockyReef => [0.400, 0.350, 0.300],
        Zone::DeepOcean => [0.020, 0.050, 0.150],
        Zone::Custom(_) => unreachable!(),
    }
}

fn mix_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn gradient(stops: &[[f32; 3]], t: f32) -> [f32; 3] {
    let n = stops.len() - 1;
    let tt = (t * n as f32).clamp(0.0, n as f32 - 0.001);
    let i = tt as usize;
    let frac = tt - i as f32;
    mix_color(stops[i], stops[i + 1], frac)
}

fn zone_stops(zone: Zone) -> &'static [[f32; 3]] {
    match zone {
        Zone::Forest => &[[0.05,0.10,0.25],[0.08,0.30,0.18],[0.12,0.42,0.10],[0.20,0.48,0.14],[0.38,0.42,0.22],[0.65,0.60,0.48],[1.0,1.0,1.0]],
        Zone::Plains => &[[0.05,0.10,0.25],[0.18,0.45,0.18],[0.35,0.55,0.18],[0.50,0.52,0.22],[0.65,0.60,0.38],[0.82,0.78,0.62],[1.0,1.0,1.0]],
        Zone::Desert => &[[0.05,0.10,0.25],[0.55,0.35,0.15],[0.72,0.50,0.20],[0.78,0.58,0.28],[0.72,0.48,0.22],[0.50,0.32,0.18],[1.0,1.0,1.0]],
        Zone::Tundra => &[[0.05,0.10,0.25],[0.18,0.22,0.18],[0.32,0.35,0.30],[0.50,0.55,0.50],[0.72,0.78,0.72],[0.88,0.92,0.98],[1.0,1.0,1.0]],
        Zone::Jungle => &[[0.05,0.10,0.25],[0.06,0.25,0.12],[0.06,0.40,0.08],[0.08,0.48,0.10],[0.25,0.42,0.14],[0.45,0.48,0.28],[1.0,1.0,1.0]],
        Zone::Volcanic => &[[0.05,0.10,0.25],[0.12,0.08,0.08],[0.22,0.12,0.08],[0.32,0.18,0.10],[0.48,0.22,0.08],[0.68,0.32,0.12],[1.0,1.0,1.0]],
        Zone::Crystal => &[[0.05,0.10,0.25],[0.25,0.12,0.35],[0.35,0.20,0.50],[0.45,0.30,0.65],[0.52,0.48,0.78],[0.68,0.68,0.88],[1.0,1.0,1.0]],
        Zone::Cave => &[[0.03,0.02,0.05],[0.10,0.08,0.06],[0.15,0.12,0.10],[0.20,0.16,0.13],[0.22,0.20,0.16],[0.28,0.25,0.22],[1.0,1.0,1.0]],
        Zone::Lava => &[[0.05,0.10,0.25],[0.25,0.03,0.0],[0.45,0.08,0.0],[0.65,0.18,0.03],[0.85,0.35,0.08],[0.95,0.55,0.18],[1.0,1.0,1.0]],
        Zone::Fungus => &[[0.05,0.10,0.25],[0.18,0.03,0.18],[0.28,0.08,0.28],[0.22,0.32,0.12],[0.38,0.42,0.18],[0.52,0.52,0.28],[1.0,1.0,1.0]],
        Zone::Abyss => &[[0.01,0.01,0.03],[0.02,0.02,0.06],[0.04,0.04,0.08],[0.05,0.05,0.10],[0.06,0.06,0.12],[0.08,0.08,0.15],[1.0,1.0,1.0]],
        Zone::Storm => &[[0.05,0.10,0.25],[0.12,0.15,0.20],[0.18,0.22,0.28],[0.28,0.32,0.38],[0.38,0.42,0.48],[0.52,0.52,0.58],[1.0,1.0,1.0]],
        Zone::Aurora => &[[0.05,0.10,0.25],[0.12,0.22,0.28],[0.12,0.38,0.32],[0.18,0.52,0.38],[0.38,0.58,0.48],[0.58,0.68,0.62],[1.0,1.0,1.0]],
        Zone::Magma => &[[0.05,0.10,0.25],[0.28,0.06,0.02],[0.42,0.12,0.03],[0.58,0.22,0.06],[0.78,0.38,0.10],[0.92,0.58,0.22],[1.0,1.0,1.0]],
        Zone::Ocean | Zone::DeepOcean => &[[0.01,0.02,0.08],[0.02,0.05,0.15],[0.05,0.10,0.22],[0.08,0.18,0.30],[0.12,0.25,0.38],[0.18,0.30,0.42],[0.25,0.35,0.48]],
        Zone::CoralReef => &[[0.02,0.05,0.12],[0.08,0.25,0.22],[0.20,0.45,0.30],[0.35,0.55,0.35],[0.50,0.60,0.40],[0.60,0.55,0.45],[0.70,0.60,0.55]],
        Zone::KelpForest => &[[0.02,0.05,0.12],[0.05,0.20,0.15],[0.08,0.35,0.12],[0.12,0.40,0.15],[0.20,0.40,0.20],[0.30,0.45,0.25],[0.40,0.50,0.30]],
        Zone::SandyPlain => &[[0.02,0.05,0.12],[0.15,0.30,0.20],[0.30,0.42,0.22],[0.45,0.50,0.25],[0.55,0.55,0.30],[0.60,0.58,0.35],[0.65,0.60,0.40]],
        Zone::RockyReef => &[[0.02,0.05,0.12],[0.10,0.20,0.18],[0.18,0.30,0.22],[0.25,0.35,0.25],[0.30,0.38,0.28],[0.35,0.40,0.30],[0.40,0.42,0.35]],
        Zone::Custom(_) => &[[0.05,0.10,0.25],[0.08,0.30,0.18],[0.12,0.42,0.10],[0.20,0.48,0.14],[0.38,0.42,0.22],[0.65,0.60,0.48],[1.0,1.0,1.0]],
    }
}

pub fn zone_rock_color(zone: Zone) -> [f32; 3] {
    match zone {
        Zone::Desert => [0.55, 0.35, 0.18],
        Zone::Volcanic | Zone::Lava | Zone::Magma => [0.28, 0.16, 0.08],
        Zone::Cave | Zone::Abyss => [0.12, 0.10, 0.08],
        Zone::Crystal => [0.38, 0.32, 0.48],
        Zone::Tundra => [0.30, 0.32, 0.35],
        Zone::Jungle => [0.20, 0.18, 0.12],
        Zone::Forest => [0.25, 0.22, 0.15],
        Zone::Fungus => [0.30, 0.15, 0.25],
        _ => [0.32, 0.28, 0.22],
    }
}

pub fn get_zone_terrain_color(zone: Zone, h: f64, max_h: f64, slope: f32, wx: f64, wz: f64) -> [f32; 3] {
    let t = (h / max_h.max(0.1)).clamp(0.0, 1.0) as f32;

    let zone_name = zone.as_str();
    let mod_stops = crate::engine::modding::ModContext::with(|ctx| {
        if let Zone::Custom(id) = zone {
            ctx.get_custom_biome(id).map(|b| b.gradient_stops.clone())
        } else {
            ctx.get_biome_gradient(zone_name).cloned()
        }
    });
    let stops: &[[f32; 3]] = if let Some(ref s) = mod_stops {
        // We can't return a reference to Vec's data easily; use heap
        let boxed: Box<[[f32; 3]]> = s.clone().into_boxed_slice();
        // Leak it to get a static ref (small, ok for mod loading)
        Box::leak(boxed)
    } else {
        zone_stops(zone)
    };
    let mut color = gradient(stops, t);

    let rock = crate::engine::modding::ModContext::with(|ctx| {
        if let Zone::Custom(id) = zone {
            ctx.get_custom_biome(id).map(|b| b.rock_color)
        } else {
            ctx.get_biome_rock_color(zone_name)
        }
    }).unwrap_or_else(|| zone_rock_color(zone));
    let slope_factor = (1.0 - slope).clamp(0.0, 0.6); // 0=flat, 0.6=max rock blend
    color = mix_color(color, rock, slope_factor);

    // Subtle noise variation to break up uniformity
    let variation = ((wx * 0.3).sin() * (wz * 0.5).cos() * 0.5 + (wx * 0.7 + wz * 0.4).sin() * 0.3) * 0.04;
    color[0] = (color[0] + variation as f32).clamp(0.0, 1.0);
    color[1] = (color[1] + variation as f32 * 0.8).clamp(0.0, 1.0);
    color[2] = (color[2] + variation as f32 * 0.6).clamp(0.0, 1.0);

    color
}

pub fn get_terrain_color(h: f64, max_h: f64) -> [f32; 3] {
    get_zone_terrain_color(Zone::Forest, h, max_h, 0.8, 0.0, 0.0)
}

pub fn zone_effects(params: &WorldParams, wx: f64, wz: f64, h: &mut f64) {
    match params.zone {
        Zone::Crystal => {
            let crystal = (wx * 0.8).sin() * (wz * 0.8).cos();
            if crystal.abs() > 0.7 {
                *h += 3.0 + crystal * 2.0;
            }
        }
        Zone::Cave => {
            let cave_n = fbm(wx * 0.04 + params.seed as f64 * 0.01, wz * 0.04, 3);
            let canyon = (wx * 0.08).sin() * (wz * 0.08).cos() + (wx * 0.12 + wz * 0.15).sin() * 0.5;
            if canyon < -0.3 || cave_n < -0.2 {
                let depth = (-canyon).max(0.0) * 3.0 + (-cave_n).max(0.0) * 2.0;
                *h = (*h - depth).max(params.water_level - 4.0);
            }
            let pillar = (wx * 0.2).sin() * (wz * 0.2).cos();
            if pillar > 0.6 && *h > params.water_level {
                *h += (pillar - 0.6) * 3.0;
            }
        }
        Zone::Fungus => {
            let spore = (wx * 1.5).sin() * (wz * 1.5).cos();
            if spore.abs() > 0.6 {
                *h += 2.0 + spore * 1.5;
            }
        }
        Zone::Abyss => {
            let pillar = (wx * 0.3).sin() * (wz * 0.3).cos();
            if pillar.abs() > 0.8 {
                *h += 4.0;
            }
            *h = h.min(4.0);
        }
        Zone::Storm => {
            let spike = (wx * 2.0).sin().abs() * (wz * 2.0).cos().abs();
            *h += spike * 2.0;
        }
        Zone::Aurora => {
            let wave = (wx * 0.5).sin() + (wz * 0.7).cos();
            *h += wave * 0.5;
        }
        Zone::Magma => {
            let fissure = (wx * 0.4).sin() * (wz * 0.4).cos();
            if fissure.abs() > 0.5 {
                *h += 2.0;
            }
        }
        Zone::Custom(_) => {}
        _ => {}
    }
}
