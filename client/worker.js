const PERM = new Uint8Array([
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
]);

function setNoiseSeed(seed) {
    let rng = seed >>> 0;
    for (let i = 255; i > 0; i--) {
        rng = Math.imul(rng, 6364136223846793005) + 1442695040888963407;
        const j = (rng >>> 32) % (i + 1);
        const tmp = PERM[i];
        PERM[i] = PERM[j];
        PERM[j] = tmp;
    }
}

function perm(i) { return PERM[i & 255]; }

function grad2(hash, x, z) {
    const h = hash & 7;
    const u = h < 4 ? x : z;
    const v = h < 4 ? z : x;
    return ((h & 1) === 0 ? u : -u) + ((h & 2) === 0 ? v : -v);
}

function grad3(hash, x, y, z) {
    const h = hash & 15;
    const u = h < 8 ? x : y;
    const v = h < 4 ? y : (h === 12 || h === 14 ? x : z);
    return ((h & 1) === 0 ? u : -u) + ((h & 2) === 0 ? v : -v);
}

function perlinNoise(x, z) {
    const ix = Math.floor(x), iz = Math.floor(z);
    const fx = x - ix, fz = z - iz;
    const ux = fx * fx * fx * (fx * (fx * 6 - 15) + 10);
    const uz = fz * fz * fz * (fz * (fz * 6 - 15) + 10);
    const v00 = grad2(perm(perm(ix) + iz), fx, fz);
    const v10 = grad2(perm(perm(ix + 1) + iz), fx - 1, fz);
    const v01 = grad2(perm(perm(ix) + iz + 1), fx, fz - 1);
    const v11 = grad2(perm(perm(ix + 1) + iz + 1), fx - 1, fz - 1);
    return ((v00 + (v10 - v00) * ux) + ((v01 + (v11 - v01) * ux) - (v00 + (v10 - v00) * ux)) * uz + 1) * 0.5;
}

function perlinNoise3D(x, y, z) {
    const ix = Math.floor(x), iy = Math.floor(y), iz = Math.floor(z);
    const fx = x - ix, fy = y - iy, fz = z - iz;
    const ux = fx * fx * fx * (fx * (fx * 6 - 15) + 10);
    const uy = fy * fy * fy * (fy * (fy * 6 - 15) + 10);
    const uz = fz * fz * fz * (fz * (fz * 6 - 15) + 10);
    const p3 = (x, y, z) => perm(perm(perm(x) + y) + z);
    const g = [
        grad3(p3(ix, iy, iz), fx, fy, fz), grad3(p3(ix + 1, iy, iz), fx - 1, fy, fz),
        grad3(p3(ix, iy + 1, iz), fx, fy - 1, fz), grad3(p3(ix + 1, iy + 1, iz), fx - 1, fy - 1, fz),
        grad3(p3(ix, iy, iz + 1), fx, fy, fz - 1), grad3(p3(ix + 1, iy, iz + 1), fx - 1, fy, fz - 1),
        grad3(p3(ix, iy + 1, iz + 1), fx, fy - 1, fz - 1), grad3(p3(ix + 1, iy + 1, iz + 1), fx - 1, fy - 1, fz - 1),
    ];
    const v00 = g[0] + (g[1] - g[0]) * ux, v10 = g[2] + (g[3] - g[2]) * ux;
    const v01 = g[4] + (g[5] - g[4]) * ux, v11 = g[6] + (g[7] - g[6]) * ux;
    return ((v00 + (v10 - v00) * uy) + ((v01 + (v11 - v01) * uy) - (v00 + (v10 - v00) * uy)) * uz + 1) * 0.5;
}

function fbm(x, z, octaves) {
    let v = 0, a = 1, f = 1, m = 0;
    for (let i = 0; i < octaves; i++) { v += perlinNoise(x * f, z * f) * a; m += a; a *= 0.5; f *= 2; }
    return m > 0 ? v / m : 0;
}

