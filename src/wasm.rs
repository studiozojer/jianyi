// src/wasm.rs — thin wasm-bindgen wrappers over json_api
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn cast(request: &str) -> Result<String, JsError> {
    crate::json_api::cast_json(request).map_err(|e| JsError::new(&e))
}

#[wasm_bindgen]
pub fn hexagram(request: &str) -> Result<String, JsError> {
    crate::json_api::hexagram_json(request).map_err(|e| JsError::new(&e))
}
