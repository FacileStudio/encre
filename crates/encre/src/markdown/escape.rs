/// Where inline content sits: it decides which characters are special.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ctx {
    Paragraph,
    Heading,
    Cell,
}

/// A decimal character reference for `c`.
pub fn char_ref(c: char) -> String {
    format!("&#{};", u32::from(c))
}

fn line_start_special(c: char, i: usize, digits: usize) -> bool {
    if i == 0 {
        return matches!(c, '#' | '>' | '+' | '-' | '=' | '|');
    }
    matches!(c, '.' | ')') && digits >= i
}

fn special(c: char, (prev, next): (Option<char>, Option<char>), i: usize, ctx: Ctx, line_start: Option<usize>) -> bool {
    let edge = |x: Option<char>| x.is_none_or(|x| !x.is_alphanumeric());
    match c {
        '\\' | '*' | '`' | '[' | ']' => true,
        '_' => edge(prev) || edge(next),
        '~' => true,
        '=' => prev == Some('=') || next == Some('='),
        '<' => next.is_some_and(|n| n.is_ascii_alphabetic() || "/!?".contains(n)),
        '&' => next.is_some_and(|n| n.is_ascii_alphanumeric() || n == '#'),
        '|' => ctx == Ctx::Cell,
        _ => ctx == Ctx::Paragraph && line_start.is_some_and(|digits| line_start_special(c, i, digits)),
    }
}

/// Escapes `text`. `full` backslash-escapes every ASCII punctuation character; otherwise only what
/// is likely to change the parse is escaped. `line_start` says the text begins a paragraph line.
pub fn escape(text: &str, ctx: Ctx, full: bool, line_start: bool) -> String {
    let mut out = String::with_capacity(text.len());
    escape_into(text, (ctx, full, line_start), &mut out);
    out
}

/// [`escape`] onto the end of `out`; `how` is the context, the full flag and the line start flag.
pub fn escape_into(text: &str, (ctx, full, line_start): (Ctx, bool, bool), out: &mut String) {
    let digits = line_start.then(|| text.chars().take_while(char::is_ascii_digit).count());
    let (mut chars, mut prev) = (text.chars().peekable(), None);
    let mut i = 0;
    while let Some(c) = chars.next() {
        let next = chars.peek().copied();
        if if full { c.is_ascii_punctuation() } else { special(c, (prev, next), i, ctx, digits) } {
            out.push('\\');
        }
        out.push(c);
        (prev, i) = (Some(c), i + 1);
    }
}

/// Writes a link or image destination and optional title: `(dest "title")`.
pub fn destination(href: &str, title: &str, full: bool) -> String {
    let bare_ok = |c: char| !c.is_ascii_control() && !" <>\\()&".contains(c) && !c.is_whitespace();
    let dest = if !full && !href.is_empty() && href.chars().all(bare_ok) {
        href.to_owned()
    } else {
        let mut out = String::from("<");
        href.chars().for_each(|c| {
            if "<>\\&".contains(c) {
                out.push('\\');
            }
            out.push(c);
        });
        out.push('>');
        out
    };
    if title.is_empty() {
        return format!("({dest})");
    }
    let quoted: String =
        title.chars().map(|c| if "\"\\&".contains(c) { format!("\\{c}") } else { c.to_string() }).collect();
    format!("({dest} \"{quoted}\")")
}

/// A code span holding `text`, with a fence longer than any backtick run inside.
pub fn code_span(text: &str, ctx: Ctx) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(longest + 1);
    let all_spaces = text.chars().all(|c| c == ' ');
    let pad = text.starts_with('`')
        || text.ends_with('`')
        || text.contains(']')
        || (text.starts_with(' ') && text.ends_with(' ') && !all_spaces);
    let body = if ctx == Ctx::Cell { text.replace('|', "\\|") } else { text.to_owned() };
    let space = if pad { " " } else { "" };
    format!("{fence}{space}{body}{space}{fence}")
}

/// The delimiter of an emphasis-like mark. Full form writes italic with `_` so runs never merge.
pub fn delimiter(kind: &str, full: bool) -> &'static str {
    match kind {
        "bold" => "**",
        "italic" if full => "_",
        "italic" => "*",
        "highlight" => "==",
        _ => "~~",
    }
}