// ── Zone ID conversion (JSON from Rust uses strings like "Forest") ──
const ZONE_IDS = {
    forest: 0, plains: 1, desert: 2, tundra: 3, jungle: 4, volcanic: 5,
    ocean: 6, crystal: 7, cave: 8, lava: 9, fungus: 10, abyss: 11,
    storm: 12, aurora: 13, magma: 14,
    coral_reef: 15, kelp_forest: 16, sandy_plain: 17, rocky_reef: 18, deep_ocean: 19,
};
const ZONE_FOREST = 0; const ZONE_PLAINS = 1; const ZONE_DESERT = 2; const ZONE_TUNDRA = 3;
const ZONE_JUNGLE = 4; const ZONE_VOLCANIC = 5; const ZONE_OCEAN = 6; const ZONE_CRYSTAL = 7;
const ZONE_CAVE = 8; const ZONE_LAVA = 9; const ZONE_FUNGUS = 10; const ZONE_ABYSS = 11;
const ZONE_STORM = 12; const ZONE_AURORA = 13; const ZONE_MAGMA = 14;
const ZONE_CORAL_REEF = 15; const ZONE_KELP_FOREST = 16; const ZONE_SANDY_PLAIN = 17;
const ZONE_ROCKY_REEF = 18; const ZONE_DEEP_OCEAN = 19;

function zoneId(params) {
    if (typeof params.zone === 'number') return params.zone;
    return ZONE_IDS[params.zone] ?? 0;
}

function isSpecificZone(params) {
    return zoneId(params) !== ZONE_FOREST;
}

// ── Block type constants ──
const BLK_AIR = 0; const BLK_GRASS = 1; const BLK_DIRT = 2; const BLK_STONE = 3;
const BLK_SAND = 4; const BLK_SNOW = 5; const BLK_COAL_ORE = 6; const BLK_IRON_ORE = 7;
const BLK_GOLD_ORE = 8; const BLK_DIAMOND_ORE = 9; const BLK_GRAVEL = 10; const BLK_CLAY = 11;
const BLK_WATER = 12; const BLK_LAVA = 13;
const BLK_PACKED_ICE = 20; const BLK_OBSIDIAN = 21; const BLK_MOSS = 22;
const BLK_GLOW_SHROOM = 23; const BLK_MAGMA_BLOCK = 24; const BLK_SOUL_SAND = 25; const BLK_BASALT = 26;

function riverCarve(params, wx, wz) {
    const seed = params.seed;
    const s = seed * 0.01;
    const river_a = (Math.sin(wx * 0.004 + wz * 0.002 + s) + Math.cos(wx * 0.002 - wz * 0.004 + s * 0.7)) * 0.5;
    const river_b = (Math.sin(wx * 0.006 - wz * 0.003 + s * 1.3) + Math.cos(wx * 0.003 + wz * 0.005 + s * 0.5)) * 0.5;
    const river_val = Math.abs(river_a * 0.65 + river_b * 0.35);
    const base_h = fbm(wx * params.scale, wz * params.scale, params.octaves) * params.amplitude + params.water_level;
    if (base_h <= params.water_level + 0.5) return 0;
    const width_vary = (perlinNoise(wx * 0.005 + 100, wz * 0.005) + perlinNoise(wx * 0.005, wz * 0.005 + 100)) * 0.25 + 0.5;
    const base_width = 0.12 + width_vary * 0.13;
    if (river_val < base_width) {
        const t = 1 - Math.max(0, Math.min(1, river_val / base_width));
        const depth = t * t * (3 + width_vary * 3);
        const edge = Math.max(0, Math.min(1, river_val / base_width));
        return depth * (1 - edge * edge * edge);
    }
    return 0;
}

function applyMutation(params, cx, cz) {
    if (params.mutation <= 0) return params;
    const h = Math.sin((params.seed * 374761393 + cx * 668265263 + cz * 1274126177) * 0.001) * 43758.5453;
    const norm = Math.abs(h % 1);
    const offset = (norm - 0.5) * 2 * params.mutation;
    return { ...params, scale: params.scale * (1 + offset * 0.1), amplitude: params.amplitude * (1 + offset * 0.15) };
}

function getZone(params, wx, wz) {
    if (isSpecificZone(params)) return zoneId(params);
    const h = _getHeightRaw(params, wx, wz);
    const water = params.water_level;
    if (h <= water) {
        const depth = water - h;
        const n = fbm(wx * 0.008, wz * 0.008, 3);
        const n2 = fbm(wx * 0.012 + 100, wz * 0.012, 2);
        if (depth < 0.8 && n > 0.2 && n2 > -0.1) return ZONE_CORAL_REEF;
        if (depth < 2 && Math.abs(n) < 0.25 && n2 < 0.2) return ZONE_KELP_FOREST;
        if (depth > 4 || n < -0.3) return ZONE_DEEP_OCEAN;
        if (n > 0) return ZONE_ROCKY_REEF;
        return ZONE_SANDY_PLAIN;
    }
    const t = fbm(wx * 0.008, wz * 0.008, 2);
    const h2 = fbm(wx * 0.008 + 50, wz * 0.008, 2);
    if (t < -0.35) return ZONE_TUNDRA;
    if (t > 0.45) return h2 < -0.25 ? ZONE_DESERT : ZONE_VOLCANIC;
    if (h2 > 0.45) return h2 > 0.6 ? ZONE_LAVA : ZONE_JUNGLE;
    if (h2 < -0.35) return ZONE_PLAINS;
    if (h2 < 0) return ZONE_OCEAN;
    return ZONE_FOREST;
}

