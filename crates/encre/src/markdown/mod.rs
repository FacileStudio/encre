//! Facile Markdown: the lossless markdown projection, and the portable CommonMark export.

mod badge;
mod blocks;
mod compare;
mod escape;
mod inline;
mod json;
mod md_tree;
mod parse;
mod parse_alert;
mod parse_columns;
mod parse_compare;
mod parse_custom;
mod parse_flow;
mod parse_highlight;
mod parse_inline;
mod parse_span;
mod parse_table;
mod plain;
mod serialize;
mod source;
mod tokens;
mod track;
mod write_code;
mod write_list;
mod write_table;

use crate::model::Node;
use serialize::Writer;
use track::Track;

pub use parse::ParseError;
pub use plain::plain_text;
pub use source::SourceError;

/// Reads Facile Markdown. Any text is accepted; blocks get no ids (see [`crate::assign_ids`]).
pub fn parse(markdown: &str) -> Result<Node, ParseError> {
    parse::parse_doc(markdown)
}

/// Writes `doc` as canonical Facile Markdown: the same tree always gives the same bytes.
pub fn serialize(doc: &Node) -> String {
    let minimal = Writer { full: false, commonmark: false }.blocks(&doc.content);
    let same = parse::parse_doc(&minimal).is_ok_and(|back| compare::comparable(&back) == compare::comparable(doc));
    if same { minimal } else { Writer { full: true, commonmark: false }.blocks(&doc.content) }
}

/// Writes `doc` as portable CommonMark with GFM. Lossy: see `spec/FORMAT.md` section 3.
pub fn serialize_commonmark(doc: &Node) -> String {
    Writer { full: false, commonmark: true }.blocks(&doc.content)
}

/// Reads Facile Markdown and validates the document it gives. The error names the tree path and the 1-based
/// line and column of the source block that holds the problem.
pub fn validate(markdown: &str) -> Result<Node, SourceError> {
    let mut doc = parse::Reader { source: markdown, track: Some(Track::new(markdown, 0, 0)) }.doc();
    let mut positions = Vec::new();
    track::take(&mut doc, "doc", &mut positions);
    crate::validate(&doc).map(|()| doc).map_err(|e| source::locate(e, &positions))
}
