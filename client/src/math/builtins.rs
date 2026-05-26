static mut PERM: [u8; 256] = [
    151,160,137,91,90,15,131,13,201,95,96,53,194,233,7,225,
    140,36,103,30,69,142,8,99,37,240,21,10,23,190,6,148,
    247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,
    57,177,33,88,237,149,56,87,174,20,125,136,171,168,68,175,
    74,165,71,134,139,48,27,166,77,146,158,231,83,111,229,122,
    60,211,133,230,220,105,92,41,55,46,245,40,244,102,143,54,
    65,25,63,161,1,216,80,73,209,76,132,187,208,89,18,169,
    200,196,135,130,116,188,159,86,164,100,109,198,173,186,3,64,
    52,217,226,250,124,123,5,202,38,147,118,126,255,82,85,212,
    207,206,59,227,47,16,58,17,182,189,28,42,223,183,170,213,
    119,248,152,2,44,154,163,70,221,153,101,155,167,43,172,9,
    129,22,39,253,19,98,108,110,79,113,224,232,178,185,112,104,
    218,246,97,228,251,34,242,193,238,210,144,12,191,179,162,241,
    81,51,145,235,249,14,239,107,49,192,214,31,181,199,106,157,
    184,84,204,176,115,121,50,45,127,4,150,254,138,236,205,93,
    222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180,
];

pub fn set_noise_seed(seed: u32) {
    unsafe {
        let mut rng = seed as u64;
        for i in (1..256).rev() {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = (rng >> 32) as usize % (i + 1);
            PERM.swap(i, j);
        }
    }
}

fn perm_index(i: i64) -> usize {
    (i & 255) as usize
}

#[inline]
fn perm(idx: usize) -> u8 {
    unsafe { PERM[idx] }
}

fn grad2(hash: u8, x: f64, z: f64) -> f64 {
    let h = hash & 7;
    let u = if h < 4 { x } else { z };
    let v = if h < 4 { z } else { x };
    let u = if h & 1 == 0 { u } else { -u };
    let v = if h & 2 == 0 { v } else { -v };
    u + v
}

pub fn perlin_noise(x: f64, z: f64) -> f64 {
    let ix = x.floor() as i64;
    let iz = z.floor() as i64;
    let fx = x - ix as f64;
    let fz = z - iz as f64;

    let ux = fx * fx * fx * (fx * (fx * 6.0 - 15.0) + 10.0);
    let uz = fz * fz * fz * (fz * (fz * 6.0 - 15.0) + 10.0);

    let xi = perm_index(ix);

    let xi1 = (xi + 1) & 255;
    let v00 = grad2(perm(perm_index(perm(xi) as i64 + iz)), fx, fz);
    let v10 = grad2(perm(perm_index(perm(xi1) as i64 + iz)), fx - 1.0, fz);
    let v01 = grad2(perm(perm_index(perm(xi) as i64 + iz + 1)), fx, fz - 1.0);
    let v11 = grad2(perm(perm_index(perm(xi1) as i64 + iz + 1)), fx - 1.0, fz - 1.0);

    let v0 = v00 + (v10 - v00) * ux;
    let v1 = v01 + (v11 - v01) * ux;
    (v0 + (v1 - v0) * uz + 1.0) * 0.5
}

pub fn simplex_noise(x: f64, z: f64) -> f64 {
    let f2 = 0.5 * (3.0_f64.sqrt() - 1.0);
    let g2 = (3.0 - 3.0_f64.sqrt()) / 6.0;

    let s = (x + z) * f2;
    let i = (x + s).floor() as i64;
    let j = (z + s).floor() as i64;

    let t = (i + j) as f64 * g2;
    let x0 = x - (i as f64 - t);
    let z0 = z - (j as f64 - t);

    let (i1, j1) = if x0 > z0 { (1, 0) } else { (0, 1) };

    let x1 = x0 - i1 as f64 + g2;
    let z1 = z0 - j1 as f64 + g2;
    let x2 = x0 - 1.0 + 2.0 * g2;
    let z2 = z0 - 1.0 + 2.0 * g2;

    let gi0 = perm(perm_index(perm(perm_index(i)) as i64 + j)) as u8;
    let gi1 = perm(perm_index(perm(perm_index(i + i1)) as i64 + j + j1)) as u8;
    let gi2 = perm(perm_index(perm(perm_index(i + 1)) as i64 + j + 1)) as u8;

    let n0 = {
        let t = 0.5 - x0 * x0 - z0 * z0;
        if t < 0.0 { 0.0 } else { let t = t * t; t * t * grad2(gi0, x0, z0) }
    };
    let n1 = {
        let t = 0.5 - x1 * x1 - z1 * z1;
        if t < 0.0 { 0.0 } else { let t = t * t; t * t * grad2(gi1, x1, z1) }
    };
    let n2 = {
        let t = 0.5 - x2 * x2 - z2 * z2;
        if t < 0.0 { 0.0 } else { let t = t * t; t * t * grad2(gi2, x2, z2) }
    };

    (n0 + n1 + n2) * 0.5 + 0.5
}

