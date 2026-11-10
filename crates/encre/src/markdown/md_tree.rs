use pulldown_cmark::{Event, Options, Parser, Tag};
use std::ops::Range;

/// A pulldown-cmark event stream folded into a tree, with source ranges. `depth` counts the blocks that hold it.
pub enum Md<'a> {
    Block { tag: Tag<'a>, range: Range<usize>, depth: usize, children: Vec<Md<'a>> },
    Leaf { event: Event<'a>, range: Range<usize> },
}

/// The markdown nests more than the room a reader has left, first at this byte offset.
pub struct TooDeep(pub usize);

fn push<'a>(siblings: &mut Vec<Md<'a>>, node: Md<'a>, source: &'a str) {
    let literal = |text: &str, range: &Range<usize>| {
        let at = source.get(range.clone());
        at.is_some_and(|at| std::ptr::eq(at, text)) || at == Some(text)
    };
    if let (Some(Md::Leaf { event: Event::Text(last), range }), Md::Leaf { event: Event::Text(next), range: after }) =
        (siblings.last_mut(), &node)
        && range.end == after.start
        && literal(last, range)
        && literal(next, after)
    {
        range.end = after.end;
        *last = source[range.clone()].into();
        return;
    }
    siblings.push(node);
}

/// The top-level nodes of `source` read as CommonMark with GFM tables, task lists and strikethrough, one at a
/// time. Adjacent text that the source writes literally becomes one leaf. Reading stops, with `too_deep` set,
/// once blocks and spans nest more than `room` deep.
pub struct Tree<'a> {
    source: &'a str,
    events: pulldown_cmark::OffsetIter<'a, pulldown_cmark::DefaultBrokenLinkCallback>,
    stack: Vec<(Tag<'a>, Range<usize>, Vec<Md<'a>>)>,
    room: usize,
    /// Where the source first nests too deep.
    pub too_deep: Option<TooDeep>,
}

impl<'a> Tree<'a> {
    /// A reader of the top-level nodes of `source`.
    pub fn new(source: &'a str, room: usize) -> Tree<'a> {
        let options = Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;
        let events = Parser::new_ext(source, options).into_offset_iter();
        Tree { source, events, stack: Vec::new(), room, too_deep: None }
    }
}

impl<'a> Iterator for Tree<'a> {
    type Item = Md<'a>;

    fn next(&mut self) -> Option<Md<'a>> {
        while self.too_deep.is_none() {
            let (event, range) = self.events.next()?;
            let node = match event {
                Event::Start(_) if self.stack.len() >= self.room => {
                    self.too_deep = Some(TooDeep(range.start));
                    return None;
                }
                Event::Start(tag) => {
                    self.stack.push((tag, range, Vec::new()));
                    continue;
                }
                Event::End(_) => match self.stack.pop() {
                    Some((tag, range, children)) => Md::Block { tag, range, depth: self.stack.len(), children },
                    None => continue,
                },
                event => Md::Leaf { event, range },
            };
            match self.stack.last_mut() {
                Some((_, _, children)) => push(children, node, self.source),
                None => return Some(node),
            }
        }
        None
    }
}

/// Every top-level node of `source`, or where it nests more than `room` deep.
pub fn tree(source: &str, room: usize) -> Result<Vec<Md<'_>>, TooDeep> {
    let mut tree = Tree::new(source, room);
    let nodes: Vec<Md> = tree.by_ref().collect();
    tree.too_deep.map_or(Ok(nodes), Err)
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
