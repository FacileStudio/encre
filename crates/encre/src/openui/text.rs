use crate::model::{Mark, Node};
use serde_json::{Map, Value};

fn printable(c: char) -> bool {
    c == '\t' || !c.is_control()
}

/// `text` on one line: line breaks become spaces and control characters other than tab go.
pub fn line(text: &str) -> String {
    text.replace("\r\n", "\n")
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .filter(|&c| printable(c))
        .collect()
}

/// `text` as a `label` attribute: one line, without backticks.
pub fn label(text: &str) -> String {
    line(text).replace('`', "'")
}

/// Inline content for `text`, each line break a `hardBreak`, every text node carrying `marks`.
pub fn inline(text: &str, marks: &[Mark]) -> Vec<Node> {
    let mut out = Vec::new();
    for (i, part) in text.replace("\r\n", "\n").split(['\n', '\r']).enumerate() {
        if i > 0 {
            out.push(Node::new("hardBreak", Map::new(), Vec::new()));
        }
        let clean: String = part.chars().filter(|&c| printable(c)).collect();
        if !clean.is_empty() {
            out.push(Node::text(&clean, marks.to_vec()));
        }
    }
    out
}

/// A paragraph holding `text`.
pub fn paragraph(text: &str, marks: &[Mark]) -> Node {
    Node::new("paragraph", Map::new(), inline(text.trim_matches(['\n', '\r']), marks))
}

/// One paragraph per run of lines between blank lines; at least one paragraph.
pub fn paragraphs(text: &str) -> Vec<Node> {
    let normal = text.replace("\r\n", "\n");
    let mut runs: Vec<Vec<&str>> = vec![Vec::new()];
    for l in normal.split('\n') {
        match (l.trim().is_empty(), runs.last_mut()) {
            (true, Some(run)) if !run.is_empty() => runs.push(Vec::new()),
            (false, Some(run)) => run.push(l),
            _ => {}
        }
    }
    runs.retain(|run| !run.is_empty());
    if runs.is_empty() {
        return vec![paragraph("", &[])];
    }
    runs.iter().map(|run| paragraph(&run.join("\n"), &[])).collect()
}

/// An attribute map from `(name, value)` pairs, leaving out empty strings.
pub fn attrs(pairs: Vec<(&str, Value)>) -> Map<String, Value> {
    pairs.into_iter().filter(|(_, v)| v.as_str() != Some("")).map(|(k, v)| (k.to_owned(), v)).collect()
}
