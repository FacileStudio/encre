use super::md_tree::Md;
use super::parse_inline::Scan;
pub(super) use super::reader::Reader;
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

impl Reader<'_> {
    /// Converts sibling nodes; a run of inline nodes (a tight list item) is one paragraph.
    pub fn blocks(&self, nodes: &[Md]) -> Vec<Node> {
        let mut out = Vec::new();
        self.blocks_into(nodes, &mut out);
        out
    }

    /// Converts sibling nodes onto the end of `out`.
    pub fn blocks_into(&self, nodes: &[Md], out: &mut Vec<Node>) {
        let mut start = 0;
        while start < nodes.len() {
            let run = nodes[start..].iter().take_while(|n| n.is_inline()).count().max(1);
            let at = out.len();
            if nodes[start].is_inline() {
                self.paragraph(&nodes[start..start + run], out);
            } else {
                self.block(&nodes[start], out);
            }
            self.stamp_from(out, at, &nodes[start].range());
            start += run;
        }
    }

    /// Converts sibling nodes, never returning an empty list.
    pub fn filled(&self, nodes: &[Md]) -> Vec<Node> {
        let blocks = self.blocks(nodes);
        if blocks.is_empty() { vec![empty_paragraph()] } else { blocks }
    }

    fn block(&self, node: &Md, out: &mut Vec<Node>) {
        let Md::Block { tag, range, depth, children } = node else {
            if let Md::Leaf { event: Event::Rule, .. } = node {
                out.push(Node::new("horizontalRule", Map::new(), Vec::new()));
            }
            return;
        };
        let built = match tag {
            Tag::Paragraph => return self.paragraph(children, out),
            Tag::Heading { level: l, .. } => {
                Node::new("heading", attr("level", (*l as i64).min(3)), self.inline(children))
            }
            Tag::BlockQuote(_) => self.quote(children),
            Tag::CodeBlock(kind) => self.code(kind, range.clone(), *depth, children),
            Tag::List(start) => return out.extend(self.list(*start, children)),
            Tag::Table(aligns) => super::parse_table::table(self, aligns, children),
            Tag::HtmlBlock => super::parse_flow::html(children),
            _ => return self.blocks_into(children, out),
        };
        out.push(built);
    }

    /// Converts inline nodes where no image block can appear.
    pub fn inline(&self, nodes: &[Md]) -> Vec<Node> {
        Scan { source: self.source, images: false }.inline(nodes)
    }
}
