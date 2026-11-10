use super::md_tree::Md;
use super::parse::{Reader, attr, empty_paragraph};
use super::parse_inline::{self, Item};
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
        *text = text.trim_start_matches([' ', '\t']).to_owned();
    }
    if let Some(text) = nodes.last_mut().filter(|_| end).and_then(|n| n.text.as_mut()) {
        *text = text.trim_end_matches([' ', '\t']).to_owned();
    }
    parse_inline::normalize(nodes)
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
    Node::new("paragraph", Map::new(), parse_inline::normalize(content))
}

impl Reader<'_> {
    /// Converts a paragraph, splitting it around images.
    pub fn paragraph(&self, children: &[Md]) -> Vec<Node> {
        let keep = lone_backslash(children, self.source);
        let mut items = Vec::new();
        parse_inline::convert(&children[..keep], &[], true, &mut items);
        let (mut blocks, mut current, mut after_image) = (Vec::new(), Vec::new(), false);
        for item in items {
            match item {
                Item::Inline(node) => current.push(node),
                Item::Image(image) => {
                    let part = trimmed(std::mem::take(&mut current), after_image, true);
                    blocks.extend((!part.is_empty()).then(|| Node::new("paragraph", Map::new(), part)));
                    blocks.push(image);
                    after_image = true;
                }
            }
        }
        let part = trimmed(current, after_image, false);
        if !part.is_empty() || blocks.is_empty() {
            blocks.push(Node::new("paragraph", Map::new(), part));
        }
        blocks
    }

    fn item(&self, item: &Md, task: Option<bool>) -> Node {
        let mut content = self.blocks(item.children());
        if content.first().is_none_or(|block| block.kind != "paragraph") {
            content.insert(0, empty_paragraph());
        }
        match task {
            Some(checked) => Node::new("taskItem", attr("checked", checked), content),
            None => Node::new("listItem", Map::new(), content),
        }
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
