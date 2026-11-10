use super::md_tree::{Md, tree};
use super::parse::Reader;
use super::parse_custom::info_attrs;
use crate::model::Node;
use pulldown_cmark::{CodeBlockKind, Tag};
use serde_json::{Map, Value};

/// A `columns` block of `built`, None unless it holds two to four columns.
pub fn columns(built: Vec<Node>) -> Option<Node> {
    (2..=4).contains(&built.len()).then(|| Node::new("columns", Map::new(), built))
}

/// A `column` fence whose info string reads: its attributes, content, offset and depth.
type Head = (Map<String, Value>, String, usize, usize);

impl Reader<'_> {
    fn head(&self, node: &Md) -> Option<Head> {
        let Md::Block { tag: Tag::CodeBlock(CodeBlockKind::Fenced(_)), range, depth, children } = node else {
            return None;
        };
        let (name, rest) = self.fence_info(range.start).filter(|(name, _)| *name == "column")?;
        let mut text: String = children.iter().map(Md::plain).collect();
        if text.ends_with('\n') {
            text.pop();
        }
        Some((info_attrs(name, rest)?, text, range.start, *depth))
    }

    /// The `columns` block this fence content holds: two to four `column` fences and nothing else. No column
    /// content is read unless every fence qualifies; content too deep for column fences is not columns.
    pub fn columns(&self) -> Option<Node> {
        let nodes = tree(self.source, self.room).ok()?;
        if !(2..=4).contains(&nodes.len()) {
            return None;
        }
        let heads = nodes.iter().map(|node| self.head(node)).collect::<Option<Vec<_>>>()?;
        let built = heads.into_iter().map(|(attrs, text, offset, depth)| {
            Node::new("column", attrs, self.nested(&text, offset, depth, 1).doc().content)
        });
        columns(built.collect())
    }
}
