(function () {
    var THREE = window.THREE;
    if (!THREE) { console.error("[bridge] THREE not loaded"); return; }

    var scene = null, camera = null, renderer = null;
    var meshes = new Map();

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

            resizeRenderer();
            new ResizeObserver(resizeRenderer).observe(canvas);

            renderer.toneMapping = THREE.ACESFilmicToneMapping;
            renderer.toneMappingExposure = 1.2;

            // Warm main light (sunset)
            var sun = new THREE.DirectionalLight(0xff8844, 1.4);
            sun.position.set(-40, 60, -20);
            scene.add(sun);

            // Cool fill light
            var fill = new THREE.DirectionalLight(0x4488ff, 0.6);
            fill.position.set(30, 10, 40);
            scene.add(fill);

            // Deep ambient
            var ambient = new THREE.AmbientLight(0x222244, 0.4);
            scene.add(ambient);

            // Rim light from below
            var rim = new THREE.DirectionalLight(0x6644aa, 0.3);
            rim.position.set(0, -30, 0);
            scene.add(rim);

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

    var frameCount = 0;
    window.threeBridgeRender = function () {
        if (!renderer || !scene || !camera) return;
        frameCount++;
        if (frameCount % 120 === 1) console.log("[bridge] render frame", frameCount);
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
})();
