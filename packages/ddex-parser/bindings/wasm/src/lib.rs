// packages/ddex-parser/bindings/wasm/src/lib.rs
use ddex_parser::DDEXParser as CoreParser;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

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
    pub fn parse(&mut self, xml: &str) -> Result<JsValue, JsValue> {
        let cursor = std::io::Cursor::new(xml.as_bytes());
        let result = self
            .inner
            .parse(cursor)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}
