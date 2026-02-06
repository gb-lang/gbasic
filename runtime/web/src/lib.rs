//! G-Basic web runtime — wasm-bindgen stubs.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn runtime_init(_width: i32, _height: i32) {
    log("G-Basic web runtime initialized");
}

#[wasm_bindgen]
pub fn runtime_clear_screen(_r: u8, _g: u8, _b: u8) {
    // Will be implemented via JS glue calling canvas context
}

#[wasm_bindgen]
pub fn runtime_present() {
    // No-op for web — requestAnimationFrame handles presentation
}

#[wasm_bindgen]
pub fn runtime_should_quit() -> i32 {
    0 // Web apps don't quit via this mechanism
}

#[wasm_bindgen]
pub fn runtime_shutdown() {
    log("G-Basic web runtime shutdown");
}
