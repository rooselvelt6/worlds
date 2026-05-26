(function () {
    var THREE = window.THREE;
    if (!THREE) {
        console.error("Three.js not loaded. Load Three.js before three_bridge.js");
        return;
    }

    var scene = null;
    var camera = null;
    var renderer = null;
    var meshes = new Map();

    // --- Init ---

    window.threeBridgeInit = function (canvas, w, h) {
        scene = new THREE.Scene();
        scene.background = new THREE.Color(0x87ceeb);
        scene.fog = new THREE.Fog(0x87ceeb, 15, 100);

        w = w || canvas.width || window.innerWidth;
        h = h || canvas.height || window.innerHeight;
        camera = new THREE.PerspectiveCamera(75, w / h, 0.1, 250);

        renderer = new THREE.WebGLRenderer({ canvas: canvas, antialias: true });
        renderer.setPixelRatio(window.devicePixelRatio);
        renderer.setSize(w, h);

        var ambient = new THREE.AmbientLight(0x6699cc, 0.5);
        scene.add(ambient);

        var sun = new THREE.DirectionalLight(0xffffdd, 1.0);
        sun.position.set(30, 50, 20);
        scene.add(sun);

        var hemi = new THREE.HemisphereLight(0x87ceeb, 0x2d5a27, 0.3);
        scene.add(hemi);
    };

    // --- Generic mesh upload (positions, normals, indices, colors) ---

    window.threeBridgeUploadMesh = function (key, posArr, normArr, idxArr, colArr) {
        if (meshes.has(key)) return;
        var geo = new THREE.BufferGeometry();
        geo.setAttribute("position", new THREE.BufferAttribute(posArr, 3));
        geo.setAttribute("normal", new THREE.BufferAttribute(normArr, 3));
        if (colArr) {
            geo.setAttribute("color", new THREE.BufferAttribute(colArr, 3));
        }
        geo.setIndex(new THREE.BufferAttribute(idxArr, 1));

        var mat = colArr
            ? new THREE.MeshLambertMaterial({ vertexColors: true })
            : new THREE.MeshLambertMaterial({ color: 0xcccccc });

        var mesh = new THREE.Mesh(geo, mat);
        scene.add(mesh);
        meshes.set(key, mesh);
    };

    // --- Set mesh world position ---

    window.threeBridgeSetMeshPosition = function (key, x, y, z) {
        var mesh = meshes.get(key);
        if (mesh) {
            mesh.position.set(x, y, z);
        }
    };

    // --- Set mesh rotation ---

    window.threeBridgeSetMeshRotation = function (key, x, y, z) {
        var mesh = meshes.get(key);
        if (mesh) {
            mesh.rotation.set(x, y, z);
        }
    };

    // --- Update mesh vertex positions (for animation) ---

    window.threeBridgeUpdateMeshPositions = function (key, posArr) {
        var mesh = meshes.get(key);
        if (mesh) {
            mesh.geometry.attributes.position.array.set(posArr);
            mesh.geometry.attributes.position.needsUpdate = true;
            mesh.geometry.computeVertexNormals();
        }
    };

    // --- Remove mesh ---

    window.threeBridgeRemoveMesh = function (key) {
        var mesh = meshes.get(key);
        if (mesh) {
            scene.remove(mesh);
            mesh.geometry.dispose();
            mesh.material.dispose();
            meshes.delete(key);
        }
    };

    // --- Set camera ---

    window.threeBridgeSetCamera = function (x, y, z, yaw, pitch) {
        if (!camera) return;
        camera.position.set(x, y, z);
        camera.rotation.order = "YXZ";
        camera.rotation.y = yaw;
        camera.rotation.x = pitch;
    };

    // --- Upload texture ---

    window.threeBridgeUploadTexture = function (key, width, height, data) {
        var texture = new THREE.DataTexture(data, width, height, THREE.RGBAFormat);
        texture.needsUpdate = true;
        if (!window.__textures) window.__textures = new Map();
        window.__textures.set(key, texture);
    };

    // --- Sky color ---

    window.threeBridgeSetSky = function (r, g, b) {
        if (!scene) return;
        scene.background = new THREE.Color(r, g, b);
    };

    // --- Fog ---

    window.threeBridgeSetFog = function (r, g, b, density) {
        if (!scene) return;
        scene.fog = new THREE.FogExp2(r, g, b, density);
    };

    // --- Upload sky mesh (dome) ---

    window.threeBridgeUploadSkyMesh = function (key, posArr, normArr, idxArr, colArr) {
        if (meshes.has(key)) return;
        var geo = new THREE.BufferGeometry();
        geo.setAttribute("position", new THREE.BufferAttribute(posArr, 3));
        geo.setAttribute("normal", new THREE.BufferAttribute(normArr, 3));
        geo.setAttribute("color", new THREE.BufferAttribute(colArr, 3));
        geo.setIndex(new THREE.BufferAttribute(idxArr, 1));
        var mat = new THREE.MeshLambertMaterial({ vertexColors: true, side: THREE.BackSide });
        var mesh = new THREE.Mesh(geo, mat);
        scene.add(mesh);
        meshes.set(key, mesh);
    };

    // --- Upload water mesh ---

    window.threeBridgeUploadWaterMesh = function (key, posArr, normArr, idxArr) {
        if (meshes.has(key)) return;
        var geo = new THREE.BufferGeometry();
        geo.setAttribute("position", new THREE.BufferAttribute(posArr, 3));
        geo.setAttribute("normal", new THREE.BufferAttribute(normArr, 3));
        geo.setIndex(new THREE.BufferAttribute(idxArr, 1));
        var mat = new THREE.MeshPhongMaterial({
            color: 0x2a6f8f,
            transparent: true,
            opacity: 0.65,
            shininess: 30,
            side: THREE.DoubleSide,
        });
        var mesh = new THREE.Mesh(geo, mat);
        scene.add(mesh);
        meshes.set(key, mesh);
    };

    // --- Set mesh color ---

    window.threeBridgeSetMeshColor = function (key, r, g, b) {
        var mesh = meshes.get(key);
        if (mesh) {
            if (mesh.material.color) {
                mesh.material.color.setRGB(r, g, b);
            }
        }
    };

    // --- Particles ---

    window.threeBridgeCreateParticles = function (key, count, r, g, b, size) {
        if (meshes.has(key)) return;
        var geo = new THREE.BufferGeometry();
        var positions = new Float32Array(count * 3);
        geo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
        var mat = new THREE.PointsMaterial({
            color: new THREE.Color(r, g, b),
            size: size,
            transparent: true,
            opacity: 1.0,
            blending: THREE.AdditiveBlending,
            depthWrite: false,
        });
        var points = new THREE.Points(geo, mat);
        scene.add(points);
        meshes.set(key, points);
    };

    window.threeBridgeUpdateParticles = function (key, posArr) {
        var mesh = meshes.get(key);
        if (mesh && mesh.geometry.attributes.position) {
            mesh.geometry.attributes.position.array.set(posArr);
            mesh.geometry.attributes.position.needsUpdate = true;
        }
    };

    window.threeBridgeSetParticlesOpacity = function (key, opacity) {
        var mesh = meshes.get(key);
        if (mesh && mesh.material) {
            mesh.material.opacity = opacity;
        }
    };

    // --- Sun light ---

    window.threeBridgeSetSunLight = function (x, y, z, r, g, b, intensity) {
        var sun = null;
        scene.children.forEach(function (child) {
            if (child.isDirectionalLight) {
                sun = child;
            }
        });
        if (sun) {
            sun.position.set(x, y, z);
            sun.color.setRGB(r, g, b);
            sun.intensity = intensity;
        }
    };

    // --- Render frame ---

    window.threeBridgeRenderFrame = function () {
        if (!renderer || !scene || !camera) return;
        renderer.render(scene, camera);
    };

    // --- Set mesh visible ---

    window.threeBridgeSetMeshVisible = function (key, visible) {
        var mesh = meshes.get(key);
        if (mesh) {
            mesh.visible = visible;
        }
    };

    // --- Set mesh opacity ---

    window.threeBridgeSetMeshOpacity = function (key, opacity) {
        var mesh = meshes.get(key);
        if (mesh && mesh.material) {
            mesh.material.transparent = true;
            mesh.material.opacity = opacity;
        }
    };

    // --- Legacy / backward-compat aliases ---

    window.threeBridgeAddChunk = function (key, posArr, colArr, idxArr, ox, oz) {
        var geo = new THREE.BufferGeometry();
        geo.setAttribute("position", new THREE.BufferAttribute(posArr, 3));
        geo.setAttribute("color", new THREE.BufferAttribute(colArr, 3));
        geo.setIndex(new THREE.BufferAttribute(idxArr, 1));
        geo.computeVertexNormals();
        var mat = new THREE.MeshLambertMaterial({ vertexColors: true });
        var mesh = new THREE.Mesh(geo, mat);
        mesh.position.set(ox, 0, oz);
        scene.add(mesh);
        meshes.set(key, mesh);
    };

    window.threeBridgeRemoveChunk = function (key) {
        window.threeBridgeRemoveMesh(key);
    };

    window.threeBridgeUpdateCamera = window.threeBridgeSetCamera;

    window.threeBridgeRender = window.threeBridgeRenderFrame;

    window.threeBridgeClearAll = function () {
        for (var entry = meshes.entries(), item; !(item = entry.next()).done;) {
            var key = item.value[0];
            var mesh = item.value[1];
            scene.remove(mesh);
            mesh.geometry.dispose();
            if (mesh.material) mesh.material.dispose();
        }
        meshes.clear();
    };

})();
