use super::md_tree::{Md, tree};
use super::parse::Reader;
use super::parse_custom::info_attrs;
use crate::model::Node;
use pulldown_cmark::{CodeBlockKind, Tag};
use serde_json::Map;

/// A `columns` block of `built`, None unless it holds two to four columns.
pub fn columns(built: Vec<Node>) -> Option<Node> {
    (2..=4).contains(&built.len()).then(|| Node::new("columns", Map::new(), built))
}

impl Reader<'_> {
    fn column(&self, node: &Md) -> Option<Node> {
        let Md::Block { tag: Tag::CodeBlock(CodeBlockKind::Fenced(_)), range, children } = node else {
            return None;
        };
        let (name, rest) = self.fence_info(range.start).filter(|(name, _)| *name == "column")?;
        let mut text: String = children.iter().map(Md::plain).collect();
        if text.ends_with('\n') {
            text.pop();
        }
        Some(Node::new(name, info_attrs(name, rest)?, self.nested(&text, range.start).doc().content))
    }

    /// The `columns` block this fence content holds: two to four `column` fences and nothing else.
    pub fn columns(&self) -> Option<Node> {
        columns(tree(self.source).iter().map(|node| self.column(node)).collect::<Option<Vec<_>>>()?)
    }
}
