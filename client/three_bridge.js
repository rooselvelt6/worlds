import * as THREE from 'three';
import { EffectComposer } from 'three/addons/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/addons/postprocessing/RenderPass.js';
import { UnrealBloomPass } from 'three/addons/postprocessing/UnrealBloomPass.js';
import { SSRPass } from 'three/addons/postprocessing/SSRPass.js';
import { SSAOPass } from 'three/addons/postprocessing/SSAOPass.js';
import { ShaderPass } from 'three/addons/postprocessing/ShaderPass.js';
import { CSM } from 'three/addons/csm/CSM.js';

let scene = null;
let camera = null;
let renderer = null;
let composer = null;
const meshes = new Map();
const detailMeshes = new Map();
let textureAtlas = null;
let detailTexture = null;

// ── R2 Ocean / SSR / Underwater ──
let ssrPass = null;
let ssaoPass = null;
let finalPass = null;
let waterMesh = null;
let underwaterActive = false;
const UNDERWATER_COLOR = new THREE.Color(0x006688);

// ── R4 Cinematographic Post-Processing (LUT, vignette, grain, exposure, flare) ──
let lutCanvas = null;
let lutCtx = null;
let lutTexture = null;
let currentLutBiome = 0;
let exposureCurrent = 1.0;
let exposureTarget = 1.0;
let lensFlareIntensity = 0.0;

// ── R6 Cascaded Shadow Maps ──
let csm = null;

// ── R5 Vegetation Wind + Grass Instancing ──
let vegWindUniforms = [];
let grassMeshes = new Map();

// Biome IDs matching Rust Zone enum index
const BIOME = {
    FOREST: 0, PLAINS: 1, DESERT: 2, TUNDRA: 3, JUNGLE: 4,
    VOLCANIC: 5, OCEAN: 6, CRYSTAL: 7, CAVE: 8, LAVA: 9,
    FUNGUS: 10, ABYSS: 11, STORM: 12, AURORA: 13, MAGMA: 14,
    CORAL_REEF: 15, KELP_FOREST: 16, SANDY_PLAIN: 17, ROCKY_REEF: 18, DEEP_OCEAN: 19,
};

function biomeCategory(id) {
    if (id === BIOME.DESERT || id === BIOME.VOLCANIC) return 'warm';
    if (id === BIOME.TUNDRA || id === BIOME.AURORA) return 'cool';
    if (id === BIOME.CAVE || id === BIOME.ABYSS || id === BIOME.LAVA || id === BIOME.MAGMA || id === BIOME.FUNGUS) return 'cave';
    if (id === BIOME.OCEAN || id === BIOME.CORAL_REEF || id === BIOME.KELP_FOREST || id === BIOME.SANDY_PLAIN || id === BIOME.ROCKY_REEF || id === BIOME.DEEP_OCEAN) return 'aquatic';
    if (id === BIOME.CRYSTAL || id === BIOME.STORM) return 'special';
    return 'neutral'; // Forest, Plains, Jungle
}

function generateIdentityLUT() {
    const size = 32;
    const width = size * size;
    const height = size;
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d');
    const imgData = ctx.createImageData(width, height);
    const data = imgData.data;
    for (let b = 0; b < size; b++) {
        for (let g = 0; g < size; g++) {
            for (let r = 0; r < size; r++) {
                const px = b * size + r;
                const py = g;
                const i = (py * width + px) * 4;
                data[i] = (r / (size - 1)) * 255;
                data[i+1] = (g / (size - 1)) * 255;
                data[i+2] = (b / (size - 1)) * 255;
                data[i+3] = 255;
            }
        }
    }
    ctx.putImageData(imgData, 0, 0);
    const tex = new THREE.CanvasTexture(canvas);
    tex.wrapS = tex.wrapT = THREE.ClampToEdgeWrapping;
    tex.magFilter = THREE.LinearFilter;
    tex.minFilter = THREE.LinearFilter;
    tex.needsUpdate = true;
    lutCanvas = canvas;
    lutCtx = ctx;
    lutTexture = tex;
    return { canvas, ctx, texture: tex };
}

function applyBiomeLUT(ctx, canvas, biome) {
    const imgData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    const data = imgData.data;
    const cat = biomeCategory(biome);
    let contrast = 0, sat = 1, temp = 0, bright = 0;
    switch (cat) {
        case 'warm':   contrast = 0.12; sat = 1.10; temp = 0.25; bright = 0.03; break;
        case 'cool':   contrast = 0.08; sat = 0.85; temp = -0.25; bright = 0.05; break;
        case 'cave':   contrast = 0.20; sat = 0.55; temp = -0.05; bright = -0.12; break;
        case 'aquatic': contrast = 0.05; sat = 1.20; temp = -0.10; bright = 0.02; break;
        case 'special': contrast = 0.15; sat = 1.30; temp = 0.10; bright = 0.05; break;
        default:       contrast = 0.05; sat = 1.00; temp = 0.00; bright = 0.00; break;
    }
    for (let i = 0; i < data.length; i += 4) {
        let r = data[i] / 255, g = data[i+1] / 255, b = data[i+2] / 255;
        r = (r - 0.5) * (1 + contrast) + 0.5;
        g = (g - 0.5) * (1 + contrast) + 0.5;
        b = (b - 0.5) * (1 + contrast) + 0.5;
        const lum = r * 0.299 + g * 0.587 + b * 0.114;
        r = lum + (r - lum) * sat;
        g = lum + (g - lum) * sat;
        b = lum + (b - lum) * sat;
        r += temp * 0.05 + bright;
        g += bright;
        b -= temp * 0.05 + bright * 0.5;
        data[i] = Math.max(0, Math.min(255, r * 255));
        data[i+1] = Math.max(0, Math.min(255, g * 255));
        data[i+2] = Math.max(0, Math.min(255, b * 255));
    }
    ctx.putImageData(imgData, 0, 0);
}