fn grad3(hash: u8, x: f64, y: f64, z: f64) -> f64 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 { y } else { if h == 12 || h == 14 { x } else { z } };
    let u = if h & 1 == 0 { u } else { -u };
    let v = if h & 2 == 0 { v } else { -v };
    u + v
}

fn p3(ix: i64, iy: i64, iz: i64) -> u8 {
    let a = perm(perm_index(ix)) as i64;
    let b = perm(perm_index(a + iy)) as i64;
    perm(perm_index(b + iz))
}

pub fn perlin_noise_3d(x: f64, y: f64, z: f64) -> f64 {
    let ix = x.floor() as i64;
    let iy = y.floor() as i64;
    let iz = z.floor() as i64;
    let fx = x - ix as f64;
    let fy = y - iy as f64;
    let fz = z - iz as f64;

    let ux = fx * fx * fx * (fx * (fx * 6.0 - 15.0) + 10.0);
    let uy = fy * fy * fy * (fy * (fy * 6.0 - 15.0) + 10.0);
    let uz = fz * fz * fz * (fz * (fz * 6.0 - 15.0) + 10.0);

    let v000 = grad3(p3(ix, iy, iz), fx, fy, fz);
    let v100 = grad3(p3(ix + 1, iy, iz), fx - 1.0, fy, fz);
    let v010 = grad3(p3(ix, iy + 1, iz), fx, fy - 1.0, fz);
    let v110 = grad3(p3(ix + 1, iy + 1, iz), fx - 1.0, fy - 1.0, fz);
    let v001 = grad3(p3(ix, iy, iz + 1), fx, fy, fz - 1.0);
    let v101 = grad3(p3(ix + 1, iy, iz + 1), fx - 1.0, fy, fz - 1.0);
    let v011 = grad3(p3(ix, iy + 1, iz + 1), fx, fy - 1.0, fz - 1.0);
    let v111 = grad3(p3(ix + 1, iy + 1, iz + 1), fx - 1.0, fy - 1.0, fz - 1.0);

    let v00 = v000 + (v100 - v000) * ux;
    let v10 = v010 + (v110 - v010) * ux;
    let v01 = v001 + (v101 - v001) * ux;
    let v11 = v011 + (v111 - v011) * ux;
    let v0 = v00 + (v10 - v00) * uy;
    let v1 = v01 + (v11 - v01) * uy;
    (v0 + (v1 - v0) * uz + 1.0) * 0.5
}

pub fn fbm_3d(x: f64, y: f64, z: f64, octaves: u32) -> f64 {
    let mut v = 0.0;
    let mut a = 1.0;
    let mut f = 1.0;
    let mut m = 0.0;
    for _ in 0..octaves {
        v += perlin_noise_3d(x * f, y * f, z * f) * a;
        m += a;
        a *= 0.5;
        f *= 2.0;
    }
    if m > 0.0 { v / m } else { 0.0 }
}

pub fn fbm(x: f64, z: f64, octaves: u32) -> f64 {
    let mut v = 0.0;
    let mut a = 1.0;
    let mut f = 1.0;
    let mut m = 0.0;
    for _ in 0..octaves {
        v += perlin_noise(x * f, z * f) * a;
        m += a;
        a *= 0.5;
        f *= 2.0;
    }
    if m > 0.0 { v / m } else { 0.0 }
}

