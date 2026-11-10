use crate::model::Node;
use serde_json::Value;

fn inline(nodes: &[Node]) -> String {
    nodes
        .iter()
        .map(|n| match n.kind.as_str() {
            "hardBreak" => "\n".to_owned(),
            "badge" => inline(&n.content),
            _ => n.text.clone().unwrap_or_default(),
        })
        .collect()
}

fn data(node: &Node) -> String {
    let mut parts: Vec<String> = ["title", "label", "value", "delta", "sub"]
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
        "paragraph" | "heading" | "codeBlock" | "diagram" => inline(&node.content),
        "tableRow" => node.content.iter().map(|cell| blocks(&cell.content)).collect::<Vec<_>>().join("\t"),
        "image" => node.attr_str("alt").to_owned(),
        "stat" | "chart" => data(node),
        "callout" | "column" => [node.attr_str("title").to_owned(), blocks(&node.content)]
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