function generateTextureAtlas() {
    const cols = 6, rows = 4, tileSize = 128;
    const canvas = document.createElement('canvas');
    canvas.width = cols * tileSize;
    canvas.height = rows * tileSize;
    const ctx = canvas.getContext('2d');

    function tile(u, v) { return { x: u * tileSize, y: v * tileSize, w: tileSize, h: tileSize }; }

    function fillRect(t, r, g, b) {
        ctx.fillStyle = `rgb(${r*255|0},${g*255|0},${b*255|0})`;
        ctx.fillRect(t.x, t.y, t.w, t.h);
    }

    function hash2(x, y) {
        return (Math.sin(x*127.1 + y*311.7)*43758.5453) & 0xffff;
    }

    function fbm(x, y, octaves) {
        let v = 0, a = 1, t = 0;
        for (let i = 0; i < octaves; i++) {
            const h = (Math.sin(x*12.9898 + y*78.233)*43758.5453);
            v += (h - Math.floor(h)) * a;
            t += a; x *= 2.1; y *= 2.1; a *= 0.5;
        }
        return v / t;
    }

    function addNoiseDetail(t, r, g, b, strength) {
        const img = ctx.getImageData(t.x, t.y, t.w, t.h);
        for (let y = 0; y < t.h; y++) {
            for (let x = 0; x < t.w; x++) {
                const i = (y * t.w + x) * 4;
                const nx = (t.x + x) * 0.05, ny = (t.y + y) * 0.05;
                const n = fbm(nx, ny, 4) * 2 - 1;
                img.data[i] = Math.max(0, Math.min(255, img.data[i] + n * strength * 255));
                img.data[i+1] = Math.max(0, Math.min(255, img.data[i+1] + n * strength * 255));
                img.data[i+2] = Math.max(0, Math.min(255, img.data[i+2] + n * strength * 255));
            }
        }
        ctx.putImageData(img, t.x, t.y);
    }

    function addDetailNoise(t, strength) {
        const img = ctx.getImageData(t.x, t.y, t.w, t.h);
        for (let y = 0; y < t.h; y++) {
            for (let x = 0; x < t.w; x++) {
                const i = (y * t.w + x) * 4;
                const n = (Math.sin((t.x+x)*(t.y+y)*0.001 + 1.0)*43758.5453 -
                           Math.floor(Math.sin((t.x+x)*(t.y+y)*0.001 + 1.0)*43758.5453)) * 2 - 1;
                img.data[i] = Math.max(0, Math.min(255, img.data[i] + n * strength * 255));
                img.data[i+1] = Math.max(0, Math.min(255, img.data[i+1] + n * strength * 255));
                img.data[i+2] = Math.max(0, Math.min(255, img.data[i+2] + n * strength * 255));
            }
        }
        ctx.putImageData(img, t.x, t.y);
    }

    function addStripes(t, r, g, b, dx, dy, spacing, width) {
        const img = ctx.getImageData(t.x, t.y, t.w, t.h);
        for (let y = 0; y < t.h; y++) {
            for (let x = 0; x < t.w; x++) {
                const v = (x*dx + y*dy) % spacing;
                if (v < spacing * width) {
                    const i = (y * t.w + x) * 4;
                    const f = 1 - v / (spacing * width);
                    img.data[i] = Math.max(0, Math.min(255, img.data[i] + r * f * 80));
                    img.data[i+1] = Math.max(0, Math.min(255, img.data[i+1] + g * f * 80));
                    img.data[i+2] = Math.max(0, Math.min(255, img.data[i+2] + b * f * 80));
                }
            }
        }
        ctx.putImageData(img, t.x, t.y);
    }

    // Tile 0: Grass - green with fine detail
    let t = tile(0,0); fillRect(t, 0.22, 0.5, 0.12); addNoiseDetail(t, 0.1, 0.3, 0.05, 0.2); addDetailNoise(t, 0.05);
    // Tile 1: Dirt
    t = tile(1,0); fillRect(t, 0.42, 0.28, 0.13); addNoiseDetail(t, 0, 0, 0, 0.15); addDetailNoise(t, 0.08);
    // Tile 2: Stone - gray with cracks
    t = tile(2,0); fillRect(t, 0.48, 0.45, 0.42); addNoiseDetail(t, 0, 0, 0, 0.1); addStripes(t, 0.6, 0.58, 0.55, 1, 2, 10, 0.25); addDetailNoise(t, 0.04);
    // Tile 3: Sand
    t = tile(3,0); fillRect(t, 0.75, 0.68, 0.42); addNoiseDetail(t, 0, 0, 0, 0.08); addDetailNoise(t, 0.12);
    // Tile 4: Snow
    t = tile(4,0); fillRect(t, 0.88, 0.9, 0.93); addNoiseDetail(t, 0, 0, 0, 0.05); addDetailNoise(t, 0.03);
    // Tile 5: Gravel
    t = tile(5,0); fillRect(t, 0.48, 0.43, 0.38); addNoiseDetail(t, 0, 0, 0, 0.2); addDetailNoise(t, 0.15);
    // Tile 6: Clay
    t = tile(0,1); fillRect(t, 0.52, 0.48, 0.4); addNoiseDetail(t, 0, 0, 0, 0.08); addDetailNoise(t, 0.06);
    // Tile 7: Coal Ore
    t = tile(1,1); fillRect(t, 0.25, 0.25, 0.25); addNoiseDetail(t, 0, 0, 0, 0.15); addDetailNoise(t, 0.18);
    // Tile 8: Iron Ore
    t = tile(2,1); fillRect(t, 0.55, 0.5, 0.45); addNoiseDetail(t, 0, 0, 0, 0.1); addStripes(t, 0.8, 0.6, 0.4, 3, 1, 7, 0.3); addDetailNoise(t, 0.05);
    // Tile 9: Gold Ore
    t = tile(3,1); fillRect(t, 0.65, 0.55, 0.3); addNoiseDetail(t, 0, 0, 0, 0.08); addStripes(t, 1.0, 0.8, 0.2, 2, 3, 6, 0.25); addDetailNoise(t, 0.04);
    // Tile 10: Diamond Ore
    t = tile(4,1); fillRect(t, 0.35, 0.55, 0.65); addNoiseDetail(t, 0.1, 0, 0, 0.08); addStripes(t, 0.6, 0.9, 1.0, 1, 1, 5, 0.3); addDetailNoise(t, 0.04);
    // Tile 11: Lava
    t = tile(5,1); fillRect(t, 0.75, 0.18, 0.04); addNoiseDetail(t, 0.2, 0, 0, 0.15); addDetailNoise(t, 0.1);
    // Tile 12: Packed Ice
    t = tile(0,2); fillRect(t, 0.5, 0.6, 0.78); addNoiseDetail(t, 0, 0, 0, 0.05); addDetailNoise(t, 0.03);
    // Tile 13: Obsidian
    t = tile(1,2); fillRect(t, 0.06, 0.05, 0.1); addNoiseDetail(t, 0, 0, 0, 0.12); addDetailNoise(t, 0.08);
    // Tile 14: Moss
    t = tile(2,2); fillRect(t, 0.22, 0.38, 0.18); addNoiseDetail(t, 0.1, 0.2, 0.05, 0.15); addDetailNoise(t, 0.08);
    // Tile 15: Glow Shroom
    t = tile(3,2); fillRect(t, 0.35, 0.55, 0.22); addNoiseDetail(t, 0.2, 0.3, 0.1, 0.12); addStripes(t, 0.6, 0.9, 0.4, 1, 2, 6, 0.2); addDetailNoise(t, 0.05);
    // Tile 16: Magma
    t = tile(4,2); fillRect(t, 0.55, 0.12, 0.04); addNoiseDetail(t, 0.3, 0, 0, 0.15); addDetailNoise(t, 0.1);
    // Tile 17: Soul Sand
    t = tile(5,2); fillRect(t, 0.22, 0.18, 0.13); addNoiseDetail(t, 0, 0, 0, 0.1); addDetailNoise(t, 0.12);
    // Tile 18: Basalt
    t = tile(0,3); fillRect(t, 0.15, 0.15, 0.18); addNoiseDetail(t, 0, 0, 0, 0.08); addStripes(t, 0.25, 0.25, 0.28, 1, 1, 4, 0.2); addDetailNoise(t, 0.05);

    const tex = new THREE.CanvasTexture(canvas);
    tex.wrapS = tex.wrapT = THREE.ClampToEdgeWrapping;
    tex.magFilter = THREE.LinearFilter;
    tex.minFilter = THREE.LinearMipmapLinearFilter;
    tex.generateMipmaps = true;
    tex.needsUpdate = true;
    textureAtlas = tex;

    // Normal map from height
    const nCanvas = document.createElement('canvas');
    nCanvas.width = canvas.width;
    nCanvas.height = canvas.height;
    const nCtx = nCanvas.getContext('2d');
    nCtx.drawImage(canvas, 0, 0);
    const srcData = nCtx.getImageData(0, 0, nCanvas.width, nCanvas.height);
    const nData = new Uint8Array(nCanvas.width * nCanvas.height * 4);
    for (let y = 1; y < nCanvas.height-1; y++) {
        for (let x = 1; x < nCanvas.width-1; x++) {
            const i = (y*nCanvas.width + x)*4;
            const l = (y*nCanvas.width + (x-1))*4;
            const r = (y*nCanvas.width + (x+1))*4;
            const d = ((y+1)*nCanvas.width + x)*4;
            const u = ((y-1)*nCanvas.width + x)*4;
            const hl = srcData.data[l]*0.3 + srcData.data[l+1]*0.59 + srcData.data[l+2]*0.11;
            const hr = srcData.data[r]*0.3 + srcData.data[r+1]*0.59 + srcData.data[r+2]*0.11;
            const hd = srcData.data[d]*0.3 + srcData.data[d+1]*0.59 + srcData.data[d+2]*0.11;
            const hu = srcData.data[u]*0.3 + srcData.data[u+1]*0.59 + srcData.data[u+2]*0.11;
            const dx2 = (hl - hr) / 255;
            const dy2 = (hd - hu) / 255;
            nData[i] = Math.round((dx2*0.5+0.5)*255);
            nData[i+1] = Math.round((dy2*0.5+0.5)*255);
            nData[i+2] = 255;
            nData[i+3] = 255;
        }
    }
    const nTex = new THREE.DataTexture(nData, nCanvas.width, nCanvas.height, THREE.RGBAFormat);
    nTex.wrapS = nTex.wrapT = THREE.ClampToEdgeWrapping;
    nTex.magFilter = THREE.LinearFilter;
    nTex.minFilter = THREE.LinearMipmapLinearFilter;
    nTex.generateMipmaps = true;
    nTex.needsUpdate = true;
    textureAtlas.userData = textureAtlas.userData || {};
    textureAtlas.userData.normalMap = nTex;

    // ── Detail texture (tileable noise for micro-detail overlay) ──
    const dSize = 64;
    const dCanvas = document.createElement('canvas');
    dCanvas.width = dSize;
    dCanvas.height = dSize;
    const dCtx = dCanvas.getContext('2d');
    const dImg = dCtx.createImageData(dSize, dSize);
    for (let y = 0; y < dSize; y++) {
        for (let x = 0; x < dSize; x++) {
            const i = (y * dSize + x) * 4;
            const nx = x / dSize, ny = y / dSize;
            let v = 0, a = 1;
            for (let o = 0; o < 4; o++) {
                const f = Math.pow(2, o);
                const px = nx * f + 0.5, py = ny * f + 1.3;
                const n = Math.sin(px * 12.9898 + py * 78.233) * 43758.5453;
                v += (n - Math.floor(n)) * a;
                a *= 0.5;
            }
            const val = Math.min(1, v);
            dImg.data[i] = val * 255;
            dImg.data[i+1] = val * 255;
            dImg.data[i+2] = val * 255;
            dImg.data[i+3] = val * 255; // alpha = height for parallax
        }
    }
    dCtx.putImageData(dImg, 0, 0);
    detailTexture = new THREE.CanvasTexture(dCanvas);
    detailTexture.wrapS = detailTexture.wrapT = THREE.RepeatWrapping;
    detailTexture.magFilter = THREE.LinearFilter;
    detailTexture.minFilter = THREE.LinearMipmapLinearFilter;
    detailTexture.generateMipmaps = true;
    detailTexture.needsUpdate = true;
}

const DETAIL_VERTEX_SHADER = `
varying vec2 vDetailUv;
varying vec3 vWorldNormal;
varying vec3 vWorldPos;

void main() {
    vec4 wp = modelMatrix * vec4(position, 1.0);
    vWorldPos = wp.xyz;
    vWorldNormal = normalize(mat3(modelMatrix) * normal);
    vDetailUv = position.xz;
    gl_Position = projectionMatrix * viewMatrix * wp;
}
`;

const DETAIL_FRAGMENT_SHADER = `
uniform sampler2D uDetailTex;
uniform float uTileFactor;
uniform float uHeightScale;
uniform vec3 uCameraPos;

varying vec2 vDetailUv;
varying vec3 vWorldNormal;
varying vec3 vWorldPos;

void main() {
    vec2 uv = vDetailUv * uTileFactor;
    vec3 viewDir = normalize(uCameraPos - vWorldPos);

    float height = texture2D(uDetailTex, uv).r;
    vec2 parallaxOffset = normalize(viewDir).xz * (height - 0.5) * uHeightScale;
    uv += parallaxOffset;

    vec4 detail = texture2D(uDetailTex, uv);

    float slope = 1.0 - abs(vWorldNormal.y);
    float alpha = 0.12 + slope * 0.38;
    alpha = clamp(alpha, 0.0, 0.5);

    gl_FragColor = vec4(detail.rgb, alpha);
}
`;