pub fn voronoi(x: f64, z: f64) -> f64 {
    let ix = x.floor() as i64;
    let iz = z.floor() as i64;
    let mut min_d = f64::MAX;
    let mut second_d = f64::MAX;
    for dx in -2..=2i64 {
        for dz in -2..=2i64 {
            let cx = ix + dx;
            let cz = iz + dz;
            let hx = (cx as f64 * 127.1 + cz as f64 * 311.7).sin().fract().abs();
            let hz = (cx as f64 * 269.5 + cz as f64 * 183.3).sin().fract().abs();
            let px = cx as f64 + hx;
            let pz = cz as f64 + hz;
            let d = ((x - px).powi(2) + (z - pz).powi(2)).sqrt();
            if d < min_d { second_d = min_d; min_d = d; }
            else if d < second_d { second_d = d; }
        }
    }
    (second_d - min_d).abs() * 1.5
}

pub fn juliaset(cx: f64, cy: f64, c_param: f64) -> f64 {
    let mut x = cx;
    let mut y = cy;
    for _ in 0..8 {
        let xt = x * x - y * y + c_param;
        y = 2.0 * x * y + 0.1;
        x = xt;
        if x * x + y * y > 4.0 { break; }
    }
    (x * x + y * y).sqrt().min(1.0)
}

pub fn mandelbrot(x: f64, z: f64) -> f64 {
    let mut r = x;
    let mut i = z;
    for _ in 0..12 {
        let rt = r * r - i * i + x;
        i = 2.0 * r * i + z;
        r = rt;
        if r * r + i * i > 16.0 { break; }
    }
    (r * r + i * i).sqrt().min(1.0) / 4.0
}

pub fn tetrahedron(x: f64, z: f64) -> f64 {
    let mut a = 0.0;
    let mut px = x;
    let mut pz = z;
    for _ in 0..4 {
        let nx = (px - pz).abs();
        let nz = (px + pz).abs();
        px = nx * 0.5 - 0.3;
        pz = nz * 0.5 - 0.2;
        a += (px * px + pz * pz).sqrt() * 0.3;
    }
    (a * 2.0).min(1.0)
}

pub fn cube_fractal(x: f64, z: f64) -> f64 {
    let cx = (x * 0.5).round();
    let cz = (z * 0.5).round();
    let dx = x - cx * 2.0;
    let dz = z - cz * 2.0;
    let dist = (dx * dx + dz * dz).sqrt();
    let cell = (cx + cz * 1.7).sin() * 0.5 + 0.5;
    let edge = (1.0 - dist).max(0.0) * cell;
    let detail = perlin_noise(x * 2.0, z * 2.0) * 0.2;
    (edge + detail).max(0.0).min(1.0)
}

pub fn sphere_field(x: f64, z: f64) -> f64 {
    let ix = (x * 0.3).floor() as i64;
    let iz = (z * 0.3).floor() as i64;
    let mut val = 0.0;
    for dx in -2..=2i64 {
        for dz in -2..=2i64 {
            let cx = ix + dx;
            let cz = iz + dz;
            let hx = (cx as f64 * 127.1 + cz as f64 * 311.7).sin().fract().abs();
            let hz = (cx as f64 * 269.5 + cz as f64 * 183.3).sin().fract().abs();
            let hs = (cx as f64 * 419.2 + cz as f64 * 571.1).sin().fract().abs() * 0.8 + 0.2;
            let px = cx as f64 * 3.33 + hx;
            let pz = cz as f64 * 3.33 + hz;
            let d = ((x - px).powi(2) + (z - pz).powi(2)).sqrt();
            if d < hs {
                let t = 1.0 - (d / hs);
                let s = 1.0 - t * t;
                val = f64::max(val, s * s);
            }
        }
    }
    val
}

pub fn menger_sponge(x: f64, z: f64) -> f64 {
    let mut h = 1.0;
    let mut px = x;
    let mut pz = z;
    for _ in 0..4 {
        px = (px * 3.0).abs();
        pz = (pz * 3.0).abs();
        let cx = (px - 1.0).abs() - 0.5;
        let cz = (pz - 1.0).abs() - 0.5;
        let dist = cx.max(cz);
        if dist < 0.5 {
            h *= dist * 2.0;
        } else {
            h *= 0.5;
        }
        let cell = px.floor() as i64 + pz.floor() as i64;
        let skip = cell % 3 == 0;
        if skip { h *= 0.3; }
        px = px.fract();
        pz = pz.fract();
    }
    h.max(0.0).min(1.0)
}

