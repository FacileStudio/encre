mod attrs;
mod inline;
mod table;

use crate::content;
use crate::model::Node;
use crate::schema::{Schema, schema};
use std::collections::HashSet;

/// Why a document does not validate, and where.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{path}: {message}")]
pub struct ValidationError {
    /// Where the problem is, as a JSON path from the root (`doc.content[2]`).
    pub path: String,
    /// What is wrong.
    pub message: String,
}

struct Walk<'a> {
    schema: &'a Schema,
    ids: HashSet<String>,
}

impl Walk<'_> {
    fn node(&mut self, node: &Node, path: &str) -> Result<(), ValidationError> {
        self.rules(node).map_err(|message| ValidationError { path: path.to_owned(), message })?;
        for (i, child) in node.content.iter().enumerate() {
            self.node(child, &format!("{path}.content[{i}]"))?;
        }
        Ok(())
    }

    fn rules(&mut self, node: &Node) -> Result<(), String> {
        let node_type = self.schema.node(&node.kind).ok_or_else(|| format!("unknown node type `{}`", node.kind))?;
        attrs::check(&node_type.attrs, &node.attrs)?;
        if node.text.is_some() != (node.kind == "text") {
            return Err("only `text` nodes carry text".to_owned());
        }
        let kinds: Vec<&str> = node.content.iter().map(|child| child.kind.as_str()).collect();
        if kinds.contains(&"doc") || !content::matches(self.schema, &node_type.content, &kinds) {
            return Err(format!("content {kinds:?} does not match `{}`", node_type.content));
        }
        inline::check(self.schema, node, node_type)?;
        if node.kind == "table" {
            table::check(node)?;
        }
        match node.id() {
            Some(id) if !self.ids.insert(id.to_owned()) => Err(format!("duplicate block id `{id}`")),
            _ => Ok(()),
        }
    }
}

/// Whether `attrs` are valid attributes for a node of type `kind`.
pub(crate) fn attrs_valid(kind: &str, attrs: &serde_json::Map<String, serde_json::Value>) -> bool {
    schema().node(kind).is_some_and(|t| attrs::check(&t.attrs, attrs).is_ok())
}

/// Validates `doc` against the current schema and the rules of `spec/FORMAT.md` section 1.
pub fn validate(doc: &Node) -> Result<(), ValidationError> {
    if doc.kind != "doc" {
        return Err(ValidationError { path: "doc".to_owned(), message: "the root is not a `doc`".to_owned() });
    }
    if !doc.marks.is_empty() {
        return Err(ValidationError { path: "doc".to_owned(), message: "the root carries marks".to_owned() });
    }
    Walk { schema: schema(), ids: HashSet::new() }.node(doc, "doc")
}
