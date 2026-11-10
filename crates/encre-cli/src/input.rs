//! Telling markdown from JSON, and loading a document from either.

use crate::cli::Format;
use crate::error::CliError;
use encre::{Node, validate};

/// The format of `text`: `from` when given, else JSON when the first non-blank character is `{`.
/// Input that starts with `{` but is not JSON is ambiguous and refused as a usage error.
pub fn detect(text: &str, from: Option<Format>) -> Result<Format, CliError> {
    if let Some(format) = from {
        return Ok(format);
    }
    if !text.trim_start().starts_with('{') {
        return Ok(Format::Markdown);
    }
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(_) => Ok(Format::Json),
        Err(_) => Err(CliError::usage("input starts with { but is not JSON; pass --from markdown or --from json")),
    }
}

/// An encre JSON document, validated against schema v2.
pub fn json_doc(text: &str) -> Result<Node, CliError> {
    let doc: Node = serde_json::from_str(text)
        .map_err(|e| CliError::invalid(format!("not an encre JSON document: {e}")).at("doc"))?;
    validate(&doc)?;
    Ok(doc)
}

/// A Facile Markdown document, without block ids.
pub fn markdown_doc(text: &str) -> Result<Node, CliError> {
    encre::markdown::parse(text).map_err(|e| CliError::invalid(format!("cannot parse markdown: {}", e.0)))
}

/// A document from `text` in the format [`detect`] finds.
pub fn load(text: &str, from: Option<Format>) -> Result<Node, CliError> {
    match detect(text, from)? {
        Format::Json => json_doc(text),
        Format::Markdown => markdown_doc(text),
    }
}
