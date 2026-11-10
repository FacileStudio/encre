use super::badge;
use super::inline::{Style, render};
use crate::model::{Mark, Node};

/// One piece of written inline markdown.
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Open(String),
    Close(String),
    LinkOpen,
    LinkClose(String),
    Code(String),
    Break,
    Badge { head: String, text: String, spaced: bool },
    Text { raw: String, first: bool, last: bool },
}

/// The first and last character each token renders, worked out when first asked for.
struct Edges {
    style: Style,
    known: Vec<Option<Option<(char, char)>>>,
}

impl Edges {
    fn get(&mut self, tokens: &[Tok], i: Option<usize>, from_end: bool) -> Option<char> {
        let i = i.filter(|i| *i < tokens.len())?;
        let style = self.style;
        let known = self.known[i].get_or_insert_with(|| {
            let mut written = String::new();
            (tokens[i] != Tok::Break).then(|| render(&tokens[i], style, false, &mut written))?;
            Some((written.chars().next()?, written.chars().next_back()?))
        });
        let (first, last) = (*known)?;
        Some(if from_end { last } else { first })
    }
}

fn set(tokens: &mut [Tok], i: Option<usize>, on_first: bool) -> Option<usize> {
    match tokens.get_mut(i?) {
        Some(Tok::Text { first, last, .. }) => {
            let flag = if on_first { first } else { last };
            let changed = !*flag;
            *flag = true;
            changed.then_some(i?)
        }
        _ => None,
    }
}

fn delimiter_fix(tokens: &mut [Tok], edges: &mut Edges, i: usize, style: Style, opener: bool) -> Option<usize> {
    let (inner_at, outer_at) = if opener { (Some(i + 1), i.checked_sub(1)) } else { (i.checked_sub(1), Some(i + 1)) };
    let inner = edges.get(tokens, inner_at, !opener);
    if inner.is_some_and(char::is_whitespace) {
        return set(tokens, inner_at, opener);
    }
    let needed = style.full || inner.is_none_or(|c| !c.is_alphanumeric());
    let outer = edges.get(tokens, outer_at, opener);
    if needed && outer.is_some_and(|c| !(c.is_ascii_punctuation() || c == ' ' || c == '\t')) {
        return set(tokens, outer_at, !opener);
    }
    None
}

fn line_fix(tokens: &mut [Tok], i: usize) -> Option<usize> {
    let Some(Tok::Text { raw, .. }) = tokens.get(i) else {
        return None;
    };
    let blank = |c: char| c == ' ' || c == '\t';
    let starts = raw.starts_with(blank) && (i == 0 || tokens[i - 1] == Tok::Break);
    let ends = raw.ends_with(blank) && tokens.get(i + 1).is_none_or(|t| *t == Tok::Break);
    let first = if starts { set(tokens, Some(i), true) } else { None };
    let last = if ends { set(tokens, Some(i), false) } else { None };
    first.or(last)
}

fn fix(tokens: &mut [Tok], edges: &mut Edges, i: usize, style: Style) -> Option<usize> {
    match tokens[i] {
        Tok::Open(_) => delimiter_fix(tokens, edges, i, style, true),
        Tok::Close(_) => delimiter_fix(tokens, edges, i, style, false),
        Tok::Text { .. } => line_fix(tokens, i),
        _ => None,
    }
}

/// Marks the characters that must become character references so every delimiter flanks as intended
/// and no line starts or ends with whitespace the parser would strip. Gives the result of sweeping every token
/// in order until a sweep changes nothing, but after the first sweep re-checks only the neighbours of a token
/// that changed.
pub fn flank(tokens: &mut [Tok], style: Style) {
    let mut edges = Edges { style, known: vec![None; tokens.len()] };
    let mut pending = std::collections::BTreeSet::new();
    let mut check = |tokens: &mut [Tok], i: usize, pending: &mut std::collections::BTreeSet<usize>| {
        if let Some(changed) = fix(tokens, &mut edges, i, style) {
            edges.known[changed] = None;
            pending.extend(changed.checked_sub(1).into_iter().chain([changed + 1]).filter(|k| *k < tokens.len()));
        }
    };
    for i in 0..tokens.len() {
        check(tokens, i, &mut pending);
    }
    let mut cursor = 0;
    while let Some(i) = pending.range(cursor..).next().copied().or_else(|| pending.first().copied()) {
        pending.remove(&i);
        cursor = i + 1;
        check(tokens, i, &mut pending);
    }
}

/// Whether `mark` is written as a delimiter: code is a span of its own, and CommonMark drops highlights.
pub fn written(mark: &Mark, style: Style) -> bool {
    mark.kind != "code" && !(style.commonmark && mark.kind == "highlight")
}

/// The token of an inline node that closes every open mark: a hard break or a badge.
pub fn standalone(node: &Node) -> Option<Tok> {
    match node.kind.as_str() {
        "hardBreak" => Some(Tok::Break),
        "badge" => {
            let (head, spaced) = badge::head(node);
            let text: String = node.content.iter().filter_map(|t| t.text.as_deref()).collect();
            Some(Tok::Badge { head, text, spaced })
        }
        _ => None,
    }
}
