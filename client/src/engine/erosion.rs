use crate::math::*;
use crate::state::WorldParams;
use std::cell::RefCell;

/// ── R9.1: Plate Tectonics ──────────────────────────────────────
/// Generates 3 plates from seed, computes uplift at convergent boundaries,
/// subsidence at divergent boundaries.

const NUM_PLATES: usize = 3;

thread_local! {
    static CACHED_PLATE_CENTERS: RefCell<Option<(u32, [(f64, f64); 3])>> = const { RefCell::new(None) };
}

fn get_plate_centers(params: &WorldParams) -> [(f64, f64); NUM_PLATES] {
    CACHED_PLATE_CENTERS.with(|c| {
        if let Some((seed, centers)) = *c.borrow() {
            if seed == params.seed { return centers; }
        }
        let centers = plate_centers(params.seed);
        *c.borrow_mut() = Some((params.seed, centers));
        centers
    })
}

fn plate_centers(seed: u32) -> [(f64, f64); NUM_PLATES] {
    let mut centers = [(0.0, 0.0); NUM_PLATES];
    let mut rng = seed as u64;
    for i in 0..NUM_PLATES {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let cx = ((rng >> 16) & 0xFFFF) as f64 / 65536.0 * 200.0 - 100.0;
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let cz = ((rng >> 16) & 0xFFFF) as f64 / 65536.0 * 200.0 - 100.0;
        centers[i] = (cx, cz);
    }
    centers
}

fn plate_motion(seed: u32, idx: usize) -> (f64, f64) {
    let mut rng = (seed as u64).wrapping_add((idx as u64).wrapping_mul(374761393));
    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let angle = ((rng >> 16) & 0xFFFF) as f64 / 65536.0 * std::f64::consts::TAU;
    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let speed = 0.3 + ((rng >> 16) & 0xFFFF) as f64 / 65536.0 * 0.7;
    (angle.cos() * speed, angle.sin() * speed)
}

fn plate_id_at(wx: f64, wz: f64, centers: &[(f64, f64); NUM_PLATES]) -> (usize, f64, f64) {
    let mut best = 0usize;
    let mut best_dist = f64::MAX;
    let mut second_dist = f64::MAX;
    for (i, &(cx, cz)) in centers.iter().enumerate() {
        let dx = wx - cx;
        let dz = wz - cz;
        let d = dx * dx + dz * dz;
        if d < best_dist {
            second_dist = best_dist;
            best_dist = d;
            best = i;
        } else if d < second_dist {
            second_dist = d;
        }
    }
    (best, best_dist.sqrt(), second_dist.sqrt())
}

fn plate_boundary(wx: f64, wz: f64, params: &WorldParams) -> f64 {
    let centers = get_plate_centers(params);
    let (pid, d1, d2) = plate_id_at(wx, wz, &centers);
    let boundary_width = 8.0;
    let dist_to_boundary = d2 - d1;
    if dist_to_boundary > boundary_width {
        return 0.0;
    }
    let (mx, mz) = plate_motion(params.seed, pid);
    let neighbor_idx = (pid + 1) % NUM_PLATES;
    let (nx, nz) = plate_motion(params.seed, neighbor_idx);

    let rel_dx = mx - nx;
    let rel_dz = mz - nz;
    let convergence = (rel_dx * rel_dx + rel_dz * rel_dz).sqrt();

    let (cx, cz) = centers[pid];
    let (ncx, ncz) = centers[neighbor_idx];
    let toward_neighbor_x = ncx - cx;
    let toward_neighbor_z = ncz - cz;
    let toward_len = (toward_neighbor_x * toward_neighbor_x + toward_neighbor_z * toward_neighbor_z).sqrt() + 0.001;
    let dot = (mx * toward_neighbor_x + mz * toward_neighbor_z) / toward_len;

    let t = 1.0 - (dist_to_boundary / boundary_width).min(1.0);
    let smooth = t * t * (3.0 - 2.0 * t);

    if dot > 0.2 {
        smooth * convergence * 8.0
    } else if dot < -0.2 {
        -(smooth * 4.0)
    } else {
        smooth * dot * 2.0
    }
}

/// ── R9.2: Thermal Erosion ──────────────────────────────────────
/// Steep slopes erode, smoothing terrain. Approximated per-point
/// using slope measured via sampling offsets.

