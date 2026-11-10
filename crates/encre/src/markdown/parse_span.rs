use super::badge;
use super::md_tree::Md;
use super::parse_highlight::delimiters;
use crate::model::Node;
use pulldown_cmark::Event;

/// One character of plain text: `syntax` when it is a bracket or `=` the source wrote bare, so it may delimit;
/// `space` when it is literal whitespace.
#[derive(Clone, Copy)]
pub struct Ch {
    pub c: char,
    pub syntax: bool,
    pub space: bool,
}

/// A sibling list flattened: plain text characters, other inline nodes, and the badges read so far.
pub enum Unit<'m, 'a> {
    Ch(Ch),
    Node(&'m Md<'a>),
    Badge(Box<Node>),
}

/// A piece of a sibling list after badges and highlights are read; the flag says it is highlighted.
pub enum Part<'m, 'a> {
    Text(String, bool),
    Node(&'m Md<'a>, bool),
    Badge(Node),
}

impl Part<'_, '_> {
    /// Whether the highlight mark covers this piece.
    pub fn highlighted(&self) -> bool {
        matches!(self, Part::Text(_, true) | Part::Node(_, true))
    }
}

fn bare(source: &str, at: usize) -> bool {
    source.as_bytes()[..at].iter().rev().take_while(|b| **b == b'\\').count() % 2 == 0
}

fn flatten<'m, 'a>(nodes: &'m [Md<'a>], source: &str) -> Vec<Unit<'m, 'a>> {
    let mut units = Vec::new();
    for node in nodes {
        let Md::Leaf { event: Event::Text(t), range } = node else {
            units.push(Unit::Node(node));
            continue;
        };
        let literal = source.get(range.clone()) == Some(t.as_ref());
        units.extend(t.char_indices().map(|(k, c)| {
            let syntax = literal && matches!(c, '[' | ']' | '=') && bare(source, range.start + k);
            Unit::Ch(Ch { c, syntax, space: literal && c.is_whitespace() })
        }));
    }
    units
}

fn badge_at(units: &[Unit], i: usize) -> Option<(Box<Node>, usize)> {
    let close = units[i + 1..].iter().position(|unit| match unit {
        Unit::Ch(ch) => ch.syntax && (ch.c == ']' || ch.c == '['),
        _ => true,
    })? + i
        + 1;
    let Unit::Ch(Ch { c: ']', .. }) = units[close] else {
        return None;
    };
    let inner: String =
        units[i + 1..close].iter().filter_map(|u| if let Unit::Ch(ch) = u { Some(ch.c) } else { None }).collect();
    badge::read(&inner).map(|node| (Box::new(node), close + 1))
}

fn badges<'m, 'a>(units: Vec<Unit<'m, 'a>>) -> Vec<Unit<'m, 'a>> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < units.len() {
        let opens = matches!(&units[i], Unit::Ch(ch) if ch.c == '[' && ch.syntax);
        match opens.then(|| badge_at(&units, i)).flatten() {
            Some((node, end)) => {
                spans.push((i, end, node));
                i = end;
            }
            None => i += 1,
        }
    }
    let mut spans = spans.into_iter().peekable();
    let mut out = Vec::with_capacity(units.len());
    for (j, unit) in units.into_iter().enumerate() {
        match spans.peek() {
            Some((start, end, _)) if (*start..*end).contains(&j) => {
                if j + 1 == *end {
                    out.extend(spans.next().map(|(_, _, node)| Unit::Badge(node)));
                }
            }
            _ => out.push(unit),
        }
    }
    out
}

fn plain_parts<'m, 'a>(nodes: &'m [Md<'a>]) -> Vec<Part<'m, 'a>> {
    let mut out: Vec<Part> = Vec::new();
    for node in nodes {
        match (node, out.last_mut()) {
            (Md::Leaf { event: Event::Text(t), .. }, Some(Part::Text(text, _))) => text.push_str(t),
            (Md::Leaf { event: Event::Text(t), .. }, _) => out.push(Part::Text(t.to_string(), false)),
            _ => out.push(Part::Node(node, false)),
        }
    }
    out
}

/// Splits sibling inline nodes into text, nodes and badges, with `==` highlight delimiters applied.
pub fn parts<'m, 'a>(nodes: &'m [Md<'a>], source: &str) -> Vec<Part<'m, 'a>> {
    let marked = |node: &Md| matches!(node, Md::Leaf { event: Event::Text(t), .. } if t.contains(['[', '=']));
    if !nodes.iter().any(marked) {
        return plain_parts(nodes);
    }
    let units = badges(flatten(nodes, source));
    let toggles: std::collections::HashSet<usize> = delimiters(&units).into_iter().collect();
    toggled(units, &toggles)
}

fn toggled<'m, 'a>(units: Vec<Unit<'m, 'a>>, toggles: &std::collections::HashSet<usize>) -> Vec<Part<'m, 'a>> {
    let mut out: Vec<Part> = Vec::new();
    let (mut on, mut skip) = (false, false);
    for (i, unit) in units.into_iter().enumerate() {
        if std::mem::take(&mut skip) {
            continue;
        }
        if toggles.contains(&i) {
            (on, skip) = (!on, true);
            continue;
        }
        match unit {
            Unit::Ch(ch) => match out.last_mut() {
                Some(Part::Text(text, hl)) if *hl == on => text.push(ch.c),
                _ => out.push(Part::Text(ch.c.to_string(), on)),
            },
            Unit::Node(node) => out.push(Part::Node(node, on)),
            Unit::Badge(badge) => out.push(Part::Badge(*badge)),
        }
    }
    out
}
