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
    var lutPass = null;
    var dofPass = null;
    var heatHazePass = null;
    var currentBiome = 'plains';
    var addonsLoaded = false;

    // Dynamic import for Three.js addon modules (jsm ES modules)
    (async function loadThreeAddons() {
        try {
            var mods = await Promise.all([
                import('https://unpkg.com/three@0.160.0/examples/jsm/postprocessing/EffectComposer.js'),
                import('https://unpkg.com/three@0.160.0/examples/jsm/postprocessing/RenderPass.js'),
                import('https://unpkg.com/three@0.160.0/examples/jsm/postprocessing/UnrealBloomPass.js'),
                import('https://unpkg.com/three@0.160.0/examples/jsm/postprocessing/ShaderPass.js'),
                import('https://unpkg.com/three@0.160.0/examples/jsm/shaders/CopyShader.js'),
                import('https://unpkg.com/three@0.160.0/examples/jsm/shaders/LuminosityHighPassShader.js'),
            ]);
            THREE.EffectComposer = mods[0].EffectComposer;
            THREE.RenderPass = mods[1].RenderPass;
            THREE.UnrealBloomPass = mods[2].UnrealBloomPass;
            THREE.ShaderPass = mods[3].ShaderPass;
            THREE.CopyShader = mods[4].CopyShader;
            THREE.LuminosityHighPassShader = mods[5].LuminosityHighPassShader;
            addonsLoaded = true;
        } catch (e) {
            console.warn('[bridge] Three addon imports failed, running without post-processing:', e);
        }
    })();

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

    function tryInitComposer() {
        if (composer || !renderer || !scene || !camera) return;
        if (!addonsLoaded || typeof THREE.EffectComposer === 'undefined') return;
        try {
            composer = new THREE.EffectComposer(renderer);
            var renderPass = new THREE.RenderPass(scene, camera);
            composer.addPass(renderPass);

            bloomPass = new THREE.UnrealBloomPass(
                new THREE.Vector2(renderer.domElement.clientWidth, renderer.domElement.clientHeight),
                0.3, 0.2, 0.1
            );
            composer.addPass(bloomPass);

            // Phase 3: LUT Color Grading
            initLUTPass();

            // Phase 3: Depth of Field
            initDoFPass();

            // Phase 3: Heat Haze (disabled by default)
            initHeatHazePass();

            console.log('[bridge] Post-processing initialized with LUT + DoF + Heat Haze');
        } catch (e) {
            console.warn('[bridge] Composer init error:', e);
        }
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

            renderer.toneMapping = THREE.ACESFilmicToneMapping;
            renderer.toneMappingExposure = 1.2;

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
            vignetteEl.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;pointer-events:none;z-index:9999;background:radial-gradient(ellipse at center, transparent 55%, rgba(0,0,0,0.5) 100%);opacity:0.8;transition:opacity 0.5s;';
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

            // Post-processing (lazy init when addons load)
            tryInitComposer();

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

        // Bloom intensity varies with time
        if (bloomPass) {
            bloomPass.strength = 0.15 + nightFactor * 0.35;
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

        // Lazy composer init (addons may load after first frame)
        if (!composer && addonsLoaded) {
            tryInitComposer();
        }

        if (waterMesh && waterMesh.material.uniforms) {
            waterMesh.material.uniforms.uTime.value = frameCount * 0.016;
        }

        // Update heat haze time uniform
        if (heatHazePass && heatHazePass.enabled && heatHazePass.material.uniforms) {
            heatHazePass.material.uniforms.uTime.value = frameCount * 0.016;
        }

        // Update DoF focal distance based on camera height
        if (dofPass && dofPass.enabled && camera && dofPass.material.uniforms) {
            dofPass.material.uniforms.uFocusDistance.value = Math.max(10, camera.position.length() * 0.5);
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
        // Vignette intensity per biome
        var vg = document.getElementById('vignette-overlay');
        if (vg) {
            var darkBiomes = ['abyss', 'cave', 'storm'];
            var brightBiomes = ['desert', 'tundra', 'crystal'];
            if (darkBiomes.indexOf(biome) >= 0) {
                vg.style.opacity = '0.95';
            } else if (brightBiomes.indexOf(biome) >= 0) {
                vg.style.opacity = '0.5';
            } else {
                vg.style.opacity = '0.75';
            }
        }
    };

    // ============================================================
    // CINEMATIC POST-PROCESSING (Phase 3)
    // ============================================================

    // --- LUT Color Grading ---

    // Enhanced LUT data per biome with richer color transformations
    var lutPresets = {
        forest:   { warmth: 0.05, vibrance: 1.1, contrast: 1.05, hueShift: 0.02 },
        plains:   { warmth: 0.02, vibrance: 1.0, contrast: 1.0,  hueShift: 0.0  },
        desert:   { warmth: 0.15, vibrance: 1.15, contrast: 1.1,  hueShift: -0.02 },
        tundra:   { warmth: -0.05, vibrance: 0.9, contrast: 1.15, hueShift: 0.01 },
        jungle:   { warmth: 0.08, vibrance: 1.2, contrast: 1.05, hueShift: 0.03 },
        volcanic: { warmth: 0.2,  vibrance: 1.25, contrast: 1.2,  hueShift: -0.03 },
        ocean:    { warmth: -0.08, vibrance: 1.05, contrast: 1.0, hueShift: 0.04 },
        crystal:  { warmth: -0.02, vibrance: 1.3, contrast: 1.1,  hueShift: 0.06 },
        cave:     { warmth: -0.1,  vibrance: 0.85, contrast: 1.3,  hueShift: 0.01 },
        lava:     { warmth: 0.25, vibrance: 1.3, contrast: 1.25, hueShift: -0.04 },
        fungus:   { warmth: 0.0,  vibrance: 1.15, contrast: 1.05, hueShift: 0.05 },
        abyss:    { warmth: -0.15, vibrance: 0.8, contrast: 1.4,  hueShift: 0.0  },
        storm:    { warmth: -0.05, vibrance: 0.9, contrast: 1.2,  hueShift: -0.01 },
        aurora:   { warmth: -0.1,  vibrance: 1.2, contrast: 1.0,  hueShift: 0.08 },
        magma:    { warmth: 0.25, vibrance: 1.3, contrast: 1.25, hueShift: -0.04 },
    };

    var lutTexture = null;
    var lutBlend = 1.0;
    var lutTargetBiome = 'plains';

    function generateLUT(biome) {
        var preset = lutPresets[biome] || lutPresets.plains;
        var canvas = document.createElement('canvas');
        canvas.width = 256;
        canvas.height = 16;
        var ctx = canvas.getContext('2d');
        var imgData = ctx.createImageData(256, 16);
        var d = imgData.data;
        for (var b = 0; b < 16; b++) {
            for (var g = 0; g < 16; g++) {
                for (var r = 0; r < 16; r++) {
                    var idx = (b * 256 + g * 16 + r) * 4;
                    var ri = r / 15, gi = g / 15, bi = b / 15;
                    // Apply preset transformations
                    var ro = Math.pow(ri, 1 / preset.contrast);
                    ro += preset.warmth * 0.15;
                    ro = Math.max(0, Math.min(1, ro));
                    var go = Math.pow(gi, 1 / preset.contrast);
                    go += preset.warmth * 0.05;
                    go = Math.max(0, Math.min(1, go));
                    var bo = Math.pow(bi, 1 / preset.contrast);
                    bo -= preset.warmth * 0.1;
                    bo = Math.max(0, Math.min(1, bo));
                    // Vibrance boost
                    var gray = (ro + go + bo) / 3;
                    var vib = preset.vibrance;
                    ro = gray + (ro - gray) * vib;
                    go = gray + (go - gray) * vib;
                    bo = gray + (bo - gray) * vib;
                    ro = Math.max(0, Math.min(1, ro));
                    go = Math.max(0, Math.min(1, go));
                    bo = Math.max(0, Math.min(1, bo));
                    d[idx] = ro * 255;
                    d[idx + 1] = go * 255;
                    d[idx + 2] = bo * 255;
                    d[idx + 3] = 255;
                }
            }
        }
        ctx.putImageData(imgData, 0, 0);
        var tex = new THREE.CanvasTexture(canvas);
        tex.magFilter = THREE.LinearFilter;
        tex.minFilter = THREE.LinearFilter;
        tex.wrapS = THREE.ClampToEdgeWrapping;
        tex.wrapT = THREE.ClampToEdgeWrapping;
        return tex;
    }

    function initLUTPass() {
        if (!THREE.ShaderPass) return;
        lutTexture = generateLUT('plains');
        lutPass = new THREE.ShaderPass({
            uniforms: {
                tDiffuse: { value: null },
                tLUT: { value: lutTexture },
                uLUTSize: { value: 16.0 },
                uBlend: { value: 1.0 },
            },
            vertexShader: [
                "varying vec2 vUv;",
                "void main() {",
                "  vUv = uv;",
                "  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);",
                "}"
            ].join("\n"),
            fragmentShader: [
                "uniform sampler2D tDiffuse;",
                "uniform sampler2D tLUT;",
                "uniform float uLUTSize;",
                "uniform float uBlend;",
                "varying vec2 vUv;",
                "void main() {",
                "  vec4 col = texture2D(tDiffuse, vUv);",
                "  float lutSize = uLUTSize;",
                "  float scale = (lutSize - 1.0) / lutSize;",
                "  float offset = 0.5 / lutSize;",
                "  vec3 lutCoord = col.rgb * scale + offset;",
                "  float b = floor(lutCoord.b * lutSize);",
                "  float tb = b / lutSize + offset;",
                "  float tn = (b + 1.0) / lutSize + offset;",
                "  float bb = fract(lutCoord.b * lutSize);",
                "  vec3 lutPosB = vec3(lutCoord.r, lutCoord.g, tb);",
                "  vec3 lutPosN = vec3(lutCoord.r, lutCoord.g, tn);",
                "  vec4 lutB = texture2D(tLUT, lutPosB.xy);",
                "  vec4 lutN = texture2D(tLUT, lutPosN.xy);",
                "  vec3 lut = mix(lutB.rgb, lutN.rgb, bb);",
                "  gl_FragColor = vec4(mix(col.rgb, lut, uBlend), col.a);",
                "}"
            ].join("\n"),
        });
        lutPass.needsSwap = true;
        // Insert LUT after bloom, before DoF
        var idx = composer.passes.indexOf(bloomPass);
        if (idx >= 0) {
            composer.insertPass(lutPass, idx + 1);
        } else {
            composer.addPass(lutPass);
        }
    }

    // --- Depth of Field ---

    function initDoFPass() {
        if (!THREE.ShaderPass) return;
        dofPass = new THREE.ShaderPass({
            uniforms: {
                tDiffuse: { value: null },
                uFocusDistance: { value: 30.0 },
                uAperture: { value: 0.015 },
                uMaxBlur: { value: 0.02 },
            },
            vertexShader: [
                "varying vec2 vUv;",
                "void main() {",
                "  vUv = uv;",
                "  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);",
                "}"
            ].join("\n"),
            fragmentShader: [
                "uniform sampler2D tDiffuse;",
                "uniform float uFocusDistance;",
                "uniform float uAperture;",
                "uniform float uMaxBlur;",
                "varying vec2 vUv;",
                "void main() {",
                "  vec2 dim = vec2(textureSize(tDiffuse, 0));",
                "  float aspect = dim.x / dim.y;",
                "  vec2 texel = vec2(1.0 / dim.x, 1.0 / dim.y);",
                "  vec4 col = texture2D(tDiffuse, vUv);",
                "  float depth = col.a;",
                "  float coc = abs(depth - 0.5) * uAperture * 50.0;",
                "  coc = min(coc, uMaxBlur);",
                "  int samples = 9;",
                "  vec3 blur = vec3(0.0);",
                "  float total = 0.0;",
                "  for (int x = -4; x <= 4; x++) {",
                "    for (int y = -4; y <= 4; y++) {",
                "      float d = float(x * x + y * y);",
                "      float weight = exp(-d * 20.0 * coc);",
                "      vec2 off = vec2(float(x) * texel.x * coc * 50.0, float(y) * texel.y * coc * 50.0 / aspect);",
                "      blur += texture2D(tDiffuse, vUv + off).rgb * weight;",
                "      total += weight;",
                "    }",
                "  }",
                "  blur /= max(total, 0.001);",
                "  gl_FragColor = vec4(blur, 1.0);",
                "}"
            ].join("\n"),
        });
        dofPass.needsSwap = true;
        // Insert DoF after LUT
        var idx = composer.passes.indexOf(lutPass || bloomPass);
        if (idx >= 0) {
            composer.insertPass(dofPass, idx + 1);
        } else {
            composer.addPass(dofPass);
        }
    }

    // --- Heat Haze ---

    function initHeatHazePass() {
        if (!THREE.ShaderPass) return;
        heatHazePass = new THREE.ShaderPass({
            uniforms: {
                tDiffuse: { value: null },
                uTime: { value: 0 },
                uIntensity: { value: 0.0 },
            },
            vertexShader: [
                "varying vec2 vUv;",
                "void main() {",
                "  vUv = uv;",
                "  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);",
                "}"
            ].join("\n"),
            fragmentShader: [
                "uniform sampler2D tDiffuse;",
                "uniform float uTime;",
                "uniform float uIntensity;",
                "varying vec2 vUv;",
                "void main() {",
                "  vec2 uv = vUv;",
                "  float distortX = sin(uv.y * 40.0 + uTime * 2.0) * 0.003 * uIntensity;",
                "  float distortY = sin(uv.x * 30.0 + uTime * 1.5 + 1.2) * 0.002 * uIntensity;",
                "  uv += vec2(distortX, distortY);",
                "  gl_FragColor = texture2D(tDiffuse, uv);",
                "}"
            ].join("\n"),
        });
        heatHazePass.enabled = false;
        heatHazePass.needsSwap = true;
        composer.addPass(heatHazePass);
    }

    // Bridge: set LUT per biome
    window.threeBridgeSetLUT = function (biome) {
        if (biome === lutTargetBiome) return;
        lutTargetBiome = biome;
        if (lutPass && lutPass.material && lutPass.material.uniforms) {
            lutTexture = generateLUT(biome);
            lutPass.material.uniforms.tLUT.value = lutTexture;
            lutPass.material.uniforms.uBlend.value = 1.0;
            // Also adjust bloom per biome
            var bloomStrength = 0.3;
            var hotBiomes = ['volcanic', 'magma', 'lava', 'desert'];
            var darkBiomes = ['cave', 'abyss', 'storm'];
            if (hotBiomes.indexOf(biome) >= 0) bloomStrength = 0.5;
            else if (darkBiomes.indexOf(biome) >= 0) bloomStrength = 0.15;
            else bloomStrength = 0.3;
            if (bloomPass) {
                bloomPass.strength = bloomStrength;
            }
        }
    };

    // Bridge: toggle heat haze
    window.threeBridgeSetHeatHaze = function (active) {
        if (!heatHazePass) return;
        heatHazePass.enabled = active;
        if (heatHazePass.material && heatHazePass.material.uniforms) {
            heatHazePass.material.uniforms.uIntensity.value = active ? 1.0 : 0.0;
        }
    };

    // Bridge: set DoF parameters
    window.threeBridgeSetDoF = function (focus, aperture) {
        if (!dofPass || !dofPass.material || !dofPass.material.uniforms) return;
        dofPass.material.uniforms.uFocusDistance.value = focus;
        dofPass.material.uniforms.uAperture.value = aperture;
    };
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

    // Hook into existing render function
    var frameCount = 0;
    window.threeBridgeRender = (function (origRender) {
        return function () {
            if (wsConnected) {
                updateRemotePlayerPositions();
            }
            return origRender.apply(this, arguments);
        };
    })(window.threeBridgeRender);
})();