let detailMaterial = null;

function ensureDetailMaterial() {
    if (detailMaterial) return;
    detailMaterial = new THREE.ShaderMaterial({
        uniforms: {
            uDetailTex: { value: detailTexture },
            uTileFactor: { value: 32.0 },
            uHeightScale: { value: 0.03 },
            uCameraPos: { value: new THREE.Vector3() },
        },
        vertexShader: DETAIL_VERTEX_SHADER,
        fragmentShader: DETAIL_FRAGMENT_SHADER,
        transparent: true,
        depthWrite: false,
        blending: THREE.MultiplyBlending,
        side: THREE.FrontSide,
    });
}

function createDetailOverlay(key, geometry) {
    if (!detailTexture || detailMeshes.has(key)) return;
    ensureDetailMaterial();
    const mat = detailMaterial.clone();
    const mesh = new THREE.Mesh(geometry, mat);
    mesh.renderOrder = 1;
    mesh.frustumCulled = true;
    scene.add(mesh);
    detailMeshes.set(key, mesh);
}

window.threeBridgeInit = function (canvas) {
    if (!textureAtlas) generateTextureAtlas();
    scene = new THREE.Scene();
    scene.background = new THREE.Color(0x1a1a2e);

    const w = canvas.clientWidth || 800;
    const h = canvas.clientHeight || 600;
    camera = new THREE.PerspectiveCamera(60, w / h, 0.1, 500);

    renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: "high-performance" });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(w, h, false);
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 1.0;

    composer = new EffectComposer(renderer);
    composer.addPass(new RenderPass(scene, camera));

    // SSR Pass
    try {
        ssrPass = new SSRPass({
            renderer,
            scene,
            camera,
            width: w,
            height: h,
            selects: [],
        });
        ssrPass.opacity = 0.6;
        ssrPass.maxDistance = 120;
        ssrPass.thickness = 0.02;
        composer.addPass(ssrPass);
    } catch (e) {
        console.warn('SSRPass not available:', e.message);
        ssrPass = null;
    }

    // SSAO Pass
    try {
        ssaoPass = new SSAOPass(scene, camera, Math.floor(w / 2), Math.floor(h / 2), 32);
        ssaoPass.kernelRadius = 4;
        ssaoPass.minDistance = 0.005;
        ssaoPass.maxDistance = 0.08;
        ssaoPass.output = SSAOPass.OUTPUT.Default;
        composer.addPass(ssaoPass);
    } catch (e) {
        console.warn('SSAOPass not available:', e.message);
        ssaoPass = null;
    }

    const bloom = new UnrealBloomPass(new THREE.Vector2(w, h), 0.3, 0.2, 0.1);
    composer.addPass(bloom);

    // R4: Combined final pass (LUT color grading + vignette + film grain + auto exposure + lens flare + underwater)
    generateIdentityLUT();
    applyBiomeLUT(lutCtx, lutCanvas, 0);
    lutTexture.needsUpdate = true;

    const finalMaterial = new THREE.ShaderMaterial({
        uniforms: {
            tDiffuse: { value: null },
            uLutTex: { value: lutTexture },
            uLutSize: { value: 32.0 },
            uTime: { value: 0 },
            uVignetteIntensity: { value: 0.25 },
            uFilmGrainIntensity: { value: 0.04 },
            uExposure: { value: 1.0 },
            uSunUV: { value: new THREE.Vector2(-1, -1) },
            uLensFlareIntensity: { value: 0.0 },
            uUnderwaterIntensity: { value: 0.0 },
            uUnderwaterColor: { value: new THREE.Color(0x006688) },
        },
        vertexShader: `
            varying vec2 vUv;
            void main() {
                vUv = uv;
                gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
            }
        `,
        fragmentShader: `
            uniform sampler2D tDiffuse;
            uniform sampler2D uLutTex;
            uniform float uLutSize;
            uniform float uTime;
            uniform float uVignetteIntensity;
            uniform float uFilmGrainIntensity;
            uniform float uExposure;
            uniform vec2 uSunUV;
            uniform float uLensFlareIntensity;
            uniform float uUnderwaterIntensity;
            uniform vec3 uUnderwaterColor;
            varying vec2 vUv;

            vec3 lutLookup(vec3 color, sampler2D lut, float size) {
                float bIdx = color.b * (size - 1.0);
                float b0 = floor(bIdx);
                float b1 = min(b0 + 1.0, size - 1.0);
                float t = bIdx - b0;
                float rPos = color.r * (size - 1.0);
                float gPos = color.g * (size - 1.0);
                float tileW = size;
                float texW = size * size;
                float texH = size;
                vec2 uv0 = vec2((b0 * tileW + rPos + 0.5) / texW, (gPos + 0.5) / texH);
                vec2 uv1 = vec2((b1 * tileW + rPos + 0.5) / texW, (gPos + 0.5) / texH);
                vec3 c0 = texture2D(lut, uv0).rgb;
                vec3 c1 = texture2D(lut, uv1).rgb;
                return mix(c0, c1, t);
            }

            void main() {
                vec2 uv = vUv;

                // 1. Underwater distortion
                float uw = uUnderwaterIntensity;
                if (uw > 0.01) {
                    uv.x += sin(uv.y * 30.0 + uTime * 2.0) * 0.003 * uw;
                    uv.y += cos(uv.x * 30.0 + uTime * 1.8) * 0.003 * uw;
                }

                vec4 color = texture2D(tDiffuse, uv);

                // 2. LUT color grading
                color.rgb = lutLookup(color.rgb, uLutTex, uLutSize);

                // 3. Auto exposure
                float avgLum = dot(color.rgb, vec3(0.299, 0.587, 0.114));
                color.rgb *= uExposure / max(avgLum, 0.01);

                // 4. Underwater tint + bubbles
                if (uw > 0.01) {
                    color.rgb = mix(color.rgb, uUnderwaterColor, 0.3 * uw);
                    float bubble = sin(uv.x * 80.0 + uTime * 3.0) * sin(uv.y * 80.0 + uTime * 2.5);
                    bubble = clamp(bubble * 2.0, 0.0, 1.0) * 0.15 * uw;
                    color.rgb += vec3(bubble);
                }

                // 5. Vignette
                float dist = length(uv - 0.5);
                float vignette = 1.0 - dist * dist * uVignetteIntensity;
                color.rgb *= vignette;

                // 6. Film grain (stronger in dark areas)
                float grain = uFilmGrainIntensity * (1.0 - avgLum * 0.5);
                float noise = fract(sin(dot(uv + uTime * 0.1, vec2(12.9898, 78.233))) * 43758.5453);
                color.rgb += (noise - 0.5) * grain;

                // 7. Lens flare
                if (uLensFlareIntensity > 0.01 && uSunUV.x > 0.0 && uSunUV.x < 1.0 && uSunUV.y > 0.0 && uSunUV.y < 1.0) {
                    vec2 flareDir = uSunUV - uv;
                    float flareDist = length(flareDir);
                    float flare = smoothstep(0.5, 0.0, flareDist) * uLensFlareIntensity * 0.4;
                    color.rgb += vec3(1.0, 0.8, 0.4) * flare;
                    // Ghost artifacts
                    for (int i = 0; i < 3; i++) {
                        vec2 goff = vec2(float(i + 1) * 0.04, 0.0);
                        vec2 gpos = uSunUV - sign(uSunUV - 0.5) * goff;
                        float gd = length(gpos - uv);
                        float g = smoothstep(0.025, 0.0, gd) * uLensFlareIntensity * 0.12;
                        color.rgb += vec3(1.0, 0.6, 0.2) * g;
                    }
                }

                gl_FragColor = vec4(color.rgb, 1.0);
            }
        `,
    });
    finalPass = new ShaderPass(finalMaterial);
    finalPass.renderToScreen = true;
    composer.addPass(finalPass);

    new ResizeObserver(() => {
        if (!renderer || !camera) return;
        const rw = renderer.domElement.clientWidth;
        const rh = renderer.domElement.clientHeight;
        if (rw === 0 || rh === 0) return;
        renderer.setSize(rw, rh, false);
        composer.setSize(rw, rh);
        if (ssaoPass) ssaoPass.setSize(Math.floor(rw / 2), Math.floor(rh / 2));
        camera.aspect = rw / rh;
        camera.updateProjectionMatrix();
    }).observe(canvas);

    const ambient = new THREE.AmbientLight(0x404060, 0.6);
    scene.add(ambient);

    const hemi = new THREE.HemisphereLight(0x87ceeb, 0x3a2a1a, 0.8);
    scene.add(hemi);

    const sun = new THREE.DirectionalLight(0xffeedd, 2.5);
    sun.position.set(50, 80, 50);
    // R6: sun not added to scene — CSM manages directional lights for shadowed illumination
    globalThis.__sunLight = sun;

    csm = new CSM({
        camera,
        parent: scene,
        cascades: 4,
        maxFar: 150,
        shadowMapSize: 2048,
        shadowBias: -0.0005,
        lightDirection: new THREE.Vector3(50, 80, 50).normalize(),
        lightIntensity: 2.5,
        lightFar: 300,
        lightMargin: 100,
    });

    const fill = new THREE.DirectionalLight(0x4488ff, 0.4);
    fill.position.set(-30, 10, -30);
    scene.add(fill);

    initSunGlow();
    initClouds();
};

