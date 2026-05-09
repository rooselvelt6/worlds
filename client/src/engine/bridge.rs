use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = threeBridgeInit)]
    pub fn init(canvas: &web_sys::HtmlCanvasElement);

    #[wasm_bindgen(js_name = threeBridgeAddChunk)]
    pub fn add_chunk(key: &str, positions: &js_sys::Float32Array, colors: &js_sys::Float32Array, indices: &js_sys::Uint16Array, ox: f32, oz: f32);

    #[wasm_bindgen(js_name = threeBridgeRemoveChunk)]
    pub fn remove_chunk(key: &str);

    #[wasm_bindgen(js_name = threeBridgeUpdateCamera)]
    pub fn update_camera(x: f64, y: f64, z: f64, yaw: f64, pitch: f64);

    #[wasm_bindgen(js_name = threeBridgeRender)]
    pub fn render();

    #[wasm_bindgen(js_name = threeBridgeResize)]
    pub fn resize();

    #[wasm_bindgen(js_name = threeBridgeClearAll)]
    pub fn clear_all();
}
