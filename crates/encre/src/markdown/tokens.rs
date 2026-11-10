use super::inline::{Style, render};

/// One piece of written inline markdown.
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Open(String),
    Close(String),
    LinkOpen,
    LinkClose(String),
    Code(String),
    Break,
    Text { raw: String, first: bool, last: bool },
}

fn edge(tokens: &[Tok], i: Option<usize>, style: Style, from_end: bool) -> Option<char> {
    let tok = tokens.get(i?).filter(|t| **t != Tok::Break)?;
    let written = render(tok, style, false);
    if from_end { written.chars().last() } else { written.chars().next() }
}

fn safe_outer(c: char) -> bool {
    c.is_ascii_punctuation() || c == ' ' || c == '\t'
}

fn set(tokens: &mut [Tok], i: Option<usize>, on_first: bool) -> bool {
    match i.and_then(|i| tokens.get_mut(i)) {
        Some(Tok::Text { first, last, .. }) => {
            let flag = if on_first { first } else { last };
            let changed = !*flag;
            *flag = true;
            changed
        }
        _ => false,
    }
}

fn delimiter_fix(tokens: &mut [Tok], i: usize, style: Style, opener: bool) -> bool {
    let (inner_at, outer_at) = if opener { (Some(i + 1), i.checked_sub(1)) } else { (i.checked_sub(1), Some(i + 1)) };
    let inner = edge(tokens, inner_at, style, !opener);
    if inner.is_some_and(char::is_whitespace) {
        return set(tokens, inner_at, opener);
    }
    let needed = style.full || inner.is_none_or(|c| !c.is_alphanumeric());
    let outer = edge(tokens, outer_at, style, opener);
    if needed && outer.is_some_and(|c| !safe_outer(c)) {
        return set(tokens, outer_at, !opener);
    }
    false
}

fn line_fix(tokens: &mut [Tok], i: usize) -> bool {
    let Some(Tok::Text { raw, .. }) = tokens.get(i) else {
        return false;
    };
    let blank = |c: char| c == ' ' || c == '\t';
    let starts = raw.starts_with(blank) && (i == 0 || tokens[i - 1] == Tok::Break);
    let ends = raw.ends_with(blank) && tokens.get(i + 1).is_none_or(|t| *t == Tok::Break);
    let mut changed = false;
    if starts {
        changed |= set(tokens, Some(i), true);
    }
    if ends {
        changed |= set(tokens, Some(i), false);
    }
    changed
}

/// Marks the characters that must become character references so every delimiter flanks as intended
/// and no line starts or ends with whitespace the parser would strip.
pub fn flank(tokens: &mut [Tok], style: Style) {
    loop {
        let mut changed = false;
        for i in 0..tokens.len() {
            changed |= match tokens[i] {
                Tok::Open(_) => delimiter_fix(tokens, i, style, true),
                Tok::Close(_) => delimiter_fix(tokens, i, style, false),
                Tok::Text { .. } => line_fix(tokens, i),
                _ => false,
            };
        }
        if !changed {
            return;
        }
    }
}
