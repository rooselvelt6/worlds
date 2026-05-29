use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = threeBridgeInit)]
    pub fn init(canvas: &web_sys::HtmlCanvasElement);

    #[wasm_bindgen(js_name = threeBridgeUploadMesh)]
    pub fn upload_mesh(
        key: &str,
        positions: &js_sys::Float32Array,
        normals: &js_sys::Float32Array,
        indices: &js_sys::Uint32Array,
        colors: &js_sys::Float32Array,
        uvs: &js_sys::Float32Array,
    );

    #[wasm_bindgen(js_name = threeBridgeSetMeshPosition)]
    pub fn set_mesh_position(key: &str, x: f64, y: f64, z: f64);

    #[wasm_bindgen(js_name = threeBridgeSetMeshRotation)]
    pub fn set_mesh_rotation(key: &str, x: f64, y: f64, z: f64);

    #[wasm_bindgen(js_name = threeBridgeUpdateMeshPositions)]
    pub fn update_mesh_positions(key: &str, positions: &js_sys::Float32Array);

    #[wasm_bindgen(js_name = threeBridgeRemoveMesh)]
    pub fn remove_mesh(key: &str);

    #[wasm_bindgen(js_name = threeBridgeSetCamera)]
    pub fn set_camera(x: f64, y: f64, z: f64, yaw: f64, pitch: f64);

    #[wasm_bindgen(js_name = threeBridgeUploadTexture)]
    pub fn upload_texture(key: &str, width: u32, height: u32, data: &js_sys::Uint8Array);

    #[wasm_bindgen(js_name = threeBridgeSetSky)]
    pub fn set_sky(r: f64, g: f64, b: f64);

    #[wasm_bindgen(js_name = threeBridgeSetFog)]
    pub fn set_fog(r: f64, g: f64, b: f64, density: f64);

    #[wasm_bindgen(js_name = threeBridgeUploadSkyMesh)]
    pub fn upload_sky_mesh(
        key: &str,
        positions: &js_sys::Float32Array,
        normals: &js_sys::Float32Array,
        indices: &js_sys::Uint32Array,
        colors: &js_sys::Float32Array,
    );

    #[wasm_bindgen(js_name = threeBridgeUploadWaterMesh)]
    pub fn upload_water_mesh(
        key: &str,
        positions: &js_sys::Float32Array,
        normals: &js_sys::Float32Array,
        indices: &js_sys::Uint32Array,
        alphas: &js_sys::Float32Array,
    );

    #[wasm_bindgen(js_name = threeBridgeUpdateWaterMesh)]
    pub fn update_water_mesh(
        key: &str,
        positions: &js_sys::Float32Array,
        normals: &js_sys::Float32Array,
        alphas: &js_sys::Float32Array,
    );

    #[wasm_bindgen(js_name = threeBridgeSetUnderwater)]
    pub fn set_underwater(active: bool);

    #[wasm_bindgen(js_name = threeBridgeSetWaterColor)]
    pub fn set_water_color(r: f64, g: f64, b: f64);

    #[wasm_bindgen(js_name = threeBridgeSetMeshColor)]
    pub fn set_mesh_color(key: &str, r: f64, g: f64, b: f64);

    #[wasm_bindgen(js_name = threeBridgeSetParticlesOpacity)]
    pub fn set_particles_opacity(key: &str, opacity: f64);

    #[wasm_bindgen(js_name = threeBridgeCreateParticles)]
    pub fn create_particles(key: &str, count: u32, r: f64, g: f64, b: f64, size: f64);

    #[wasm_bindgen(js_name = threeBridgeUpdateParticles)]
    pub fn update_particles(key: &str, positions: &js_sys::Float32Array);

    #[wasm_bindgen(js_name = threeBridgeSetSunLight)]
    pub fn set_sun_light(x: f64, y: f64, z: f64, r: f64, g: f64, b: f64, intensity: f64);

    #[wasm_bindgen(js_name = threeBridgeRenderFrame)]
    pub fn render_frame();

    #[wasm_bindgen(js_name = threeBridgeSetMeshVisible)]
    pub fn set_mesh_visible(key: &str, visible: bool);

    #[wasm_bindgen(js_name = threeBridgeSetMeshOpacity)]
    pub fn set_mesh_opacity(key: &str, opacity: f64);

    #[wasm_bindgen(js_name = threeBridgeSetMeshFrustumCulled)]
    pub fn set_mesh_frustum_culled(key: &str, value: bool);

    #[wasm_bindgen(js_name = threeBridgeSetWind)]
    pub fn set_wind(dir: f64, strength: f64);

    #[wasm_bindgen(js_name = threeBridgeSetSunPosition)]
    pub fn set_sun_position(x: f64, y: f64, z: f64, elevation: f64);

    #[wasm_bindgen(js_name = threeBridgeSetNightParams)]
    pub fn set_night_params(r: f64, g: f64, b: f64, stars_opacity: f64);

    #[wasm_bindgen(js_name = threeBridgeSetBiome)]
    pub fn set_biome(zone_id: i32);

    #[wasm_bindgen(js_name = threeBridgeUploadGrass)]
    pub fn upload_grass(
        key: &str,
        instance_data: &js_sys::Float32Array,
        count: u32,
        height: f32,
    );

    #[wasm_bindgen(js_name = threeBridgeRemoveGrass)]
    pub fn remove_grass(key: &str);

    #[wasm_bindgen(js_name = threeBridgeUploadPortalMesh)]
    pub fn upload_portal_mesh(
        key: &str,
        positions: &js_sys::Float32Array,
        normals: &js_sys::Float32Array,
        indices: &js_sys::Uint32Array,
        colors: &js_sys::Float32Array,
        target_seed: u32,
        radius: f32,
    );

    #[wasm_bindgen(js_name = threeBridgeSetFade)]
    pub fn set_fade(amount: f64);

    // WebSocket / Multiplayer
    #[wasm_bindgen(js_name = threeBridgeWsConnect)]
    pub fn ws_connect(url: &str, seed: u32, on_message: &js_sys::Function);

    #[wasm_bindgen(js_name = threeBridgeWsSendPos)]
    pub fn ws_send_pos(x: f64, y: f64, z: f64, yaw: f64, pitch: f64);

    #[wasm_bindgen(js_name = threeBridgeWsSendChat)]
    pub fn ws_send_chat(text: &str);

    #[wasm_bindgen(js_name = threeBridgeWsDisconnect)]
    pub fn ws_disconnect();

    #[wasm_bindgen(js_name = threeBridgeUpdateRemotePlayer)]
    pub fn ws_update_remote_player(id: &str, name: &str, x: f64, y: f64, z: f64, yaw: f64, pitch: f64);

    #[wasm_bindgen(js_name = threeBridgeWsRemovePlayer)]
    pub fn ws_remove_player(id: &str);

    // Web Worker chunk generation
    #[wasm_bindgen(js_name = threeBridgeWorkerInit)]
    pub fn worker_init();

    #[wasm_bindgen(js_name = threeBridgeWorkerGenChunk)]
    pub fn worker_gen_chunk(params_json: &str, cx: i32, cz: i32, lod: u32) -> i32;

    #[wasm_bindgen(js_name = threeBridgeWorkerGetReady)]
    pub fn worker_get_ready() -> Option<String>;

    #[wasm_bindgen(js_name = threeBridgeWorkerPending)]
    pub fn worker_pending() -> u32;

    #[wasm_bindgen(js_name = threeBridgeWorkerSetSeed)]
    pub fn worker_set_seed(seed: u32);

    #[wasm_bindgen(js_name = threeBridgeWorkerTerminate)]
    pub fn worker_terminate();

    // ── R10 Living World ──
    #[wasm_bindgen(js_name = threeBridgeCreateFootprint)]
    pub fn create_footprint(key: &str, x: f64, y: f64, z: f64, rot: f64);

    #[wasm_bindgen(js_name = threeBridgeSetFootprintOpacity)]
    pub fn set_footprint_opacity(key: &str, opacity: f64);

    #[wasm_bindgen(js_name = threeBridgeRemoveFootprint)]
    pub fn remove_footprint(key: &str);

    #[wasm_bindgen(js_name = threeBridgeCreateMeteor)]
    pub fn create_meteor(key: &str, x: f64, y: f64, z: f64);

    #[wasm_bindgen(js_name = threeBridgeUpdateMeteor)]
    pub fn update_meteor(key: &str, x: f64, y: f64, z: f64);

    #[wasm_bindgen(js_name = threeBridgeRemoveMeteor)]
    pub fn remove_meteor(key: &str);

    #[wasm_bindgen(js_name = threeBridgePushFlora)]
    pub fn push_flora(key: &str, push_x: f64, push_z: f64, strength: f64);

    #[wasm_bindgen(js_name = threeBridgeResetFlora)]
    pub fn reset_flora(key: &str);
}
