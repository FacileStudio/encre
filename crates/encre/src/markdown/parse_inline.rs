use super::md_tree::Md;
use crate::model::{Mark, Node};
use crate::schema::schema;
use pulldown_cmark::{Event, Tag};
use serde_json::{Map, Value};

/// One converted inline item: inline content, or an image that splits a paragraph.
pub enum Item {
    Inline(Node),
    Image(Node),
}

fn with_mark(marks: &[Mark], mark: Mark) -> Vec<Mark> {
    let mut next = marks.to_vec();
    if !next.iter().any(|m| m.kind == mark.kind) {
        next.push(mark);
        next.sort_by_key(|m| schema().mark_rank(&m.kind));
    }
    next
}

fn pairs(pairs: &[(&str, &str)]) -> Map<String, Value> {
    pairs.iter().map(|(k, v)| ((*k).to_owned(), Value::from(*v))).collect()
}

/// The image block for `src`, `alt` and `title`.
pub fn image(src: &str, alt: &str, title: &str) -> Node {
    Node::new("image", pairs(&[("src", src), ("alt", alt), ("title", title)]), Vec::new())
}

fn text(value: &str, marks: &[Mark]) -> Item {
    Item::Inline(Node::text(&value.replace('\n', " "), marks.to_vec()))
}

fn leaf(event: &Event, marks: &[Mark]) -> Option<Item> {
    let hard_break = || Item::Inline(Node::new("hardBreak", Map::new(), Vec::new()));
    Some(match event {
        Event::Text(t) => text(t, marks),
        Event::Code(c) => text(c, &with_mark(marks, Mark::new("code", Map::new()))),
        Event::InlineHtml(h) | Event::Html(h)
            if matches!(h.to_ascii_lowercase().as_str(), "<br>" | "<br/>" | "<br />") =>
        {
            hard_break()
        }
        Event::InlineHtml(h) | Event::Html(h) => text(h, marks),
        Event::SoftBreak => text(" ", marks),
        Event::HardBreak => hard_break(),
        _ => return None,
    })
}

/// Converts inline nodes to items, carrying `marks` down. With `images` false, an image becomes its alt text.
pub fn convert(nodes: &[Md], marks: &[Mark], images: bool, out: &mut Vec<Item>) {
    for node in nodes {
        let (tag, children) = match node {
            Md::Leaf { event, .. } => {
                out.extend(leaf(event, marks));
                continue;
            }
            Md::Block { tag, children, .. } => (tag, children),
        };
        let mark = match tag {
            Tag::Emphasis => Mark::new("italic", Map::new()),
            Tag::Strong => Mark::new("bold", Map::new()),
            Tag::Strikethrough => Mark::new("strike", Map::new()),
            Tag::Link { dest_url, title, .. } => Mark::new("link", pairs(&[("href", dest_url), ("title", title)])),
            Tag::Image { dest_url, title, .. } if images => {
                out.push(Item::Image(image(dest_url, &node.plain(), title)));
                continue;
            }
            _ => {
                out.push(text(&node.plain(), marks));
                continue;
            }
        };
        convert(children, &with_mark(marks, mark), images, out);
    }
}

/// Merges adjacent text nodes with equal marks and drops empty text.
pub fn normalize(nodes: Vec<Node>) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::new();
    for node in nodes {
        if node.kind == "text" && node.text.as_deref().unwrap_or("").is_empty() {
            continue;
        }
        match out.last_mut() {
            Some(last) if last.kind == "text" && node.kind == "text" && last.marks == node.marks => {
                last.text.get_or_insert_default().push_str(node.text.as_deref().unwrap_or(""));
            }
            _ => out.push(node),
        }
    }
    out
}
