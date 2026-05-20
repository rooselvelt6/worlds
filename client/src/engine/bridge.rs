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

    #[wasm_bindgen(js_name = threeBridgeSetTime)]
    pub fn set_time(t: f64);

    #[wasm_bindgen(js_name = threeBridgeSetWaterLevel)]
    pub fn set_water_level(level: f64);

    #[wasm_bindgen(js_name = threeBridgeSetUnderwater)]
    pub fn set_underwater(active: bool, depth: f32);

    #[wasm_bindgen(js_name = threeBridgeSetFog)]
    pub fn set_fog(r: f32, g: f32, b: f32, density: f32);

    #[wasm_bindgen(js_name = threeBridgeSpawnParticles)]
    pub fn spawn_particles(key: &str, positions: &js_sys::Float32Array, colors: &js_sys::Float32Array, count: u32);

    #[wasm_bindgen(js_name = threeBridgeRemoveParticles)]
    pub fn remove_particles(key: &str);

    #[wasm_bindgen(js_name = threeBridgeSetBloom)]
    pub fn set_bloom(strength: f32, radius: f32, threshold: f32);

    #[wasm_bindgen(js_name = threeBridgeCaptureScreenshot)]
    pub fn capture_screenshot(seed: u32, formula: &str, zone: &str, x: f64, y: f64, z: f64);

    // Audio
    #[wasm_bindgen(js_name = threeBridgeAudioInit)]
    pub fn audio_init();

    #[wasm_bindgen(js_name = threeBridgeAudioPlayAmbient)]
    pub fn audio_play_ambient(zone: &str);

    #[wasm_bindgen(js_name = threeBridgeAudioStopAmbient)]
    pub fn audio_stop_ambient();

    #[wasm_bindgen(js_name = threeBridgeAudioPlayFootstep)]
    pub fn audio_play_footstep(intensity: f32);

    #[wasm_bindgen(js_name = threeBridgeAudioPlayEffect)]
    pub fn audio_play_effect(effect_type: &str);

    #[wasm_bindgen(js_name = threeBridgeAudioSetMasterVolume)]
    pub fn audio_set_master_volume(vol: f32);

    // Weather
    #[wasm_bindgen(js_name = threeBridgeSetWeather)]
    pub fn set_weather(weather_type: &str, intensity: f32);

    #[wasm_bindgen(js_name = threeBridgeClearWeather)]
    pub fn clear_weather();

    // Visual
    #[wasm_bindgen(js_name = threeBridgeSetBiomeTint)]
    pub fn set_biome_tint(biome: &str);

    // Phase 3: Cinematic Post-Processing
    #[wasm_bindgen(js_name = threeBridgeSetLUT)]
    pub fn set_lut(biome: &str);

    #[wasm_bindgen(js_name = threeBridgeSetHeatHaze)]
    pub fn set_heat_haze(active: bool);

    // Phase 4: Multiplayer
    #[wasm_bindgen(js_name = threeBridgeWsConnect)]
    pub fn ws_connect(seed: u32);

    #[wasm_bindgen(js_name = threeBridgeWsSendPosition)]
    pub fn ws_send_position(x: f64, y: f64, z: f64, yaw: f64, pitch: f64);

    #[wasm_bindgen(js_name = threeBridgeWsDisconnect)]
    pub fn ws_disconnect();

    // Phase 5: Save/Load
    #[wasm_bindgen(js_name = threeBridgeSaveSlot)]
    pub fn save_slot(slot: u32, json: &str);

    #[wasm_bindgen(js_name = threeBridgeLoadSlot)]
    pub fn load_slot(slot: u32) -> String;

    #[wasm_bindgen(js_name = threeBridgeDeleteSlot)]
    pub fn delete_slot(slot: u32);

    // Vegetation
    #[wasm_bindgen(js_name = threeBridgeSpawnVegetation)]
    pub fn spawn_vegetation(key: &str, pos_data: &js_sys::Float32Array, size_data: &js_sys::Float32Array, type_data: &js_sys::Uint8Array, count: u32, zone: &str);

    #[wasm_bindgen(js_name = threeBridgeRemoveVegetation)]
    pub fn remove_vegetation(key: &str);

    #[wasm_bindgen(js_name = threeBridgeUpdateWind)]
    pub fn update_wind(time: f32);

    // Structures
    #[wasm_bindgen(js_name = threeBridgeSpawnStructure)]
    pub fn spawn_structure(key: &str, struct_data: &js_sys::Float32Array, count: u32, zone: &str);

    #[wasm_bindgen(js_name = threeBridgeRemoveStructure)]
    pub fn remove_structure(key: &str);

    // Export
    #[wasm_bindgen(js_name = threeBridgeExportOBJ)]
    pub fn export_obj();

    // Discovery
    #[wasm_bindgen(js_name = threeBridgeCheckDiscovery)]
    pub fn check_discovery(px: f32, py: f32, pz: f32) -> String;

    // Minerals
    #[wasm_bindgen(js_name = threeBridgeSpawnMinerals)]
    pub fn spawn_minerals(key: &str, min_data: &js_sys::Float32Array, count: u32);

    #[wasm_bindgen(js_name = threeBridgeRemoveMinerals)]
    pub fn remove_minerals(key: &str);

    // F6: Mining & Building
    #[wasm_bindgen(js_name = threeBridgeMineAt)]
    pub fn mine_at(ox: f64, oy: f64, oz: f64, yaw: f64, pitch: f64) -> f64;

    #[wasm_bindgen(js_name = threeBridgePlaceBlock)]
    pub fn place_block(ox: f64, oy: f64, oz: f64, yaw: f64, pitch: f64, block_type: u8) -> bool;

    #[wasm_bindgen(js_name = threeBridgeGetBlocks)]
    pub fn get_blocks() -> String;

    #[wasm_bindgen(js_name = threeBridgeSetBlocks)]
    pub fn set_blocks(json: &str);

    // F8: Weather & Seasons
    #[wasm_bindgen(js_name = threeBridgeSetSeason)]
    pub fn set_season(season: &str);

    #[wasm_bindgen(js_name = threeBridgeSetTreeGrowth)]
    pub fn set_tree_growth(key: &str, growth: f32);

    // F9: Creatures
    #[wasm_bindgen(js_name = threeBridgeSpawnCreature)]
    pub fn spawn_creature(id: &str, x: f64, y: f64, z: f64, creature_type: u8, biome: &str);

    #[wasm_bindgen(js_name = threeBridgeUpdateCreature)]
    pub fn update_creature(id: &str, x: f64, y: f64, z: f64, rot: f64);

    #[wasm_bindgen(js_name = threeBridgeRemoveCreature)]
    pub fn remove_creature(id: &str);

    // F11: Portals
    #[wasm_bindgen(js_name = threeBridgeSpawnPortal)]
    pub fn spawn_portal(id: &str, x: f64, y: f64, z: f64, target_seed: u32);

    #[wasm_bindgen(js_name = threeBridgeRemovePortal)]
    pub fn remove_portal(id: &str);

    // F13: Water features
    #[wasm_bindgen(js_name = threeBridgeSpawnWaterfall)]
    pub fn spawn_waterfall(key: &str, x: f64, y: f64, z: f64, height: f64);

    #[wasm_bindgen(js_name = threeBridgeRemoveWaterfall)]
    pub fn remove_waterfall(key: &str);

    // F18: WebXR
    #[wasm_bindgen(js_name = threeBridgeIsVRSupported)]
    pub fn is_vr_supported() -> bool;

    #[wasm_bindgen(js_name = threeBridgeEnterVR)]
    pub fn enter_vr();

    #[wasm_bindgen(js_name = threeBridgeExitVR)]
    pub fn exit_vr();
}
