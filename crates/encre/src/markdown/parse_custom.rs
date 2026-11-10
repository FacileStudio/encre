use super::blocks::{CUSTOM, DIAGRAMS, attrs_from, read_info};
use super::md_tree::Md;
use super::parse::{Reader, attr};
use crate::model::Node;
use crate::validate::attrs_valid;
use pulldown_cmark::CodeBlockKind;
use serde_json::{Map, Value};
use std::ops::Range;

fn data_block(name: &str, rest: &str, value: &str) -> Option<Node> {
    if !rest.trim().is_empty() {
        return None;
    }
    let Value::Object(map) = serde_json::from_str(value).ok()? else {
        return None;
    };
    let attrs = attrs_from(name, map.into_iter().collect())?;
    attrs_valid(name, &attrs).then(|| Node::new(name, attrs, Vec::new()))
}

/// The attributes of a fence whose info string holds `key="value"` pairs; None when malformed.
pub fn info_attrs(name: &str, rest: &str) -> Option<Map<String, Value>> {
    let pairs = read_info(rest)?.into_iter().map(|(k, v)| (k, Value::String(v))).collect();
    attrs_from(name, pairs).filter(|attrs| attrs_valid(name, attrs))
}

fn diagram(name: &str, rest: &str, value: &str) -> Option<Node> {
    let content = (!value.is_empty()).then(|| Node::text(value, Vec::new()));
    rest.trim().is_empty().then(|| Node::new("diagram", attr("language", name), content.into_iter().collect()))
}

impl Reader<'_> {
    /// Reads a custom block from its raw info string (after the name) and the content of a fence opening at
    /// `at` inside `depth` blocks; None when malformed.
    fn custom(&self, name: &str, rest: &str, value: &str, at: (usize, usize)) -> Option<Node> {
        let inner = |fences| self.nested(value, at.0, at.1, fences);
        match name {
            "callout" => Some(Node::new(name, info_attrs(name, rest)?, inner(1).doc().content)),
            "columns" => rest.trim().is_empty().then(|| inner(1).columns()).flatten(),
            "compare" | "pros-cons" | "vs" => rest.trim().is_empty().then(|| inner(2).compare()).flatten(),
            _ if DIAGRAMS.contains(&name) => diagram(name, rest, value),
            _ => data_block(name, rest, value),
        }
    }

    /// The name and the rest of the info string of the backtick fence opening at `start`, read raw.
    pub fn fence_info(&self, start: usize) -> Option<(&str, &str)> {
        let line = self.source.get(start..).unwrap_or("").lines().next().unwrap_or("").trim_start();
        let raw = line.strip_prefix("```")?.trim_start_matches('`').trim();
        let name = raw.split_whitespace().next().unwrap_or("");
        Some((name, &raw[name.len()..]))
    }

    /// Converts a code block, or the custom block a backtick fence names.
    pub fn code(&self, kind: &CodeBlockKind, range: Range<usize>, depth: usize, children: &[Md]) -> Node {
        let mut text: String = children.iter().map(Md::plain).collect();
        if text.ends_with('\n') {
            text.pop();
        }
        let info = match kind {
            CodeBlockKind::Fenced(info) => info.to_string(),
            CodeBlockKind::Indented => String::new(),
        };
        let custom = self.fence_info(range.start).filter(|(name, _)| CUSTOM.contains(name));
        if matches!(kind, CodeBlockKind::Fenced(_))
            && let Some((name, rest)) = custom
            && let Some(node) = self.custom(name, rest, &text, (range.start, depth))
        {
            return node;
        }
        let language = info.split_ascii_whitespace().collect::<Vec<_>>().join(" ");
        let content = (!text.is_empty()).then(|| Node::text(&text, Vec::new()));
        Node::new("codeBlock", attr("language", language), content.into_iter().collect())
    }
}
