use super::blocks::CUSTOM;
use super::blocks::{attrs_from, read_info};
use super::md_tree::Md;
use super::parse::{Reader, attr, parse_doc};
use crate::model::Node;
use crate::validate::attrs_valid;
use pulldown_cmark::CodeBlockKind;
use serde_json::Value;
use std::ops::Range;

/// Reads a custom block from its raw info string (after the name) and fence content; None when malformed.
pub fn custom(name: &str, rest: &str, value: &str) -> Option<Node> {
    let attrs = if name == "callout" {
        let pairs = read_info(rest)?.into_iter().map(|(k, v)| (k, Value::String(v))).collect();
        attrs_from(name, pairs)?
    } else {
        if !rest.trim().is_empty() {
            return None;
        }
        let Value::Object(map) = serde_json::from_str(value).ok()? else {
            return None;
        };
        attrs_from(name, map.into_iter().collect())?
    };
    if !attrs_valid(name, &attrs) {
        return None;
    }
    let content = if name == "callout" { parse_doc(value).ok()?.content } else { Vec::new() };
    Some(Node::new(name, attrs, content))
}

impl Reader<'_> {
    /// Converts a code block, or the custom block a backtick fence names.
    pub fn code(&self, kind: &CodeBlockKind, range: Range<usize>, children: &[Md]) -> Node {
        let mut text: String = children.iter().map(Md::plain).collect();
        if text.ends_with('\n') {
            text.pop();
        }
        let info = match kind {
            CodeBlockKind::Fenced(info) => info.to_string(),
            CodeBlockKind::Indented => String::new(),
        };
        let line = self.source.get(range.start..).unwrap_or("").lines().next().unwrap_or("").trim_start();
        let raw = line.strip_prefix("```").map(|rest| rest.trim_start_matches('`').trim());
        let name = raw.and_then(|r| r.split_whitespace().next()).unwrap_or("");
        if matches!(kind, CodeBlockKind::Fenced(_))
            && CUSTOM.contains(&name)
            && let Some(node) = custom(name, &raw.unwrap_or("")[name.len()..], &text)
        {
            return node;
        }
        let language = info.split_ascii_whitespace().collect::<Vec<_>>().join(" ");
        let content = (!text.is_empty()).then(|| Node::text(&text, Vec::new()));
        Node::new("codeBlock", attr("language", language), content.into_iter().collect())
    }
}
