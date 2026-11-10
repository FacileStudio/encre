use super::escape::{Ctx, char_ref, code_span, delimiter, destination, escape, escape_into};
use super::tokens::{Tok, flank, standalone, written};
use crate::model::{Mark, Node};
use crate::schema::schema;

/// How inline content is written.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub ctx: Ctx,
    pub full: bool,
    pub commonmark: bool,
}

fn extents(nodes: &[Node]) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    for i in (0..nodes.len()).rev() {
        let next = nodes.get(i + 1).map(|n| (&n.marks, &out[i + 1]));
        let run = |mark: &Mark| next.and_then(|(marks, runs)| marks.iter().position(|m| m == mark).map(|k| runs[k]));
        out[i] = nodes[i].marks.iter().map(|mark| 1 + run(mark).unwrap_or(0)).collect();
    }
    out
}

fn open(tokens: &mut Vec<Tok>, stack: &mut Vec<Mark>, mark: Mark) {
    tokens.push(if mark.kind == "link" { Tok::LinkOpen } else { Tok::Open(mark.kind.clone()) });
    stack.push(mark);
}

fn close(tokens: &mut Vec<Tok>, stack: &mut Vec<Mark>, keep: usize, full: bool) {
    while stack.len() > keep {
        let mark = stack.pop().expect("stack is longer than keep");
        tokens.push(match mark.kind.as_str() {
            "link" => Tok::LinkClose(destination(attr(&mark, "href"), attr(&mark, "title"), full)),
            _ => Tok::Close(mark.kind.clone()),
        });
    }
}

fn tokenize(nodes: &[Node], style: Style) -> Vec<Tok> {
    let mut tokens = Vec::new();
    let mut stack: Vec<Mark> = Vec::new();
    let runs = extents(nodes);
    for (i, node) in nodes.iter().enumerate() {
        let marks: Vec<&Mark> = node.marks.iter().filter(|m| written(m, style)).collect();
        let keep = stack.iter().take_while(|m| marks.contains(m)).count();
        let alone = standalone(node);
        close(&mut tokens, &mut stack, if alone.is_some() { 0 } else { keep }, style.full);
        if let Some(tok) = alone {
            tokens.push(tok);
            continue;
        }
        let mut opening: Vec<&Mark> = marks.into_iter().filter(|m| !stack.contains(m)).collect();
        let extent = |m: &Mark| node.marks.iter().position(|x| std::ptr::eq(x, m)).map_or(0, |k| runs[i][k]);
        opening.sort_by_key(|m| (usize::MAX - extent(m), schema().mark_rank(&m.kind)));
        opening.into_iter().for_each(|m| open(&mut tokens, &mut stack, m.clone()));
        let text = node.text.clone().unwrap_or_default();
        let is_code = node.marks.iter().any(|m| m.kind == "code");
        tokens.push(if is_code { Tok::Code(text) } else { Tok::Text { raw: text, first: false, last: false } });
    }
    close(&mut tokens, &mut stack, 0, style.full);
    tokens
}

fn text(raw: &str, (first, last): (bool, bool), style: Style, line_start: bool, out: &mut String) {
    let lead = raw.chars().next().filter(|_| first);
    let tail = raw.chars().next_back().filter(|_| last && (raw.chars().nth(1).is_some() || !first));
    let start = lead.map_or(0, char::len_utf8);
    let end = raw.len() - tail.map_or(0, char::len_utf8);
    if let Some(c) = lead {
        out.push_str(&char_ref(c));
    }
    escape_into(raw.get(start..end).unwrap_or(""), (style.ctx, style.full, line_start && !first), out);
    if let Some(c) = tail {
        out.push_str(&char_ref(c));
    }
}

fn attr<'a>(mark: &'a Mark, name: &str) -> &'a str {
    mark.attrs.get(name).and_then(serde_json::Value::as_str).unwrap_or("")
}

/// Renders one token onto the end of `out`; `line_start` says it begins a paragraph line.
pub fn render(tok: &Tok, style: Style, line_start: bool, out: &mut String) {
    let how = (style.ctx, style.full, line_start);
    match tok {
        Tok::Open(kind) | Tok::Close(kind) => out.push_str(delimiter(kind, style.full)),
        Tok::LinkOpen => out.push('['),
        Tok::LinkClose(dest) => {
            out.push(']');
            out.push_str(dest);
        }
        Tok::Code(text) => out.push_str(&code_span(text, style.ctx)),
        Tok::Break if style.ctx == Ctx::Paragraph => out.push_str("\\\n"),
        Tok::Break if style.commonmark => out.push(' '),
        Tok::Break => out.push_str("<br>"),
        Tok::Text { raw, first, last } => text(raw, (*first, *last), style, line_start, out),
        Tok::Badge { text, .. } if style.commonmark => escape_into(&format!("[{text}]"), how, out),
        Tok::Badge { head, spaced: false, .. } => out.push_str(&format!("[{head}]")),
        Tok::Badge { head, text, .. } => {
            out.push_str(&format!("[{head} {}]", escape(text, style.ctx, style.full, false)))
        }
    }
}

/// Writes inline content, with every flanking fix applied.
pub fn write(nodes: &[Node], style: Style) -> String {
    let mut tokens = tokenize(nodes, style);
    flank(&mut tokens, style);
    let mut out = String::new();
    let mut line_start = true;
    for tok in &tokens {
        render(tok, style, line_start, &mut out);
        line_start = matches!(tok, Tok::Break);
    }
    out
}