pub fn vortex(x: f64, z: f64) -> f64 {
    let r = (x * x + z * z).sqrt();
    let a = z.atan2(x);
    (a * 3.0 + r * 0.5).sin() * (1.0 / (r + 1.0))
}

pub fn ice(x: f64, z: f64) -> f64 {
    let p = perlin_noise(x * 0.3, z * 0.3);
    let v = voronoi(x * 0.5, z * 0.5);
    (p * 0.6 + v * 0.4).abs()
}

pub fn wave(x: f64, z: f64) -> f64 {
    wave_param(x, z, 0.5)
}

pub fn wave_param(x: f64, z: f64, freq: f64) -> f64 {
    (x * freq).sin() * (z * freq).cos() * 0.5 + 0.5
}

pub fn spiral(x: f64, z: f64) -> f64 {
    spiral_param(x, z, 2.0)
}

pub fn spiral_param(x: f64, z: f64, turns: f64) -> f64 {
    let r = (x * x + z * z).sqrt();
    (r * turns - z.atan2(x)).sin() * 0.5
}

pub fn hexagonal(x: f64, z: f64) -> f64 {
    let hx = x * 0.5;
    let hz = z * 0.5 + (x * 0.25).sin();
    let dx = (hx - hx.round()).abs();
    let dz = (hz - hz.round()).abs();
    (dx.max(dz) * 2.0).min(1.0)
}

