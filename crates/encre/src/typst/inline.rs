use crate::escape::quoted;
use crate::model::{Mark, Node};

fn wrap(mark: &Mark, inner: String) -> String {
    match mark.kind.as_str() {
        "bold" => format!("#strong[{inner}]"),
        "italic" => format!("#emph[{inner}]"),
        "strike" => format!("#strike[{inner}]"),
        "link" => format!("#link({})[{inner}]", quoted(mark.attrs.get("href").and_then(|v| v.as_str()).unwrap_or(""))),
        _ => inner,
    }
}

fn text(node: &Node) -> String {
    let value = node.text.as_deref().unwrap_or("");
    let code = node.marks.iter().any(|m| m.kind == "code");
    let leaf = if code { format!("#raw({})", quoted(value)) } else { format!("#{}", quoted(value)) };
    node.marks.iter().rev().fold(leaf, |inner, mark| wrap(mark, inner))
}

/// Inline content as Typst markup: text as string literals, marks as function calls.
pub fn inline(nodes: &[Node]) -> String {
    nodes
        .iter()
        .map(|node| match node.kind.as_str() {
            "hardBreak" => "#linebreak()".to_owned(),
            _ => text(node),
        })
        .collect()
}
