use crate::validate::ValidationError;
use std::fmt;

/// One place a `patch` found its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchMatch {
    /// The block id, or `#index` when the block has none.
    pub block: String,
    /// The match with a little surrounding text.
    pub context: String,
}

/// Why an operation was refused. Messages say what matched and how to narrow the request.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum OpError {
    /// No top-level block has this id.
    #[error("{op}: no top-level block has id {id}; read_blocks lists the block ids")]
    UnknownBlock {
        /// Operation name.
        op: &'static str,
        /// The id asked for.
        id: String,
    },
    /// The id names a block inside another block.
    #[error("{op}: block {id} is inside block {parent}; operations address top-level blocks, so target {parent}")]
    NestedBlock {
        /// Operation name.
        op: &'static str,
        /// The id asked for.
        id: String,
        /// The top-level block that holds it.
        parent: String,
    },
    /// The markdown holds no block.
    #[error("{op}: the markdown holds no block; to remove a block use delete_block")]
    EmptyMarkdown {
        /// Operation name.
        op: &'static str,
    },
    /// The edit would leave the document without blocks.
    #[error("{op}: the document would have no block left; use replace_block or replace_all instead")]
    EmptyDocument {
        /// Operation name.
        op: &'static str,
    },
    /// `read_blocks` asked for blocks that do not exist.
    #[error("read_blocks: range {start}..{end} is outside the {len} blocks; use a range within 0..{len}")]
    Range {
        /// Requested start.
        start: usize,
        /// Requested end.
        end: usize,
        /// How many top-level blocks there are.
        len: usize,
    },
    /// `patch` with an empty `find` or an `expect` of zero.
    #[error("patch: {0}")]
    BadPatch(&'static str),
    /// `patch` found a different number of matches than expected.
    #[error("{}", patch_message(*.expect, .found))]
    PatchMatches {
        /// Matches required.
        expect: usize,
        /// Matches found.
        found: Vec<PatchMatch>,
    },
    /// `replace_all` against a document that changed since the agent read it.
    #[error("replace_all expected version {expected}, the document is at version {actual}; read it again and retry")]
    Version {
        /// The version the agent read.
        expected: u64,
        /// The current version.
        actual: u64,
    },
    /// The edit would produce an invalid document.
    #[error("{op}: the result would be invalid ({error}); change the markdown")]
    Invalid {
        /// Operation name.
        op: &'static str,
        /// The validation failure.
        error: ValidationError,
    },
    /// The open document could not be read.
    #[cfg(feature = "yjs")]
    #[error("the open document cannot be read: {0}")]
    Yjs(#[from] crate::yjs::YjsError),
}

fn patch_message(expect: usize, found: &[PatchMatch]) -> String {
    let plural = if expect == 1 { "match" } else { "matches" };
    let head = format!("patch expected {expect} {plural}, found {}", found.len());
    if found.is_empty() {
        return format!("{head}; find must be exact text of one block's markdown, as read_blocks shows it");
    }
    let list: Vec<String> = found.iter().map(|m| m.to_string()).collect();
    let hint = if found.len() > expect { "add more surrounding text to find" } else { "lower expect or check find" };
    format!("{head}: {}; {hint}", list.join(", "))
}

impl fmt::Display for PatchMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "block {} \"{}\"", self.block, self.context)
    }
}
