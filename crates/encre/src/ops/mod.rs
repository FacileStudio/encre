//! Agent operations (`spec/FORMAT.md` section 4): read markdown, write through guarded edits.
//!
//! Every operation is checked completely before anything changes. Operations address top-level blocks by id.
//! [`apply`] edits a stored [`Node`]; with feature `yjs`, [`apply_yjs`] edits an open document in place,
//! inserting, removing or replacing single elements so other people's cursors survive.

mod edit;
mod error;
mod patch;
mod plan;

use crate::model::Node;
use serde::{Deserialize, Serialize};

pub use error::{LISTED, OpError, PatchMatch};

/// One agent operation, in the JSON form agents send: `{"op": "insert_after", "block_id": …, "markdown": …}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Op {
    /// The document as Facile Markdown.
    Read,
    /// Top-level blocks with their ids, optionally only blocks `range[0]..range[1]`.
    ReadBlocks {
        /// Start and end (exclusive) block index.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        range: Option<[usize; 2]>,
    },
    /// Adds blocks at the end.
    Append {
        /// Facile Markdown of the new blocks.
        markdown: String,
    },
    /// Adds blocks at the start.
    Prepend {
        /// Facile Markdown of the new blocks.
        markdown: String,
    },
    /// Adds blocks after one block.
    InsertAfter {
        /// The top-level block to insert after.
        block_id: String,
        /// Facile Markdown of the new blocks.
        markdown: String,
    },
    /// Replaces one block, and only that block: a heading's section stays.
    ReplaceBlock {
        /// The top-level block to replace.
        block_id: String,
        /// Facile Markdown of the replacement.
        markdown: String,
    },
    /// Removes one block.
    DeleteBlock {
        /// The top-level block to remove.
        block_id: String,
    },
    /// Replaces exact markdown text inside blocks, when it matches exactly `expect` times.
    Patch {
        /// Exact text of a block's markdown, as `read_blocks` shows it.
        find: String,
        /// The text to put in its place.
        replace: String,
        /// How many matches there must be.
        #[serde(default = "one")]
        expect: usize,
    },
    /// Replaces the whole document, only while it is still at `expect_version`.
    ReplaceAll {
        /// Facile Markdown of the new document.
        markdown: String,
        /// The version the agent read.
        expect_version: u64,
    },
}

fn one() -> usize {
    1
}

/// One top-level block as `read_blocks` returns it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockView {
    /// The block id, null until assigned.
    pub id: Option<String>,
    /// The node type.
    #[serde(rename = "type")]
    pub kind: String,
    /// The block alone as Facile Markdown.
    pub markdown: String,
}

/// What an operation answers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// `read`: the document as Facile Markdown.
    Markdown(String),
    /// `read_blocks`: the blocks asked for.
    Blocks(Vec<BlockView>),
    /// A write: the ids of the top-level blocks it inserted.
    Changed(Vec<String>),
}

impl Op {
    /// The operation name, as agents write it.
    pub fn name(&self) -> &'static str {
        match self {
            Op::Read => "read",
            Op::ReadBlocks { .. } => "read_blocks",
            Op::Append { .. } => "append",
            Op::Prepend { .. } => "prepend",
            Op::InsertAfter { .. } => "insert_after",
            Op::ReplaceBlock { .. } => "replace_block",
            Op::DeleteBlock { .. } => "delete_block",
            Op::Patch { .. } => "patch",
            Op::ReplaceAll { .. } => "replace_all",
        }
    }
}

/// Applies `op` to `doc`, whose stored version is `version`. On error `doc` is unchanged.
pub fn apply(doc: &mut Node, version: u64, op: &Op) -> Result<Outcome, OpError> {
    let plan = plan::plan(doc, version, op)?;
    if let Some(edit) = plan.edit {
        *doc = edit.next;
    }
    Ok(plan.outcome)
}

/// Applies `op` to the document open in `fragment`, inside the caller's transaction.
#[cfg(feature = "yjs")]
pub fn apply_yjs(
    txn: &mut yrs::TransactionMut,
    fragment: &yrs::XmlFragmentRef,
    version: u64,
    op: &Op,
) -> Result<Outcome, OpError> {
    use yrs::XmlFragment;
    let doc = crate::yjs::read(txn, fragment)?;
    let plan = plan::plan(&doc, version, op)?;
    if let Some(edit) = &plan.edit {
        for s in edit.splices.iter().rev() {
            if s.removed > 0 {
                fragment.remove_range(txn, s.at as u32, s.removed as u32);
            }
            crate::yjs::insert_blocks(txn, fragment, s.at as u32, &edit.next.content[s.new_at..s.new_at + s.inserted]);
        }
    }
    Ok(plan.outcome)
}
