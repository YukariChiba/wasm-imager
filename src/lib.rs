use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

mod disk;
mod generator;
mod layout;
mod resolver;
mod schema;
mod utils;

use schema::SchemaLayout;

#[wasm_bindgen]
pub fn generate_layout(schema_js: JsValue, file_sizes_js: JsValue) -> Result<JsValue, JsValue> {
    let schema: SchemaLayout = serde_wasm_bindgen::from_value(schema_js)
        .map_err(|e| JsValue::from_str(&format!("Schema resolve failed: {}", e)))?;

    let file_sizes: BTreeMap<String, u64> = serde_wasm_bindgen::from_value(file_sizes_js)
        .map_err(|e| JsValue::from_str(&format!("File Sizes resolve failed: {}", e)))?;

    let resolved = resolver::resolve(&schema, &file_sizes)
        .map_err(|e| JsValue::from_str(&format!("Config resolve failed: {}", e)))?;

    let result = generator::generate(&schema, resolved)
        .map_err(|e| JsValue::from_str(&format!("Layout generation failed: {}", e)))?;

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
}
