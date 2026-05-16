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

    // Add clamp polyfill for older browsers
    if (!Math.clamp) {
        Math.clamp = function (v, min, max) { return Math.min(Math.max(v, min), max); };
    }
})();