function _getHeightRaw(params, wx, wz) {
    const h = fbm(wx * params.scale, wz * params.scale, params.octaves) * params.amplitude + params.water_level;
    return Math.max(0, h);
}

function getHeight(params, wx, wz) {
    let h = _getHeightRaw(params, wx, wz);
    h -= riverCarve(params, wx, wz);
    if (params.canyons) {
        const canyon = Math.sin(wx * 0.04) * Math.cos(wz * 0.04) + Math.sin(wx * 0.06 + wz * 0.08) * 0.5;
        if (canyon < -0.2) h = Math.max(h - (-canyon - 0.2) * 12, params.water_level - 6);
    }
    const z = zoneId(params);
    switch (z) {
        case ZONE_OCEAN: h += params.water_level; break;
        case ZONE_VOLCANIC: h = h * 1.5 + 2; break;
        case ZONE_CRYSTAL: h *= 0.5; break;
        case ZONE_CAVE: { const cn = Math.sin(wx * 0.3) * Math.cos(wz * 0.25) * 3; const e = Math.max(0.3, Math.abs(Math.sin(wx * 0.8) * Math.cos(wz * 0.7))) * 4; h = 5 + cn - Math.max(0, Math.min(2, e)); break; }
        case ZONE_FUNGUS: h = h * 0.6 + 1 + Math.sin(wx * 0.3) * Math.cos(wz * 0.3) * 1.5; break;
        case ZONE_ABYSS: h = h * 0.2 + params.water_level * 0.3; break;
        case ZONE_STORM: h = h * 1.8 + 1; break;
        case ZONE_AURORA: h = h * 0.5 + 0.5; break;
        case ZONE_MAGMA: h = h * 2 + 3 + Math.abs(Math.sin(wx * 0.2)) * 2; break;
    }
    const mh = { val: h };
    zoneEffects(z, params, wx, wz, mh);
    return Math.max(0, mh.val);
}

function zoneEffects(zone, params, wx, wz, h) {
    switch (zone) {
        case ZONE_CRYSTAL: { const c = Math.sin(wx * 0.8) * Math.cos(wz * 0.8); if (Math.abs(c) > 0.7) h.val += 3 + c * 2; break; }
        case ZONE_CAVE: {
            const cn = fbm(wx * 0.04 + params.seed * 0.01, wz * 0.04, 3);
            const canyon = Math.sin(wx * 0.08) * Math.cos(wz * 0.08) + Math.sin(wx * 0.12 + wz * 0.15) * 0.5;
            if (canyon < -0.3 || cn < -0.2) { const d = Math.max(0, -canyon) * 3 + Math.max(0, -cn) * 2; h.val = Math.max(h.val - d, params.water_level - 4); }
            const pillar = Math.sin(wx * 0.2) * Math.cos(wz * 0.2);
            if (pillar > 0.6 && h.val > params.water_level) h.val += (pillar - 0.6) * 3;
            break;
        }
        case ZONE_FUNGUS: { const s = Math.sin(wx * 1.5) * Math.cos(wz * 1.5); if (Math.abs(s) > 0.6) h.val += 2 + s * 1.5; break; }
        case ZONE_ABYSS: { const p = Math.sin(wx * 0.3) * Math.cos(wz * 0.3); if (Math.abs(p) > 0.8) h.val += 4; h.val = Math.min(h.val, 4); break; }
        case ZONE_STORM: h.val += Math.abs(Math.sin(wx * 2)) * Math.abs(Math.cos(wz * 2)) * 2; break;
        case ZONE_AURORA: h.val += (Math.sin(wx * 0.5) + Math.cos(wz * 0.7)) * 0.5; break;
        case ZONE_MAGMA: { const f = Math.sin(wx * 0.4) * Math.cos(wz * 0.4); if (Math.abs(f) > 0.5) h.val += 2; break; }
    }
}

