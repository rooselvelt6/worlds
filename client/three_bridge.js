(function () {
    var THREE = window.THREE;
    if (!THREE) { console.error("[bridge] THREE not loaded"); return; }

    var scene = null, camera = null, renderer = null;
    var meshes = new Map();
    var waterMesh = null;
    var sunLight = null, fillLight = null, ambientLight = null;
    var timeOfDay = 0.5;
    var particles = [];
    var composer = null;
    var bloomPass = null;

    function resizeRenderer() {
        if (!renderer || !camera || !renderer.domElement) return;
        var w = renderer.domElement.clientWidth;
        var h = renderer.domElement.clientHeight;
        if (w === 0 || h === 0) return;
        renderer.setSize(w, h, false);
        camera.aspect = w / h;
        camera.updateProjectionMatrix();
        if (composer) composer.setSize(w, h);
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

            resizeRenderer();
            new ResizeObserver(resizeRenderer).observe(canvas);

            renderer.toneMapping = THREE.ACESFilmicToneMapping;
            renderer.toneMappingExposure = 1.2;

            sunLight = new THREE.DirectionalLight(0xff8844, 1.4);
            sunLight.position.set(-40, 60, -20);
            scene.add(sunLight);

            fillLight = new THREE.DirectionalLight(0x4488ff, 0.6);
            fillLight.position.set(30, 10, 40);
            scene.add(fillLight);

            ambientLight = new THREE.AmbientLight(0x222244, 0.4);
            scene.add(ambientLight);

            var rim = new THREE.DirectionalLight(0x6644aa, 0.3);
            rim.position.set(0, -30, 0);
            scene.add(rim);

            // Water
            var waterGeo = new THREE.PlaneGeometry(500, 500, 1, 1);
            var waterMat = new THREE.MeshLambertMaterial({
                color: 0x1a5276,
                transparent: true,
                opacity: 0.4,
                side: THREE.DoubleSide,
            });
            waterMesh = new THREE.Mesh(waterGeo, waterMat);
            waterMesh.rotation.x = -Math.PI / 2;
            waterMesh.position.y = 0;
            scene.add(waterMesh);

            // Post-processing
            if (typeof THREE.EffectComposer !== 'undefined') {
                composer = new THREE.EffectComposer(renderer);
                var renderPass = new THREE.RenderPass(scene, camera);
                composer.addPass(renderPass);

                bloomPass = new THREE.UnrealBloomPass(
                    new THREE.Vector2(
                        renderer.domElement.clientWidth,
                        renderer.domElement.clientHeight
                    ),
                    0.3, 0.2, 0.1
                );
                composer.addPass(bloomPass);
            }

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
        }

        // Bloom intensity varies with time
        if (bloomPass) {
            bloomPass.strength = 0.15 + nightFactor * 0.35;
        }
    };

    window.threeBridgeSetWaterLevel = function (level) {
        if (waterMesh) {
            waterMesh.position.y = level;
        }
    };

    window.threeBridgeSetFog = function (r, g, b, density) {
        if (scene && scene.fog) {
            scene.fog.color.setRGB(r, g, b);
            scene.fog.density = density;
        }
    };

    window.threeBridgeSetBloom = function (strength, radius, threshold) {
        if (bloomPass) {
            bloomPass.strength = strength;
            bloomPass.radius = radius;
            bloomPass.threshold = threshold;
        }
    };

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

        if (waterMesh && waterMesh.material) {
            waterMesh.material.opacity = 0.3 + Math.sin(frameCount * 0.01) * 0.05;
        }

        if (composer) {
            composer.render();
        } else {
            renderer.render(scene, camera);
        }
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
    // END AUDIO + WEATHER
    // ============================================================

    // Add clamp polyfill for older browsers
    if (!Math.clamp) {
        Math.clamp = function (v, min, max) { return Math.min(Math.max(v, min), max); };
    }
})();
