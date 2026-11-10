use super::edit::{Edit, Replace, edit};
use super::error::OpError;
use super::{BlockView, Op, Outcome, patch};
use crate::markdown::{parse, serialize};
use crate::model::Node;
use crate::validate::ValidationError;
use serde_json::Map;

/// What an operation answers, and the edit it makes when it writes.
pub struct Plan {
    pub outcome: Outcome,
    pub edit: Option<Edit>,
}

/// Checks `op` against `doc` and works out its full effect, without changing anything.
pub fn plan(doc: &Node, version: u64, op: &Op) -> Result<Plan, OpError> {
    let name = op.name();
    let replace = match op {
        Op::Read => return Ok(Plan { outcome: Outcome::Markdown(serialize(doc)), edit: None }),
        Op::ReadBlocks { range } => return read_blocks(doc, *range),
        Op::Append { markdown } => Replace::insert(doc.content.len(), blocks(name, markdown)?),
        Op::Prepend { markdown } => Replace::insert(0, blocks(name, markdown)?),
        Op::InsertAfter { block_id, markdown } => {
            Replace::insert(index(doc, name, block_id)? + 1, blocks(name, markdown)?)
        }
        Op::ReplaceBlock { block_id, markdown } => Replace::one(index(doc, name, block_id)?, blocks(name, markdown)?),
        Op::DeleteBlock { block_id } => Replace::one(index(doc, name, block_id)?, Vec::new()),
        Op::Patch { find, replace, expect } => return finish(doc, name, patch::replaces(doc, find, replace, *expect)?),
        Op::ReplaceAll { markdown, expect_version } => replace_all(doc, version, *expect_version, markdown)?,
    };
    finish(doc, name, vec![replace])
}

fn finish(doc: &Node, op: &'static str, replaces: Vec<Replace>) -> Result<Plan, OpError> {
    let edit = edit(doc, op, replaces)?;
    Ok(Plan { outcome: Outcome::Changed(edit.inserted_ids()), edit: Some(edit) })
}

fn replace_all(doc: &Node, version: u64, expected: u64, markdown: &str) -> Result<Replace, OpError> {
    if version != expected {
        return Err(OpError::Version { expected, actual: version });
    }
    Ok(Replace { at: 0, removed: doc.content.len(), nodes: blocks("replace_all", markdown)?, keep_id: false })
}

/// The top-level blocks `markdown` holds.
pub fn blocks(op: &'static str, markdown: &str) -> Result<Vec<Node>, OpError> {
    if markdown.trim().is_empty() {
        return Err(OpError::EmptyMarkdown { op });
    }
    let refused = |e: crate::markdown::ParseError| ValidationError { path: "doc".to_owned(), message: e.0 };
    parse(markdown).map(|doc| doc.content).map_err(|e| OpError::Invalid { op, error: refused(e) })
}

/// The markdown of one block on its own.
pub fn block_markdown(block: &Node) -> String {
    serialize(&Node::new("doc", Map::new(), vec![block.clone()]))
}

fn read_blocks(doc: &Node, range: Option<[usize; 2]>) -> Result<Plan, OpError> {
    let len = doc.content.len();
    let [start, end] = range.unwrap_or([0, len]);
    if start > end || end > len {
        return Err(OpError::Range { start, end, len });
    }
    let views = doc.content[start..end]
        .iter()
        .map(|block| BlockView {
            id: block.id().map(str::to_owned),
            kind: block.kind.clone(),
            markdown: block_markdown(block),
        })
        .collect();
    Ok(Plan { outcome: Outcome::Blocks(views), edit: None })
}

fn index(doc: &Node, op: &'static str, id: &str) -> Result<usize, OpError> {
    if let Some(i) = doc.content.iter().position(|block| block.id() == Some(id)) {
        return Ok(i);
    }
    let parent = doc.content.iter().find(|block| contains(block, id)).map(|block| block.id().unwrap_or("without id"));
    Err(match parent {
        Some(parent) => OpError::NestedBlock { op, id: id.to_owned(), parent: parent.to_owned() },
        None => OpError::UnknownBlock { op, id: id.to_owned() },
    })
}

fn contains(node: &Node, id: &str) -> bool {
    node.content.iter().any(|child| child.id() == Some(id) || contains(child, id))
}
