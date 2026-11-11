use crate::escape::quoted;
use crate::model::{Mark, Node};

const SCHEMES: [&str; 4] = ["http", "https", "mailto", "tel"];

/// Whether a reader may follow `href` from the PDF: http, https, mailto, tel or relative, as `Document` allows.
fn followable(href: &str) -> bool {
    if href.chars().any(char::is_control) {
        return false;
    }
    let trimmed = href.trim();
    let scheme = trimmed.split_once(':').map(|(scheme, _)| scheme).filter(|scheme| {
        scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'))
    });
    scheme.is_none_or(|scheme| SCHEMES.iter().any(|allowed| scheme.eq_ignore_ascii_case(allowed)))
}

fn link(mark: &Mark, inner: String) -> String {
    match mark.attrs.get("href").and_then(|v| v.as_str()).filter(|href| followable(href)) {
        Some(href) => format!("#link({})[{inner}]", quoted(href)),
        None => inner,
    }
}

fn wrap(mark: &Mark, inner: String) -> String {
    match mark.kind.as_str() {
        "bold" => format!("#strong[{inner}]"),
        "italic" => format!("#emph[{inner}]"),
        "strike" => format!("#strike[{inner}]"),
        "highlight" => format!("#highlight[{inner}]"),
        "link" => link(mark, inner),
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
