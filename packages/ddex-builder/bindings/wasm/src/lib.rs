use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::{from_value, to_value};
use ddex_builder::{DdexBuilder as RustBuilder, BuildRequest, BuildOptions};

#[wasm_bindgen]
pub struct DDEXBuilder {
    inner: RustBuilder,
}

#[wasm_bindgen]
impl DDEXBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<DDEXBuilder, JsValue> {
        console_error_panic_hook::set_once();
        
        Ok(DDEXBuilder {
            inner: RustBuilder::new(),
        })
    }
    
    #[wasm_bindgen]
    pub async fn build(&self, request: JsValue, options: JsValue) -> Result<JsValue, JsValue> {
        let req: BuildRequest = from_value(request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        let opts: BuildOptions = if !options.is_undefined() && !options.is_null() {
            from_value(options).map_err(|e| JsValue::from_str(&e.to_string()))?
        } else {
            BuildOptions::default()
        };
            
        let result = self.inner.build(req, opts)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn build_sync(&self, request: JsValue, options: JsValue) -> Result<JsValue, JsValue> {
        let req: BuildRequest = from_value(request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        let opts: BuildOptions = if !options.is_undefined() && !options.is_null() {
            from_value(options).map_err(|e| JsValue::from_str(&e.to_string()))?
        } else {
            BuildOptions::default()
        };
            
        let result = self.inner.build_sync(req, opts)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }
    
    #[wasm_bindgen]
    pub fn canonicalize(&self, xml: &str) -> Result<String, JsValue> {
        self.inner.canonicalize(xml)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}