// ── R5 Vegetation Wind Shader ──
const VEG_WIND_VERTEX = `
uniform float uWindTime;
uniform float uWindDir;
uniform float uWindStrength;
varying vec3 vVNormal;
varying vec3 vVPos;
#ifdef USE_COLOR
varying vec3 vVColor;
#endif
void main() {
    vec3 pos = position;
    float windAngle = uWindDir;
    vec2 windVec = vec2(cos(windAngle), sin(windAngle));
    float heightFactor = max(pos.y, 0.0) * 0.3;
    float phase = uWindTime * 1.8 + pos.x * 0.7 + pos.z * 1.1;
    float sway = sin(phase) * uWindStrength * heightFactor;
    pos.x += windVec.x * sway;
    pos.z += windVec.y * sway;
    vec4 wp = modelMatrix * vec4(pos, 1.0);
    vVPos = wp.xyz;
    vVNormal = normalize(normalMatrix * normal);
    #ifdef USE_COLOR
    vVColor = color;
    #endif
    gl_Position = projectionMatrix * viewMatrix * wp;
}
`;

const VEG_WIND_FRAGMENT = `
uniform vec3 uAmbient;
uniform vec3 uSunDir;
uniform vec3 uSunColor;
varying vec3 vVNormal;
varying vec3 vVPos;
#ifdef USE_COLOR
varying vec3 vVColor;
#endif
void main() {
    vec3 N = normalize(vVNormal);
    vec3 L = normalize(uSunDir);
    float diff = max(dot(N, L), 0.0) * 0.5 + 0.5;
    vec3 light = uAmbient + uSunColor * diff;
    #ifdef USE_COLOR
    vec3 col = vVColor * light;
    #else
    vec3 col = vec3(0.3, 0.6, 0.2) * light;
    #endif
    gl_FragColor = vec4(col, 1.0);
}
`;

let windVegMatCache = null;
function getWindVegMaterial(hasColors) {
    if (windVegMatCache && windVegMatCache.defines.USE_COLOR === hasColors) return windVegMatCache;
    const defines = {};
    if (hasColors) defines.USE_COLOR = '';
    windVegMatCache = new THREE.ShaderMaterial({
        uniforms: {
            uWindTime: { value: 0 },
            uWindDir: { value: 0 },
            uWindStrength: { value: 0.4 },
            uAmbient: { value: new THREE.Color(0x404060) },
            uSunDir: { value: new THREE.Vector3(0.5, 0.8, 0.5).normalize() },
            uSunColor: { value: new THREE.Color(1, 0.95, 0.85) },
        },
        vertexShader: VEG_WIND_VERTEX,
        fragmentShader: VEG_WIND_FRAGMENT,
        defines,
        side: THREE.FrontSide,
    });
    return windVegMatCache;
}

// ── R5 Grass InstancedMesh ──
const GRASS_BLADE_VERTEX = `
uniform float uWindTime;
uniform float uWindDir;
uniform float uWindStrength;
uniform float uGrassHeight;
uniform float uGrassWidth;
varying float vHeightFrac;
void main() {
    vec3 pos = position;
    float windAngle = uWindDir;
    vec2 windVec = vec2(cos(windAngle), sin(windAngle));
    float heightFactor = pos.y / uGrassHeight;
    float phase = uWindTime * 2.5 + (pos.x + position.x) * 1.3;
    float sway = sin(phase) * uWindStrength * heightFactor * 0.5;
    vec3 instancePos = (instanceMatrix * vec4(pos, 1.0)).xyz;
    instancePos.x += windVec.x * sway;
    instancePos.z += windVec.y * sway;
    vHeightFrac = heightFactor;
    gl_Position = projectionMatrix * viewMatrix * vec4(instancePos, 1.0);
}
`;

const GRASS_BLADE_FRAGMENT = `
uniform vec3 uColor1;
uniform vec3 uColor2;
uniform vec3 uSunDir;
uniform vec3 uAmbient;
varying float vHeightFrac;
void main() {
    vec3 col = mix(uColor1, uColor2, vHeightFrac);
    col *= uAmbient + vec3(1.0, 0.95, 0.85) * 0.7;
    gl_FragColor = vec4(col, 1.0);
}
`;

let grassBladeGeo = null;
function getGrassBladeGeometry(height, width) {
    if (grassBladeGeo) return grassBladeGeo;
    const h = height || 0.4;
    const w = width || 0.04;
    const verts = new Float32Array([
        -w/2, 0, 0,   w/2, 0, 0,
        -w/4, h, 0,   w/4, h, 0,
    ]);
    const idx = new Uint16Array([0, 2, 1, 1, 2, 3]);
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.Float32BufferAttribute(verts, 3));
    geo.setIndex(new THREE.BufferAttribute(idx, 1));
    geo.computeVertexNormals();
    grassBladeGeo = geo;
    return geo;
}

function getGrassMaterial() {
    return new THREE.ShaderMaterial({
        uniforms: {
            uWindTime: { value: 0 },
            uWindDir: { value: 0 },
            uWindStrength: { value: 0.5 },
            uGrassHeight: { value: 0.4 },
            uGrassWidth: { value: 0.04 },
            uColor1: { value: new THREE.Color(0.3, 0.7, 0.15) },
            uColor2: { value: new THREE.Color(0.5, 0.85, 0.2) },
            uSunDir: { value: new THREE.Vector3(0.5, 0.8, 0.5).normalize() },
            uAmbient: { value: new THREE.Color(0.404060) },
        },
        vertexShader: GRASS_BLADE_VERTEX,
        fragmentShader: GRASS_BLADE_FRAGMENT,
        side: THREE.DoubleSide,
    });
}

window.threeBridgeUploadGrass = function (key, instanceData, count, height) {
    if (grassMeshes.has(key)) return;
    const h = height || 0.4;
    const geo = getGrassBladeGeometry(h, 0.04);
    const mat = getGrassMaterial();
    mat.uniforms.uGrassHeight.value = h;
    const mesh = new THREE.InstancedMesh(geo, mat, count);
    const dummy = new THREE.Object3D();
    for (let i = 0; i < count; i++) {
        const off = i * 4;
        dummy.position.set(instanceData[off], instanceData[off + 1], instanceData[off + 2]);
        dummy.scale.set(1, instanceData[off + 3], 1);
        dummy.rotation.set(0, instanceData[off + 4] || 0, 0);
        dummy.updateMatrix();
        mesh.setMatrixAt(i, dummy.matrix);
    }
    mesh.instanceMatrix.needsUpdate = true;
    mesh.receiveShadow = true;
    scene.add(mesh);
    grassMeshes.set(key, mesh);
    vegWindUniforms.push(mat.uniforms);
};

window.threeBridgeRemoveGrass = function (key) {
    const mesh = grassMeshes.get(key);
    if (!mesh) return;
    scene.remove(mesh);
    mesh.geometry && mesh.geometry.dispose();
    mesh.material.dispose();
    const idx = vegWindUniforms.indexOf(mesh.material.uniforms);
    if (idx >= 0) vegWindUniforms.splice(idx, 1);
    grassMeshes.delete(key);
};

window.threeBridgeUploadMesh = function (key, positions, normals, indices, colors, uvs) {
    if (meshes.has(key)) return;
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geo.setAttribute('normal', new THREE.Float32BufferAttribute(normals, 3));
    geo.setIndex(new THREE.BufferAttribute(indices, 1));
    if (colors && colors.length > 0) {
        geo.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
    }
    if (uvs && uvs.length > 0) {
        geo.setAttribute('uv', new THREE.Float32BufferAttribute(uvs, 2));
    }
    geo.computeBoundingSphere();
    const hasUVs = uvs && uvs.length > 0;
    const hasColors = colors && colors.length > 0;
    const isTerrain = key.startsWith('chunk_');
    const isVeg = key.startsWith('veg_');
    let mat;
    if (isVeg) {
        mat = getWindVegMaterial(hasColors).clone();
        vegWindUniforms.push(mat.uniforms);
    } else {
        mat = new THREE.MeshStandardMaterial({
            color: 0xffffff,
            vertexColors: hasColors,
            map: hasUVs ? textureAtlas : undefined,
            normalMap: hasUVs && textureAtlas && textureAtlas.userData ? textureAtlas.userData.normalMap : undefined,
            roughness: isTerrain ? 0.8 : 0.6,
            metalness: isTerrain ? 0.05 : 0.0,
            envMapIntensity: 0.3,
        });
        if (csm) csm.setupMaterial(mat);
    }
    const mesh = new THREE.Mesh(geo, mat);
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    scene.add(mesh);
    meshes.set(key, mesh);

    if (isTerrain) {
        createDetailOverlay(key, geo);
    }
};

// ── Portal Shared ──
const portalTime = { value: 0 };

