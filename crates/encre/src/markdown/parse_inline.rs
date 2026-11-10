use super::md_tree::Md;
use super::parse_span::{Part, parts};
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

/// Converts inline markdown nodes, reading badges and highlights from the `source` text around them.
pub struct Scan<'s> {
    pub source: &'s str,
    pub images: bool,
}

impl Scan<'_> {
    /// Converts inline nodes to items, carrying `marks` down. With `images` false, an image becomes its alt text.
    pub fn convert(&self, nodes: &[Md], marks: &[Mark], out: &mut Vec<Item>) {
        let highlighted = with_mark(marks, Mark::new("highlight", Map::new()));
        for part in parts(nodes, self.source) {
            let marks = if part.highlighted() { &highlighted[..] } else { marks };
            match part {
                Part::Text(value, _) => out.push(text(&value, marks)),
                Part::Badge(badge) => out.push(Item::Inline(badge)),
                Part::Node(node, _) => self.node(node, marks, out),
            }
        }
    }

    fn node(&self, node: &Md, marks: &[Mark], out: &mut Vec<Item>) {
        let (tag, children) = match node {
            Md::Leaf { event, .. } => return out.extend(leaf(event, marks)),
            Md::Block { tag, children, .. } => (tag, children),
        };
        let mark = match tag {
            Tag::Emphasis => Mark::new("italic", Map::new()),
            Tag::Strong => Mark::new("bold", Map::new()),
            Tag::Strikethrough => Mark::new("strike", Map::new()),
            Tag::Link { dest_url, title, .. } => Mark::new("link", pairs(&[("href", dest_url), ("title", title)])),
            Tag::Image { dest_url, title, .. } if self.images => {
                return out.push(Item::Image(image(dest_url, &node.plain(), title)));
            }
            _ => return out.push(text(&node.plain(), marks)),
        };
        self.convert(children, &with_mark(marks, mark), out);
    }

    /// Converts inline nodes where no image block can appear.
    pub fn inline(&self, nodes: &[Md]) -> Vec<Node> {
        let mut items = Vec::new();
        self.convert(nodes, &[], &mut items);
        let nodes = items.into_iter().filter_map(|item| if let Item::Inline(n) = item { Some(n) } else { None });
        super::parse_flow::normalize(nodes.collect())
    }
}
