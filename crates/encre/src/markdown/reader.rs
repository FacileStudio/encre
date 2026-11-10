use super::md_tree::{TooDeep, Tree};
use super::parse::{ParseError, empty_paragraph};
use super::track::Track;
use crate::model::Node;
use serde_json::Map;
use std::cell::Cell;
use std::rc::Rc;

/// How deep blocks and inline spans may nest in markdown (`spec/FORMAT.md` section 2.6).
pub const MARKDOWN_DEPTH: usize = 64;

/// Why a reader stopped: the rule broken and the byte offset in the reader's source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Refusal {
    pub message: &'static str,
    pub at: Option<(usize, usize)>,
}

/// The refusal for markdown that nests deeper than [`MARKDOWN_DEPTH`].
pub const TOO_DEEP: &str = "blocks and inline spans nest more than 64 levels deep";

/// The refusal for custom blocks that nest deeper than [`crate::validate::MAX_CUSTOM`].
pub const TOO_MANY_FENCES: &str = "custom blocks nest more than 8 levels deep";

/// Reads Facile Markdown into a document without ids.
pub fn parse_doc(markdown: &str) -> Result<Node, ParseError> {
    let reader = Reader::root(markdown, None);
    let doc = reader.doc();
    match reader.refused.get() {
        Some(refusal) => Err(ParseError(refusal.message.to_owned())),
        None => Ok(doc),
    }
}

pub(super) struct Reader<'a> {
    pub source: &'a str,
    pub track: Option<Track>,
    pub room: usize,
    pub fences: usize,
    pub refused: Rc<Cell<Option<Refusal>>>,
}

impl<'a> Reader<'a> {
    /// A reader for a whole document.
    pub fn root(source: &'a str, track: Option<Track>) -> Reader<'a> {
        Reader { source, track, room: MARKDOWN_DEPTH, fences: 0, refused: Rc::default() }
    }

    /// Records the first rule the source breaks, at byte `offset`.
    pub fn refuse(&self, message: &'static str, offset: usize) {
        if self.refused.get().is_none() {
            let at = self.track.as_ref().map(|t| t.position(self.source, offset));
            self.refused.set(Some(Refusal { message, at }));
        }
    }

    /// The whole source as a document, never empty; empty when it nests too deep. Converts one top-level block
    /// at a time, so the parser's tree of a block is gone before the next one is read.
    pub fn doc(&self) -> Node {
        let mut tree = Tree::new(self.source, self.room);
        let (mut blocks, mut inline) = (Vec::new(), Vec::new());
        for node in tree.by_ref() {
            if node.is_inline() {
                inline.push(node);
                continue;
            }
            self.blocks_into(&std::mem::take(&mut inline), &mut blocks);
            self.blocks_into(std::slice::from_ref(&node), &mut blocks);
        }
        self.blocks_into(&inline, &mut blocks);
        if let Some(TooDeep(offset)) = tree.too_deep {
            self.refuse(TOO_DEEP, offset);
            blocks.clear();
        }
        Node::new("doc", Map::new(), if blocks.is_empty() { vec![empty_paragraph()] } else { blocks })
    }

    /// A reader for `inner`, the content of a fence opening at `offset` inside `depth` blocks that opens
    /// `fences` custom blocks; tracked when this one is. Past the custom block limit it reads nothing.
    pub fn nested<'b>(&self, inner: &'b str, offset: usize, depth: usize, fences: usize) -> Reader<'b> {
        let track = self.track.as_ref().map(|t| {
            let (line, column) = t.position(self.source, offset);
            Track::new(inner, line, column - 1)
        });
        let fences = self.fences + fences;
        let over = fences > crate::validate::MAX_CUSTOM;
        if over {
            self.refuse(TOO_MANY_FENCES, offset);
        }
        let room = if over { 0 } else { self.room.saturating_sub(depth + 1) };
        Reader { source: inner, track, room, fences, refused: self.refused.clone() }
    }

    /// An untracked reader for text built from this reader's source, with the same limits.
    pub fn part<'b>(&self, text: &'b str) -> Reader<'b> {
        Reader { source: text, track: None, room: self.room, fences: self.fences, refused: self.refused.clone() }
    }
}
