use super::md_tree::{Md, tree};
use super::parse_inline::{self, Item};
use crate::model::Node;
use pulldown_cmark::{Event, HeadingLevel, Tag};
use serde_json::{Map, Value};

/// Why markdown could not be read. Facile Markdown accepts any text, so this is an internal failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("markdown could not be read: {0}")]
pub struct ParseError(pub String);

/// An empty paragraph.
pub fn empty_paragraph() -> Node {
    Node::new("paragraph", Map::new(), Vec::new())
}

/// One attribute map.
pub fn attr(name: &str, value: impl Into<Value>) -> Map<String, Value> {
    let mut attrs = Map::new();
    attrs.insert(name.to_owned(), value.into());
    attrs
}

/// Reads Facile Markdown into a document without ids.
pub fn parse_doc(markdown: &str) -> Result<Node, ParseError> {
    let nodes = tree(markdown);
    let mut blocks = Reader { source: markdown }.blocks(&nodes);
    if blocks.is_empty() {
        blocks.push(empty_paragraph());
    }
    Ok(Node::new("doc", Map::new(), blocks))
}

fn level(level: &HeadingLevel) -> i64 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        _ => 3,
    }
}

pub(super) struct Reader<'a> {
    pub source: &'a str,
}

impl Reader<'_> {
    /// Converts sibling nodes; a run of inline nodes (a tight list item) is one paragraph.
    pub fn blocks(&self, nodes: &[Md]) -> Vec<Node> {
        let mut out = Vec::new();
        let mut start = 0;
        while start < nodes.len() {
            let run = nodes[start..].iter().take_while(|n| n.is_inline()).count();
            if run > 0 {
                out.extend(self.paragraph(&nodes[start..start + run]));
                start += run;
            } else {
                out.extend(self.block(&nodes[start]));
                start += 1;
            }
        }
        out
    }

    /// Converts sibling nodes, never returning an empty list.
    pub fn filled(&self, nodes: &[Md]) -> Vec<Node> {
        let blocks = self.blocks(nodes);
        if blocks.is_empty() { vec![empty_paragraph()] } else { blocks }
    }

    fn block(&self, node: &Md) -> Vec<Node> {
        let Md::Block { tag, range, children } = node else {
            return match node {
                Md::Leaf { event: Event::Rule, .. } => vec![Node::new("horizontalRule", Map::new(), Vec::new())],
                _ => Vec::new(),
            };
        };
        match tag {
            Tag::Paragraph => self.paragraph(children),
            Tag::Heading { level: l, .. } => vec![Node::new("heading", attr("level", level(l)), self.inline(children))],
            Tag::BlockQuote(_) => vec![Node::new("blockquote", Map::new(), self.filled(children))],
            Tag::CodeBlock(kind) => vec![self.code(kind, range.clone(), children)],
            Tag::List(start) => self.list(*start, children),
            Tag::Table(aligns) => vec![super::parse_table::table(self, aligns, children)],
            Tag::HtmlBlock => vec![super::parse_flow::html(children)],
            _ => self.blocks(children),
        }
    }

    /// Converts inline nodes where no image block can appear.
    pub fn inline(&self, nodes: &[Md]) -> Vec<Node> {
        let mut items = Vec::new();
        parse_inline::convert(nodes, &[], false, &mut items);
        let nodes = items.into_iter().filter_map(|item| if let Item::Inline(n) = item { Some(n) } else { None });
        parse_inline::normalize(nodes.collect())
    }
}
