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

fn line_start_special(chars: &[char], i: usize) -> bool {
    let c = chars[i];
    if i == 0 {
        return matches!(c, '#' | '>' | '+' | '-' | '=' | '|');
    }
    matches!(c, '.' | ')') && chars[..i].iter().all(char::is_ascii_digit)
}

fn special(chars: &[char], i: usize, ctx: Ctx, line_start: bool) -> bool {
    let c = chars[i];
    let prev = i.checked_sub(1).map(|p| chars[p]);
    let next = chars.get(i + 1).copied();
    let edge = |x: Option<char>| x.is_none_or(|x| !x.is_alphanumeric());
    match c {
        '\\' | '*' | '`' | '[' | ']' => true,
        '_' => edge(prev) || edge(next),
        '~' => true,
        '=' => prev == Some('=') || next == Some('='),
        '<' => next.is_some_and(|n| n.is_ascii_alphabetic() || "/!?".contains(n)),
        '&' => next.is_some_and(|n| n.is_ascii_alphanumeric() || n == '#'),
        '|' => ctx == Ctx::Cell,
        _ => ctx == Ctx::Paragraph && line_start && line_start_special(chars, i),
    }
}

/// Escapes `text`. `full` backslash-escapes every ASCII punctuation character; otherwise only what
/// is likely to change the parse is escaped. `line_start` says the text begins a paragraph line.
pub fn escape(text: &str, ctx: Ctx, full: bool, line_start: bool) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (i, &c) in chars.iter().enumerate() {
        let escaped = if full { c.is_ascii_punctuation() } else { special(&chars, i, ctx, line_start) };
        if escaped {
            out.push('\\');
        }
        out.push(c);
    }
    out
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