// ── Color functions (matching Rust terrain.rs) ──
function mixColor(a, b, t) { return [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]; }

function gradient(stops, t) {
    const n = stops.length - 1;
    const tt = Math.max(0, Math.min(n - 0.001, t * n));
    const i = Math.floor(tt);
    return mixColor(stops[i], stops[i + 1], tt - i);
}

function zoneStops(zone) {
    const all = {
        [ZONE_FOREST]: [[0.05,0.10,0.25],[0.08,0.30,0.18],[0.12,0.42,0.10],[0.20,0.48,0.14],[0.38,0.42,0.22],[0.65,0.60,0.48],[1.0,1.0,1.0]],
        [ZONE_PLAINS]: [[0.05,0.10,0.25],[0.18,0.45,0.18],[0.35,0.55,0.18],[0.50,0.52,0.22],[0.65,0.60,0.38],[0.82,0.78,0.62],[1.0,1.0,1.0]],
        [ZONE_DESERT]: [[0.05,0.10,0.25],[0.55,0.35,0.15],[0.72,0.50,0.20],[0.78,0.58,0.28],[0.72,0.48,0.22],[0.50,0.32,0.18],[1.0,1.0,1.0]],
        [ZONE_TUNDRA]: [[0.05,0.10,0.25],[0.18,0.22,0.18],[0.32,0.35,0.30],[0.50,0.55,0.50],[0.72,0.78,0.72],[0.88,0.92,0.98],[1.0,1.0,1.0]],
        [ZONE_JUNGLE]: [[0.05,0.10,0.25],[0.06,0.25,0.12],[0.06,0.40,0.08],[0.08,0.48,0.10],[0.25,0.42,0.14],[0.45,0.48,0.28],[1.0,1.0,1.0]],
        [ZONE_VOLCANIC]: [[0.05,0.10,0.25],[0.12,0.08,0.08],[0.22,0.12,0.08],[0.32,0.18,0.10],[0.48,0.22,0.08],[0.68,0.32,0.12],[1.0,1.0,1.0]],
        [ZONE_CRYSTAL]: [[0.05,0.10,0.25],[0.25,0.12,0.35],[0.35,0.20,0.50],[0.45,0.30,0.65],[0.52,0.48,0.78],[0.68,0.68,0.88],[1.0,1.0,1.0]],
        [ZONE_CAVE]: [[0.03,0.02,0.05],[0.10,0.08,0.06],[0.15,0.12,0.10],[0.20,0.16,0.13],[0.22,0.20,0.16],[0.28,0.25,0.22],[1.0,1.0,1.0]],
        [ZONE_LAVA]: [[0.05,0.10,0.25],[0.25,0.03,0.0],[0.45,0.08,0.0],[0.65,0.18,0.03],[0.85,0.35,0.08],[0.95,0.55,0.18],[1.0,1.0,1.0]],
        [ZONE_FUNGUS]: [[0.05,0.10,0.25],[0.18,0.03,0.18],[0.28,0.08,0.28],[0.22,0.32,0.12],[0.38,0.42,0.18],[0.52,0.52,0.28],[1.0,1.0,1.0]],
        [ZONE_ABYSS]: [[0.01,0.01,0.03],[0.02,0.02,0.06],[0.04,0.04,0.08],[0.05,0.05,0.10],[0.06,0.06,0.12],[0.08,0.08,0.15],[1.0,1.0,1.0]],
        [ZONE_STORM]: [[0.05,0.10,0.25],[0.12,0.15,0.20],[0.18,0.22,0.28],[0.28,0.32,0.38],[0.38,0.42,0.48],[0.52,0.52,0.58],[1.0,1.0,1.0]],
        [ZONE_AURORA]: [[0.05,0.10,0.25],[0.12,0.22,0.28],[0.12,0.38,0.32],[0.18,0.52,0.38],[0.38,0.58,0.48],[0.58,0.68,0.62],[1.0,1.0,1.0]],
        [ZONE_MAGMA]: [[0.05,0.10,0.25],[0.28,0.06,0.02],[0.42,0.12,0.03],[0.58,0.22,0.06],[0.78,0.38,0.10],[0.92,0.58,0.22],[1.0,1.0,1.0]],
        [ZONE_OCEAN]: [[0.01,0.02,0.08],[0.02,0.05,0.15],[0.05,0.10,0.22],[0.08,0.18,0.30],[0.12,0.25,0.38],[0.18,0.30,0.42],[0.25,0.35,0.48]],
        [ZONE_DEEP_OCEAN]: [[0.01,0.02,0.08],[0.02,0.05,0.15],[0.05,0.10,0.22],[0.08,0.18,0.30],[0.12,0.25,0.38],[0.18,0.30,0.42],[0.25,0.35,0.48]],
        [ZONE_CORAL_REEF]: [[0.02,0.05,0.12],[0.08,0.25,0.22],[0.20,0.45,0.30],[0.35,0.55,0.35],[0.50,0.60,0.40],[0.60,0.55,0.45],[0.70,0.60,0.55]],
        [ZONE_KELP_FOREST]: [[0.02,0.05,0.12],[0.05,0.20,0.15],[0.08,0.35,0.12],[0.12,0.40,0.15],[0.20,0.40,0.20],[0.30,0.45,0.25],[0.40,0.50,0.30]],
        [ZONE_SANDY_PLAIN]: [[0.02,0.05,0.12],[0.15,0.30,0.20],[0.30,0.42,0.22],[0.45,0.50,0.25],[0.55,0.55,0.30],[0.60,0.58,0.35],[0.65,0.60,0.40]],
        [ZONE_ROCKY_REEF]: [[0.02,0.05,0.12],[0.10,0.20,0.18],[0.18,0.30,0.22],[0.25,0.35,0.25],[0.30,0.38,0.28],[0.35,0.40,0.30],[0.40,0.42,0.35]],
    };
    return all[zone] || all[ZONE_FOREST];
}

