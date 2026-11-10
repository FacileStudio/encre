//! Facile Markdown: the lossless markdown projection, and the portable CommonMark export.

mod blocks;
mod compare;
mod escape;
mod inline;
mod json;
mod md_tree;
mod parse;
mod parse_custom;
mod parse_flow;
mod parse_inline;
mod parse_table;
mod plain;
mod serialize;
mod tokens;
mod write_code;
mod write_list;
mod write_table;

use crate::model::Node;
use serialize::Writer;

pub use parse::ParseError;
pub use plain::plain_text;

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