pub fn ridged_fbm(x: f64, z: f64, octaves: u32) -> f64 {
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut prev = 1.0;
    for _ in 0..octaves {
        let n = 1.0 - perlin_noise(x * frequency, z * frequency).abs();
        let ridge = n * n;
        total += ridge * amplitude * prev;
        prev = ridge;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    total
}

pub fn domain_warp(x: f64, z: f64) -> f64 {
    domain_warp_strength(x, z, 4.0)
}

pub fn domain_warp_strength(x: f64, z: f64, strength: f64) -> f64 {
    let warp_x = x + perlin_noise(x + 10.0, z) * strength;
    let warp_z = z + perlin_noise(x, z + 10.0) * strength;
    let inner = fbm(warp_x * 0.8, warp_z * 0.8, 5);
    let outer = fbm(warp_x, warp_z, 3) * 0.3;
    inner * 0.7 + outer
}

pub fn hybrid_terrain(x: f64, z: f64) -> f64 {
    let f = fbm(x, z, 5);
    let r = ridged_fbm(x * 1.5, z * 1.5, 4);
    let v = voronoi(x * 0.5, z * 0.5);
    (f + r + v) / 3.0
}

pub fn sierpinski_triangle(x: f64, z: f64) -> f64 {
    let mut px = x.abs() as u32;
    let mut pz = z.abs() as u32;
    let mut count = 0;
    while (px | pz) != 0 {
        if (px & 1) == 1 && (pz & 1) == 1 {
            count += 1;
        }
        px >>= 1;
        pz >>= 1;
    }
    if count % 2 == 0 { 0.0 } else { 1.0 }
}

pub fn plasma(x: f64, z: f64) -> f64 {
    let freq = 3.0;
    let t = x.sin();
    let v1 = (x * freq + z * freq * 0.5 + t).sin();
    let v2 = (z * freq - x * freq * 0.3).cos();
    v1 * v2 * 0.5 + 0.5
}

pub fn cellular(x: f64, z: f64) -> f64 {
    let ix = x.floor() as i64;
    let iz = z.floor() as i64;
    let fx = x - ix as f64;
    let fz = z - iz as f64;
    let rule = ((ix.wrapping_mul(127).wrapping_add(iz.wrapping_mul(311)) as f64).sin() * 43758.5453).fract().abs();
    let state = (rule * 8.0) as u8;
    let mut v = 0.0;
    for dy in -1..=1i64 {
        for dx in -1..=1i64 {
            let nx = (ix + dx).wrapping_mul(157).wrapping_add((iz + dy).wrapping_mul(113)) as f64;
            let n = (nx.sin() * 43758.5453).fract().abs();
            if (state >> ((n * 7.0) as u8 % 8)) & 1 == 1 {
                let d = ((fx - dx as f64).powi(2) + (fz - dy as f64).powi(2)).sqrt();
                v += (1.0 - d * 0.5).max(0.0);
            }
        }
    }
    (v / 3.0).min(1.0)
}

pub fn strange_attractor(x: f64, z: f64, a: f64, b: f64) -> f64 {
    let c = 1.5 + a * 2.0;
    let d = 0.5 + b * 1.5;
    let mut px = x;
    let mut pz = z;
    for _ in 0..6 {
        let nx = (a * pz + c * (a * px).cos()).sin();
        let nz = (b * px + d * (b * pz).sin()).cos();
        px = nx;
        pz = nz;
    }
    (px * pz).abs() * 0.5 + 0.3
}

pub fn worley(x: f64, z: f64) -> f64 {
    let ix = x.floor() as i64;
    let iz = z.floor() as i64;
    let mut dists = [f64::MAX; 3];
    for dx in -2..=2i64 {
        for dz in -2..=2i64 {
            let cx = ix + dx;
            let cz = iz + dz;
            let hx = (cx as f64 * 127.1 + cz as f64 * 311.7).sin().fract().abs();
            let hz = (cx as f64 * 269.5 + cz as f64 * 183.3).sin().fract().abs();
            let px = cx as f64 + hx;
            let pz = cz as f64 + hz;
            let d = ((x - px).powi(2) + (z - pz).powi(2)).sqrt();
            if d < dists[0] { dists[2] = dists[1]; dists[1] = dists[0]; dists[0] = d; }
            else if d < dists[1] { dists[2] = dists[1]; dists[1] = d; }
            else if d < dists[2] { dists[2] = d; }
        }
    }
    (dists[1] - dists[0]).min(1.0)
}

pub fn marble(x: f64, z: f64) -> f64 {
    let f = fbm(x, z, 5);
    let detail = fbm(x * 2.0, z * 2.0, 4);
    let v = (f * 4.0 + detail * 2.0).sin() * 0.5 + 0.5;
    v
}

pub fn terrazas(x: f64, z: f64, levels: f64) -> f64 {
    let h = fbm(x, z, 5);
    let levels = levels.max(1.0);
    (h * levels).floor() / levels
}

pub fn erosion(x: f64, z: f64) -> f64 {
    let h = fbm(x, z, 6);
    let slope_x = (fbm(x + 0.1, z, 6) - fbm(x - 0.1, z, 6)) / 0.2;
    let slope_z = (fbm(x, z + 0.1, 6) - fbm(x, z - 0.1, 6)) / 0.2;
    let slope = (slope_x * slope_x + slope_z * slope_z).sqrt();
    let erosion_factor = (1.0 - slope).max(0.0).powi(2);
    h * (0.3 + erosion_factor * 0.7)
}

pub fn thermal(x: f64, z: f64) -> f64 {
    let h = fbm(x, z, 5);
    let temp = perlin_noise(x * 2.0 + 5.0, z * 2.0 + 5.0) * 0.5 + 0.5;
    let thermal_noise = perlin_noise(x * 10.0, z * 10.0) * 0.15;
    h * temp + thermal_noise
}

pub fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let r = r.clamp(0.0, 1.0);
    let g = g.clamp(0.0, 1.0);
    let b = b.clamp(0.0, 1.0);
    let mx = r.max(g).max(b);
    let mn = r.min(g).min(b);
    let c = mx - mn;
    let l = (mx + mn) / 2.0;
    if c == 0.0 { return (0.0, 0.0, l); }
    let h = if mx == r { ((g - b) / c).rem_euclid(6.0) / 6.0 }
            else if mx == g { ((b - r) / c + 2.0) / 6.0 }
            else { ((r - g) / c + 4.0) / 6.0 };
    let s = c / (1.0 - (2.0 * l - 1.0).abs());
    (h, s, l)
}

pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 { return (l, l, l); }
    let hue_to_rgb = |p: f32, q: f32, t: f32| -> f32 {
        let t = t.rem_euclid(1.0);
        if t < 1.0 / 6.0 { p + (q - p) * 6.0 * t }
        else if t < 0.5 { q }
        else if t < 2.0 / 3.0 { p + (q - p) * (2.0 / 3.0 - t) * 6.0 }
        else { p }
    };
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    (hue_to_rgb(p, q, h + 1.0 / 3.0), hue_to_rgb(p, q, h), hue_to_rgb(p, q, h - 1.0 / 3.0))
}
