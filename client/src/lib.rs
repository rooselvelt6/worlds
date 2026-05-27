pub mod app;
pub mod engine;
pub mod i18n;
pub mod math;
pub mod state;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    i18n::init();

    #[cfg(feature = "parallel")]
    {
        let n = web_sys::window()
            .and_then(|w| Some(w.navigator().hardware_concurrency()))
            .unwrap_or(4.0) as usize;
        let _ = wasm_bindgen_rayon::init_thread_pool(n);
    }

    leptos::mount::mount_to_body(app::App);
}