window.threeBridgeUploadPortalMesh = function (key, positions, normals, indices, colors, targetSeed, radius) {
    if (meshes.has(key)) return;
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geo.setAttribute('normal', new THREE.Float32BufferAttribute(normals, 3));
    geo.setIndex(new THREE.BufferAttribute(indices, 1));
    if (colors) {
        geo.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
    }
    const hue = ((targetSeed % 200) / 200) * 0.8;
    const c1 = new THREE.Color().setHSL(hue, 0.8, 0.5);
    const c2 = new THREE.Color().setHSL((hue + 0.5) % 1.0, 1.0, 0.7);
    const mat = new THREE.ShaderMaterial({
        uniforms: {
            uTime: portalTime,
            uColor1: { value: c1 },
            uColor2: { value: c2 },
        },
        vertexShader: `
            uniform float uTime;
            varying vec3 vPos;
            void main() {
                vec3 p = position;
                float pulse = sin(length(p.xz) * 3.0 - uTime * 2.0) * 0.04;
                p.y += pulse;
                vPos = p;
                gl_Position = projectionMatrix * modelViewMatrix * vec4(p, 1.0);
            }
        `,
        fragmentShader: `
            uniform vec3 uColor1;
            uniform vec3 uColor2;
            uniform float uTime;
            varying vec3 vPos;
            void main() {
                float t = sin(vPos.x * 5.0 + vPos.z * 5.0 + uTime * 2.5) * 0.5 + 0.5;
                vec3 col = mix(uColor1, uColor2, t);
                float glow = 0.6 + 0.4 * sin(uTime * 2.0 + length(vPos.xz) * 6.0);
                col *= glow;
                gl_FragColor = vec4(col, 1.0);
            }
        `,
        side: THREE.DoubleSide,
    });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.renderOrder = 2;
    scene.add(mesh);
    meshes.set(key, mesh);

    // Floating particles around portal
    const pCount = 30;
    const pGeo = new THREE.BufferGeometry();
    const pPos = new Float32Array(pCount * 3);
    const r = radius * 0.8;
    for (let i = 0; i < pCount; i++) {
        const theta = (i / pCount) * Math.PI * 2 + Math.random() * 0.3;
        const pr = r * (0.5 + Math.random() * 0.5);
        pPos[i * 3] = Math.cos(theta) * pr;
        pPos[i * 3 + 1] = (Math.random() - 0.5) * 0.8;
        pPos[i * 3 + 2] = Math.sin(theta) * pr;
    }
    pGeo.setAttribute('position', new THREE.Float32BufferAttribute(pPos, 3));
    const pMat = new THREE.PointsMaterial({
        color: new THREE.Color().setHSL(hue, 1.0, 0.7),
        size: 0.08,
        transparent: true,
        opacity: 0.8,
        blending: THREE.AdditiveBlending,
        depthWrite: false,
        sizeAttenuation: true,
    });
    const particles = new THREE.Points(pGeo, pMat);
    particles.userData.isPortalParticles = true;
    scene.add(particles);
    meshes.set(key + "_p", particles);
};

// ── Fade overlay ──
let fadeOverlay = null;
window.threeBridgeSetFade = function (amount) {
    const val = Math.max(0, Math.min(1, amount));
    if (!fadeOverlay) {
        fadeOverlay = document.createElement('div');
        fadeOverlay.style.cssText = 'position:fixed;top:0;left:0;width:100%;height:100%;z-index:9999;pointer-events:none;background:#000;opacity:0';
        document.body.appendChild(fadeOverlay);
    }
    fadeOverlay.style.opacity = val;
};

window.threeBridgeSetMeshFrustumCulled = function (key, value) {
    const mesh = meshes.get(key);
    if (!mesh) return;
    mesh.frustumCulled = value;
};

window.threeBridgeSetMeshPosition = function (key, x, y, z) {
    const mesh = meshes.get(key);
    if (!mesh) return;
    mesh.position.set(x, y, z);
};

window.threeBridgeSetMeshRotation = function (key, x, y, z) {
    const mesh = meshes.get(key);
    if (!mesh) return;
    mesh.rotation.set(x, y, z);
};

window.threeBridgeUpdateMeshPositions = function (key, positions) {
    const mesh = meshes.get(key);
    if (!mesh) return;
    const attr = new THREE.Float32BufferAttribute(positions, 3);
    mesh.geometry.setAttribute('position', attr);
    mesh.geometry.computeVertexNormals();
};

function disposeObject(obj) {
    if (obj.geometry) obj.geometry.dispose();
    if (obj.material) {
        if (Array.isArray(obj.material)) obj.material.forEach(m => m.dispose());
        else obj.material.dispose();
    }
    if (obj.children) {
        for (let i = obj.children.length - 1; i >= 0; i--) {
            disposeObject(obj.children[i]);
        }
    }
}

// ── Web Worker for Chunk Generation ──
let chunkWorker = null;
let chunkCallbacks = new Map(); // id -> { resolve, reject }
let chunkReadyQueue = []; // completed chunks ready for pickup
let chunkIdCounter = 0;
let workerBusy = false;
let workerQueue = []; // pending chunk generation tasks

window.threeBridgeWorkerInit = function () {
    if (chunkWorker) return;
    try {
        chunkWorker = new Worker('worker.js');
        chunkWorker.onmessage = function (e) {
            const msg = e.data;
            if (msg.type === 'chunk_data') {
                workerBusy = false;
                // Store completed chunk in ready queue
                chunkReadyQueue.push(msg);
                // Process next in queue
                processWorkerQueue();
            }
        };
        chunkWorker.onerror = function (err) {
            console.error('Chunk worker error:', err);
            workerBusy = false;
            processWorkerQueue();
        };
    } catch (e) {
        console.warn('Worker not available, falling back to main thread:', e);
    }
};

function processWorkerQueue() {
    if (!chunkWorker || workerBusy || workerQueue.length === 0) return;
    const task = workerQueue.shift();
    workerBusy = true;
    chunkWorker.postMessage(task);
}

window.threeBridgeWorkerGenChunk = function (paramsJson, cx, cz, lod) {
    if (!chunkWorker) return -1;
    const id = chunkIdCounter++;
    const task = {
        type: 'gen_chunk',
        id: id,
        params: JSON.parse(paramsJson),
        cx: cx,
        cz: cz,
        lod: lod || 0,
    };
    workerQueue.push(task);
    processWorkerQueue();
    return id;
};

window.threeBridgeWorkerGetReady = function () {
    if (chunkReadyQueue.length === 0) return null;
    const msg = chunkReadyQueue.shift();
    // Create the mesh directly from worker data
    if (msg.positions && msg.positions.byteLength > 0) {
        const key = msg.key;
        const posArr = new Float32Array(msg.positions);
        const normArr = new Float32Array(msg.normals);
        const idxArr = new Uint32Array(msg.indices);
        const colArr = msg.colors ? new Float32Array(msg.colors) : null;
        
        if (!meshes.has(key)) {
            const geo = new THREE.BufferGeometry();
            geo.setAttribute('position', new THREE.Float32BufferAttribute(posArr, 3));
            geo.setAttribute('normal', new THREE.Float32BufferAttribute(normArr, 3));
            geo.setIndex(new THREE.BufferAttribute(idxArr, 1));
            if (colArr) geo.setAttribute('color', new THREE.Float32BufferAttribute(colArr, 3));
            geo.computeBoundingSphere();
            const mat = new THREE.MeshStandardMaterial({
                color: 0xffffff,
                vertexColors: !!colArr,
                roughness: 0.8,
                metalness: 0.05,
            });
            if (csm) csm.setupMaterial(mat);
            const mesh = new THREE.Mesh(geo, mat);
            mesh.castShadow = true;
            mesh.receiveShadow = true;
            scene.add(mesh);
            meshes.set(key, mesh);
            createDetailOverlay(key, geo);
        }
    }
    return JSON.stringify({ key: msg.key, cx: msg.cx, cz: msg.cz });
};

window.threeBridgeWorkerPending = function () {
    return workerQueue.length + (workerBusy ? 1 : 0);
};

window.threeBridgeWorkerSetSeed = function (seed) {
    if (chunkWorker) {
        chunkWorker.postMessage({ type: 'set_seed', seed });
    }
};

window.threeBridgeWorkerTerminate = function () {
    if (chunkWorker) {
        chunkWorker.terminate();
        chunkWorker = null;
    }
    workerQueue = [];
    chunkReadyQueue = [];
    workerBusy = false;
};

window.threeBridgeRemoveMesh = function (key) {
    // Also remove associated particles
    const pKey = key + "_p";
    const pMesh = meshes.get(pKey);
    if (pMesh) {
        scene.remove(pMesh);
        disposeObject(pMesh);
        meshes.delete(pKey);
    }
    const obj = meshes.get(key);
    if (!obj) return;
    scene.remove(obj);
    if (obj.material) {
        if (obj.material.uniforms) {
            const idx = vegWindUniforms.indexOf(obj.material.uniforms);
            if (idx >= 0) vegWindUniforms.splice(idx, 1);
        }
        if (csm && obj.material.type === 'MeshStandardMaterial') {
            csm.shaders.delete(obj.material);
        }
    }
    disposeObject(obj);
    meshes.delete(key);

    // Remove detail overlay
    const dMesh = detailMeshes.get(key);
    if (dMesh) {
        scene.remove(dMesh);
        disposeObject(dMesh);
        detailMeshes.delete(key);
    }
};

window.threeBridgeSetCamera = function (x, y, z, yaw, pitch) {
    if (!camera) return;
    camera.position.set(x, y, z);
    const euler = new THREE.Euler(pitch, yaw, 0, 'YXZ');
    camera.quaternion.setFromEuler(euler);
};

window.threeBridgeUploadTexture = function (key, width, height, data) {
    const tex = new THREE.DataTexture(data, width, height, THREE.RGBAFormat);
    tex.needsUpdate = true;
    tex.wrapS = tex.wrapT = THREE.RepeatWrapping;
    meshes.set('__tex_' + key, tex);
};

window.threeBridgeSetSky = function (r, g, b) {
    if (!scene) return;
    scene.background = new THREE.Color(r, g, b);
};

window.threeBridgeSetFog = function (r, g, b, density) {
    if (!scene) return;
    scene.fog = new THREE.FogExp2(new THREE.Color(r, g, b), density);
};

// ── R3 Atmosphere Scattering ──
const ATMOSPHERE_VERTEX_SHADER = `
varying vec3 vWorldPos;
void main() {
    vec4 wp = modelMatrix * vec4(position, 1.0);
    vWorldPos = wp.xyz;
    gl_Position = projectionMatrix * viewMatrix * wp;
}
`;

