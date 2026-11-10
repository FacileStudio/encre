/// Characters a Typst string must not carry: C0 controls but tab, line feed and carriage return, and DEL.
fn is_dropped(c: char) -> bool {
    matches!(c, '\u{0}'..='\u{8}' | '\u{b}' | '\u{c}' | '\u{e}'..='\u{1f}' | '\u{7f}')
}

/// Escapes `value` for the inside of a Typst string literal. Port of Ardoise's `escapeTypstString`:
/// controls are dropped, `\` and `"` are escaped, `\r\n` and `\n` become `\n`, tab becomes `\t`.
/// A lone carriage return becomes `\r` (Ardoise left it raw).
pub fn escape_string(value: &str) -> String {
    let kept: String = value.chars().filter(|c| !is_dropped(*c)).collect();
    let mut out = String::with_capacity(kept.len() + 2);
    let mut chars = kept.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' if chars.peek() == Some(&'\n') => {}
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out
}

/// `value` as a quoted Typst string literal.
pub fn quoted(value: &str) -> String {
    format!("\"{}\"", escape_string(value))
}
