//! WASM bindings for DDEX Builder: the core `ddex-builder` crate compiled to WebAssembly.
//!
//! `WasmDdexBuilder.build(request)` takes the same build request as the core builder and
//! the Node binding (the core `BuildRequest` fields, snake_case; see `build-request.d.ts`)
//! and returns the core builder's XML. Validation and errors are the core's.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

// The build request type, shared with the Node binding.
#[wasm_bindgen(typescript_custom_section)]
const BUILD_REQUEST_TS: &'static str = include_str!("../../node/build-request.d.ts");

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "BuildRequest")]
    pub type BuildRequestJs;
}

/// DDEX builder (the core builder).
#[wasm_bindgen]
pub struct WasmDdexBuilder {
    builder: ddex_builder::builder::DDEXBuilder,
}

#[wasm_bindgen]
impl WasmDdexBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmDdexBuilder {
        console_error_panic_hook::set_once();
        WasmDdexBuilder {
            builder: ddex_builder::builder::DDEXBuilder::new(),
        }
    }

    /// Build DDEX XML from a build request. `header` and `version` are required; missing
    /// `releases` / `deals` mean none. Nothing is filled in: a value the target ERN version
    /// requires but the request does not carry is reported as an error naming the field.
    pub fn build(&self, request: BuildRequestJs) -> Result<String, JsError> {
        let request = build_request(&request)?;
        let result = self
            .builder
            .build(request, ddex_builder::builder::BuildOptions::default())
            .map_err(|e| JsError::new(&format!("Build failed: {}", e)))?;
        Ok(result.xml)
    }
}

impl Default for WasmDdexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The core build request for a JS value, read the same way as by the Node binding: through
/// JSON (so integers stay integers and `undefined` fields are absent), with missing
/// `releases` / `deals` as empty lists.
fn build_request(request: &JsValue) -> Result<ddex_builder::builder::BuildRequest, JsError> {
    let json = js_sys::JSON::stringify(request)
        .map_err(|_| JsError::new("the build request must be JSON-serializable"))?
        .as_string()
        .ok_or_else(|| JsError::new("the build request must be an object"))?;
    let mut value: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| JsError::new(&format!("Invalid build request: {}", e)))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| JsError::new("the build request must be an object"))?;
    object
        .entry("releases")
        .or_insert_with(|| serde_json::json!([]));
    object
        .entry("deals")
        .or_insert_with(|| serde_json::json!([]));
    serde_json::from_value(value)
        .map_err(|e| JsError::new(&format!("Invalid build request: {}", e)))
}

/// The package version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
