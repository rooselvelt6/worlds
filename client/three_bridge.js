(function () {
    var THREE = window.THREE;
    if (!THREE) { console.error("[bridge] THREE not loaded"); return; }

    var scene = null, camera = null, renderer = null;
    var meshes = new Map();
    var waterMesh = null;
    var sunLight = null, fillLight = null, ambientLight = null;
    var timeOfDay = 0.5;
    var particles = [];
    var currentBiome = 'plains';

    // Post-processing disabled for color accuracy; using direct render

    function resizeRenderer() {
        if (!renderer || !camera || !renderer.domElement) return;
        var w = renderer.domElement.clientWidth;
        var h = renderer.domElement.clientHeight;
        if (w === 0 || h === 0) return;
        renderer.setSize(w, h, false);
        camera.aspect = w / h;
        camera.updateProjectionMatrix();
    }

    window.threeBridgeInit = function (canvas) {
        try {
            scene = new THREE.Scene();
            scene.background = new THREE.Color(0x0d0d1a);
            scene.fog = new THREE.FogExp2(0x0d0d1a, 0.008);

            camera = new THREE.PerspectiveCamera(75, 1, 0.1, 300);
            camera.position.set(50, 25, 50);

            renderer = new THREE.WebGLRenderer({ canvas: canvas, antialias: true });
            renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
            renderer.shadowMap.enabled = true;
            renderer.shadowMap.type = THREE.PCFSoftShadowMap;

            resizeRenderer();
            new ResizeObserver(resizeRenderer).observe(canvas);

            renderer.toneMapping = THREE.NoToneMapping;
            renderer.outputColorSpace = THREE.SRGBColorSpace;

            sunLight = new THREE.DirectionalLight(0xff8844, 1.4);
            sunLight.position.set(-40, 60, -20);
            sunLight.castShadow = true;
            sunLight.shadow.mapSize.width = 2048;
            sunLight.shadow.mapSize.height = 2048;
            sunLight.shadow.camera.near = 0.5;
            sunLight.shadow.camera.far = 200;
            sunLight.shadow.camera.left = -80;
            sunLight.shadow.camera.right = 80;
            sunLight.shadow.camera.top = 80;
            sunLight.shadow.camera.bottom = -80;
            sunLight.shadow.bias = -0.001;
            scene.add(sunLight);

            fillLight = new THREE.DirectionalLight(0x4488ff, 0.6);
            fillLight.position.set(30, 10, 40);
            scene.add(fillLight);

            ambientLight = new THREE.AmbientLight(0x222244, 0.4);
            scene.add(ambientLight);

            var rim = new THREE.DirectionalLight(0x6644aa, 0.3);
            rim.position.set(0, -30, 0);
            scene.add(rim);

            // Vignette overlay
            var vignetteEl = document.createElement('div');
            vignetteEl.id = 'vignette-overlay';
            vignetteEl.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;pointer-events:none;z-index:9999;background:radial-gradient(ellipse at center, transparent 55%, rgba(0,0,0,0.5) 100%);opacity:0.3;transition:opacity 0.5s;';
            document.body.appendChild(vignetteEl);

            // Water with custom shader
            var waterGeo = new THREE.PlaneGeometry(500, 500, 64, 64);
            var waterMat = new THREE.ShaderMaterial({
                uniforms: {
                    uTime: { value: 0 },
                    uColorDeep: { value: new THREE.Color(0x0a2a4a) },
                    uColorShallow: { value: new THREE.Color(0x1a8a9a) },
                    uColorFoam: { value: new THREE.Color(0x8ad4e8) },
                    uWaterLevel: { value: 0 },
                },
                vertexShader: [
                    "uniform float uTime;",
                    "varying vec2 vUv;",
                    "varying float vHeight;",
                    "void main() {",
                    "  vUv = uv;",
                    "  vec3 pos = position;",
                    "  float wave1 = sin(pos.x * 0.05 + uTime * 0.8) * 0.3;",
                    "  float wave2 = sin(pos.y * 0.08 + uTime * 0.5 + 1.3) * 0.2;",
                    "  float wave3 = sin((pos.x + pos.y) * 0.03 + uTime * 0.3) * 0.4;",
                    "  pos.z += wave1 + wave2 + wave3;",
                    "  vHeight = pos.z;",
                    "  gl_Position = projectionMatrix * modelViewMatrix * vec4(pos, 1.0);",
                    "}"
                ].join("\n"),
                fragmentShader: [
                    "uniform vec3 uColorDeep;",
                    "uniform vec3 uColorShallow;",
                    "uniform vec3 uColorFoam;",
                    "uniform float uTime;",
                    "varying vec2 vUv;",
                    "varying float vHeight;",
                    "void main() {",
                    "  vec2 uv = vUv * 8.0;",
                    "  float wave = sin(uv.x * 3.0 + uTime) * cos(uv.y * 3.0 + uTime * 0.7);",
                    "  float foam = smoothstep(0.3, 0.7, wave);",
                    "  float depth = (vHeight + 1.5) / 3.0;",
                    "  depth = clamp(depth, 0.0, 1.0);",
                    "  vec3 col = mix(uColorDeep, uColorShallow, depth);",
                    "  col = mix(col, uColorFoam, foam * 0.3);",
                    "  float shimmer = sin(uv.x * 10.0 + uTime * 2.0) * sin(uv.y * 10.0 + uTime * 1.7);",
                    "  col += vec3(0.1, 0.15, 0.2) * max(0.0, shimmer * 0.3);",
                    "  float alpha = 0.35 + depth * 0.25;",
                    "  gl_FragColor = vec4(col, alpha);",
                    "}"
                ].join("\n"),
                transparent: true,
                side: THREE.DoubleSide,
                depthWrite: false,
            });
            waterMesh = new THREE.Mesh(waterGeo, waterMat);
            waterMesh.rotation.x = -Math.PI / 2;
            waterMesh.position.y = 0;
            waterMesh.receiveShadow = true;
            scene.add(waterMesh);

            // Post-processing disabled

        } catch (e) {
            console.error("[bridge] init error:", e);
        }
    };

    window.threeBridgeAddChunk = function (key, posArr, colArr, idxArr, ox, oz) {
        if (meshes.has(key)) return;
        try {
            var geo = new THREE.BufferGeometry();
            geo.setAttribute("position", new THREE.BufferAttribute(posArr, 3));
            geo.setAttribute("color", new THREE.BufferAttribute(colArr, 3));
            geo.setIndex(new THREE.BufferAttribute(idxArr, 1));
            geo.computeVertexNormals();

            var mat = new THREE.MeshLambertMaterial({
                vertexColors: true,
                flatShading: false,
            });
            var mesh = new THREE.Mesh(geo, mat);
            mesh.position.set(ox, 0, oz);
            mesh.receiveShadow = true;
            mesh.castShadow = true;
            scene.add(mesh);
            meshes.set(key, mesh);
        } catch (e) {
            console.error("[bridge] addChunk error:", e);
        }
    };

    window.threeBridgeRemoveChunk = function (key) {
        var mesh = meshes.get(key);
        if (mesh) {
            scene.remove(mesh);
            mesh.geometry.dispose();
            mesh.material.dispose();
            meshes.delete(key);
        }
    };

    window.threeBridgeUpdateCamera = function (x, y, z, yaw, pitch) {
        if (!camera) return;
        camera.position.set(x, y, z);
        var euler = new THREE.Euler(pitch, yaw, 0, "YXZ");
        camera.quaternion.setFromEuler(euler);
    };

    window.threeBridgeSetTime = function (t) {
        timeOfDay = t;
        if (!sunLight || !fillLight || !ambientLight || !scene) return;

        var angle = t * Math.PI * 2;
        var sunY = Math.sin(angle) * 60;
        var sunX = Math.cos(angle) * 60;
        sunLight.position.set(sunX, Math.max(sunY, -10), -20);

        var dayFactor = Math.min(Math.max(Math.sin(angle), 0), 1);
        var nightFactor = 1 - dayFactor;

        var sunColor = new THREE.Color().setHSL(0.08 - dayFactor * 0.05, 0.8, 0.5 + dayFactor * 0.3);
        sunLight.color.copy(sunColor);
        sunLight.intensity = 0.4 + dayFactor * 1.2;

        fillLight.color.setHSL(0.6, 0.5, 0.3 + nightFactor * 0.2);
        fillLight.intensity = 0.2 + nightFactor * 0.4;

        var ambientColor = (dayFactor > 0.5)
            ? new THREE.Color(0x222244)
            : new THREE.Color(0x0a0a1a);
        ambientLight.color.copy(ambientColor);
        ambientLight.intensity = 0.15 + dayFactor * 0.35;

        var skyColor = new THREE.Color();
        if (dayFactor > 0.3) {
            skyColor.setHSL(0.6, 0.4, 0.3 + dayFactor * 0.4);
        } else {
            skyColor.setHSL(0.7, 0.5, 0.05 + nightFactor * 0.08);
        }
        scene.background.copy(skyColor);

        if (scene.fog) {
            scene.fog.color.copy(skyColor);
            scene.fog.density = 0.004 + nightFactor * 0.006;
        }

        if (waterMesh) {
            waterMesh.position.y = -1 + dayFactor * 0.5;
            if (waterMesh.material.uniforms) {
                var deepCol = new THREE.Color().setHSL(0.6, 0.6, 0.05 + dayFactor * 0.1);
                var shallowCol = new THREE.Color().setHSL(0.55, 0.6, 0.2 + dayFactor * 0.25);
                waterMesh.material.uniforms.uColorDeep.value.copy(deepCol);
                waterMesh.material.uniforms.uColorShallow.value.copy(shallowCol);
            }
        }

    };

    window.threeBridgeSetWaterLevel = function (level) {
        if (waterMesh) {
            waterMesh.position.y = level;
            if (waterMesh.material.uniforms) {
                waterMesh.material.uniforms.uWaterLevel.value = level;
            }
        }
    };

    window.threeBridgeSetFog = function (r, g, b, density) {
        if (scene && scene.fog) {
            scene.fog.color.setRGB(r, g, b);
            scene.fog.density = density;
        }
    };

    window.threeBridgeSetBloom = function () {};
    window.threeBridgeSetLUT = function () {};
    window.threeBridgeSetHeatHaze = function () {};

    window.threeBridgeSpawnParticles = function (key, posArr, colArr, count) {
        if (!scene) return;
        var existing = particles.find(function (p) { return p.key === key; });
        if (existing) {
            scene.remove(existing.points);
            existing.points.geometry.dispose();
            existing.points.material.dispose();
            particles = particles.filter(function (p) { return p.key !== key; });
        }
        try {
            var geo = new THREE.BufferGeometry();
            geo.setAttribute("position", new THREE.BufferAttribute(posArr, 3));
            geo.setAttribute("color", new THREE.BufferAttribute(colArr, 3));
            var mat = new THREE.PointsMaterial({
                size: 0.5,
                vertexColors: true,
                transparent: true,
                opacity: 0.6,
                blending: THREE.AdditiveBlending,
                depthWrite: false,
            });
            var points = new THREE.Points(geo, mat);
            scene.add(points);
            particles.push({ key: key, points: points });
        } catch (e) {
            console.error("[bridge] spawnParticles error:", e);
        }
    };

    window.threeBridgeRemoveParticles = function (key) {
        var found = particles.find(function (p) { return p.key === key; });
        if (found) {
            scene.remove(found.points);
            found.points.geometry.dispose();
            found.points.material.dispose();
            particles = particles.filter(function (p) { return p.key !== key; });
        }
    };

    var frameCount = 0;
    window.threeBridgeRender = function () {
        if (!renderer || !scene || !camera) return;
        frameCount++;

        if (waterMesh && waterMesh.material.uniforms) {
            waterMesh.material.uniforms.uTime.value = frameCount * 0.016;
        }

        renderer.render(scene, camera);
    };

    window.threeBridgeResize = function () {
        resizeRenderer();
    };

    window.threeBridgeClearAll = function () {
        for (var entry = meshes.entries(), item; !(item = entry.next()).done;) {
            var key = item.value[0], mesh = item.value[1];
            scene.remove(mesh);
            mesh.geometry.dispose();
            mesh.material.dispose();
        }
        meshes.clear();
    };

    window.threeBridgeCaptureScreenshot = function (seed, formula, zone, x, y, z) {
        if (!renderer) return;
        renderer.render(scene, camera);
        var link = document.createElement('a');
        link.download = 'worlds_' + seed + '_' + formula + '_' + zone + '_' + Math.round(x) + '_' + Math.round(z) + '.png';
        link.href = renderer.domElement.toDataURL('image/png');
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
    };

    // Vignette opacity set in init after element creation

    // ============================================================
    // AUDIO SYSTEM
    // ============================================================
    var audioCtx = null;
    var masterGain = null;
    var ambientNodes = [];
    var lastFootstepTime = 0;

    function createNoiseBuffer(ctx, duration) {
        var sr = ctx.sampleRate;
        var len = Math.floor(sr * duration);
        var buf = ctx.createBuffer(1, len, sr);
        var data = buf.getChannelData(0);
        for (var i = 0; i < len; i++) {
            data[i] = Math.random() * 2 - 1;
        }
        return buf;
    }

    function connectAndStart(node, destination, delay) {
        node.connect(destination);
        if (node.start) {
            try { node.start(delay || 0); } catch (e) {}
        }
    }

    function makeAmplitudeEnvelope(ctx, duration, attack, sustain, release) {
        var env = ctx.createGain();
        var now = ctx.currentTime;
        env.gain.setValueAtTime(0, now);
        env.gain.linearRampToValueAtTime(1, now + attack);
        env.gain.setValueAtTime(1, now + attack + sustain);
        env.gain.linearRampToValueAtTime(0, now + attack + sustain + release);
        return env;
    }

    function buildAmbient(ctx, zone) {
        var nodes = [];
        function connectChain(src) {
            var prev = src;
            for (var i = 1; i < arguments.length; i++) {
                prev.connect(arguments[i]);
                prev = arguments[i];
            }
            return prev;
        }

        try {
            switch (zone) {
                case "forest": {
                    // Wind: filtered noise
                    var buf = createNoiseBuffer(ctx, 4);
                    var src = ctx.createBufferSource();
                    src.buffer = buf;
                    src.loop = true;
                    var filt = ctx.createBiquadFilter();
                    filt.type = "lowpass";
                    filt.frequency.value = 800;
                    filt.Q.value = 0.5;
                    var lfo = ctx.createOscillator();
                    lfo.frequency.value = 0.15;
                    var lfoGain = ctx.createGain();
                    lfoGain.gain.value = 400;
                    lfo.connect(lfoGain);
                    lfoGain.connect(filt.frequency);
                    var amp = ctx.createGain();
                    amp.gain.value = 0.12;
                    connectChain(src, filt, amp, masterGain);
                    nodes.push(src, filt, lfo, lfoGain, amp);

                    // Bird chirps
                    for (var i = 0; i < 3; i++) {
                        var chirp = ctx.createOscillator();
                        chirp.type = "sine";
                        chirp.frequency.value = 1200 + i * 400 + Math.random() * 300;
                        var cAmp = ctx.createGain();
                        cAmp.gain.value = 0.02;
                        var cLfo = ctx.createOscillator();
                        cLfo.frequency.value = 4 + Math.random() * 2;
                        var cLfoG = ctx.createGain();
                        cLfoG.gain.value = 0.02;
                        cLfo.connect(cLfoG);
                        cLfoG.connect(cAmp.gain);
                        connectChain(chirp, cAmp, masterGain);
                        nodes.push(chirp, cAmp, cLfo, cLfoG);
                    }
                    break;
                }
                case "plains": {
                    var buf = createNoiseBuffer(ctx, 4);
                    var src = ctx.createBufferSource();
                    src.buffer = buf;
                    src.loop = true;
                    var filt = ctx.createBiquadFilter();
                    filt.type = "bandpass";
                    filt.frequency.value = 400;
                    filt.Q.value = 1.0;
                    var amp = ctx.createGain();
                    amp.gain.value = 0.08;
                    connectChain(src, filt, amp, masterGain);
                    nodes.push(src, filt, amp);
                    break;
                }
                case "desert": {
                    var buf = createNoiseBuffer(ctx, 4);
                    var src = ctx.createBufferSource();
                    src.buffer = buf;
                    src.loop = true;
                    var filt = ctx.createBiquadFilter();
                    filt.type = "highpass";
                    filt.frequency.value = 1500;
                    var amp = ctx.createGain();
                    amp.gain.value = 0.06;
                    connectChain(src, filt, amp, masterGain);
                    nodes.push(src, filt, amp);

                    // Occasional low wind gust
                    var gust = ctx.createOscillator();
                    gust.type = "sine";
                    gust.frequency.value = 60;
                    var gAmp = ctx.createGain();
                    gAmp.gain.value = 0.03;
                    var gLfo = ctx.createOscillator();
                    gLfo.frequency.value = 0.08;
                    var gLfoG = ctx.createGain();
                    gLfoG.gain.value = 0.03;
                    gLfo.connect(gLfoG);
                    gLfoG.connect(gAmp.gain);
                    connectChain(gust, gAmp, masterGain);
                    nodes.push(gust, gAmp, gLfo, gLfoG);
                    break;
                }
                case "tundra": {
                    var buf = createNoiseBuffer(ctx, 4);
                    var src = ctx.createBufferSource();
                    src.buffer = buf;
                    src.loop = true;
                    var filt = ctx.createBiquadFilter();
                    filt.type = "lowpass";
                    filt.frequency.value = 500;
                    var amp = ctx.createGain();
                    amp.gain.value = 0.15;
                    connectChain(src, filt, amp, masterGain);
                    nodes.push(src, filt, amp);

                    // Cold wind howl
                    var howl = ctx.createOscillator();
                    howl.type = "sine";
                    howl.frequency.value = 200;
                    var hAmp = ctx.createGain();
                    hAmp.gain.value = 0.04;
                    var hLfo = ctx.createOscillator();
                    hLfo.frequency.value = 0.1;
                    var hLfoG = ctx.createGain();
                    hLfoG.gain.value = 0.04;
                    hLfo.connect(hLfoG);
                    hLfoG.connect(hAmp.gain);
                    connectChain(howl, hAmp, masterGain);
                    nodes.push(howl, hAmp, hLfo, hLfoG);
                    break;
                }
                case "jungle": {
                    var buf = createNoiseBuffer(ctx, 4);
                    var src = ctx.createBufferSource();
                    src.buffer = buf;
                    src.loop = true;
                    var filt = ctx.createBiquadFilter();
                    filt.type = "lowpass";
                    filt.frequency.value = 1200;
                    var amp = ctx.createGain();
                    amp.gain.value = 0.1;
                    connectChain(src, filt, amp, masterGain);
                    nodes.push(src, filt, amp);

                    // Insect buzzes
                    for (var i = 0; i < 5; i++) {
                        var buzz = ctx.createOscillator();
                        buzz.type = "sawtooth";
                        buzz.frequency.value = 300 + i * 100;
                        var bAmp = ctx.createGain();
                        bAmp.gain.value = 0.015;
                        connectChain(buzz, bAmp, masterGain);
                        nodes.push(buzz, bAmp);
                    }
                    break;
                }
                case "volcanic":
                case "magma": {
                    var buf = createNoiseBuffer(ctx, 4);
                    var src = ctx.createBufferSource();
                    src.buffer = buf;
                    src.loop = true;
                    var filt = ctx.createBiquadFilter();
                    filt.type = "lowpass";
                    filt.frequency.value = 150;
                    var amp = ctx.createGain();
                    amp.gain.value = 0.2;
                    connectChain(src, filt, amp, masterGain);
                    nodes.push(src, filt, amp);

                    var rumble = ctx.createOscillator();
                    rumble.type = "sine";
                    rumble.frequency.value = 55;
                    var rAmp = ctx.createGain();
                    rAmp.gain.value = 0.15;
                    var rLfo = ctx.createOscillator();
                    rLfo.frequency.value = 0.3;
                    var rLfoG = ctx.createGain();
                    rLfoG.gain.value = 0.1;
                    rLfo.connect(rLfoG);
                    rLfoG.connect(rAmp.gain);
                    connectChain(rumble, rAmp, masterGain);
                    nodes.push(rumble, rAmp, rLfo, rLfoG);

                    // Crackle
                    var cBuf = createNoiseBuffer(ctx, 0.5);
                    var cSrc = ctx.createBufferSource();
                    cSrc.buffer = cBuf;
                    cSrc.loop = true;
                    var cFilt = ctx.createBiquadFilter();
                    cFilt.type = "highpass";
                    cFilt.frequency.value = 2000;
                    var cAmp2 = ctx.createGain();
                    cAmp2.gain.value = 0.06;
                    connectChain(cSrc, cFilt, cAmp2, masterGain);
                    nodes.push(cSrc, cFilt, cAmp2);
                    break;
                }
                case "ocean": {
                    var buf = createNoiseBuffer(ctx, 4);
                    var src = ctx.createBufferSource();
                    src.buffer = buf;
                    src.loop = true;
                    var filt = ctx.createBiquadFilter();
                    filt.type = "lowpass";
                    filt.frequency.value = 200;
                    var amp = ctx.createGain();
                    amp.gain.value = 0.18;
                    var lfo2 = ctx.createOscillator();
                    lfo2.frequency.value = 0.05;
                    var lfoG2 = ctx.createGain();
                    lfoG2.gain.value = 0.1;
                    lfo2.connect(lfoG2);
                    lfoG2.connect(amp.gain);
                    connectChain(src, filt, amp, masterGain);
                    nodes.push(src, filt, amp, lfo2, lfoG2);
                    break;
                }
                case "crystal": {
                    for (var i = 0; i < 6; i++) {
                        var ring = ctx.createOscillator();
                        ring.type = "sine";
                        ring.frequency.value = 800 + i * 300 + Math.random() * 100;
                        var rAmp = ctx.createGain();
                        rAmp.gain.value = 0.025;
                        var rLfo = ctx.createOscillator();
                        rLfo.frequency.value = 2 + Math.random() * 3;
                        var rLfoG = ctx.createGain();
                        rLfoG.gain.value = 0.02;
                        rLfo.connect(rLfoG);
                        rLfoG.connect(rAmp.gain);
                        connectChain(ring, rAmp, masterGain);
                        nodes.push(ring, rAmp, rLfo, rLfoG);
                    }
                    break;
                }
                case "cave": {
                    var buf = createNoiseBuffer(ctx, 4);
                    var src = ctx.createBufferSource();
                    src.buffer = buf;
                    src.loop = true;
                    var filt = ctx.createBiquadFilter();
                    filt.type = "lowpass";
                    filt.frequency.value = 300;
                    var amp = ctx.createGain();
                    amp.gain.value = 0.1;
                    connectChain(src, filt, amp, masterGain);
                    nodes.push(src, filt, amp);

                    var drone = ctx.createOscillator();
                    drone.type = "sine";
                    drone.frequency.value = 75;
                    var dAmp = ctx.createGain();
                    dAmp.gain.value = 0.08;
                    connectChain(drone, dAmp, masterGain);
                    nodes.push(drone, dAmp);

                    var drip2 = ctx.createOscillator();
                    drip2.type = "sine";
                    drip2.frequency.value = 1800;
                    var dripAmp = ctx.createGain();
                    dripAmp.gain.value = 0.005;
                    var dripLfo = ctx.createOscillator();
                    dripLfo.frequency.value = 0.2;
                    var dripLfoG = ctx.createGain();
                    dripLfoG.gain.value = 0.005;
                    dripLfo.connect(dripLfoG);
                    dripLfoG.connect(dripAmp.gain);
                    connectChain(drip2, dripAmp, masterGain);
                    nodes.push(drip2, dripAmp, dripLfo, dripLfoG);
                    break;
                }
                case "lava": {
                    var buf = createNoiseBuffer(ctx, 4);
                    var src = ctx.createBufferSource();
                    src.buffer = buf;
                    src.loop = true;
                    var filt = ctx.createBiquadFilter();
                    filt.type = "bandpass";
                    filt.frequency.value = 300;
                    filt.Q.value = 2;
                    var amp = ctx.createGain();
                    amp.gain.value = 0.15;
                    connectChain(src, filt, amp, masterGain);
                    nodes.push(src, filt, amp);

                    var bubble = ctx.createOscillator();
                    bubble.type = "sawtooth";
                    bubble.frequency.value = 100;
                    var bAmp = ctx.createGain();
                    bAmp.gain.value = 0.05;
                    var bLfo = ctx.createOscillator();
                    bLfo.frequency.value = 2;
                    var bLfoG = ctx.createGain();
                    bLfoG.gain.value = 0.04;
                    bLfo.connect(bLfoG);
                    bLfoG.connect(bAmp.gain);
                    connectChain(bubble, bAmp, masterGain);
                    nodes.push(bubble, bAmp, bLfo, bLfoG);
                    break;
                }
                case "fungus": {
                    var buf = createNoiseBuffer(ctx, 4);
                    var src = ctx.createBufferSource();
                    src.buffer = buf;
                    src.loop = true;
                    var filt = ctx.createBiquadFilter();
                    filt.type = "bandpass";
                    filt.frequency.value = 600;
                    filt.Q.value = 3;
                    var amp = ctx.createGain();
                    amp.gain.value = 0.12;
                    var fLfo = ctx.createOscillator();
                    fLfo.frequency.value = 0.4;
                    var fLfoG = ctx.createGain();
                    fLfoG.gain.value = 300;
                    fLfo.connect(fLfoG);
                    fLfoG.connect(filt.frequency);
                    connectChain(src, filt, amp, masterGain);
                    nodes.push(src, filt, amp, fLfo, fLfoG);

                    var pulse = ctx.createOscillator();
                    pulse.type = "sine";
                    pulse.frequency.value = 150;
                    var pAmp = ctx.createGain();
                    pAmp.gain.value = 0.03;
                    var pLfo = ctx.createOscillator();
                    pLfo.frequency.value = 0.5;
                    var pLfoG = ctx.createGain();
                    pLfoG.gain.value = 0.03;
                    pLfo.connect(pLfoG);
                    pLfoG.connect(pAmp.gain);
                    connectChain(pulse, pAmp, masterGain);
                    nodes.push(pulse, pAmp, pLfo, pLfoG);
                    break;
                }
                case "abyss": {
                    var buf = createNoiseBuffer(ctx, 4);
                    var src = ctx.createBufferSource();
                    src.buffer = buf;
                    src.loop = true;
                    var filt = ctx.createBiquadFilter();
                    filt.type = "lowpass";
                    filt.frequency.value = 100;
                    var amp = ctx.createGain();
                    amp.gain.value = 0.08;
                    connectChain(src, filt, amp, masterGain);
                    nodes.push(src, filt, amp);

                    var drone2 = ctx.createOscillator();
                    drone2.type = "sine";
                    drone2.frequency.value = 40;
                    var dAmp2 = ctx.createGain();
                    dAmp2.gain.value = 0.1;
                    connectChain(drone2, dAmp2, masterGain);
                    nodes.push(drone2, dAmp2);
                    break;
                }
                case "storm": {
                    var buf = createNoiseBuffer(ctx, 4);
                    var src = ctx.createBufferSource();
                    src.buffer = buf;
                    src.loop = true;
                    var filt = ctx.createBiquadFilter();
                    filt.type = "lowpass";
                    filt.frequency.value = 1000;
                    var amp = ctx.createGain();
                    amp.gain.value = 0.25;
                    var sLfo = ctx.createOscillator();
                    sLfo.frequency.value = 0.2;
                    var sLfoG = ctx.createGain();
                    sLfoG.gain.value = 0.15;
                    sLfo.connect(sLfoG);
                    sLfoG.connect(amp.gain);
                    connectChain(src, filt, amp, masterGain);
                    nodes.push(src, filt, amp, sLfo, sLfoG);

                    var thunder = ctx.createOscillator();
                    thunder.type = "sine";
                    thunder.frequency.value = 50;
                    var tAmp = ctx.createGain();
                    tAmp.gain.value = 0.12;
                    var tLfo = ctx.createOscillator();
                    tLfo.frequency.value = 0.07;
                    var tLfoG = ctx.createGain();
                    tLfoG.gain.value = 0.1;
                    tLfo.connect(tLfoG);
                    tLfoG.connect(tAmp.gain);
                    connectChain(thunder, tAmp, masterGain);
                    nodes.push(thunder, tAmp, tLfo, tLfoG);
                    break;
                }
                case "aurora": {
                    for (var i = 0; i < 4; i++) {
                        var ethereal = ctx.createOscillator();
                        ethereal.type = "sine";
                        ethereal.frequency.value = 200 + i * 150;
                        var eAmp = ctx.createGain();
                        eAmp.gain.value = 0.03;
                        var eLfo = ctx.createOscillator();
                        eLfo.frequency.value = 0.3 + i * 0.2;
                        var eLfoG = ctx.createGain();
                        eLfoG.gain.value = 0.03;
                        eLfo.connect(eLfoG);
                        eLfoG.connect(eAmp.gain);
                        connectChain(ethereal, eAmp, masterGain);
                        nodes.push(ethereal, eAmp, eLfo, eLfoG);
                    }
                    var buf4 = createNoiseBuffer(ctx, 4);
                    var src4 = ctx.createBufferSource();
                    src4.buffer = buf4;
                    src4.loop = true;
                    var filt4 = ctx.createBiquadFilter();
                    filt4.type = "bandpass";
                    filt4.frequency.value = 2000;
                    filt4.Q.value = 0.5;
                    var amp4 = ctx.createGain();
                    amp4.gain.value = 0.04;
                    connectChain(src4, filt4, amp4, masterGain);
                    nodes.push(src4, filt4, amp4);
                    break;
                }
                default: {
                    var buf = createNoiseBuffer(ctx, 4);
                    var src = ctx.createBufferSource();
                    src.buffer = buf;
                    src.loop = true;
                    var filt = ctx.createBiquadFilter();
                    filt.type = "lowpass";
                    filt.frequency.value = 600;
                    var amp = ctx.createGain();
                    amp.gain.value = 0.06;
                    connectChain(src, filt, amp, masterGain);
                    nodes.push(src, filt, amp);
                }
            }
        } catch (e) {
            console.warn("[audio] build error:", e);
        }
        return nodes;
    }

    function stopNodes(nodes) {
        for (var i = 0; i < nodes.length; i++) {
            try {
                nodes[i].disconnect();
                if (nodes[i].stop) nodes[i].stop();
            } catch (e) {}
        }
    }

    window.threeBridgeAudioInit = function () {
        try {
            audioCtx = new (window.AudioContext || window.webkitAudioContext)();
            masterGain = audioCtx.createGain();
            masterGain.gain.value = 0.3;
            masterGain.connect(audioCtx.destination);
        } catch (e) {
            console.warn("[audio] init error:", e);
        }
    };

    window.threeBridgeAudioPlayAmbient = function (zone) {
        if (!audioCtx || !masterGain) return;
        if (audioCtx.state === "suspended") {
            audioCtx.resume();
        }
        stopNodes(ambientNodes);
        ambientNodes = [];
        ambientNodes = buildAmbient(audioCtx, zone);
        for (var i = 0; i < ambientNodes.length; i++) {
            if (ambientNodes[i].start) {
                try { ambientNodes[i].start(); } catch (e) {}
            }
        }
    };

    window.threeBridgeAudioStopAmbient = function () {
        stopNodes(ambientNodes);
        ambientNodes = [];
    };

    window.threeBridgeAudioPlayFootstep = function (intensity) {
        if (!audioCtx || !masterGain) return;
        var now = audioCtx.currentTime;
        if (now - lastFootstepTime < 0.15) return;
        lastFootstepTime = now;
        try {
            var buf = createNoiseBuffer(audioCtx, 0.08);
            var src = audioCtx.createBufferSource();
            src.buffer = buf;
            var filt = audioCtx.createBiquadFilter();
            filt.type = "lowpass";
            filt.frequency.value = 800 + intensity * 500;
            var env = makeAmplitudeEnvelope(audioCtx, 0.08, 0.002, 0.01, 0.068);
            src.connect(filt);
            filt.connect(env);
            env.connect(masterGain);
            src.start();
            // Cleanup after done
            setTimeout(function () {
                try { src.disconnect(); filt.disconnect(); env.disconnect(); } catch (e) {}
            }, 200);
        } catch (e) {}
    };

    window.threeBridgeAudioPlayEffect = function (type) {
        if (!audioCtx || !masterGain) return;
        if (audioCtx.state === "suspended") audioCtx.resume();
        try {
            if (type === "formula" || type === "zone") {
                // Rising tone
                var osc = audioCtx.createOscillator();
                osc.type = "sine";
                osc.frequency.setValueAtTime(400, audioCtx.currentTime);
                osc.frequency.exponentialRampToValueAtTime(800, audioCtx.currentTime + 0.2);
                var env = makeAmplitudeEnvelope(audioCtx, 0.3, 0.01, 0.1, 0.19);
                osc.connect(env);
                env.connect(masterGain);
                osc.start();
                setTimeout(function () {
                    try { osc.disconnect(); env.disconnect(); } catch (e) {}
                }, 500);
            } else if (type === "click") {
                var osc2 = audioCtx.createOscillator();
                osc2.type = "sine";
                osc2.frequency.value = 600;
                var env2 = makeAmplitudeEnvelope(audioCtx, 0.05, 0.001, 0.01, 0.039);
                osc2.connect(env2);
                env2.connect(masterGain);
                osc2.start();
                setTimeout(function () {
                    try { osc2.disconnect(); env2.disconnect(); } catch (e) {}
                }, 100);
            }
        } catch (e) {}
    };

    window.threeBridgeAudioSetMasterVolume = function (vol) {
        if (masterGain) masterGain.gain.value = Math.clamp(vol, 0, 1);
    };

    // ============================================================
    // WEATHER SYSTEM
    // ============================================================
    var weatherActive = false;
    var weatherTimeout = null;

    window.threeBridgeSetWeather = function (type, intensity) {
        if (!scene) return;
        weatherActive = (type !== "none");

        // Set fog based on weather
        if (scene.fog) {
            switch (type) {
                case "rain":
                    scene.fog.color.setHSL(0.6, 0.2, 0.15);
                    scene.fog.density = 0.012 + intensity * 0.008;
                    break;
                case "snow":
                    scene.fog.color.setHSL(0, 0, 0.25);
                    scene.fog.density = 0.006 + intensity * 0.006;
                    break;
                case "storm":
                    scene.fog.color.setHSL(0.6, 0.1, 0.08);
                    scene.fog.density = 0.02 + intensity * 0.015;
                    break;
                case "dust":
                    scene.fog.color.setHSL(0.08, 0.3, 0.2);
                    scene.fog.density = 0.01 + intensity * 0.01;
                    break;
                case "ash":
                    scene.fog.color.setHSL(0.03, 0.1, 0.12);
                    scene.fog.density = 0.015 + intensity * 0.015;
                    break;
                default:
                    // Reset to time-of-day based fog
                    break;
            }
        }
    };

    window.threeBridgeClearWeather = function () {
        weatherActive = false;
        if (scene && scene.fog) {
            // Fog will be reset by next setTime call in game loop
        }
    };

    // ============================================================
    // BIOME TINT
    // ============================================================
    var biomeColors = {
        forest:   [0.0, 0.03, 0.0],
        plains:   [0.0, 0.01, 0.0],
        desert:   [0.04, 0.02, 0.0],
        tundra:   [0.0, 0.0, 0.02],
        jungle:   [0.0, 0.04, 0.0],
        volcanic: [0.04, 0.01, 0.0],
        ocean:    [0.0, 0.0, 0.03],
        crystal:  [0.01, 0.0, 0.04],
        cave:     [0.0, 0.0, 0.0],
        lava:     [0.05, 0.01, 0.0],
        fungus:   [0.03, 0.0, 0.03],
        abyss:    [0.0, 0.0, 0.0],
        storm:    [0.0, 0.0, 0.02],
        aurora:   [0.0, 0.02, 0.04],
        magma:    [0.04, 0.01, 0.0],
    };

    window.threeBridgeSetBiomeTint = function (biome) {
        if (!ambientLight) return;
        var tint = biomeColors[biome] || [0, 0, 0];
        ambientLight.color.setRGB(
            0.15 + tint[0] * 0.5,
            0.15 + tint[1] * 0.5,
            0.15 + tint[2] * 0.5
        );
        // Vignette subtle per biome
        var vg = document.getElementById('vignette-overlay');
        if (vg) {
            var darkBiomes = ['abyss', 'cave', 'storm'];
            var brightBiomes = ['desert', 'tundra', 'crystal'];
            if (darkBiomes.indexOf(biome) >= 0) {
                vg.style.opacity = '0.2';
            } else if (brightBiomes.indexOf(biome) >= 0) {
                vg.style.opacity = '0.1';
            } else {
                vg.style.opacity = '0.15';
            }
        }
    };

    // ============================================================
    // CINEMATIC POST-PROCESSING (Phase 3)
    // ============================================================

    // Post-processing (LUT, DoF, Heat Haze) disabled for color accuracy

    var vegetation = new Map(); // key -> { meshes: [], windData: [] }
    var vegGeos = {};

    function buildVegGeometries() {
        if (vegGeos.tree) return;

        // Tree: merged trunk + canopy
        var trunk = new THREE.CylinderGeometry(0.08, 0.12, 0.8, 5);
        trunk.translate(0, 0.6, 0);
        var canopy = new THREE.ConeGeometry(0.5, 0.9, 5);
        canopy.translate(0, 1.6, 0);
        vegGeos.tree = mergeBufferGeos([trunk, canopy]);

        // Bush
        vegGeos.bush = new THREE.SphereGeometry(0.4, 5, 5);
        vegGeos.bush.scale(1, 0.6, 1);

        // Rock
        vegGeos.rock = new THREE.IcosahedronGeometry(0.3, 0);
        vegGeos.rock = new THREE.IcosahedronGeometry(0.3, 1);

        // Cactus
        vegGeos.cactus = new THREE.CylinderGeometry(0.1, 0.15, 1.2, 6);
        vegGeos.cactus.translate(0, 0.6, 0);

        // Mushroom
        var stem = new THREE.CylinderGeometry(0.05, 0.08, 0.5, 5);
        stem.translate(0, 0.35, 0);
        var cap = new THREE.SphereGeometry(0.3, 5, 4);
        cap.scale(1, 0.3, 1);
        cap.translate(0, 0.7, 0);
        vegGeos.mushroom = mergeBufferGeos([stem, cap]);

        // Crystal spire
        vegGeos.crystal = new THREE.ConeGeometry(0.15, 0.8, 4);
        vegGeos.crystal.translate(0, 0.4, 0);

        // Dead tree
        vegGeos.deadTree = new THREE.CylinderGeometry(0.06, 0.1, 0.8, 4);
        vegGeos.deadTree.translate(0, 0.4, 0);

        // Flower
        vegGeos.flower = new THREE.SphereGeometry(0.08, 4, 4);
    }

    function mergeBufferGeos(geos) {
        var totalVerts = 0, totalIdx = 0;
        for (var i = 0; i < geos.length; i++) {
            totalVerts += geos[i].getAttribute('position').count;
            totalIdx += geos[i].index.count;
        }
        var pos = new Float32Array(totalVerts * 3);
        var idx = new (totalVerts > 65535 ? Uint32Array : Uint16Array)(totalIdx);
        var vertOffset = 0, idxOffset = 0;
        for (var i = 0; i < geos.length; i++) {
            var g = geos[i];
            var p = g.getAttribute('position').array;
            pos.set(p, vertOffset * 3);
            var ind = g.index.array;
            for (var j = 0; j < ind.length; j++) {
                idx[idxOffset + j] = ind[j] + vertOffset;
            }
            vertOffset += g.getAttribute('position').count;
            idxOffset += ind.length;
        }
        var geo = new THREE.BufferGeometry();
        geo.setAttribute('position', new THREE.BufferAttribute(pos, 3));
        geo.setIndex(new THREE.BufferAttribute(idx, 1));
        geo.computeVertexNormals();
        return geo;
    }

    var vegColors = {
        tree:     [new THREE.Color(0x4a7a3a), new THREE.Color(0x6a9a4a), new THREE.Color(0x8aba5a)],
        bush:     [new THREE.Color(0x5a8a3a), new THREE.Color(0x7aaa4a)],
        rock:     [new THREE.Color(0x888888), new THREE.Color(0x666666), new THREE.Color(0x999999)],
        cactus:   [new THREE.Color(0x4a8a3a), new THREE.Color(0x5a9a4a)],
        mushroom: [new THREE.Color(0xaa66aa), new THREE.Color(0xcc88cc), new THREE.Color(0x884488)],
        crystal:  [new THREE.Color(0x88aaff), new THREE.Color(0xaa88ff), new THREE.Color(0x66ccff)],
        deadTree: [new THREE.Color(0x554433), new THREE.Color(0x443322)],
        flower:   [new THREE.Color(0xff6688), new THREE.Color(0xffaa44), new THREE.Color(0xff4488)],
    };

    function getVegTypeName(type) {
        var names = ['tree', 'bush', 'rock', 'cactus', 'mushroom', 'crystal', 'deadTree', 'flower'];
        return names[type] || 'rock';
    }

    window.threeBridgeSpawnVegetation = function (key, posArr, sizeArr, typeArr, count, zone) {
        removeVegetation(key);
        if (count === 0) return;
        buildVegGeometries();

        // Group instances by type for InstancedMesh
        var byType = {};
        for (var i = 0; i < count; i++) {
            var t = typeArr[i];
            if (!byType[t]) byType[t] = [];
            byType[t].push({
                pos: [posArr[i*3], posArr[i*3+1], posArr[i*3+2]],
                size: sizeArr[i],
            });
        }

        var meshes = [];
        var windData = [];
        var dummy = new THREE.Object3D();

        for (var typeId in byType) {
            var typeName = getVegTypeName(parseInt(typeId));
            var geo = vegGeos[typeName];
            if (!geo) continue;

            var cols = vegColors[typeName] || [new THREE.Color(0x888888)];
            var inst = byType[typeId];
            var im = new THREE.InstancedMesh(geo, new THREE.MeshLambertMaterial({
                color: cols[0],
                flatShading: true,
            }), inst.length);
            im.receiveShadow = true;
            im.castShadow = true;

            for (var j = 0; j < inst.length; j++) {
                var p = inst[j];
                var s = p.size * (0.8 + Math.random() * 0.4);
                dummy.position.set(p.pos[0], p.pos[1], p.pos[2]);
                dummy.scale.set(s, s, s);
                dummy.rotation.set(0, Math.random() * Math.PI * 2, 0);
                dummy.updateMatrix();
                im.setMatrixAt(j, dummy.matrix);

                // Store wind data per instance
                windData.push({
                    idx: meshes.length + j,
                    meshIdx: meshes.length,
                    windPhase: Math.random() * Math.PI * 2,
                    windAmp: 0.02 + Math.random() * 0.04,
                    basePos: [p.pos[0], p.pos[1], p.pos[2]],
                    baseRot: dummy.rotation.y,
                });
            }
            im.instanceMatrix.needsUpdate = true;
            scene.add(im);
            meshes.push(im);
        }

        vegetation.set(key, { meshes: meshes, windData: windData });
    };

    function removeVegetation(key) {
        var entry = vegetation.get(key);
        if (entry) {
            for (var i = 0; i < entry.meshes.length; i++) {
                scene.remove(entry.meshes[i]);
                entry.meshes[i].geometry.dispose();
                entry.meshes[i].material.dispose();
            }
            vegetation.delete(key);
        }
    }

    window.threeBridgeRemoveVegetation = function (key) {
        removeVegetation(key);
    };

    window.threeBridgeUpdateWind = function (time) {
        var dummy = new THREE.Object3D();
        for (var entry of vegetation.values()) {
            if (!entry.windData || entry.windData.length === 0) continue;
            var meshes = entry.meshes;
            for (var wd of entry.windData) {
                var im = meshes[wd.meshIdx];
                if (!im) continue;
                // Reconstruct matrix from stored data
                var sway = Math.sin(time * 1.5 + wd.windPhase) * wd.windAmp;
                dummy.position.set(wd.basePos[0], wd.basePos[1], wd.basePos[2]);
                dummy.scale.set(1, 1, 1);
                dummy.rotation.set(sway, wd.baseRot + sway * 0.2, sway * 0.3);
                dummy.updateMatrix();
                im.setMatrixAt(wd.idx, dummy.matrix);
                im.instanceMatrix.needsUpdate = true;
            }
        }
    };

    // ============================================================
    // STRUCTURE SYSTEM
    // ============================================================
    var structures = new Map();
    var structGeos = {};

    function buildStructGeometries() {
        if (structGeos.hut) return;

        // Hut: box body + cone roof
        var hutBody = new THREE.BoxGeometry(1.2, 0.8, 1.2);
        hutBody.translate(0, 0.4, 0);
        var hutRoof = new THREE.ConeGeometry(0.9, 0.8, 4);
        hutRoof.translate(0, 1.0, 0);
        structGeos.hut = mergeBufferGeos([hutBody, hutRoof]);

        // Tower: tall box + crenellation
        var twBase = new THREE.BoxGeometry(0.6, 2.0, 0.6);
        twBase.translate(0, 1.0, 0);
        var twTop = new THREE.BoxGeometry(0.7, 0.2, 0.7);
        twTop.translate(0, 2.2, 0);
        structGeos.tower = mergeBufferGeos([twBase, twTop]);

        // Ruins: U-shape walls
        var rWall1 = new THREE.BoxGeometry(1.2, 0.6, 0.1);
        rWall1.translate(0, 0.3, -0.55);
        var rWall2 = new THREE.BoxGeometry(0.1, 0.6, 1.2);
        rWall2.translate(-0.55, 0.3, 0);
        var rWall3 = new THREE.BoxGeometry(0.1, 0.6, 1.2);
        rWall3.translate(0.55, 0.3, 0);
        structGeos.ruins = mergeBufferGeos([rWall1, rWall2, rWall3]);

        // Arch: two pillars + beam
        var aPillar = new THREE.BoxGeometry(0.12, 0.8, 0.12);
        aPillar.translate(-0.4, 0.4, 0);
        var aPillar2 = new THREE.BoxGeometry(0.12, 0.8, 0.12);
        aPillar2.translate(0.4, 0.4, 0);
        var aBeam = new THREE.BoxGeometry(0.92, 0.1, 0.12);
        aBeam.translate(0, 0.85, 0);
        structGeos.arch = mergeBufferGeos([aPillar, aPillar2, aBeam]);

        // Pillar: tall cylinder
        structGeos.pillar = new THREE.CylinderGeometry(0.15, 0.2, 1.5, 6);
        structGeos.pillar.translate(0, 0.75, 0);

        // Dome: hemisphere (approximate with sphere)
        structGeos.dome = new THREE.SphereGeometry(0.8, 8, 6, 0, Math.PI * 2, 0, Math.PI / 2);
        structGeos.dome.scale(1, 0.5, 1);

        // Pyramid
        structGeos.pyramid = new THREE.ConeGeometry(0.8, 1.2, 4);
        structGeos.pyramid.translate(0, 0.6, 0);

        // Crystal spire
        structGeos.crystalSpire = new THREE.ConeGeometry(0.2, 1.8, 5);
        structGeos.crystalSpire.translate(0, 0.9, 0);

        // Mushroom hut
        var mStem = new THREE.CylinderGeometry(0.2, 0.3, 0.6, 6);
        mStem.translate(0, 0.3, 0);
        var mCap = new THREE.SphereGeometry(0.7, 6, 5);
        mCap.scale(1, 0.35, 1);
        mCap.translate(0, 0.7, 0);
        structGeos.mushroomHut = mergeBufferGeos([mStem, mCap]);

        // Obelisk: tall thin pyramid
        structGeos.obelisk = new THREE.ConeGeometry(0.15, 1.5, 4);
        structGeos.obelisk.translate(0, 0.75, 0);
    }

    var structColorSets = {
        hut:         [0x8B5E3C, 0xA0703C, 0x6B4226],
        tower:       [0x7A7A7A, 0x8A8A8A, 0x6A6A6A],
        ruins:       [0x9A8A7A, 0x8A7A6A, 0xAA9A8A],
        arch:        [0xCCCCCC, 0xAAAAAA, 0xDDDDDD],
        pillar:      [0x555555, 0x666666, 0x777777],
        dome:        [0xCCEEFF, 0xCCDDEE, 0xBBCCDD],
        pyramid:     [0xD4A853, 0xC49A44, 0xE4B862],
        crystalSpire:[0x88AAFF, 0xAA88FF, 0x66CCFF],
        mushroomHut: [0xAA66AA, 0xCC88CC, 0x884488],
        obelisk:     [0x444444, 0x555555, 0x666666],
    };

    function getStructTypeName(type) {
        var names = ['hut', 'tower', 'ruins', 'arch', 'pillar', 'dome', 'pyramid', 'crystalSpire', 'mushroomHut', 'obelisk'];
        return names[type] || 'hut';
    }

    window.threeBridgeSpawnStructure = function (key, structArr, count, zone) {
        removeStructureGroup(key);
        if (count === 0) return;
        buildStructGeometries();

        var group = new THREE.Group();

        for (var i = 0; i < count; i++) {
            var x = structArr[i * 6];
            var y = structArr[i * 6 + 1];
            var z = structArr[i * 6 + 2];
            var rot = structArr[i * 6 + 3];
            var scale = structArr[i * 6 + 4];
            var type = Math.round(structArr[i * 6 + 5]);

            var typeName = getStructTypeName(type);
            var geo = structGeos[typeName];
            if (!geo) continue;

            var colors = structColorSets[typeName] || [0x888888];
            var color = colors[i % colors.length];

            var mat = new THREE.MeshLambertMaterial({
                color: color,
                flatShading: true,
            });
            var mesh = new THREE.Mesh(geo, mat);
            mesh.position.set(x, y, z);
            mesh.rotation.y = rot;
            mesh.scale.set(scale, scale, scale);
            mesh.castShadow = true;
            mesh.receiveShadow = true;
            group.add(mesh);

            // Register hidden structures for discovery (type 3 = Arch, type 7 = CrystalSpire)
            if (type === 3 || type === 7) {
                window.threeBridgeRegisterHidden(x, y, z, 'Ancient ' + typeName);
            }
        }

        scene.add(group);
        structures.set(key, group);
    };

    function removeStructureGroup(key) {
        var group = structures.get(key);
        if (group) {
            scene.remove(group);
            group.traverse(function (child) {
                if (child.isMesh) {
                    child.geometry.dispose();
                    child.material.dispose();
                }
            });
            structures.delete(key);
        }
    }

    window.threeBridgeRemoveStructure = function (key) {
        removeStructureGroup(key);
    };

    // ============================================================
    // MINERAL SYSTEM
    // ============================================================
    var minerals = new Map();
    var mineralColors = [
        0x44aa88, // cave emerald
        0x8888cc, // cave sapphire
        0xcc8844, // cave copper
        0x88aaff, // crystal quartz
        0xaa88ff, // crystal amethyst
        0xff6644, // volcanic ruby
        0xffaa33, // volcanic topaz
        0xcc88cc, // fungus pearl
    ];

    window.threeBridgeSpawnMinerals = function (key, minArr, count) {
        removeMineralGroup(key);
        if (count === 0) return;

        var group = new THREE.Group();
        var geo = new THREE.OctahedronGeometry(0.15, 0);

        for (var i = 0; i < count; i++) {
            var x = minArr[i * 5];
            var y = minArr[i * 5 + 1];
            var z = minArr[i * 5 + 2];
            var type = Math.round(minArr[i * 5 + 3]);
            var size = minArr[i * 5 + 4];

            var color = mineralColors[type % mineralColors.length];
            var mat = new THREE.MeshLambertMaterial({
                color: color,
                emissive: color,
                emissiveIntensity: 0.3,
                flatShading: true,
            });
            var mesh = new THREE.Mesh(geo, mat);
            mesh.position.set(x, y, z);
            mesh.scale.set(size, size * (1.2 + Math.random() * 0.5), size);
            mesh.rotation.set(Math.random() * 6, Math.random() * 6, Math.random() * 6);
            mesh.castShadow = true;
            group.add(mesh);
        }

        scene.add(group);
        minerals.set(key, group);
    };

    function removeMineralGroup(key) {
        var group = minerals.get(key);
        if (group) {
            scene.remove(group);
            group.traverse(function (child) {
                if (child.isMesh) {
                    child.geometry.dispose();
                    child.material.dispose();
                }
            });
            minerals.delete(key);
        }
    }

    window.threeBridgeRemoveMinerals = function (key) {
        removeMineralGroup(key);
    };

    // ============================================================
    // EXPORT + SAVE/LOAD
    // ============================================================

    function downloadBlob(content, filename, mimeType) {
        var blob = new Blob([content], {type: mimeType});
        var url = URL.createObjectURL(blob);
        var a = document.createElement('a');
        a.href = url;
        a.download = filename;
        a.click();
        URL.revokeObjectURL(url);
    }

    window.threeBridgeExportOBJ = function () {
        if (!scene) return;
        var vertices = [];
        var faces = [];
        var norms = [];
        var offset = 0;

        scene.traverse(function (child) {
            if (!child.isMesh || !child.geometry) return;
            var geo = child.geometry;
            var pos = geo.getAttribute('position');
            var idx = geo.index;
            if (!pos) return;

            var v = pos.array;
            for (var i = 0; i < v.length; i += 3) {
                var p = new THREE.Vector3(v[i], v[i+1], v[i+2]);
                p.applyMatrix4(child.matrixWorld);
                vertices.push('v ' + p.x + ' ' + p.y + ' ' + p.z);
            }

            if (geo.getAttribute('normal')) {
                var n = geo.getAttribute('normal').array;
                for (var i = 0; i < n.length; i += 3) {
                    var np = new THREE.Vector3(n[i], n[i+1], n[i+2]);
                    np.applyQuaternion(child.quaternion);
                    norms.push('vn ' + np.x + ' ' + np.y + ' ' + np.z);
                }
            }

            if (idx) {
                var ind = idx.array;
                for (var i = 0; i < ind.length; i += 3) {
                    faces.push('f ' + (ind[i]+1+offset) + '//' + (ind[i]+1+offset) +
                               ' ' + (ind[i+1]+1+offset) + '//' + (ind[i+1]+1+offset) +
                               ' ' + (ind[i+2]+1+offset) + '//' + (ind[i+2]+1+offset));
                }
            }
            offset += pos.count;
        });

        var obj = '# WORLDS Export\n' +
                  vertices.join('\n') + '\n' +
                  norms.join('\n') + '\n' +
                  faces.join('\n');
        downloadBlob(obj, 'worlds_export.obj', 'text/plain');
    };

    window.threeBridgeExportSTL = function () {
        if (!scene) return;
        var triangles = [];

        scene.traverse(function (child) {
            if (!child.isMesh || !child.geometry) return;
            var geo = child.geometry;
            var pos = geo.getAttribute('position');
            var idx = geo.index;
            if (!pos) return;

            var v = pos.array;
            var ind = idx ? idx.array : null;

            function getVertex(i) {
                var p = new THREE.Vector3(v[i*3], v[i*3+1], v[i*3+2]);
                p.applyMatrix4(child.matrixWorld);
                return p;
            }

            if (ind) {
                for (var i = 0; i < ind.length; i += 3) {
                    var a = getVertex(ind[i]);
                    var b = getVertex(ind[i+1]);
                    var c = getVertex(ind[i+2]);
                    var edge1 = new THREE.Vector3().subVectors(b, a);
                    var edge2 = new THREE.Vector3().subVectors(c, a);
                    var n = new THREE.Vector3().crossVectors(edge1, edge2).normalize();
                    triangles.push({n: n, a: a, b: b, c: c});
                }
            } else {
                for (var i = 0; i < pos.count; i += 3) {
                    var a = getVertex(i);
                    var b = getVertex(i+1);
                    var c = getVertex(i+2);
                    var edge1 = new THREE.Vector3().subVectors(b, a);
                    var edge2 = new THREE.Vector3().subVectors(c, a);
                    var n = new THREE.Vector3().crossVectors(edge1, edge2).normalize();
                    triangles.push({n: n, a: a, b: b, c: c});
                }
            }
        });

        // Binary STL
        var header = new Uint8Array(80);
        var count = triangles.length;
        var data = new Uint8Array(84 + count * 50);
        data.set(header, 0);
        var dv = new DataView(data.buffer);
        dv.setUint32(80, count, true);
        var off = 84;
        for (var i = 0; i < count; i++) {
            var t = triangles[i];
            dv.setFloat32(off, t.n.x, true); off += 4;
            dv.setFloat32(off, t.n.y, true); off += 4;
            dv.setFloat32(off, t.n.z, true); off += 4;
            dv.setFloat32(off, t.a.x, true); off += 4;
            dv.setFloat32(off, t.a.y, true); off += 4;
            dv.setFloat32(off, t.a.z, true); off += 4;
            dv.setFloat32(off, t.b.x, true); off += 4;
            dv.setFloat32(off, t.b.y, true); off += 4;
            dv.setFloat32(off, t.b.z, true); off += 4;
            dv.setFloat32(off, t.c.x, true); off += 4;
            dv.setFloat32(off, t.c.y, true); off += 4;
            dv.setFloat32(off, t.c.z, true); off += 4;
            dv.setUint16(off, 0, true); off += 2; // attribute byte count
        }
        downloadBlob(data, 'worlds_export.stl', 'application/octet-stream');
    };

    // ============================================================
    // MINING SYSTEM
    // ============================================================
    var mineTarget = null;
    var raycaster = new THREE.Raycaster();

    window.threeBridgeMineAt = function (screenX, screenY) {
        if (!camera || !scene) return null;
        raycaster.setFromCamera(new THREE.Vector2(screenX, screenY), camera);
        var intersects = raycaster.intersectObjects(scene.children, true);
        for (var i = 0; i < intersects.length; i++) {
            var obj = intersects[i].object;
            if (obj.isMesh && obj.geometry && obj.geometry.getAttribute('position')) {
                var pos = obj.position.clone();
                return { x: pos.x, y: pos.y, z: pos.z };
            }
        }
        return null;
    };

    // ============================================================
    // HIDDEN STRUCTURES
    // ============================================================
    var foundStructures = [];

    window.threeBridgeRegisterHidden = function (x, y, z, name) {
        foundStructures.push({x: x, y: y, z: z, name: name, found: false});
    };

    window.threeBridgeCheckDiscovery = function (px, py, pz) {
        var radius = 5;
        for (var i = 0; i < foundStructures.length; i++) {
            var s = foundStructures[i];
            if (!s.found) {
                var dx = s.x - px;
                var dy = s.y - py;
                var dz = s.z - pz;
                var dist = Math.sqrt(dx*dx + dy*dy + dz*dz);
                if (dist < radius) {
                    s.found = true;
                    return s.name || 'Hidden Structure';
                }
            }
        }
        return '';
    };

    // ============================================================
    // END
    // ============================================================

    // Add clamp polyfill for older browsers
    if (!Math.clamp) {
        Math.clamp = function (v, min, max) { return Math.min(Math.max(v, min), max); };
    }

    // ============================================================
    // MULTIPLAYER (Phase 4)
    // ============================================================
    var ws = null;
    var wsConnected = false;
    var wsPlayerId = '';
    var remotePlayers = new Map(); // id -> { mesh, target, current }

    function createRemotePlayerMesh(name) {
        var group = new THREE.Group();

        // Capsule-like body: cylinder + 2 hemispheres
        var bodyGeo = new THREE.CylinderGeometry(0.3, 0.3, 0.6, 8);
        bodyGeo.translate(0, 0.3, 0);
        var bodyMat = new THREE.MeshLambertMaterial({ color: 0x4488ff, flatShading: true });
        var body = new THREE.Mesh(bodyGeo, bodyMat);
        body.castShadow = true;
        group.add(body);

        // Head
        var headGeo = new THREE.SphereGeometry(0.2, 6, 6);
        headGeo.translate(0, 0.7, 0);
        var headMat = new THREE.MeshLambertMaterial({ color: 0xffccaa, flatShading: true });
        var head = new THREE.Mesh(headGeo, headMat);
        head.castShadow = true;
        group.add(head);

        // Name indicator (small sphere on top)
        var pingGeo = new THREE.SphereGeometry(0.04, 4, 4);
        var pingMat = new THREE.MeshBasicMaterial({ color: 0x22ff88 });
        var ping = new THREE.Mesh(pingGeo, pingMat);
        ping.position.y = 0.9;
        group.add(ping);

        // Glow ring at feet
        var ringGeo = new THREE.RingGeometry(0.2, 0.35, 16);
        var ringMat = new THREE.MeshBasicMaterial({ color: 0x4488ff, transparent: true, opacity: 0.3, side: THREE.DoubleSide });
        var ring = new THREE.Mesh(ringGeo, ringMat);
        ring.rotation.x = -Math.PI / 2;
        ring.position.y = 0.02;
        group.add(ring);

        group.visible = false;
        scene.add(group);
        return group;
    }

    function updateRemotePlayerPositions() {
        for (var entry of remotePlayers.entries()) {
            var id = entry[0];
            var rp = entry[1];
            if (!rp.mesh.visible) continue;

            // Interpolate
            var lerpFactor = 0.15;
            rp.current.x += (rp.target.x - rp.current.x) * lerpFactor;
            rp.current.y += (rp.target.y - rp.current.y) * lerpFactor;
            rp.current.z += (rp.target.z - rp.current.z) * lerpFactor;
            rp.current.yaw += (rp.target.yaw - rp.current.yaw) * lerpFactor;
            rp.current.pitch += (rp.target.pitch - rp.current.pitch) * lerpFactor;

            rp.mesh.position.set(rp.current.x, rp.current.y, rp.current.z);
            rp.mesh.rotation.y = rp.current.yaw;
        }
    }

    window.threeBridgeWsConnect = function (seed) {
        if (ws) return;
        var protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        var host = window.location.host;
        var url = protocol + '//' + host + '/ws';

        try {
            ws = new WebSocket(url);
            ws.binaryType = 'arraybuffer';

            ws.onopen = function () {
                wsConnected = true;
                ws.send(JSON.stringify({ type: 'join', seed: seed }));
                console.log('[mp] Connected to', url);
            };

            ws.onmessage = function (event) {
                try {
                    var msg = JSON.parse(event.data);
                    switch (msg.type) {
                        case 'welcome':
                            wsPlayerId = msg.your_id;
                            console.log('[mp] Welcome! ID:', wsPlayerId.slice(0, 6), 'Players:', msg.count);
                            break;
                        case 'pos':
                            for (var i = 0; i < msg.players.length; i++) {
                                var p = msg.players[i];
                                if (p.id === wsPlayerId) continue;
                                if (!remotePlayers.has(p.id)) {
                                    var mesh = createRemotePlayerMesh(p.name || 'Player');
                                    remotePlayers.set(p.id, {
                                        mesh: mesh,
                                        target: { x: p.x, y: p.y, z: p.z, yaw: p.yaw, pitch: p.pitch },
                                        current: { x: p.x, y: p.y, z: p.z, yaw: p.yaw, pitch: p.pitch },
                                    });
                                }
                                var rp = remotePlayers.get(p.id);
                                rp.target.x = p.x;
                                rp.target.y = p.y;
                                rp.target.z = p.z;
                                rp.target.yaw = p.yaw;
                                rp.target.pitch = p.pitch;
                                rp.mesh.visible = true;
                            }
                            break;
                        case 'leave':
                            var leftId = msg.your_id;
                            if (remotePlayers.has(leftId)) {
                                var rp = remotePlayers.get(leftId);
                                scene.remove(rp.mesh);
                                rp.mesh.traverse(function (c) { if (c.isMesh) { c.geometry.dispose(); c.material.dispose(); } });
                                remotePlayers.delete(leftId);
                            }
                            break;
                    }
                } catch (e) {
                    console.warn('[mp] msg error:', e);
                }
            };

            ws.onclose = function () {
                wsConnected = false;
                ws = null;
                // Cleanup all remote players
                for (var entry of remotePlayers.entries()) {
                    scene.remove(entry[1].mesh);
                    entry[1].mesh.traverse(function (c) { if (c.isMesh) { c.geometry.dispose(); c.material.dispose(); } });
                }
                remotePlayers.clear();
                console.log('[mp] Disconnected');
            };

            ws.onerror = function (e) {
                console.warn('[mp] Error:', e);
            };
        } catch (e) {
            console.warn('[mp] Connection failed:', e);
        }
    };

    window.threeBridgeWsSendPosition = function (x, y, z, yaw, pitch) {
        if (!ws || ws.readyState !== WebSocket.OPEN) return;
        ws.send(JSON.stringify({
            type: 'pos',
            x: x, y: y, z: z,
            yaw: yaw, pitch: pitch,
        }));
    };

    window.threeBridgeWsDisconnect = function () {
        if (ws) {
            ws.close();
            ws = null;
        }
    };

    // ============================================================
    // SAVE/LOAD (Phase 5)
    // ============================================================

    window.threeBridgeSaveSlot = function (slot, json) {
        try {
            localStorage.setItem('worlds_save_' + slot, json);
        } catch (e) {
            console.warn('[save] localStorage error:', e);
        }
    };

    window.threeBridgeLoadSlot = function (slot) {
        try {
            return localStorage.getItem('worlds_save_' + slot) || '';
        } catch (e) {
            return '';
        }
    };

    window.threeBridgeDeleteSlot = function (slot) {
        try {
            localStorage.removeItem('worlds_save_' + slot);
        } catch (e) {}
    };

    // ============================================================
    // F6: MINING & BUILDING SYSTEM
    // ============================================================

    var placedBlocks = new Map(); // "x,y,z" -> blockType
    var removedBlocks = new Set(); // "x,y,z" removed from terrain
    var blockGeo = new THREE.BoxGeometry(0.9, 0.9, 0.9);
    var blockMeshes = new Map();

    function getBlockKey(x, y, z) { return Math.round(x)+','+Math.round(y)+','+Math.round(z); }

    function createBlockMesh(x, y, z, type) {
        var colors = [
            0x998866, 0x888888, 0x885533, 0x339933, 0x9966ff, 0xcc5500, 0x88ccff, 0xd4b87a, 0x558833
        ];
        var mat = new THREE.MeshLambertMaterial({ color: colors[type] || 0x888888, flatShading: true });
        var mesh = new THREE.Mesh(blockGeo, mat);
        mesh.position.set(Math.round(x) + 0.5, Math.round(y) + 0.5, Math.round(z) + 0.5);
        mesh.castShadow = true;
        mesh.receiveShadow = true;
        scene.add(mesh);
        return mesh;
    }

    window.threeBridgeMineAt = function (ox, oy, oz, yaw, pitch) {
        // Raycast from center of screen
        var dir = new THREE.Vector3(0, 0, -1);
        var euler = new THREE.Euler(pitch, yaw, 0, 'YXZ');
        dir.applyEuler(euler);
        var origin = new THREE.Vector3(ox, oy, oz);
        var raycaster = new THREE.Raycaster(origin, dir, 1, 8);
        var targets = [];
        // Collect all block meshes + terrain meshes
        for (var entry of meshes.entries()) {
            targets.push(entry[1]);
        }
        for (var bm of blockMeshes.values()) {
            targets.push(bm);
        }
        var hits = raycaster.intersectObjects(targets);
        if (hits.length > 0) {
            var hit = hits[0];
            var p = hit.point;
            var nx = Math.floor(p.x);
            var ny = Math.floor(p.y);
            var nz = Math.floor(p.z);
            // Check if it's a placed block
            var bk = getBlockKey(nx, ny, nz);
            if (blockMeshes.has(bk)) {
                var mesh = blockMeshes.get(bk);
                scene.remove(mesh);
                mesh.geometry && mesh.geometry.dispose();
                mesh.material && mesh.material.dispose();
                blockMeshes.delete(bk);
                placedBlocks.delete(bk);
                return nx + ny * 10000 + nz * 100000000;
            }
            // Remove from terrain
            removedBlocks.add(bk);
            return 1;
        }
        return -1;
    };

    window.threeBridgePlaceBlock = function (ox, oy, oz, yaw, pitch, blockType) {
        var dir = new THREE.Vector3(0, 0, -1);
        var euler = new THREE.Euler(pitch, yaw, 0, 'YXZ');
        dir.applyEuler(euler);
        var origin = new THREE.Vector3(ox, oy, oz);
        var raycaster = new THREE.Raycaster(origin, dir, 1, 8);
        var targets = [];
        for (var entry of meshes.entries()) {
            targets.push(entry[1]);
        }
        for (var bm of blockMeshes.values()) {
            targets.push(bm);
        }
        var hits = raycaster.intersectObjects(targets);
        if (hits.length > 0) {
            var hit = hits[0];
            var normal = hit.face.normal.clone();
            // Place block adjacent to hit face
            var px = Math.floor(hit.point.x + normal.x * 0.5);
            var py = Math.floor(hit.point.y + normal.y * 0.5);
            var pz = Math.floor(hit.point.z + normal.z * 0.5);
            var bk = getBlockKey(px, py, pz);
            if (!blockMeshes.has(bk) && !removedBlocks.has(bk)) {
                createBlockMesh(px, py, pz, blockType);
                blockMeshes.set(bk, blockMeshes.get(bk) || createBlockMesh(px, py, pz, blockType));
                placedBlocks.set(bk, blockType);
                return true;
            }
        }
        return false;
    };

    window.threeBridgeGetBlocks = function () {
        return JSON.stringify({ placed: Array.from(placedBlocks.entries()), removed: Array.from(removedBlocks) });
    };

    window.threeBridgeSetBlocks = function (json) {
        try {
            var data = JSON.parse(json);
            if (data.placed) {
                for (var entry of data.placed) {
                    var parts = entry[0].split(',');
                    createBlockMesh(parseFloat(parts[0]), parseFloat(parts[1]), parseFloat(parts[2]), entry[1]);
                    blockMeshes.set(entry[0], blockMeshes.get(entry[0]));
                    placedBlocks.set(entry[0], entry[1]);
                }
            }
            if (data.removed) {
                for (var k of data.removed) { removedBlocks.add(k); }
            }
        } catch(e) {}
    };

    // ============================================================
    // F8: SEASONS
    // ============================================================

    var seasonFoliageColors = [
        { tree: 0x44aa44, grass: 0x55bb55 },  // spring
        { tree: 0x338833, grass: 0x44aa44 },  // summer
        { tree: 0xcc6633, grass: 0xaa7733 },  // autumn
        { tree: 0x666688, grass: 0x888899 },  // winter
    ];

    window.threeBridgeSetSeason = function (season) {
        document.body.dataset.season = season;
        console.log('[season]', season);
    };

    window.threeBridgeSetTreeGrowth = function (key, growth) {
        var veg = vegetation.get(key);
        if (!veg) return;
        for (var i = 0; i < veg.meshes.length; i++) {
            var mesh = veg.meshes[i];
            if (mesh && mesh.isInstancedMesh) {
                mesh.scale.setScalar(0.5 + growth * 0.5);
            }
        }
    };

    // ============================================================
    // F9: CREATURES
    // ============================================================

    var creatureMeshes = new Map(); // id -> { mesh, target, current, type }

    function createCreatureMesh(type, biome) {
        var group = new THREE.Group();
        var bodyColor = 0x88aa44;
        switch (type) {
            case 0: bodyColor = 0xcc8844; break; // deer
            case 1: bodyColor = 0x44aa88; break; // snake
            case 2: bodyColor = 0x88ccff; break; // firefly
            case 3: bodyColor = 0xaa66dd; break; // crystal
            case 4: bodyColor = 0x444444; break; // bat
            case 5: bodyColor = 0xcc4422; break; // fire
            case 6: bodyColor = 0xddbb66; break; // lizard
            case 7: bodyColor = 0xeeeeff; break; // snow
            case 8: bodyColor = 0x66aa44; break; // rabbit
            case 9: bodyColor = 0x886644; break; // fox
        }
        var bodyGeo = new THREE.SphereGeometry(0.2, 6, 6);
        bodyGeo.scale(1, 0.6, 1.4);
        var bodyMat = new THREE.MeshLambertMaterial({ color: bodyColor, flatShading: true });
        var body = new THREE.Mesh(bodyGeo, bodyMat);
        body.position.y = 0.2;
        group.add(body);

        var headGeo = new THREE.SphereGeometry(0.1, 5, 5);
        var headMat = new THREE.MeshLambertMaterial({ color: bodyColor, flatShading: true });
        var head = new THREE.Mesh(headGeo, headMat);
        head.position.set(0.2, 0.35, 0);
        group.add(head);

        // Glow for special types
        if (type === 2 || type === 3 || type === 5) {
            var glowGeo = new THREE.SphereGeometry(0.05, 4, 4);
            var glowMat = new THREE.MeshBasicMaterial({ color: 0xffff88, transparent: true, opacity: 0.6 });
            var glow = new THREE.Mesh(glowGeo, glowMat);
            glow.position.y = 0.4;
            group.add(glow);
        }

        scene.add(group);
        return group;
    }

    window.threeBridgeSpawnCreature = function (id, x, y, z, type, biome) {
        if (creatureMeshes.has(id)) return;
        var mesh = createCreatureMesh(type, biome);
        mesh.position.set(x, y, z);
        creatureMeshes.set(id, { mesh: mesh, target: { x: x, y: y, z: z }, current: { x: x, y: y, z: z }, type: type });
    };

    window.threeBridgeUpdateCreature = function (id, x, y, z, rot) {
        var c = creatureMeshes.get(id);
        if (!c) return;
        c.target.x = x; c.target.y = y; c.target.z = z;
        c.mesh.rotation.y = rot;
    };

    window.threeBridgeRemoveCreature = function (id) {
        var c = creatureMeshes.get(id);
        if (!c) return;
        scene.remove(c.mesh);
        c.mesh.traverse(function (n) { if (n.isMesh) { n.geometry.dispose(); n.material.dispose(); } });
        creatureMeshes.delete(id);
    };

    // ============================================================
    // F11: PORTALS
    // ============================================================

    var portalMeshes = new Map(); // id -> group

    window.threeBridgeSpawnPortal = function (id, x, y, z, targetSeed) {
        if (portalMeshes.has(id)) return;
        var group = new THREE.Group();

        // Outer ring
        var ringGeo = new THREE.TorusGeometry(1.5, 0.15, 12, 24);
        var ringMat = new THREE.MeshBasicMaterial({
            color: 0x8844ff,
            transparent: true,
            opacity: 0.8,
        });
        var ring = new THREE.Mesh(ringGeo, ringMat);
        ring.rotation.x = Math.PI / 2;
        group.add(ring);

        // Inner glow disc
        var discGeo = new THREE.RingGeometry(0.2, 1.4, 24);
        var discMat = new THREE.MeshBasicMaterial({
            color: 0x6633cc,
            transparent: true,
            opacity: 0.3,
            side: THREE.DoubleSide,
        });
        var disc = new THREE.Mesh(discGeo, discMat);
        disc.rotation.x = Math.PI / 2;
        group.add(disc);

        // Particles around portal
        var pCount = 30;
        var pGeo = new THREE.BufferGeometry();
        var pPos = new Float32Array(pCount * 3);
        for (var i = 0; i < pCount; i++) {
            var a = (i / pCount) * Math.PI * 2;
            var r = 1.6 + Math.random() * 0.3;
            pPos[i*3] = Math.cos(a) * r;
            pPos[i*3+1] = Math.sin(a) * r * 0.5;
            pPos[i*3+2] = 0;
        }
        pGeo.setAttribute('position', new THREE.BufferAttribute(pPos, 3));
        var pMat = new THREE.PointsMaterial({ color: 0x9966ff, size: 0.08, transparent: true, blending: THREE.AdditiveBlending });
        var points = new THREE.Points(pGeo, pMat);
        group.add(points);
        group.userData.particles = points;

        group.position.set(x, y, z);
        scene.add(group);
        portalMeshes.set(id, group);
    };

    window.threeBridgeRemovePortal = function (id) {
        var g = portalMeshes.get(id);
        if (!g) return;
        scene.remove(g);
        g.traverse(function (n) { if (n.isMesh || n.isPoints) { n.geometry && n.geometry.dispose(); n.material && n.material.dispose(); } });
        portalMeshes.delete(id);
    };

    // Animate portals each frame
    function animatePortals() {
        var t = frameCount * 0.016;
        for (var entry of portalMeshes.entries()) {
            var g = entry[1];
            g.rotation.y += 0.01;
            if (g.userData.particles) {
                var pos = g.userData.particles.geometry.attributes.position.array;
                for (var i = 0; i < pos.length / 3; i++) {
                    var a = (i / (pos.length / 3)) * Math.PI * 2 + t;
                    pos[i*3+2] = Math.sin(a * 2 + t) * 0.3;
                }
                g.userData.particles.geometry.attributes.position.needsUpdate = true;
            }
        }
    }

    // ============================================================
    // F13: WATERFALLS
    // ============================================================

    var waterfallMeshes = new Map();

    window.threeBridgeSpawnWaterfall = function (key, x, y, z, height) {
        if (waterfallMeshes.has(key)) return;
        var group = new THREE.Group();

        // Falling water column
        var pCount = 100;
        var geo = new THREE.BufferGeometry();
        var pos = new Float32Array(pCount * 3);
        var sizes = new Float32Array(pCount);
        for (var i = 0; i < pCount; i++) {
            pos[i*3] = (Math.random() - 0.5) * 0.8;
            pos[i*3+1] = -Math.random() * height;
            pos[i*3+2] = (Math.random() - 0.5) * 0.8;
            sizes[i] = 0.1 + Math.random() * 0.15;
        }
        geo.setAttribute('position', new THREE.BufferAttribute(pos, 3));
        geo.setAttribute('size', new THREE.BufferAttribute(sizes, 1));

        var mat = new THREE.PointsMaterial({
            color: 0x88ccff,
            size: 0.2,
            transparent: true,
            opacity: 0.6,
            blending: THREE.AdditiveBlending,
        });
        var points = new THREE.Points(geo, mat);
        group.add(points);
        group.userData.offset = Math.random() * 100;

        group.position.set(x, y, z);
        scene.add(group);
        waterfallMeshes.set(key, group);
    };

    window.threeBridgeRemoveWaterfall = function (key) {
        var g = waterfallMeshes.get(key);
        if (!g) return;
        scene.remove(g);
        g.traverse(function (n) { if (n.isPoints) { n.geometry.dispose(); n.material.dispose(); } });
        waterfallMeshes.delete(key);
    };

    function animateWaterfalls() {
        var t = frameCount * 0.016;
        for (var entry of waterfallMeshes.entries()) {
            var g = entry[1];
            var pos = g.children[0].geometry.attributes.position.array;
            for (var i = 0; i < pos.length / 3; i++) {
                pos[i*3+1] = ((pos[i*3+1] - t * 2 + g.userData.offset) % 6) - 6;
            }
            g.children[0].geometry.attributes.position.needsUpdate = true;
        }
    }

    // ============================================================
    // F14: WEATHER POWERS
    // ============================================================

    window.threeBridgeInvokeWeather = function (power) {
        switch (power) {
            case 'rain': window.threeBridgeSetWeather('rain', 1.0); break;
            case 'clear': window.threeBridgeClearWeather(); break;
            case 'storm': window.threeBridgeSetWeather('storm', 1.0); break;
            case 'snow': window.threeBridgeSetWeather('snow', 1.0); break;
        }
        setTimeout(function () { window.threeBridgeClearWeather(); }, 10000);
    };

    // ============================================================
    // F16: CODEX (encyclopedia data)
    // ============================================================

    window.threeBridgeGetCodexData = function () {
        return JSON.stringify({
            biomes: ['forest','plains','desert','tundra','jungle','volcanic','ocean','crystal','cave','lava','fungus','abyss','storm','aurora','magma'],
            formulas: ['FBM','Perlin','Simplex','Voronoi','Mandelbrot','Sierpinski','Julia','Tetrahedron','Cube','Sphere','Menger','Vortex','Ice','Wave','Spiral','Hexagonal','RidgedMF','DomainWarp','Hybrid','Plasma','Cellular','Strange','Worley','Marble','Terrazas','Erosion','Thermal'],
            minerals: ['emerald','sapphire','copper','quartz','amethyst','ruby','topaz','pearl'],
            structures: ['Hut','Tower','Ruins','Arch','Pillar','Dome','Pyramid','CrystalSpire','MushroomHut','Obelisk'],
            creatures: ['Deer','Snake','Firefly','Crystal','Bat','Fire','Lizard','Snow','Rabbit','Fox'],
        });
    };

    // ============================================================
    // F18: WEBXR VR SUPPORT
    // ============================================================

    window.threeBridgeIsVRSupported = function () {
        return navigator.xr && navigator.xr.isSessionSupported ? true : false;
    };

    window.threeBridgeEnterVR = function () {
        if (!renderer) return;
        renderer.xr.enabled = true;
        if (navigator.xr) {
            navigator.xr.requestSession('immersive-vr').then(function (session) {
                renderer.xr.setSession(session);
                console.log('[vr] Session started');
            }).catch(function (e) {
                console.warn('[vr] Session failed:', e);
            });
        }
    };

    window.threeBridgeExitVR = function () {
        if (renderer && renderer.xr.getSession()) {
            renderer.xr.getSession().end();
            renderer.xr.enabled = false;
        }
    };

    // ============================================================
    // F19: MODDING API
    // ============================================================

    var activeMods = [];

    window.threeBridgeLoadMod = function (url) {
        fetch(url).then(function (r) { return r.json(); }).then(function (mod) {
            activeMods.push(mod);
            console.log('[mod] Loaded:', mod.name);
        }).catch(function (e) { console.warn('[mod] Failed:', e); });
    };

    window.threeBridgeGetActiveMods = function () {
        return JSON.stringify(activeMods);
    };

    // ============================================================
    // ANIMATION LOOPS (integrate into render hook)
    // ============================================================

    // Re-hook render with all animations
    window.threeBridgeRender = (function (origRender) {
        return function () {
            if (wsConnected) {
                updateRemotePlayerPositions();
            }
            animatePortals();
            animateWaterfalls();
            // Lerp creature positions
            for (var entry of creatureMeshes.entries()) {
                var c = entry[1];
                c.current.x += (c.target.x - c.current.x) * 0.05;
                c.current.y += (c.target.y - c.current.y) * 0.05;
                c.current.z += (c.target.z - c.current.z) * 0.05;
                c.mesh.position.set(c.current.x, c.current.y, c.current.z);
            }
            return origRender.apply(this, arguments);
        };
    })(window.threeBridgeRender);
})();
