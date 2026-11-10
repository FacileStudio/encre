use crate::model::Node;
use serde_json::{Map, Value};

const TONES: &[&str] = &["neutral", "accent", "info", "success", "warning", "danger"];

const STATES: &[(&str, &str)] = &[
    ("active", "success"),
    ("done", "success"),
    ("ready", "success"),
    ("error", "danger"),
    ("failed", "danger"),
    ("stopped", "danger"),
    ("dead", "danger"),
    ("pending", "warning"),
    ("wip", "warning"),
    ("running", "warning"),
    ("loading", "warning"),
];

fn tone(word: &str) -> Option<&'static str> {
    match word {
        "primary" => Some("accent"),
        _ => TONES.iter().find(|t| **t == word).copied(),
    }
}

fn state(word: &str) -> Option<&'static str> {
    STATES.iter().find(|(s, _)| *s == word).map(|(_, t)| *t).or_else(|| tone(word))
}

fn node(tone: &str, dot: bool, text: &str) -> Node {
    let mut attrs = Map::new();
    attrs.insert("tone".to_owned(), Value::from(tone));
    attrs.insert("dot".to_owned(), Value::from(dot));
    let content = (!text.is_empty()).then(|| Node::text(text, Vec::new()));
    Node::new("badge", attrs, content.into_iter().collect())
}

/// The badge written between brackets: `badge:<tone> <text>`, `status:<state>` or `status:<state> <text>`.
pub fn read(inner: &str) -> Option<Node> {
    let (head, text) = match inner.split_once(' ') {
        Some((head, text)) => (head, Some(text)),
        None => (inner, None),
    };
    if let Some(word) = head.strip_prefix("badge:") {
        return Some(node(tone(word)?, false, text.unwrap_or("")));
    }
    let word = head.strip_prefix("status:")?;
    Some(node(state(word)?, true, text.unwrap_or(word)))
}

/// The bracket head of `badge` and whether its text follows after a space.
pub fn head(badge: &Node) -> (String, bool) {
    let tone = match badge.attr_str("tone") {
        "" => "neutral",
        tone => tone,
    };
    let text: String = badge.content.iter().filter_map(|t| t.text.as_deref()).collect();
    let dot = badge.attrs.get("dot").and_then(Value::as_bool).unwrap_or(false);
    match (dot, text.is_empty()) {
        (false, true) => (format!("badge:{tone}"), false),
        (false, false) => (format!("badge:{tone}"), true),
        (true, _) if STATES.iter().any(|(s, t)| *s == text && *t == tone) || text == tone => {
            (format!("status:{text}"), false)
        }
        (true, _) => (format!("status:{tone}"), true),
    }
}
