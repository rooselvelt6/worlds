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

    window.threeBridgeInit = function (canvas, w, h) {
        scene = new THREE.Scene();
        scene.background = new THREE.Color(0x87ceeb);
        scene.fog = new THREE.Fog(0x87ceeb, 15, 100);

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

    window.threeBridgeAddChunk = function (key, posArr, colArr, idxArr, ox, oz) {
        if (meshes.has(key)) return;

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
        camera.rotation.order = "YXZ";
        camera.rotation.y = yaw;
        camera.rotation.x = pitch;
    };

    window.threeBridgeRender = function () {
        if (!renderer || !scene || !camera) return;
        renderer.render(scene, camera);
    };

    window.threeBridgeResize = function (w, h) {
        if (!renderer || !camera) return;
        renderer.setSize(w, h);
        camera.aspect = w / h;
        camera.updateProjectionMatrix();
    };

    window.threeBridgeClearAll = function () {
        for (var entry = meshes.entries(), item; !(item = entry.next()).done;) {
            var key = item.value[0];
            var mesh = item.value[1];
            scene.remove(mesh);
            mesh.geometry.dispose();
            mesh.material.dispose();
        }
        meshes.clear();
    };
})();
