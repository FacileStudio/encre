use crate::model::Node;
use serde_json::Value;

fn inline(nodes: &[Node]) -> String {
    nodes.iter().map(|n| if n.kind == "hardBreak" { "\n" } else { n.text.as_deref().unwrap_or("") }).collect()
}

fn data(node: &Node) -> String {
    let mut parts: Vec<String> = ["title", "label", "value", "delta"]
        .iter()
        .map(|key| node.attr_str(key).to_owned())
        .filter(|s| !s.is_empty())
        .collect();
    if let Some(Value::Array(labels)) = node.attrs.get("labels") {
        parts.extend(labels.iter().filter_map(Value::as_str).map(str::to_owned));
    }
    parts.join(" ")
}

fn block(node: &Node) -> String {
    match node.kind.as_str() {
        "paragraph" | "heading" | "codeBlock" => inline(&node.content),
        "tableRow" => node.content.iter().map(|cell| blocks(&cell.content)).collect::<Vec<_>>().join("\t"),
        "image" => node.attr_str("alt").to_owned(),
        "stat" | "chart" => data(node),
        "callout" => [node.attr_str("title").to_owned(), blocks(&node.content)]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => blocks(&node.content),
    }
}

fn blocks(nodes: &[Node]) -> String {
    nodes.iter().map(block).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n")
}

/// The text of `doc` without markup: one line per block, cells separated by tabs. For search and previews.
pub fn plain_text(doc: &Node) -> String {
    blocks(&doc.content)
}