const ATMOSPHERE_FRAGMENT_SHADER = `
uniform vec3 uSunDirection;
uniform float uSunElevation;
uniform float uTurbidity;
uniform float uRayleigh;
uniform float uMieCoeff;
uniform float uMieG;
uniform float uExposure;
uniform vec3 uNightColor;
uniform float uStarsOpacity;

varying vec3 vWorldPos;

#define PI 3.14159265359

float rayleighPhase(float cosTheta) {
    return 3.0 / (16.0 * PI) * (1.0 + cosTheta * cosTheta);
}

float miePhase(float cosTheta, float g) {
    float g2 = g * g;
    return (1.0 - g2) / (4.0 * PI * pow(1.0 + g2 - 2.0 * g * cosTheta, 1.5));
}

float sunDisk(vec3 dir, vec3 sunDir, float angularRadius) {
    return smoothstep(angularRadius, angularRadius - 0.0005, acos(dot(dir, sunDir)));
}

void main() {
    vec3 viewDir = normalize(vWorldPos - cameraPosition);
    vec3 sunDir = normalize(uSunDirection);
    float cosTheta = dot(viewDir, sunDir);

    float elev = max(uSunElevation, 0.0);
    float night = clamp(-uSunElevation * 4.0, 0.0, 1.0);
    float sunfade = 1.0 - exp(-elev * 3.0);

    // Wavelengths for RGB (680nm, 550nm, 440nm)
    vec3 lambda = vec3(680e-9, 550e-9, 440e-9);
    vec3 lambda4 = lambda * lambda * lambda * lambda;
    vec3 rayleighBeta = vec3(5.8e-6, 1.35e-5, 3.36e-5) * uRayleigh;

    // Rayleigh scattering
    float rayleighAngular = rayleighPhase(cosTheta);
    float rayleighDepth = 1.0 / (1.0 + viewDir.y * 10.0);
    vec3 rayleighColor = rayleighBeta * rayleighAngular * rayleighDepth * elev;

    // Mie scattering
    float mieAngular = miePhase(cosTheta, uMieG);
    float mieDepth = 1.0 / (1.0 + viewDir.y * 5.0);
    vec3 mieColor = vec3(1.0, 0.95, 0.9) * uMieCoeff * mieAngular * mieDepth * sunfade;

    // Sun disk
    float disk = sunDisk(viewDir, sunDir, 0.0047);
    vec3 sunColor = vec3(1.0, 0.85, 0.5) * 15.0 * (1.0 - night);

    // Horizon tint (orange/red at low sun)
    float horizonFade = exp(-abs(viewDir.y) * 2.0) * (1.0 - elev) * 0.5;
    vec3 horizonGlow = vec3(1.0, 0.6, 0.2) * horizonFade * sunfade;

    // Combine sky
    vec3 skyColor = rayleighColor + mieColor + horizonGlow + sunColor * disk;

    // Night sky
    float starDot = 1.0 - abs(viewDir.y);
    float star = pow(max(0.0, fract(sin(dot(viewDir * 1000.0, vec3(12.9898, 78.233, 45.164)) * 43758.5453) - 0.95) * 20.0), 8.0);
    vec3 nightSky = uNightColor * night + vec3(star) * uStarsOpacity * night;

    skyColor = mix(skyColor, nightSky, night);

    // Tonemap
    skyColor = 1.0 - exp(-skyColor * uExposure);

    gl_FragColor = vec4(skyColor, 1.0);
}
`;

// ── Cloud Shader ──
const CLOUD_VERTEX_SHADER = `
varying vec3 vWorldPos;
varying vec2 vUv;
void main() {
    vec4 wp = modelMatrix * vec4(position, 1.0);
    vWorldPos = wp.xyz;
    vUv = position.xz * 0.001;
    gl_Position = projectionMatrix * viewMatrix * wp;
}
`;

const CLOUD_FRAGMENT_SHADER = `
uniform vec3 uColor;
uniform float uDensity;
uniform float uCoverage;
uniform float uScale;
uniform vec2 uWindOffset;
uniform float uOpacity;

varying vec3 vWorldPos;
varying vec2 vUv;

float hash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

float fbm(vec2 p, int octaves) {
    float v = 0.0, a = 1.0, t = 0.0;
    for (int i = 0; i < 6; i++) {
        if (i >= octaves) break;
        v += a * noise(p);
        t += a;
        a *= 0.5;
        p *= 2.0;
    }
    return v / t;
}

void main() {
    vec2 uv = vUv + uWindOffset;
    float n = fbm(uv * uScale, 5);
    float cloud = smoothstep(uCoverage, uCoverage + 0.3, n);
    float alpha = cloud * uDensity * uOpacity;
    float heightFade = smoothstep(0.0, 0.1, vWorldPos.y);
    alpha *= heightFade;
    gl_FragColor = vec4(uColor, alpha);
}
`;

let skyMesh = null;
let sunGlow = null;
let cloudLayers = [];
let cloudWindOffset = [0.0, 0.0, 0.0];
let cloudWindSpeed = [0.0003, 0.0005, 0.0008];

window.threeBridgeUploadSkyMesh = function (key, positions, normals, indices, colors) {
    if (meshes.has(key)) return;
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geo.setAttribute('normal', new THREE.Float32BufferAttribute(normals, 3));
    geo.setIndex(new THREE.BufferAttribute(indices, 1));
    const mat = new THREE.ShaderMaterial({
        uniforms: {
            uSunDirection: { value: new THREE.Vector3(0.5, 0.8, 0.3) },
            uSunElevation: { value: 0.8 },
            uTurbidity: { value: 2.0 },
            uRayleigh: { value: 1.0 },
            uMieCoeff: { value: 0.005 },
            uMieG: { value: 0.8 },
            uExposure: { value: 0.6 },
            uNightColor: { value: new THREE.Color(0x050510) },
            uStarsOpacity: { value: 0.0 },
        },
        vertexShader: ATMOSPHERE_VERTEX_SHADER,
        fragmentShader: ATMOSPHERE_FRAGMENT_SHADER,
        side: THREE.BackSide,
        depthWrite: false,
        fog: false,
    });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.renderOrder = -2;
    mesh.frustumCulled = false;
    scene.add(mesh);
    meshes.set(key, mesh);
    skyMesh = mesh;
};

window.threeBridgeSetSky = function (r, g, b) {
    // No longer used for background; kept for compatibility
};

// ── Sun Glow ──
function initSunGlow() {
    const canvas = document.createElement('canvas');
    canvas.width = 128;
    canvas.height = 128;
    const ctx = canvas.getContext('2d');
    const gradient = ctx.createRadialGradient(64, 64, 0, 64, 64, 64);
    gradient.addColorStop(0, 'rgba(255,255,255,1)');
    gradient.addColorStop(0.1, 'rgba(255,240,200,0.8)');
    gradient.addColorStop(0.3, 'rgba(255,200,100,0.3)');
    gradient.addColorStop(0.6, 'rgba(255,150,50,0.05)');
    gradient.addColorStop(1, 'rgba(0,0,0,0)');
    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, 128, 128);
    const tex = new THREE.CanvasTexture(canvas);
    const spriteMat = new THREE.SpriteMaterial({
        map: tex,
        blending: THREE.AdditiveBlending,
        depthWrite: false,
        transparent: true,
    });
    const sprite = new THREE.Sprite(spriteMat);
    sprite.scale.set(60, 60, 1);
    sprite.renderOrder = -1;
    scene.add(sprite);
    sunGlow = sprite;
}

// ── Cloud Layers ──
function initClouds() {
    const cloudColors = [0xffffff, 0xf0f0f0, 0xe0e0e0];
    const cloudHeights = [180, 120, 60];
    const cloudScales = [0.8, 1.2, 0.5];
    const cloudDensities = [0.3, 0.6, 0.5];
    const cloudCoverages = [0.35, 0.45, 0.55];

    for (let i = 0; i < 3; i++) {
        const geo = new THREE.SphereGeometry(200, 32, 16);
        const mat = new THREE.ShaderMaterial({
            uniforms: {
                uColor: { value: new THREE.Color(cloudColors[i]) },
                uDensity: { value: cloudDensities[i] },
                uCoverage: { value: cloudCoverages[i] },
                uScale: { value: cloudScales[i] },
                uWindOffset: { value: new THREE.Vector2(0, 0) },
                uOpacity: { value: 0.5 },
            },
            vertexShader: CLOUD_VERTEX_SHADER,
            fragmentShader: CLOUD_FRAGMENT_SHADER,
            transparent: true,
            side: THREE.BackSide,
            depthWrite: false,
            fog: false,
        });
        const mesh = new THREE.Mesh(geo, mat);
        mesh.position.y = cloudHeights[i];
        mesh.renderOrder = -1;
        mesh.frustumCulled = false;
        scene.add(mesh);
        cloudLayers.push(mesh);
    }
}

// ── Bridge functions ──
window.threeBridgeSetSunPosition = function (x, y, z, elevation) {
    if (csm) {
        csm.lightDirection.set(x, y, z).normalize();
    }
    if (skyMesh && skyMesh.material.uniforms) {
        skyMesh.material.uniforms.uSunDirection.value.set(x, y, z).normalize();
        skyMesh.material.uniforms.uSunElevation.value = elevation;
    }
    if (sunGlow) {
        sunGlow.position.set(x * 100, y * 100, z * 100);
        const size = 20 + (1 - Math.max(elevation, 0)) * 60;
        sunGlow.scale.set(size, size, 1);
        const glowColor = new THREE.Color();
        if (elevation > 0.1) {
            glowColor.setHSL(0.08, 0.6, 0.9);
        } else {
            glowColor.setHSL(0.05, 0.8, 0.6);
        }
        sunGlow.material.color.copy(glowColor);
    }
};

