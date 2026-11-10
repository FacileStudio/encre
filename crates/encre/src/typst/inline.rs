use crate::escape::quoted;
use crate::model::{Mark, Node};

fn wrap(mark: &Mark, inner: String) -> String {
    match mark.kind.as_str() {
        "bold" => format!("#strong[{inner}]"),
        "italic" => format!("#emph[{inner}]"),
        "strike" => format!("#strike[{inner}]"),
        "highlight" => format!("#highlight[{inner}]"),
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

fn badge(node: &Node) -> String {
    let text: String = node.content.iter().filter_map(|t| t.text.as_deref()).collect();
    let tone = quoted(match node.attr_str("tone") {
        "" => "neutral",
        tone => tone,
    });
    if node.attrs.get("dot").and_then(|v| v.as_bool()).unwrap_or(false) {
        format!("#box[#status-dot(tone: {tone})#h(0.3em)#{}]", quoted(&text))
    } else {
        format!("#badge({}, tone: {tone})", quoted(&text))
    }
}

/// Inline content as Typst markup: text as string literals, marks as function calls.
pub fn inline(nodes: &[Node]) -> String {
    nodes
        .iter()
        .map(|node| match node.kind.as_str() {
            "hardBreak" => "#linebreak()".to_owned(),
            "badge" => badge(node),
            _ => text(node),
        })
        .collect()
}
