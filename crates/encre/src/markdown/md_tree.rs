use pulldown_cmark::{Event, Options, Parser, Tag};
use std::ops::Range;

/// A pulldown-cmark event stream folded into a tree, with source ranges.
pub enum Md<'a> {
    Block { tag: Tag<'a>, range: Range<usize>, children: Vec<Md<'a>> },
    Leaf { event: Event<'a>, range: Range<usize> },
}

fn options() -> Options {
    Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH
}

/// Parses `source` as CommonMark with GFM tables, task lists and strikethrough.
pub fn tree(source: &str) -> Vec<Md<'_>> {
    let mut stack: Vec<(Tag, Range<usize>, Vec<Md>)> = Vec::new();
    let mut root = Vec::new();
    for (event, range) in Parser::new_ext(source, options()).into_offset_iter() {
        let node = match event {
            Event::Start(tag) => {
                stack.push((tag, range, Vec::new()));
                continue;
            }
            Event::End(_) => match stack.pop() {
                Some((tag, range, children)) => Md::Block { tag, range, children },
                None => continue,
            },
            event => Md::Leaf { event, range },
        };
        match stack.last_mut() {
            Some((_, _, children)) => children.push(node),
            None => root.push(node),
        }
    }
    root
}

impl Md<'_> {
    /// Child nodes; a leaf has none.
    pub fn children(&self) -> &[Md<'_>] {
        match self {
            Md::Block { children, .. } => children,
            Md::Leaf { .. } => &[],
        }
    }

    /// The source range of this node.
    pub fn range(&self) -> Range<usize> {
        match self {
            Md::Block { range, .. } | Md::Leaf { range, .. } => range.clone(),
        }
    }

    /// Whether this node belongs in inline content.
    pub fn is_inline(&self) -> bool {
        match self {
            Md::Block { tag, .. } => {
                matches!(tag, Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link { .. } | Tag::Image { .. })
            }
            Md::Leaf { event, .. } => !matches!(event, Event::Rule),
        }
    }

    /// The text of every leaf below, for image alt text.
    pub fn plain(&self) -> String {
        match self {
            Md::Leaf { event: Event::Text(t) | Event::Code(t) | Event::InlineHtml(t) | Event::Html(t), .. } => {
                t.to_string()
            }
            Md::Leaf { event: Event::SoftBreak | Event::HardBreak, .. } => " ".to_owned(),
            other => other.children().iter().map(Md::plain).collect(),
        }
    }
}
