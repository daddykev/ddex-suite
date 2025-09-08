// packages/ddex-parser/bindings/wasm/src/lib.rs
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::{from_value, to_value};
use ddex_parser_core::{DDEXParser as CoreParser, ParserOptions, ParseResult};
use ddex_core::models::{ERNMessage, FlattenedRelease};

#[wasm_bindgen]
pub struct DDEXParser {
    inner: CoreParser,
}

#[wasm_bindgen]
impl DDEXParser {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<DDEXParser, JsValue> {
        console_error_panic_hook::set_once();
        
        Ok(DDEXParser {
            inner: CoreParser::new(),
        })
    }
    
    #[wasm_bindgen]
    pub fn parse(&self, xml: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let opts: ParserOptions = from_value(options)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        let result = self.inner.parse(xml, opts)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    
    #[wasm_bindgen]
    pub async fn parse_stream(
        &self, 
        stream: web_sys::ReadableStream,
        options: JsValue
    ) -> Result<JsValue, JsValue> {
        // Implement Web Streams API support
        todo!("Streaming implementation")
    }
    
    #[wasm_bindgen]
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}