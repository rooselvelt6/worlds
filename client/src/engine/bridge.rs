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
    );

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
}
