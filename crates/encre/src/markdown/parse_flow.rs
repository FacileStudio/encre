use super::md_tree::Md;
use super::parse::{Reader, attr, empty_paragraph};
use super::parse_inline::{Item, Scan};
use crate::model::Node;
use pulldown_cmark::{Event, Tag};
use serde_json::{Map, Value};

fn lone_backslash(children: &[Md], source: &str) -> usize {
    let n = children.len();
    let previous = children.get(n.wrapping_sub(2));
    let after_break =
        n == 1 || matches!(previous, Some(Md::Leaf { event: Event::HardBreak | Event::TaskListMarker(_), .. }));
    match children.last() {
        Some(Md::Leaf { event: Event::Text(t), range }) if t.as_ref() == "\\" && after_break => {
            let escaped = source.get(..range.start).is_some_and(|before| before.ends_with('\\'));
            if source.get(range.clone()) == Some("\\") && !escaped { n - 1 } else { n }
        }
        _ => n,
    }
}

fn trimmed(mut nodes: Vec<Node>, start: bool, end: bool) -> Vec<Node> {
    if let Some(text) = nodes.first_mut().filter(|_| start).and_then(|n| n.text.as_mut()) {
        let cut = text.len() - text.trim_start_matches([' ', '\t']).len();
        text.drain(..cut);
    }
    if let Some(text) = nodes.last_mut().filter(|_| end).and_then(|n| n.text.as_mut()) {
        text.truncate(text.trim_end_matches([' ', '\t']).len());
    }
    normalize(nodes)
}

fn task_marker(item: &[Md]) -> Option<bool> {
    let first = item.first()?;
    let leaf = match first {
        Md::Block { tag: Tag::Paragraph, children, .. } => children.first()?,
        other => other,
    };
    match leaf {
        Md::Leaf { event: Event::TaskListMarker(checked), .. } => Some(*checked),
        _ => None,
    }
}

/// Converts an HTML block to a paragraph of its lines, as text.
pub fn html(children: &[Md]) -> Node {
    let raw: String = children.iter().map(Md::plain).collect();
    let mut content = Vec::new();
    for (i, line) in raw.trim_end_matches('\n').split('\n').enumerate() {
        if i > 0 {
            content.push(Node::new("hardBreak", Map::new(), Vec::new()));
        }
        content.push(Node::text(line, Vec::new()));
    }
    Node::new("paragraph", Map::new(), normalize(content))
}

impl Reader<'_> {
    /// Converts a paragraph onto the end of `out`, splitting it around images.
    pub fn paragraph(&self, children: &[Md], out: &mut Vec<Node>) {
        let keep = lone_backslash(children, self.source);
        let mut items = Vec::new();
        Scan { source: self.source, images: true }.convert(&children[..keep], &[], &mut items);
        let (start, mut current, mut after_image) = (out.len(), Vec::new(), false);
        for item in items {
            match item {
                Item::Inline(node) => current.push(node),
                Item::Image(image) => {
                    let part = trimmed(std::mem::take(&mut current), after_image, true);
                    out.extend((!part.is_empty()).then(|| Node::new("paragraph", Map::new(), part)));
                    out.push(image);
                    after_image = true;
                }
            }
        }
        let part = trimmed(current, after_image, false);
        if !part.is_empty() || out.len() == start {
            out.push(Node::new("paragraph", Map::new(), part));
        }
    }

    fn item(&self, item: &Md, task: Option<bool>) -> Node {
        let mut content = self.blocks(item.children());
        if content.first().is_none_or(|block| block.kind != "paragraph") {
            content.insert(0, empty_paragraph());
        }
        let built = match task {
            Some(checked) => Node::new("taskItem", attr("checked", checked), content),
            None => Node::new("listItem", Map::new(), content),
        };
        self.stamped(vec![built], &item.range()).remove(0)
    }

    /// Converts a list; a bullet list is split into runs of task and plain items.
    pub fn list(&self, start: Option<u64>, items: &[Md]) -> Vec<Node> {
        if let Some(start) = start {
            let built = items.iter().map(|i| self.item(i, None)).collect();
            return vec![Node::new("orderedList", attr("start", Value::from(start.min(999_999_999))), built)];
        }
        items
            .chunk_by(|a, b| task_marker(a.children()).is_some() == task_marker(b.children()).is_some())
            .map(|run| {
                let task = task_marker(run[0].children()).is_some();
                let built = run.iter().map(|i| self.item(i, task_marker(i.children()))).collect();
                Node::new(if task { "taskList" } else { "bulletList" }, Map::new(), built)
            })
            .collect()
    }
}

/// Merges adjacent text nodes with equal marks and drops empty text.
pub fn normalize(mut nodes: Vec<Node>) -> Vec<Node> {
    nodes.retain(|node| node.kind != "text" || !node.text.as_deref().unwrap_or("").is_empty());
    nodes.dedup_by(|node, last| {
        let merge = last.kind == "text" && node.kind == "text" && last.marks == node.marks;
        if merge {
            last.text.get_or_insert_default().push_str(node.text.as_deref().unwrap_or(""));
        }
        merge
    });
    nodes
}