fn slope_at(wx: f64, wz: f64, params: &WorldParams) -> f64 {
    let eps = 2.0;
    let h_center = crate::engine::terrain::get_height_base(params, wx, wz);
    let h_dx = crate::engine::terrain::get_height_base(params, wx + eps, wz);
    let h_dz = crate::engine::terrain::get_height_base(params, wx, wz + eps);
    let dx = (h_dx - h_center) / eps;
    let dz = (h_dz - h_center) / eps;
    (dx * dx + dz * dz).sqrt()
}

fn thermal_erosion(wx: f64, wz: f64, _h: f64, params: &WorldParams) -> f64 {
    let slope = slope_at(wx, wz, params);
    let talus = 0.6;
    if slope > talus {
        let excess = slope - talus;
        let erosion = excess * 0.3 * 2.0;
        let curv = curvature_at(wx, wz, params);
        if curv > 0.0 {
            -erosion * (1.0 - (-curv * 0.5).exp())
        } else {
            erosion * 0.5 * (1.0 - (curv * 0.3).exp())
        }
    } else {
        0.0
    }
}

fn curvature_at(wx: f64, wz: f64, params: &WorldParams) -> f64 {
    let eps = 3.0;
    let h_center = crate::engine::terrain::get_height_base(params, wx, wz);
    let h_px = crate::engine::terrain::get_height_base(params, wx + eps, wz);
    let h_nx = crate::engine::terrain::get_height_base(params, wx - eps, wz);
    let h_pz = crate::engine::terrain::get_height_base(params, wx, wz + eps);
    let h_nz = crate::engine::terrain::get_height_base(params, wx, wz - eps);
    (h_px + h_nx + h_pz + h_nz - 4.0 * h_center) / (eps * eps)
}

/// ── R9.2: Hydraulic Erosion ────────────────────────────────────
/// Simplified: rivers carry sediment from high to low, depositing
/// in valleys. Uses water flow accumulation approximation.

fn hydraulic_erosion(wx: f64, wz: f64, _h: f64, params: &WorldParams) -> f64 {
    let eps = 3.0;
    let h_center = crate::engine::terrain::get_height_base(params, wx, wz);
    let water_level = params.water_level;

    let mut total_inflow = 0.0;
    let mut lowest = h_center;
    let dirs = [
        (eps, 0.0), (-eps, 0.0), (0.0, eps), (0.0, -eps),
        (eps, eps), (-eps, eps), (eps, -eps), (-eps, -eps),
    ];
    for &(dx, dz) in &dirs {
        let nh = crate::engine::terrain::get_height_base(params, wx + dx, wz + dz);
        if nh < h_center {
            let diff = h_center - nh;
            total_inflow += diff * 0.1;
            if nh < lowest {
                lowest = nh;
            }
        }
    }

    let water_flow = total_inflow.max(0.0);
    let sediment_capacity = water_flow * 0.3;

    if h_center > water_level && water_flow > 0.5 {
        let erosion = sediment_capacity * 0.5;
        let deposition = if lowest < h_center - 1.0 {
            let dist_to_water = (h_center - water_level).max(0.0);
            if dist_to_water < 2.0 && h_center - lowest < 2.0 {
                sediment_capacity * 0.3
            } else {
                0.0
            }
        } else {
            0.0
        };
        deposition - erosion.min(0.5)
    } else if h_center <= water_level + 1.0 && h_center > water_level - 0.5 && water_flow > 0.3 {
        water_flow * 0.15
    } else {
        0.0
    }
}

/// ── R9.3: Watershed / River Network ────────────────────────────
/// Enhances river carving with more realistic drainage patterns.
/// Replaces simple sin/cos rivers with noise-directed flow.

