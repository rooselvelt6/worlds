import * as THREE from 'three';
import { EffectComposer } from 'three/addons/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/addons/postprocessing/RenderPass.js';
import { UnrealBloomPass } from 'three/addons/postprocessing/UnrealBloomPass.js';
import { SSRPass } from 'three/addons/postprocessing/SSRPass.js';
import { ShaderPass } from 'three/addons/postprocessing/ShaderPass.js';

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
let underwaterPass = null;
let waterMesh = null;
let underwaterActive = false;
const UNDERWATER_COLOR = new THREE.Color(0x006688);

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

    const bloom = new UnrealBloomPass(new THREE.Vector2(w, h), 0.3, 0.2, 0.1);
    composer.addPass(bloom);

    // Underwater pass
    const underwaterMaterial = new THREE.ShaderMaterial({
        uniforms: {
            tDiffuse: { value: null },
            uTime: { value: 0 },
            uIntensity: { value: 0 },
            uWaterColor: { value: new THREE.Color(0x006688) },
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
            uniform float uTime;
            uniform float uIntensity;
            uniform vec3 uWaterColor;
            varying vec2 vUv;

            void main() {
                vec2 uv = vUv;
                // Caustic-like distortion
                float caustic = sin(uv.x * 20.0 + uTime * 1.5) * sin(uv.y * 20.0 + uTime * 1.2) * 0.5 + 0.5;
                uv.x += sin(uv.y * 30.0 + uTime * 2.0) * 0.003 * uIntensity;
                uv.y += cos(uv.x * 30.0 + uTime * 1.8) * 0.003 * uIntensity;

                vec4 color = texture2D(tDiffuse, uv);
                vec3 tinted = mix(color.rgb, uWaterColor, 0.3 * uIntensity);

                // Vignette
                float dist = length(uv - 0.5);
                float vignette = 1.0 - dist * 0.8 * uIntensity;

                // Bubbles
                float bubble = sin(uv.x * 80.0 + uTime * 3.0) * sin(uv.y * 80.0 + uTime * 2.5);
                bubble = clamp(bubble * 2.0, 0.0, 1.0) * 0.15 * uIntensity;

                gl_FragColor = vec4(tinted * vignette + vec3(bubble), 1.0);
            }
        `,
    });
    underwaterPass = new ShaderPass(underwaterMaterial);
    underwaterPass.renderToScreen = true;
    composer.addPass(underwaterPass);

    new ResizeObserver(() => {
        if (!renderer || !camera) return;
        const rw = renderer.domElement.clientWidth;
        const rh = renderer.domElement.clientHeight;
        if (rw === 0 || rh === 0) return;
        renderer.setSize(rw, rh, false);
        composer.setSize(rw, rh);
        camera.aspect = rw / rh;
        camera.updateProjectionMatrix();
    }).observe(canvas);

    const ambient = new THREE.AmbientLight(0x404060, 0.6);
    scene.add(ambient);

    const hemi = new THREE.HemisphereLight(0x87ceeb, 0x3a2a1a, 0.8);
    scene.add(hemi);

    const sun = new THREE.DirectionalLight(0xffeedd, 2.5);
    sun.position.set(50, 80, 50);
    sun.castShadow = true;
    sun.shadow.mapSize.width = 2048;
    sun.shadow.mapSize.height = 2048;
    sun.shadow.camera.left = -80;
    sun.shadow.camera.right = 80;
    sun.shadow.camera.top = 80;
    sun.shadow.camera.bottom = -80;
    sun.shadow.camera.near = 1;
    sun.shadow.camera.far = 250;
    sun.shadow.bias = -0.001;
    scene.add(sun);

    const fill = new THREE.DirectionalLight(0x4488ff, 0.4);
    fill.position.set(-30, 10, -30);
    scene.add(fill);

    initSunGlow();
    initClouds();

    globalThis.__sunLight = sun;
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
    const mat = new THREE.MeshStandardMaterial({
        color: 0xffffff,
        vertexColors: hasColors,
        map: hasUVs ? textureAtlas : undefined,
        normalMap: hasUVs && textureAtlas && textureAtlas.userData ? textureAtlas.userData.normalMap : undefined,
        roughness: isTerrain ? 0.8 : 0.6,
        metalness: isTerrain ? 0.05 : 0.0,
        envMapIntensity: 0.3,
    });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    scene.add(mesh);
    meshes.set(key, mesh);

    if (isTerrain) {
        createDetailOverlay(key, geo);
    }
};

window.threeBridgeUploadMeshBatch = function (batchJson, positions, normals, indices, colors, uvs) {
    const batch = JSON.parse(batchJson);
    for (const entry of batch) {
        if (meshes.has(entry.key)) continue;
        const geo = new THREE.BufferGeometry();
        const posCount = entry.pn * 3;
        const posArr = positions.slice(entry.po, entry.po + posCount);
        geo.setAttribute('position', new THREE.Float32BufferAttribute(posArr, 3));
        const normCount = entry.nn * 3;
        const normArr = normals.slice(entry.no, entry.no + normCount);
        geo.setAttribute('normal', new THREE.Float32BufferAttribute(normArr, 3));
        const idxCount = entry.in;
        const idxArr = indices.slice(entry.io, entry.io + idxCount);
        geo.setIndex(new THREE.BufferAttribute(idxArr, 1));
        if (entry.hasCol) {
            const colCount = entry.cn * 3;
            const colArr = colors.slice(entry.co, entry.co + colCount);
            geo.setAttribute('color', new THREE.Float32BufferAttribute(colArr, 3));
        }
        if (entry.hasUv) {
            const uvCount = entry.un * 2;
            const uvArr = uvs.slice(entry.uo, entry.uo + uvCount);
            geo.setAttribute('uv', new THREE.Float32BufferAttribute(uvArr, 2));
        }
        geo.computeBoundingSphere();
        const hasUVs = entry.hasUv;
        const hasColors = entry.hasCol;
        const isTerrain = entry.key.startsWith('chunk_');
        const mat = new THREE.MeshStandardMaterial({
            color: 0xffffff,
            vertexColors: hasColors,
            map: hasUVs ? textureAtlas : undefined,
            normalMap: hasUVs && textureAtlas && textureAtlas.userData ? textureAtlas.userData.normalMap : undefined,
            roughness: isTerrain ? 0.8 : 0.6,
            metalness: isTerrain ? 0.05 : 0.0,
            envMapIntensity: 0.3,
        });
        const mesh = new THREE.Mesh(geo, mat);
        mesh.castShadow = true;
        mesh.receiveShadow = true;
        scene.add(mesh);
        meshes.set(entry.key, mesh);
        if (isTerrain) {
            createDetailOverlay(entry.key, geo);
        }
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
    if (!scene) return;
    const old = scene.getObjectByName('__sun');
    if (old) scene.remove(old);
    const light = new THREE.DirectionalLight(new THREE.Color(r, g, b), intensity);
    light.name = '__sun';
    light.position.set(x, y, z);
    scene.add(light);
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

    // Update underwater pass
    if (underwaterPass) {
        underwaterPass.material.uniforms.uTime.value = now * 0.001;
        underwaterPass.material.uniforms.uIntensity.value = underwaterActive ? 1.0 : 0.0;
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

    composer.render();
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
