use super::md_tree::{Md, tree};
use super::parse_inline::Scan;
use super::track::Track;
use crate::model::Node;
use pulldown_cmark::{Event, Tag};
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
    Ok(Reader { source: markdown, track: None }.doc())
}

pub(super) struct Reader<'a> {
    pub source: &'a str,
    pub track: Option<Track>,
}

impl Reader<'_> {
    /// The whole source as a document, never empty.
    pub fn doc(&self) -> Node {
        Node::new("doc", Map::new(), self.filled(&tree(self.source)))
    }

    /// Converts sibling nodes; a run of inline nodes (a tight list item) is one paragraph.
    pub fn blocks(&self, nodes: &[Md]) -> Vec<Node> {
        let mut out = Vec::new();
        let mut start = 0;
        while start < nodes.len() {
            let run = nodes[start..].iter().take_while(|n| n.is_inline()).count().max(1);
            let built = if nodes[start].is_inline() {
                self.paragraph(&nodes[start..start + run])
            } else {
                self.block(&nodes[start])
            };
            out.extend(self.stamped(built, &nodes[start].range()));
            start += run;
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
            Tag::Heading { level: l, .. } => {
                vec![Node::new("heading", attr("level", (*l as i64).min(3)), self.inline(children))]
            }
            Tag::BlockQuote(_) => vec![self.quote(children)],
            Tag::CodeBlock(kind) => vec![self.code(kind, range.clone(), children)],
            Tag::List(start) => self.list(*start, children),
            Tag::Table(aligns) => vec![super::parse_table::table(self, aligns, children)],
            Tag::HtmlBlock => vec![super::parse_flow::html(children)],
            _ => self.blocks(children),
        }
    }

    /// Converts inline nodes where no image block can appear.
    pub fn inline(&self, nodes: &[Md]) -> Vec<Node> {
        Scan { source: self.source, images: false }.inline(nodes)
    }
}
