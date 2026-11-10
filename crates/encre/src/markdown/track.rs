use super::parse::Reader;
use crate::model::Node;
use serde_json::Value;
use std::ops::Range;

/// The attribute a tracked parse leaves on each block until [`take`] collects it.
pub const AT: &str = "\u{0}at";

/// Maps byte offsets of one source to 1-based lines and columns of the outer document.
pub struct Track {
    line: usize,
    column: usize,
    starts: Vec<usize>,
}

impl Track {
    /// A map over `source`, whose first line is line `line + 1` of the document, shifted `column` columns.
    pub fn new(source: &str, line: usize, column: usize) -> Track {
        let starts = std::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1)).collect();
        Track { line, column, starts }
    }

    /// The 1-based line and column of `offset` in `source`.
    pub fn position(&self, source: &str, offset: usize) -> (usize, usize) {
        let index = self.starts.partition_point(|start| *start <= offset).saturating_sub(1);
        let start = self.starts[index];
        let column = source.get(start..offset).map_or(0, |s| s.chars().count());
        (self.line + index + 1, self.column + column + 1)
    }
}

/// Puts the position of `offset` on every node of `nodes` that has none yet.
pub fn stamp(nodes: &mut [Node], position: (usize, usize)) {
    for node in nodes.iter_mut().filter(|n| n.kind != "text" && !n.attrs.contains_key(AT)) {
        node.attrs.insert(AT.to_owned(), Value::from(vec![position.0, position.1]));
    }
}

/// Removes every position from `node` and returns them by tree path (`doc.content[1]`).
pub fn take(node: &mut Node, path: &str, out: &mut Vec<(String, (usize, usize))>) {
    if let Some(Value::Array(at)) = node.attrs.remove(AT) {
        let n = |i: usize| at.get(i).and_then(Value::as_u64).unwrap_or(1) as usize;
        out.push((path.to_owned(), (n(0), n(1))));
    }
    for (i, child) in node.content.iter_mut().enumerate() {
        take(child, &format!("{path}.content[{i}]"), out);
    }
}

impl Reader<'_> {
    /// A reader for `inner`, the content of a fence opening at `offset`, tracked when this one is.
    pub fn nested<'b>(&self, inner: &'b str, offset: usize) -> Reader<'b> {
        let track = self.track.as_ref().map(|t| {
            let (line, column) = t.position(self.source, offset);
            Track::new(inner, line, column - 1)
        });
        Reader { source: inner, track }
    }

    /// Puts the source position of `range` on each of `nodes` when this reader tracks positions.
    pub fn stamped(&self, mut nodes: Vec<Node>, range: &Range<usize>) -> Vec<Node> {
        if let Some(track) = &self.track {
            stamp(&mut nodes, track.position(self.source, range.start));
        }
        nodes
    }
}
