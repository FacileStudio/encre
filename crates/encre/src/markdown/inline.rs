use super::escape::{Ctx, char_ref, code_span, delimiter, destination, escape};
use super::tokens::{Tok, flank};
use crate::model::{Mark, Node};
use crate::schema::schema;

/// How inline content is written.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub ctx: Ctx,
    pub full: bool,
    pub commonmark: bool,
}

fn extent(nodes: &[Node], from: usize, mark: &Mark) -> usize {
    nodes[from..].iter().take_while(|n| n.marks.contains(mark)).count()
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

fn tokenize(nodes: &[Node], full: bool) -> Vec<Tok> {
    let mut tokens = Vec::new();
    let mut stack: Vec<Mark> = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        let marks: Vec<&Mark> = node.marks.iter().filter(|m| m.kind != "code").collect();
        let keep = stack.iter().take_while(|m| marks.contains(m)).count();
        close(&mut tokens, &mut stack, if node.kind == "hardBreak" { 0 } else { keep }, full);
        if node.kind == "hardBreak" {
            tokens.push(Tok::Break);
            continue;
        }
        let mut opening: Vec<&Mark> = marks.into_iter().filter(|m| !stack.contains(m)).collect();
        opening.sort_by_key(|m| (usize::MAX - extent(nodes, i, m), schema().mark_rank(&m.kind)));
        opening.into_iter().for_each(|m| open(&mut tokens, &mut stack, m.clone()));
        let text = node.text.clone().unwrap_or_default();
        let is_code = node.marks.iter().any(|m| m.kind == "code");
        tokens.push(if is_code { Tok::Code(text) } else { Tok::Text { raw: text, first: false, last: false } });
    }
    close(&mut tokens, &mut stack, 0, full);
    tokens
}

fn text(raw: &str, first: bool, last: bool, style: Style, line_start: bool) -> String {
    let chars: Vec<char> = raw.chars().collect();
    let n = chars.len();
    let tail = last && (n > 1 || !first);
    let (start, end) = (usize::from(first).min(n), n - usize::from(tail));
    let mut out: String = chars.first().filter(|_| first).map(|c| char_ref(*c)).unwrap_or_default();
    let middle: String = chars[start..end.max(start)].iter().collect();
    out.push_str(&escape(&middle, style.ctx, style.full, line_start && !first));
    out.push_str(&chars.last().filter(|_| tail).map(|c| char_ref(*c)).unwrap_or_default());
    out
}

fn attr<'a>(mark: &'a Mark, name: &str) -> &'a str {
    mark.attrs.get(name).and_then(serde_json::Value::as_str).unwrap_or("")
}

/// Renders one token; `line_start` says it begins a paragraph line.
pub fn render(tok: &Tok, style: Style, line_start: bool) -> String {
    match tok {
        Tok::Open(kind) | Tok::Close(kind) => delimiter(kind, style.full).to_owned(),
        Tok::LinkOpen => "[".to_owned(),
        Tok::LinkClose(dest) => format!("]{dest}"),
        Tok::Code(text) => code_span(text, style.ctx),
        Tok::Break if style.ctx == Ctx::Paragraph => "\\\n".to_owned(),
        Tok::Break if style.commonmark => " ".to_owned(),
        Tok::Break => "<br>".to_owned(),
        Tok::Text { raw, first, last } => text(raw, *first, *last, style, line_start),
    }
}

/// Writes inline content, with every flanking fix applied.
pub fn write(nodes: &[Node], style: Style) -> String {
    let mut tokens = tokenize(nodes, style.full);
    flank(&mut tokens, style);
    let mut out = String::new();
    let mut line_start = true;
    for tok in &tokens {
        out.push_str(&render(tok, style, line_start));
        line_start = matches!(tok, Tok::Break);
    }
    out
}
