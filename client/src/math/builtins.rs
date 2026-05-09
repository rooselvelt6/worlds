pub fn perlin_noise(x: f64, z: f64) -> f64 {
    let ix = x.floor() as i64;
    let iz = z.floor() as i64;
    let fx = x - ix as f64;
    let fz = z - iz as f64;
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uz = fz * fz * (3.0 - 2.0 * fz);

    fn hash(ix: i64, iz: i64) -> f64 {
        let n = ix.wrapping_mul(157).wrapping_add(iz.wrapping_mul(113)) as f64;
        (n.sin() * 43758.5453).fract().abs()
    }

    let v00 = hash(ix, iz);
    let v10 = hash(ix + 1, iz);
    let v01 = hash(ix, iz + 1);
    let v11 = hash(ix + 1, iz + 1);

    let v0 = v00 + (v10 - v00) * ux;
    let v1 = v01 + (v11 - v01) * ux;
    v0 + (v1 - v0) * uz
}

pub fn simplex_noise(x: f64, z: f64) -> f64 {
    fbm(x, z, 4)
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
    let _d = (x * x + z * z).sqrt();
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
    domain_warp_strength(x, z, 2.0)
}

pub fn domain_warp_strength(x: f64, z: f64, strength: f64) -> f64 {
    let warp_x = x + perlin_noise(x + 10.0, z) * strength;
    let warp_z = z + perlin_noise(x, z + 10.0) * strength;
    fbm(warp_x, warp_z, 4)
}

pub fn hybrid_terrain(x: f64, z: f64) -> f64 {
    let f = fbm(x, z, 4);
    let r = ridged_fbm(x * 1.5, z * 1.5, 4);
    let v = voronoi(x * 0.5, z * 0.5);
    (f + r + v) / 3.0
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