function zoneRockColor(zone) {
    return ({
        [ZONE_DESERT]: [0.55,0.35,0.18], [ZONE_VOLCANIC]: [0.28,0.16,0.08], [ZONE_LAVA]: [0.28,0.16,0.08], [ZONE_MAGMA]: [0.28,0.16,0.08],
        [ZONE_CAVE]: [0.12,0.10,0.08], [ZONE_ABYSS]: [0.12,0.10,0.08],
        [ZONE_CRYSTAL]: [0.38,0.32,0.48], [ZONE_TUNDRA]: [0.30,0.32,0.35],
        [ZONE_JUNGLE]: [0.20,0.18,0.12], [ZONE_FOREST]: [0.25,0.22,0.15],
        [ZONE_FUNGUS]: [0.30,0.15,0.25],
    })[zone] || [0.32,0.28,0.22];
}

function getZoneTerrainColor(zone, h, maxH, slope, wx, wz) {
    const t = Math.max(0, Math.min(1, h / Math.max(0.1, maxH)));
    let color = gradient(zoneStops(zone), t);
    const rock = zoneRockColor(zone);
    color = mixColor(color, rock, Math.max(0, Math.min(0.6, 1 - slope)));
    const variation = (Math.sin(wx * 0.3) * Math.cos(wz * 0.5) * 0.5 + Math.sin(wx * 0.7 + wz * 0.4) * 0.3) * 0.04;
    return [
        Math.max(0, Math.min(1, color[0] + variation)),
        Math.max(0, Math.min(1, color[1] + variation * 0.8)),
        Math.max(0, Math.min(1, color[2] + variation * 0.6)),
    ];
}

function rgbToHsl(r, g, b) {
    const mx = Math.max(r, g, b), mn = Math.min(r, g, b);
    const l = (mx + mn) / 2;
    if (mx === mn) return [0, 0, l];
    const d = mx - mn;
    const s = l > 0.5 ? d / (2 - mx - mn) : d / (mx + mn);
    let h;
    if (mx === r) h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
    else if (mx === g) h = ((b - r) / d + 2) / 6;
    else h = ((r - g) / d + 4) / 6;
    return [h, s, l];
}

function hslToRgb(h, s, l) {
    if (s === 0) return [l, l, l];
    const h2rgb = (p, q, t) => {
        if (t < 0) t += 1; if (t > 1) t -= 1;
        if (t < 1/6) return p + (q - p) * 6 * t;
        if (t < 1/2) return q;
        if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
        return p;
    };
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    return [h2rgb(p, q, h + 1/3), h2rgb(p, q, h), h2rgb(p, q, h - 1/3)];
}

function applyHSL(r, g, b, hueShift, saturation, lightness) {
    const [h, s, l] = rgbToHsl(r, g, b);
    return hslToRgb((h + hueShift / 360) % 1, Math.max(0, Math.min(1, s * saturation)), Math.max(0, Math.min(1, l * lightness)));
}

