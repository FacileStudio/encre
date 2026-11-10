use super::serialize::{Writer, indent};
use crate::model::Node;
use serde_json::Value;

fn family(kind: &str) -> Option<&'static str> {
    match kind {
        "bulletList" | "taskList" => Some("bullet"),
        "orderedList" => Some("ordered"),
        _ => None,
    }
}

fn interrupts(node: &Node) -> bool {
    let start = node.attrs.get("start").and_then(Value::as_u64).unwrap_or(1);
    match node.kind.as_str() {
        "bulletList" | "taskList" | "codeBlock" | "heading" | "callout" | "stat" | "chart" | "diagram" => true,
        "orderedList" => start == 1,
        _ => false,
    }
}

impl Writer {
    /// Writes sibling blocks. In a list item (`item`), a block that can interrupt a paragraph follows it
    /// on the next line; the flag returned says a blank line was needed somewhere.
    pub fn sequence(&self, nodes: &[Node], item: bool) -> (String, bool) {
        let (mut out, mut loose, mut alternate) = (String::new(), false, false);
        let mut previous: Option<&Node> = None;
        for node in nodes {
            let listed = family(&node.kind);
            alternate = listed.is_some() && previous.and_then(|p| family(&p.kind)) == listed && !alternate;
            let written = if listed.is_some() { list(self, node, alternate) } else { self.block(node) };
            if self.commonmark && written.is_empty() {
                continue;
            }
            let tight = previous.map(|prev| item && prev.kind == "paragraph" && interrupts(node));
            loose |= tight == Some(false);
            out.push_str(match tight {
                None => "",
                Some(true) => "\n",
                Some(false) => "\n\n",
            });
            out.push_str(&written);
            previous = Some(node);
        }
        (out, loose)
    }

    /// Writes sibling blocks separated by blank lines.
    pub fn blocks(&self, nodes: &[Node]) -> String {
        self.sequence(nodes, false).0
    }
}

fn marker(list: &Node, index: usize, alternate: bool) -> String {
    if list.kind == "orderedList" {
        let start = list.attrs.get("start").and_then(Value::as_u64).unwrap_or(1);
        return format!("{}{} ", start + index as u64, if alternate { ')' } else { '.' });
    }
    format!("{} ", if alternate { '*' } else { '-' })
}

/// Writes a list; `alternate` switches the marker so it stays apart from an adjacent list.
pub fn list(writer: &Writer, node: &Node, alternate: bool) -> String {
    if family(&node.kind).is_none() {
        return String::new();
    }
    let items: Vec<(String, bool)> = node
        .content
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let mark = marker(node, i, alternate);
            let check = match item.attrs.get("checked") {
                Some(Value::Bool(true)) => "[x] ",
                _ if item.kind == "taskItem" => "[ ] ",
                _ => "",
            };
            let (body, loose) = writer.sequence(&item.content, true);
            (indent(&format!("{check}{body}"), &mark, &" ".repeat(mark.len())), loose)
        })
        .collect();
    let loose = items.iter().any(|(_, loose)| *loose);
    items.into_iter().map(|(text, _)| text).collect::<Vec<_>>().join(if loose { "\n\n" } else { "\n" })
}