window.threeBridgeSetNightParams = function (nightColor, starsOpacity) {
    if (skyMesh && skyMesh.material.uniforms) {
        skyMesh.material.uniforms.uNightColor.value.setRGB(nightColor[0], nightColor[1], nightColor[2]);
        skyMesh.material.uniforms.uStarsOpacity.value = starsOpacity;
    }
};

window.threeBridgeCloudUpdate = function (windDir, windStrength) {
    const wd = windDir || 0;
    const ws = windStrength || 0.5;
    for (let i = 0; i < cloudLayers.length && i < cloudWindOffset.length; i++) {
        cloudWindOffset[i] += ws * cloudWindSpeed[i];
        const offset = new THREE.Vector2(
            Math.cos(wd) * cloudWindOffset[i],
            Math.sin(wd) * cloudWindOffset[i]
        );
        cloudLayers[i].material.uniforms.uWindOffset.value.copy(offset);
    }
};

// ── Water Shader ──
const WATER_VERTEX_SHADER = `
attribute float aAlpha;
varying float vAlpha;
varying vec3 vNormal;
varying vec3 vWorldPos;
void main() {
    vAlpha = aAlpha;
    vNormal = normalize(normalMatrix * normal);
    vec4 wp = modelMatrix * vec4(position, 1.0);
    vWorldPos = wp.xyz;
    gl_Position = projectionMatrix * viewMatrix * wp;
}
`;

const WATER_FRAGMENT_SHADER = `
uniform vec3 uWaterColor;
uniform vec3 uWaterDeepColor;
uniform vec3 uSunDir;
uniform float uOpacity;
uniform float uTime;
varying float vAlpha;
varying vec3 vNormal;
varying vec3 vWorldPos;

void main() {
    vec3 N = normalize(vNormal);
    vec3 V = normalize(cameraPosition - vWorldPos);
    float fresnel = pow(1.0 - max(dot(N, V), 0.0), 4.0);
    vec3 col = mix(uWaterDeepColor, uWaterColor, fresnel);
    float diffuse = max(dot(N, normalize(uSunDir)), 0.0) * 0.5 + 0.5;
    col *= 0.6 + 0.4 * diffuse;
    float alpha = vAlpha * uOpacity;
    gl_FragColor = vec4(col, alpha);
}
`;

window.threeBridgeUploadWaterMesh = function (key, positions, normals, indices, alphas) {
    if (meshes.has(key)) return;
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geo.setAttribute('normal', new THREE.Float32BufferAttribute(normals, 3));
    geo.setIndex(new THREE.BufferAttribute(indices, 1));
    if (alphas && alphas.length > 0) {
        geo.setAttribute('aAlpha', new THREE.Float32BufferAttribute(alphas, 1));
    } else {
        const count = positions.length / 3;
        geo.setAttribute('aAlpha', new THREE.Float32BufferAttribute(new Float32Array(count).fill(0.5), 1));
    }
    const mat = new THREE.ShaderMaterial({
        uniforms: {
            uWaterColor: { value: new THREE.Color(0x0077aa) },
            uWaterDeepColor: { value: new THREE.Color(0x003355) },
            uSunDir: { value: new THREE.Vector3(0.5, 0.8, 0.5).normalize() },
            uOpacity: { value: 0.4 },
            uTime: { value: 0 },
        },
        vertexShader: WATER_VERTEX_SHADER,
        fragmentShader: WATER_FRAGMENT_SHADER,
        transparent: true,
        side: THREE.DoubleSide,
        depthWrite: false,
    });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.renderOrder = 1;
    mesh.frustumCulled = false;
    scene.add(mesh);
    meshes.set(key, mesh);
    waterMesh = mesh;
    if (ssrPass && ssrPass.selects) {
        ssrPass.selects = [mesh];
        ssrPass.selective = true;
    }
};

window.threeBridgeUpdateWaterMesh = function (key, positions, normals, alphas) {
    const mesh = meshes.get(key);
    if (!mesh) return;
    const geo = mesh.geometry;
    geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geo.setAttribute('normal', new THREE.Float32BufferAttribute(normals, 3));
    if (alphas) {
        geo.setAttribute('aAlpha', new THREE.Float32BufferAttribute(alphas, 1));
    }
    geo.attributes.position.needsUpdate = true;
    geo.attributes.normal.needsUpdate = true;
    if (alphas) geo.attributes.aAlpha.needsUpdate = true;
};

window.threeBridgeSetWaterColor = function (r, g, b) {
    if (waterMesh && waterMesh.material.uniforms) {
        waterMesh.material.uniforms.uWaterColor.value.setRGB(r, g, b);
    }
};

window.threeBridgeSetUnderwater = function (active) {
    const prev = underwaterActive;
    underwaterActive = active;
    if (active !== prev && waterMesh) {
        if (active) {
            waterMesh.material.uniforms.uOpacity.value = 0.15;
            waterMesh.material.side = THREE.BackSide;
        } else {
            waterMesh.material.uniforms.uOpacity.value = 0.4;
            waterMesh.material.side = THREE.DoubleSide;
        }
    }
};

window.threeBridgeSetMeshColor = function (key, r, g, b) {
    const mesh = meshes.get(key);
    if (!mesh) return;
    if (mesh.material) {
        if (Array.isArray(mesh.material)) {
            mesh.material.forEach(m => m.color.setRGB(r, g, b));
        } else {
            mesh.material.color.setRGB(r, g, b);
        }
    }
};

window.threeBridgeSetParticlesOpacity = function (key, opacity) {
    const mesh = meshes.get(key);
    if (!mesh) return;
    if (Array.isArray(mesh.material)) {
        mesh.material.forEach(m => { m.opacity = opacity; });
    } else {
        mesh.material.opacity = opacity;
    }
};

window.threeBridgeCreateParticles = function (key, count, r, g, b, size) {
    if (meshes.has(key)) return;
    const geo = new THREE.BufferGeometry();
    const pos = new Float32Array(count * 3);
    geo.setAttribute('position', new THREE.Float32BufferAttribute(pos, 3));
    const mat = new THREE.PointsMaterial({
        color: new THREE.Color(r, g, b),
        size: size,
        transparent: true,
        opacity: 0.5,
        blending: THREE.AdditiveBlending,
        depthWrite: false,
        sizeAttenuation: true,
    });
    const points = new THREE.Points(geo, mat);
    points.frustumCulled = false;
    scene.add(points);
    meshes.set(key, points);
};

window.threeBridgeUpdateParticles = function (key, positions) {
    const points = meshes.get(key);
    if (!points) return;
    points.geometry.setAttribute(
        'position',
        new THREE.Float32BufferAttribute(positions, 3)
    );
    points.geometry.attributes.position.needsUpdate = true;
};

window.threeBridgeSetSunLight = function (x, y, z, r, g, b, intensity) {
    if (globalThis.__sunLight) {
        globalThis.__sunLight.position.set(x, y, z);
        globalThis.__sunLight.color.setRGB(r, g, b);
        globalThis.__sunLight.intensity = intensity;
    }
    if (csm) {
        csm.lightDirection.set(x, y, z).normalize();
        csm.lightIntensity = intensity;
        for (const light of csm.lights) {
            light.color.setRGB(r, g, b);
            light.intensity = intensity;
        }
    }
};

window.threeBridgeSetMeshVisible = function (key, visible) {
    const mesh = meshes.get(key);
    if (!mesh) return;
    mesh.visible = visible;
};

window.threeBridgeSetMeshOpacity = function (key, opacity) {
    const mesh = meshes.get(key);
    if (!mesh) return;
    if (mesh.material) {
        if (Array.isArray(mesh.material)) {
            mesh.material.forEach(m => { m.opacity = opacity; });
        } else {
            mesh.material.opacity = opacity;
        }
        mesh.material.transparent = opacity < 0.99;
        mesh.material.needsUpdate = true;
    }
};