function blockColor(blockType, params, wx, wy, wz, zone, surfaceH, maxH) {
    const depth = surfaceH - wy;
    const darken = Math.max(0.25, 1 - Math.max(0, Math.min(0.75, depth / 20)));
    let base;
    switch (blockType) {
        case BLK_GRASS: return getZoneTerrainColor(zone, surfaceH, maxH, 0.8, wx, wz);
        case BLK_DIRT: base = [0.55,0.35,0.18]; break;
        case BLK_SAND: base = [0.85,0.75,0.5]; break;
        case BLK_SNOW: base = [0.9,0.92,0.95]; break;
        case BLK_STONE: base = zoneRockColor(zone); break;
        case BLK_COAL_ORE: base = [0.15,0.15,0.15]; break;
        case BLK_IRON_ORE: base = [0.7,0.6,0.5]; break;
        case BLK_GOLD_ORE: base = [0.9,0.75,0.2]; break;
        case BLK_DIAMOND_ORE: base = [0.4,0.7,0.9]; break;
        case BLK_GRAVEL: base = [0.5,0.45,0.4]; break;
        case BLK_CLAY: base = [0.6,0.55,0.5]; break;
        case BLK_WATER: return [0.2,0.4,0.8];
        case BLK_LAVA: return [1.0,0.3,0.05];
        case BLK_PACKED_ICE: base = [0.6,0.7,0.85]; break;
        case BLK_OBSIDIAN: base = [0.08,0.06,0.12]; break;
        case BLK_MOSS: base = [0.35,0.45,0.2]; break;
        case BLK_GLOW_SHROOM: base = [0.5,0.7,0.3]; break;
        case BLK_MAGMA_BLOCK: base = [0.7,0.2,0.05]; break;
        case BLK_SOUL_SAND: base = [0.25,0.2,0.15]; break;
        case BLK_BASALT: base = [0.18,0.18,0.2]; break;
        default: base = zoneRockColor(zone);
    }
    if (blockType === BLK_LAVA || blockType === BLK_GLOW_SHROOM || blockType === BLK_DIAMOND_ORE || blockType === BLK_GOLD_ORE) {
        const glow = blockType === BLK_LAVA ? 1 : blockType === BLK_GLOW_SHROOM ? 0.7 : 0.3;
        return [Math.max(base[0], glow), Math.max(base[1], glow), Math.max(base[2], glow)];
    }
    if (depth > 0.5) {
        const rock = zoneRockColor(zone);
        const rockT = Math.max(0, Math.min(1, (depth - 0.5) / 4));
        return [
            Math.max(0, Math.min(1, (base[0] * (1 - rockT) + rock[0] * rockT) * darken)),
            Math.max(0, Math.min(1, (base[1] * (1 - rockT) + rock[1] * rockT) * darken)),
            Math.max(0, Math.min(1, (base[2] * (1 - rockT) + rock[2] * rockT) * darken)),
        ];
    }
    return base;
}

function getSurfaceBlockType(zone) {
    if (zone === ZONE_DESERT || zone === ZONE_SANDY_PLAIN) return BLK_SAND;
    if (zone === ZONE_TUNDRA) return BLK_SNOW;
    return BLK_GRASS;
}

// ── Chunk computation ──
const CHUNK_SIZE = 24;
const BLOCK_RES = 24;
const BLOCK_SIZE = CHUNK_SIZE / BLOCK_RES;

