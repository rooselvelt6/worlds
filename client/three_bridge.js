import * as THREE from 'three';
import { EffectComposer } from 'three/addons/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/addons/postprocessing/RenderPass.js';
import { UnrealBloomPass } from 'three/addons/postprocessing/UnrealBloomPass.js';

let scene = null;
let camera = null;
let renderer = null;
let composer = null;
const meshes = new Map();

window.threeBridgeInit = function (canvas) {
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
    const bloom = new UnrealBloomPass(new THREE.Vector2(w, h), 0.3, 0.2, 0.1);
    composer.addPass(bloom);

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

    const ambient = new THREE.AmbientLight(0x404060, 1.0);
    scene.add(ambient);

    const sun = new THREE.DirectionalLight(0xffffff, 2.0);
    sun.position.set(50, 80, 50);
    sun.castShadow = true;
    sun.shadow.mapSize.width = 1024;
    sun.shadow.mapSize.height = 1024;
    sun.shadow.camera.left = -60;
    sun.shadow.camera.right = 60;
    sun.shadow.camera.top = 60;
    sun.shadow.camera.bottom = -60;
    sun.shadow.camera.near = 1;
    sun.shadow.camera.far = 200;
    scene.add(sun);

    const fill = new THREE.DirectionalLight(0x4488ff, 0.5);
    fill.position.set(-30, 10, -30);
    scene.add(fill);
};

window.threeBridgeUploadMesh = function (key, positions, normals, indices, colors) {
    if (meshes.has(key)) return;
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geo.setAttribute('normal', new THREE.Float32BufferAttribute(normals, 3));
    geo.setIndex(new THREE.BufferAttribute(indices, 1));
    if (colors) {
        geo.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
    }
    const mat = new THREE.MeshStandardMaterial({
        color: colors ? 0xffffff : 0xff3333,
        vertexColors: !!colors,
        roughness: 0.6,
        metalness: 0.0,
    });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    scene.add(mesh);
    meshes.set(key, mesh);
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

window.threeBridgeRemoveMesh = function (key) {
    const mesh = meshes.get(key);
    if (!mesh) return;
    scene.remove(mesh);
    mesh.geometry.dispose();
    mesh.material.dispose();
    meshes.delete(key);
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

window.threeBridgeUploadSkyMesh = function (key, positions, normals, indices, colors) {
    if (meshes.has(key)) return;
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geo.setAttribute('normal', new THREE.Float32BufferAttribute(normals, 3));
    geo.setIndex(new THREE.BufferAttribute(indices, 1));
    geo.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
    const mat = new THREE.MeshBasicMaterial({
        vertexColors: true,
        side: THREE.BackSide,
        depthWrite: false,
        fog: false,
    });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.renderOrder = -1;
    mesh.frustumCulled = false;
    scene.add(mesh);
    meshes.set(key, mesh);
};

window.threeBridgeUploadWaterMesh = function (key, positions, normals, indices) {
    if (meshes.has(key)) return;
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geo.setAttribute('normal', new THREE.Float32BufferAttribute(normals, 3));
    geo.setIndex(new THREE.BufferAttribute(indices, 1));
    const mat = new THREE.MeshPhysicalMaterial({
        color: 0x0077aa,
        transparent: true,
        opacity: 0.35,
        roughness: 0.2,
        metalness: 0.1,
        side: THREE.DoubleSide,
    });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.renderOrder = 1;
    scene.add(mesh);
    meshes.set(key, mesh);
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

window.threeBridgeRenderFrame = function () {
    if (!composer || !scene || !camera) return;
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