let lastFrameTime = 0;
window.threeBridgeRenderFrame = function () {
    if (!composer || !scene || !camera) return;
    const now = performance.now();
    const dt = lastFrameTime ? (now - lastFrameTime) / 1000 : 0.016;
    lastFrameTime = now;
    portalTime.value += dt;

    // Animate portal particles
    for (const [key, obj] of meshes) {
        if (obj.userData && obj.userData.isPortalParticles) {
            const pos = obj.geometry.attributes.position.array;
            for (let i = 0; i < pos.length / 3; i++) {
                const i3 = i * 3;
                const angle = dt * 0.6;
                const x = pos[i3];
                const z = pos[i3 + 2];
                pos[i3] = x * Math.cos(angle) - z * Math.sin(angle);
                pos[i3 + 2] = x * Math.sin(angle) + z * Math.cos(angle);
                pos[i3 + 1] += Math.sin(now * 0.002 + i * 1.7) * dt * 0.15;
            }
            obj.geometry.attributes.position.needsUpdate = true;
        }
    }

    // Update detail overlay camera position for parallax
    if (detailMaterial && camera) {
        detailMaterial.uniforms.uCameraPos.value.copy(camera.position);
    }

    // Update water sun direction
    if (waterMesh && globalThis.__sunLight) {
        const sunDir = globalThis.__sunLight.position.clone().normalize();
        waterMesh.material.uniforms.uSunDir.value.copy(sunDir);
    }

    // Update SSR camera projection matrices
    if (ssrPass) {
        ssrPass.ssrMaterial.uniforms.cameraProjectionMatrix.value.copy(camera.projectionMatrix);
        ssrPass.ssrMaterial.uniforms.cameraInverseProjectionMatrix.value.copy(camera.projectionMatrixInverse);
    }

    // Update SSAO camera projection matrices
    if (ssaoPass) {
        ssaoPass.ssaoMaterial.uniforms.cameraProjectionMatrix.value.copy(camera.projectionMatrix);
        ssaoPass.ssaoMaterial.uniforms.cameraInverseProjectionMatrix.value.copy(camera.projectionMatrixInverse);
    }

    // R4: Update final combined pass
    if (finalPass) {
        const u = finalPass.material.uniforms;
        u.uTime.value = now * 0.001;
        u.uUnderwaterIntensity.value = underwaterActive ? 1.0 : 0.0;

        // Auto-exposure smooth adaptation
        exposureCurrent += (exposureTarget - exposureCurrent) * 0.05;
        u.uExposure.value = exposureCurrent;

        // Lens flare: compute sun screen-space position from camera
        if (globalThis.__sunLight) {
            const sunWorld = globalThis.__sunLight.position.clone();
            const sunNDC = sunWorld.project(camera);
            if (sunNDC.z > 0 && sunNDC.z < 1) {
                u.uSunUV.value.set(sunNDC.x * 0.5 + 0.5, sunNDC.y * 0.5 + 0.5);
            } else {
                u.uSunUV.value.set(-1, -1);
            }
            const viewDir = camera.getWorldDirection(new THREE.Vector3());
            const sunDir = sunWorld.clone().normalize();
            const alignment = Math.max(0, viewDir.dot(sunDir));
            lensFlareIntensity = Math.pow(alignment, 5.0) * 0.8;
            u.uLensFlareIntensity.value = lensFlareIntensity;
        }

        // Vignette & grain from biome
        const cat = biomeCategory(currentLutBiome);
        u.uVignetteIntensity.value = cat === 'cave' ? 0.7 : 0.25;
        u.uFilmGrainIntensity.value = cat === 'cave' || underwaterActive ? 0.08 : 0.03;

        // Exposure target from biome
        switch (cat) {
            case 'cave': exposureTarget = 1.8; break;
            case 'aquatic': exposureTarget = 1.2; break;
            case 'warm': exposureTarget = 0.9; break;
            case 'special': exposureTarget = 0.85; break;
            default: exposureTarget = 1.0; break;
        }
    }

    // Cloud wind drift
    if (cloudLayers.length > 0 && scene.userData) {
        const wd = scene.userData.windDir || 0;
        const ws = scene.userData.windStrength || 0.5;
        for (let i = 0; i < cloudLayers.length; i++) {
            cloudWindOffset[i] += ws * cloudWindSpeed[i] * dt * 60;
            const off = new THREE.Vector2(
                Math.cos(wd) * cloudWindOffset[i],
                Math.sin(wd) * cloudWindOffset[i]
            );
            cloudLayers[i].material.uniforms.uWindOffset.value.copy(off);
        }
    }

    // R5: Update vegetation wind sway
    if (vegWindUniforms.length > 0) {
        const wd = scene.userData.windDir || 0;
        const ws = scene.userData.windStrength || 0.5;
        const t = now * 0.001;
        for (const u of vegWindUniforms) {
            u.uWindTime.value = t;
            u.uWindDir.value = wd;
            u.uWindStrength.value = ws * 0.6 + 0.1;
            if (u.uSunDir && globalThis.__sunLight) {
                u.uSunDir.value.copy(globalThis.__sunLight.position).normalize();
            }
        }
    }

    // R6: Update CSM shadow cameras before render
    if (csm) csm.update();

    composer.render();
};

// ── R7: SSAO intensity from biome ──
function setSSAOFromBiome(zoneId) {
    if (!ssaoPass) return;
    const cat = biomeCategory(zoneId);
    switch (cat) {
        case 'cave':
            ssaoPass.kernelRadius = 10;
            ssaoPass.maxDistance = 0.18;
            break;
        case 'aquatic':
            ssaoPass.kernelRadius = 6;
            ssaoPass.maxDistance = 0.1;
            break;
        case 'warm':
            ssaoPass.kernelRadius = 3;
            ssaoPass.maxDistance = 0.06;
            break;
        case 'special':
            ssaoPass.kernelRadius = 4;
            ssaoPass.maxDistance = 0.08;
            break;
        default:
            ssaoPass.kernelRadius = 3;
            ssaoPass.maxDistance = 0.06;
            break;
    }
}

window.threeBridgeSetSSAOIntensity = function (kernelRadius, maxDistance) {
    if (!ssaoPass) return;
    ssaoPass.kernelRadius = kernelRadius;
    ssaoPass.maxDistance = maxDistance;
};

// ── R4: Biome-based color grading ──
window.threeBridgeSetBiome = function (zoneId) {
    currentLutBiome = zoneId;
    if (lutCtx && lutCanvas && lutTexture) {
        applyBiomeLUT(lutCtx, lutCanvas, zoneId);
        lutTexture.needsUpdate = true;
    }
    const cat = biomeCategory(zoneId);
    switch (cat) {
        case 'cave': exposureTarget = 1.8; break;
        case 'aquatic': exposureTarget = 1.2; break;
        case 'warm': exposureTarget = 0.9; break;
        case 'special': exposureTarget = 0.85; break;
        default: exposureTarget = 1.0; break;
    }
    setSSAOFromBiome(zoneId);
};

// ── Wind / Weather ──
window.threeBridgeSetWind = function (dir, strength) {
    // Wind direction (radians) and strength (0-1) for visual effects
    // This will be used for particle systems, foliage animation, etc.
    if (!scene) return;
    scene.userData = scene.userData || {};
    scene.userData.windDir = dir;
    scene.userData.windStrength = strength;
};

// ── WebSocket / Multiplayer ──
let ws = null;
let wsConnected = false;
let wsPlayerId = null;
let wsYaw = 0;
let wsPitch = 0;
let remotePlayers = new Map(); // playerId -> { mesh, nameLabel }

window.threeBridgeWsConnect = function (url, seed, onMessage) {
    if (ws && ws.readyState === WebSocket.OPEN) return;
    ws = new WebSocket(url);
    ws.onopen = () => {
        wsConnected = true;
        ws.send(JSON.stringify({ type: "join", seed }));
        setInterval(() => {
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(JSON.stringify({ type: "pong" }));
            }
        }, 30000);
    };
    ws.onmessage = (ev) => {
        try {
            const msg = JSON.parse(ev.data);
            if (msg.type === "welcome") {
                wsPlayerId = msg.your_id;
            }
            onMessage(JSON.stringify(msg));
        } catch (e) {}
    };
    ws.onclose = () => {
        wsConnected = false;
        wsPlayerId = null;
        // Clear remote players
        for (const [id, obj] of remotePlayers) {
            scene.remove(obj.mesh);
            if (obj.label) scene.remove(obj.label);
        }
        remotePlayers.clear();
    };
};

window.threeBridgeWsSendPos = function (x, y, z, yaw, pitch) {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    ws.send(JSON.stringify({ type: "pos", x, y, z, yaw, pitch }));
    wsYaw = yaw;
    wsPitch = pitch;
};

window.threeBridgeWsSendChat = function (text) {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    ws.send(JSON.stringify({ type: "chat", text }));
};

window.threeBridgeWsDisconnect = function () {
    if (ws) { ws.close(); ws = null; }
    wsConnected = false;
    for (const [id, obj] of remotePlayers) {
        scene.remove(obj.mesh);
        if (obj.label) scene.remove(obj.label);
    }
    remotePlayers.clear();
};

window.threeBridgeUpdateRemotePlayer = function (id, name, x, y, z, yaw, pitch) {
    let obj = remotePlayers.get(id);
    if (!obj) {
        // Simple capsule: cylinder + sphere on top
        const bodyGeo = new THREE.CylinderGeometry(0.25, 0.25, 0.8, 8);
        const headGeo = new THREE.SphereGeometry(0.2, 6, 6);
        const mat = new THREE.MeshStandardMaterial({
            color: new THREE.Color(0.3, 0.7, 1.0),
            roughness: 0.5,
            metalness: 0.2,
        });
        if (csm) csm.setupMaterial(mat);
        const body = new THREE.Mesh(bodyGeo, mat);
        body.position.y = 0.4;
        const head = new THREE.Mesh(headGeo, mat);
        head.position.y = 1.0;
        const group = new THREE.Group();
        group.add(body);
        group.add(head);
        scene.add(group);

        // Name label (sprite)
        const canvas = document.createElement('canvas');
        canvas.width = 256;
        canvas.height = 64;
        const ctx = canvas.getContext('2d');
        ctx.fillStyle = 'rgba(0,0,0,0.5)';
        ctx.roundRect(0, 0, 256, 64, 8);
        ctx.fill();
        ctx.fillStyle = 'white';
        ctx.font = '20px monospace';
        ctx.textAlign = 'center';
        ctx.fillText(name, 128, 40);
        const tex = new THREE.CanvasTexture(canvas);
        const labelMat = new THREE.SpriteMaterial({ map: tex, transparent: true, depthTest: false });
        const label = new THREE.Sprite(labelMat);
        label.position.y = 1.6;
        label.scale.set(1, 0.25, 1);
        group.add(label);

        remotePlayers.set(id, { mesh: group, label: null });
        obj = remotePlayers.get(id);
    }
    // Lerp position
    const dx = x - obj.mesh.position.x;
    const dy = y - obj.mesh.position.y;
    const dz = z - obj.mesh.position.z;
    obj.mesh.position.x += dx * 0.15;
    obj.mesh.position.y += dy * 0.15;
    obj.mesh.position.z += dz * 0.15;
    obj.mesh.rotation.y = yaw;
};

window.threeBridgeWsRemovePlayer = function (id) {
    const obj = remotePlayers.get(id);
    if (!obj) return;
    scene.remove(obj.mesh);
    remotePlayers.delete(id);
};