function computeChunkDataLOD(params, cx, cz, lod) {
    const p = applyMutation(params, cx, cz);
    const ox = cx * CHUNK_SIZE, oz = cz * CHUNK_SIZE;
    const n = BLOCK_RES;
    const step = BLOCK_SIZE;
    const sampleStep = lod === 0 ? 1 : (lod === 1 ? 2 : 4);

    const heights = new Float64Array(n * n);
    let maxH = 0;
    for (let iz = 0; iz < n; iz++) {
        for (let ix = 0; ix < n; ix++) {
            const wx = ox + ix * step + step * 0.5, wz = oz + iz * step + step * 0.5;
            const h = getHeight(p, wx, wz);
            heights[iz * n + ix] = h;
            if (h > maxH) maxH = h;
        }
    }

    const zones = new Int8Array(n * n);
    for (let iz = 0; iz < n; iz++) {
        for (let ix = 0; ix < n; ix++) {
            zones[iz * n + ix] = getZone(p, ox + ix * step + step * 0.5, oz + iz * step + step * 0.5);
        }
    }

    const positions = [], normals = [], colors = [], uvs = [], indices = [];

    const cn = n + 1;
    const cornerHeights = new Float64Array(cn * cn);
    for (let iz = 0; iz <= n; iz++) {
        for (let ix = 0; ix <= n; ix++) {
            cornerHeights[iz * cn + ix] = getHeight(p, ox + ix * step, oz + iz * step);
        }
    }

    // First pass: accumulate normals at each corner
    const cornerNormals = new Float64Array(cn * cn * 3);
    for (let iz = 0; iz < n; iz += sampleStep) {
        for (let ix = 0; ix < n; ix += sampleStep) {
            const h00 = cornerHeights[iz * cn + ix];
            const h10 = cornerHeights[iz * cn + (ix + sampleStep)];
            const h01 = cornerHeights[(iz + sampleStep) * cn + ix];
            const h11 = cornerHeights[(iz + sampleStep) * cn + (ix + sampleStep)];

            const x0 = ox + ix * step, z0 = oz + iz * step;
            const x1 = ox + (ix + sampleStep) * step, z1 = oz + (iz + sampleStep) * step;

            // Tri 1: p0-p1-p2
            const p0 = [x0, h00, z0], p1 = [x1, h11, z1], p2 = [x1, h10, z0];
            const e1x = p1[0]-p0[0], e1y = p1[1]-p0[1], e1z = p1[2]-p0[2];
            const e2x = p2[0]-p0[0], e2y = p2[1]-p0[1], e2z = p2[2]-p0[2];
            const nx = e1y*e2z - e1z*e2y, ny = e1z*e2x - e1x*e2z, nz = e1x*e2y - e1y*e2x;

            const c00 = (iz * cn + ix) * 3, c10 = (iz * cn + (ix + sampleStep)) * 3;
            const c01 = ((iz + sampleStep) * cn + ix) * 3, c11 = ((iz + sampleStep) * cn + (ix + sampleStep)) * 3;

            cornerNormals[c00] += nx; cornerNormals[c00+1] += ny; cornerNormals[c00+2] += nz;
            cornerNormals[c11] += nx; cornerNormals[c11+1] += ny; cornerNormals[c11+2] += nz;
            cornerNormals[c10] += nx; cornerNormals[c10+1] += ny; cornerNormals[c10+2] += nz;

            // Tri 2: p0-p3-p1
            const p3 = [x0, h01, z1];
            const e3x = p3[0]-p0[0], e3y = p3[1]-p0[1], e3z = p3[2]-p0[2];
            const e4x = p1[0]-p0[0], e4y = p1[1]-p0[1], e4z = p1[2]-p0[2];
            const nx2 = e3y*e4z - e3z*e4y, ny2 = e3z*e4x - e3x*e4z, nz2 = e3x*e4y - e3y*e4x;

            cornerNormals[c00] += nx2; cornerNormals[c00+1] += ny2; cornerNormals[c00+2] += nz2;
            cornerNormals[c01] += nx2; cornerNormals[c01+1] += ny2; cornerNormals[c01+2] += nz2;
            cornerNormals[c11] += nx2; cornerNormals[c11+1] += ny2; cornerNormals[c11+2] += nz2;
        }
    }

    // Normalize corner normals
    for (let i = 0; i < cn * cn; i++) {
        const i3 = i * 3;
        const len = Math.sqrt(cornerNormals[i3]*cornerNormals[i3] + cornerNormals[i3+1]*cornerNormals[i3+1] + cornerNormals[i3+2]*cornerNormals[i3+2]) || 1;
        cornerNormals[i3] /= len; cornerNormals[i3+1] /= len; cornerNormals[i3+2] /= len;
    }

    // Second pass: output vertices with smoothed normals
    for (let iz = 0; iz < n; iz += sampleStep) {
        for (let ix = 0; ix < n; ix += sampleStep) {
            const wxCenter = ox + ix * step + step * 0.5, wzCenter = oz + iz * step + step * 0.5;
            const zone = zones[iz * n + ix];
            const surfaceH = heights[iz * n + ix];

            const h00 = cornerHeights[iz * cn + ix];
            const h10 = cornerHeights[iz * cn + (ix + sampleStep)];
            const h01 = cornerHeights[(iz + sampleStep) * cn + ix];
            const h11 = cornerHeights[(iz + sampleStep) * cn + (ix + sampleStep)];

            const dzdx = ((h10 - h00) + (h11 - h01)) / (2 * step * sampleStep);
            const dzdy = ((h01 - h00) + (h11 - h10)) / (2 * step * sampleStep);
            const slope = Math.sqrt(dzdx * dzdx + dzdy * dzdy);

            let effectiveBt;
            if (surfaceH > maxH * 0.85 && zone !== ZONE_DESERT && zone !== ZONE_TUNDRA) effectiveBt = BLK_SNOW;
            else if (slope > 0.6) effectiveBt = BLK_STONE;
            else if (slope > 0.3 && zone !== ZONE_DESERT && zone !== ZONE_SANDY_PLAIN) effectiveBt = BLK_DIRT;
            else effectiveBt = getSurfaceBlockType(zone);

            let c = effectiveBt === BLK_GRASS
                ? getZoneTerrainColor(zone, surfaceH, maxH, slope, wxCenter, wzCenter)
                : blockColor(effectiveBt, p, wxCenter, surfaceH, wzCenter, zone, surfaceH, maxH);
            c = applyHSL(c[0], c[1], c[2], p.hue_shift || 0, p.saturation || 1, p.lightness || 1);

            const x0 = ox + ix * step, z0 = oz + iz * step;
            const x1 = ox + (ix + sampleStep) * step, z1 = oz + (iz + sampleStep) * step;
            const p0 = [x0, h00, z0], p1 = [x1, h11, z1], p2 = [x1, h10, z0], p3 = [x0, h01, z1];

            const n00 = iz * cn + ix, n10 = iz * cn + (ix + sampleStep);
            const n01 = (iz + sampleStep) * cn + ix, n11 = (iz + sampleStep) * cn + (ix + sampleStep);

            const nv = positions.length / 3;

            for (const v of [p0, p1, p2]) {
                positions.push(v[0], v[1], v[2]);
                normals.push(cornerNormals[n00*3], cornerNormals[n00*3+1], cornerNormals[n00*3+2]);
            }
            for (const v of [p1, p2]) {
                normals.push(cornerNormals[n11*3], cornerNormals[n11*3+1], cornerNormals[n11*3+2]);
                normals.push(cornerNormals[n10*3], cornerNormals[n10*3+1], cornerNormals[n10*3+2]);
            }
            colors.push(c[0], c[1], c[2], c[0], c[1], c[2], c[0], c[1], c[2]);
            uvs.push(0, 0, 1, 1, 1, 0);
            indices.push(nv, nv + 1, nv + 2);

            const nv2 = positions.length / 3;

            for (const v of [p0, p3, p1]) {
                positions.push(v[0], v[1], v[2]);
            }
            normals.push(cornerNormals[n00*3], cornerNormals[n00*3+1], cornerNormals[n00*3+2]);
            normals.push(cornerNormals[n01*3], cornerNormals[n01*3+1], cornerNormals[n01*3+2]);
            normals.push(cornerNormals[n11*3], cornerNormals[n11*3+1], cornerNormals[n11*3+2]);
            colors.push(c[0], c[1], c[2], c[0], c[1], c[2], c[0], c[1], c[2]);
            uvs.push(0, 0, 0, 1, 1, 1);
            indices.push(nv2, nv2 + 1, nv2 + 2);
        }
    }

    return {
        positions: new Float32Array(positions),
        normals: new Float32Array(normals),
        indices: new Uint32Array(indices),
        colors: new Float32Array(colors),
        uvs: new Float32Array(uvs),
    };
}

// ── Message handler ──
self.onmessage = function(e) {
    const msg = e.data;
    switch (msg.type) {
        case 'set_seed':
            setNoiseSeed(msg.seed);
            break;
        case 'gen_chunk':
            setNoiseSeed(msg.params.seed);
            const result = computeChunkDataLOD(msg.params, msg.cx, msg.cz, msg.lod || 0);
            self.postMessage({
                type: 'chunk_data', id: msg.id, cx: msg.cx, cz: msg.cz,
                key: 'chunk_' + msg.cx + ',' + msg.cz,
                positions: result.positions.buffer,
                normals: result.normals.buffer,
                indices: result.indices.buffer,
                colors: result.colors.buffer,
                uvs: result.uvs.buffer,
            }, [result.positions.buffer, result.normals.buffer, result.indices.buffer, result.colors.buffer, result.uvs.buffer]);
            break;
    }
};