pub fn watershed_carve(params: &WorldParams, wx: f64, wz: f64) -> f64 {
    let seed = params.seed as f64;
    let s = seed * 0.01;

    let flow_dir = fbm(wx * 0.003 + s * 0.5, wz * 0.003, 2) * std::f64::consts::TAU;
    let (sf, cf) = flow_dir.sin_cos();

    let accumulated = fbm(wx * 0.006 + cf * 0.5, wz * 0.006 + sf * 0.5, 2);
    let flow_strength = (accumulated + 0.5).max(0.0) * 0.5;

    let base_h = crate::engine::terrain::get_height_base(params, wx, wz);
    if base_h <= params.water_level + 0.5 {
        return 0.0;
    }

    let river_val = fbm(
        wx * 0.005 + sf * flow_strength,
        wz * 0.005 + cf * flow_strength,
        2,
    );
    let threshold = 0.1 + flow_strength * 0.25;
    if river_val < threshold {
        let t = 1.0 - (river_val / threshold).min(1.0);
        let depth = t * t * (2.0 + flow_strength * 5.0);
        let edge = (river_val / threshold).clamp(0.0, 1.0);
        let smooth = 1.0 - edge * edge * edge;
        depth * smooth
    } else {
        0.0
    }
}

/// ── R9.4: Sedimentation ───────────────────────────────────────
/// Deposits sediment in valleys, floodplains, and river mouths.

fn sedimentation(wx: f64, wz: f64, h: f64, params: &WorldParams) -> f64 {
    let water_level = params.water_level;
    let is_underwater = h <= water_level;

    if is_underwater {
        let depth = water_level - h;
        if depth < 0.5 {
            let sediment_noise = fbm(wx * 0.01 + 200.0, wz * 0.01, 2);
            if sediment_noise > 0.2 {
                sediment_noise * 0.3
            } else {
                0.0
            }
        } else if depth < 2.0 {
            let sediment_noise = fbm(wx * 0.008 + 300.0, wz * 0.008 + 100.0, 2);
            (sediment_noise * 0.5 + 0.5) * 0.2 * (1.0 - depth / 2.0)
        } else {
            0.0
        }
    } else {
        let curv = curvature_at(wx, wz, params);
        if curv < 0.0 && h - water_level < 3.0 {
            let valley_floor = fbm(wx * 0.008 + 50.0, wz * 0.008 + 50.0, 2);
            if valley_floor > 0.1 {
                (valley_floor * 0.5 + 0.5) * 0.2 * (1.0 - (h - water_level) / 3.0)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

/// ── R9.5: Continental Shelf Profile ───────────────────────────
/// Creates realistic ocean floor: shelf → slope → abyssal plain.

fn continental_shelf(h: f64, water_level: f64) -> f64 {
    if h >= water_level {
        return h;
    }
    let depth = water_level - h;
    let shelf_depth = 3.0;
    let slope_depth = 8.0;
    let abyssal_depth = 15.0;

    if depth < shelf_depth {
        let t = depth / shelf_depth;
        let shelf_slope = t * t * 0.5;
        water_level - shelf_slope * shelf_depth
    } else if depth < slope_depth {
        let t = (depth - shelf_depth) / (slope_depth - shelf_depth);
        let steep_slope = shelf_depth + t * t * (abyssal_depth - shelf_depth) * 2.0;
        (water_level - steep_slope).min(h)
    } else {
        let abyssal_h = water_level - abyssal_depth;
        let noise_floor = fbm(h * 0.01 + 500.0, 0.0, 1) * 1.5;
        let transition = ((depth - slope_depth) / 3.0).min(1.0);
        let floor = abyssal_h + noise_floor;
        if h < floor {
            h + (floor - h) * transition * 0.5
        } else {
            h
        }
    }
}

/// ── Public API ─────────────────────────────────────────────────

pub fn apply_plate_tectonics(wx: f64, wz: f64, params: &WorldParams) -> f64 {
    plate_boundary(wx, wz, params)
}

pub fn apply_erosion(wx: f64, wz: f64, h: f64, params: &WorldParams) -> f64 {
    let thermal = thermal_erosion(wx, wz, h, params);
    let hydraulic = hydraulic_erosion(wx, wz, h, params);
    hydraulic + thermal
}

pub fn apply_sedimentation(wx: f64, wz: f64, h: f64, params: &WorldParams) -> f64 {
    sedimentation(wx, wz, h, params)
}

pub fn apply_continental_shelf(h: f64, water_level: f64) -> f64 {
    continental_shelf(h, water_level)
}

pub fn apply_watershed_river(wx: f64, wz: f64, params: &WorldParams) -> f64 {
    watershed_carve(params, wx, wz)
}
