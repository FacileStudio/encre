//! encre's markdown projection and OpenUI adapter for the browser. Documents cross the boundary as JSON strings.
#![deny(unused, dead_code)]

use encre::Node;
use wasm_bindgen::prelude::*;

fn read(doc: &str) -> Result<Node, JsError> {
    let node: Node = serde_json::from_str(doc).map_err(|e| JsError::new(&format!("document is not JSON: {e}")))?;
    encre::validate(&node).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(node)
}

/// Reads Facile Markdown and returns the document as ProseMirror JSON, without ids.
#[wasm_bindgen]
pub fn parse(markdown: &str) -> Result<String, JsError> {
    let doc = encre::markdown::parse(markdown).map_err(|e| JsError::new(&e.to_string()))?;
    serde_json::to_string(&doc).map_err(|e| JsError::new(&e.to_string()))
}

/// Writes a valid document as canonical Facile Markdown.
#[wasm_bindgen]
pub fn serialize(doc: &str) -> Result<String, JsError> {
    Ok(encre::markdown::serialize(&read(doc)?))
}

/// Writes a valid document as portable CommonMark.
#[wasm_bindgen]
pub fn serialize_commonmark(doc: &str) -> Result<String, JsError> {
    Ok(encre::markdown::serialize_commonmark(&read(doc)?))
}

/// Validates a document; the error message says what is wrong and where.
#[wasm_bindgen]
pub fn validate(doc: &str) -> Result<(), JsError> {
    read(doc).map(drop)
}

/// Converts an OpenUI Lang program to a document without ids; the error says why it cannot.
#[wasm_bindgen(js_name = openuiToDocument)]
pub fn openui_to_document(source: &str) -> Result<String, JsError> {
    let doc = encre::openui::to_document(source).map_err(|e| JsError::new(&e.to_string()))?;
    serde_json::to_string(&doc).map_err(|e| JsError::new(&e.to_string()))
}
