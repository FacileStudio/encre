use super::error::OpError;
use crate::ids::assign_ids;
use crate::model::Node;
use crate::validate::validate;
use serde_json::{Map, Value};

/// Top-level blocks `at..at + removed` of the old document give way to `nodes`.
pub struct Replace {
    pub at: usize,
    pub removed: usize,
    pub nodes: Vec<Node>,
    pub keep_id: bool,
}

/// One splice, in old (`at`) and new (`new_at`) block positions.
pub struct Splice {
    #[cfg(feature = "yjs")]
    pub at: usize,
    #[cfg(feature = "yjs")]
    pub removed: usize,
    pub new_at: usize,
    pub inserted: usize,
}

/// The document after the operation, and the splices that turn the old one into it.
pub struct Edit {
    pub next: Node,
    pub splices: Vec<Splice>,
}

impl Replace {
    /// Inserts `nodes` before block `at`.
    pub fn insert(at: usize, nodes: Vec<Node>) -> Self {
        Replace { at, removed: 0, nodes, keep_id: false }
    }

    /// Replaces block `at` with `nodes`; the first one keeps the old id when it has the same type.
    pub fn one(at: usize, nodes: Vec<Node>) -> Self {
        Replace { at, removed: 1, nodes, keep_id: true }
    }
}

impl Edit {
    /// Ids of every inserted top-level block, in document order.
    pub fn inserted_ids(&self) -> Vec<String> {
        self.splices
            .iter()
            .flat_map(|s| &self.next.content[s.new_at..s.new_at + s.inserted])
            .filter_map(|block| block.id().map(str::to_owned))
            .collect()
    }
}

fn with_ids(doc: &Node, nodes: Vec<Node>) -> Vec<Node> {
    let kept = doc.content.len();
    let mut scratch = Node::new("doc", Map::new(), doc.content.iter().cloned().chain(nodes).collect());
    assign_ids(&mut scratch);
    scratch.content.split_off(kept)
}

fn carry_id(old: &Node, nodes: &mut [Node]) {
    if let (Some(id), Some(first)) = (old.id(), nodes.first_mut())
        && old.kind == first.kind
    {
        first.attrs.insert("id".to_owned(), Value::from(id));
    }
}

/// Applies `replaces` (ascending, not overlapping) to a copy of `doc`, gives new blocks ids, and validates it.
pub fn edit(doc: &Node, op: &'static str, replaces: Vec<Replace>) -> Result<Edit, OpError> {
    let mut content = Vec::new();
    let mut splices = Vec::new();
    let mut cursor = 0;
    for replace in replaces {
        content.extend_from_slice(&doc.content[cursor..replace.at]);
        let mut nodes = with_ids(doc, replace.nodes);
        if replace.keep_id && replace.removed == 1 {
            carry_id(&doc.content[replace.at], &mut nodes);
        }
        splices.push(Splice {
            #[cfg(feature = "yjs")]
            at: replace.at,
            #[cfg(feature = "yjs")]
            removed: replace.removed,
            new_at: content.len(),
            inserted: nodes.len(),
        });
        content.extend(nodes);
        cursor = replace.at + replace.removed;
    }
    content.extend_from_slice(&doc.content[cursor..]);
    if content.is_empty() {
        return Err(OpError::EmptyDocument { op });
    }
    let next = Node::new("doc", doc.attrs.clone(), content);
    validate(&next).map_err(|error| OpError::Invalid { op, error })?;
    Ok(Edit { next, splices })
}
