use encre::{Mark, Node};
use proptest::prelude::*;
use serde_json::{Map, Value};

const TEXT: &[char] = &[
    'a', 'b', 'Z', '1', '9', ' ', ' ', '\t', '*', '_', '[', ']', '(', ')', '#', '-', '>', '|', '\\', '`', '~', '!',
    '<', '&', ';', ':', '.', '/', '@', '=', '+', '"', '\'', '$', '{', 'é', '€', '中', '\u{a0}', '\u{2003}', 'w', 'h',
    't', 'p',
];

/// Characters allowed in single-line attributes: text characters without the tab.
pub const TEXT_LINE: &[char] = &['a', 'Z', ' ', '*', '"', '\\', '`', '<', '&', '|', 'é', '€'];

pub fn text() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(TEXT), 1..7).prop_map(|chars| chars.into_iter().collect())
}

fn link() -> impl Strategy<Value = Mark> {
    let href = prop::sample::select(vec!["https://x.com", "a b", "", "/p(1)", "<x>", "a&amp;b", "w\\x"]);
    let title = prop::sample::select(vec!["", "t", "a \"q\"", "&lt;"]);
    (href, title).prop_map(|(h, t)| {
        let mut attrs = Map::new();
        attrs.insert("href".into(), Value::from(h));
        attrs.insert("title".into(), Value::from(t));
        Mark::new("link", attrs)
    })
}

fn marks() -> impl Strategy<Value = Vec<Mark>> {
    (prop::option::weighted(0.15, link()), prop::collection::vec(any::<bool>(), 5)).prop_map(|(link, flags)| {
        let mut marks: Vec<Mark> = link.into_iter().collect();
        for (on, kind) in flags.into_iter().zip(["bold", "italic", "strike", "highlight", "code"]) {
            if on {
                marks.push(Mark::new(kind, Map::new()));
            }
        }
        marks
    })
}

fn piece() -> impl Strategy<Value = Node> {
    prop_oneof![
        8 => (text(), marks()).prop_map(|(t, m)| Node::text(&t, m)),
        1 => Just(Node::new("hardBreak", Map::new(), Vec::new())),
        1 => super::extra::badge(),
    ]
}

/// Merges adjacent text nodes with the same marks, as a valid document requires.
pub fn normalize(nodes: Vec<Node>) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::new();
    for node in nodes {
        match out.last_mut() {
            Some(last) if last.kind == "text" && node.kind == "text" && last.marks == node.marks => {
                let more = node.text.unwrap_or_default();
                last.text.get_or_insert_default().push_str(&more);
            }
            _ => out.push(node),
        }
    }
    out
}

fn without_linked_label_end(mut node: Node) -> Node {
    if ["link", "code"].iter().all(|k| node.marks.iter().any(|m| m.kind == *k)) {
        node.text = node.text.map(|t| t.replace("]:", "] :"));
    }
    node
}

pub fn inline() -> impl Strategy<Value = Vec<Node>> {
    prop::collection::vec(piece().prop_map(without_linked_label_end), 0..6).prop_map(normalize)
}

/// Inline content with no code text holding a backslash before a pipe, which a GFM cell cannot hold.
pub fn without_code_pipes(nodes: Vec<Node>) -> Vec<Node> {
    nodes
        .into_iter()
        .map(|mut node| {
            if node.marks.iter().any(|m| m.kind == "code") {
                node.text = node.text.map(|t| t.replace("\\|", "\\ |"));
            }
            node
        })
        .collect()
}
