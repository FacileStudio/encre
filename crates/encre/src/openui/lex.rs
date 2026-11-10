use super::Error;
use serde_json::Number;

/// One token of OpenUI Lang.
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Name(String),
    Type(String),
    Str(String),
    Num(Number),
    Punct(char),
    Newline,
}

/// A token and the 1-based line it starts on.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
}

/// Splits `source` into tokens. Whitespace other than line feeds is skipped.
pub fn tokens(source: &str) -> Result<Vec<Token>, Error> {
    let mut lexer = Lexer { chars: source.chars().collect(), pos: 0, line: 1 };
    let mut out = Vec::new();
    while let Some(&c) = lexer.chars.get(lexer.pos) {
        if matches!(c, ' ' | '\t' | '\r') {
            lexer.pos += 1;
            continue;
        }
        let line = lexer.line;
        let tok = lexer.token(c)?;
        if tok == Tok::Newline {
            lexer.line += 1;
        }
        out.push(Token { tok, line });
    }
    Ok(out)
}

impl Lexer {
    fn error(&self, message: String) -> Error {
        Error::Syntax { line: self.line, message }
    }

    fn token(&mut self, c: char) -> Result<Tok, Error> {
        let digit_next = self.chars.get(self.pos + 1).is_some_and(char::is_ascii_digit);
        match c {
            '\n' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ':' | '=' => {
                self.pos += 1;
                Ok(if c == '\n' { Tok::Newline } else { Tok::Punct(c) })
            }
            '"' => self.double_quoted(),
            '\'' => self.single_quoted(),
            _ if c.is_ascii_digit() || (c == '-' && digit_next) => self.number(),
            _ if c.is_ascii_alphabetic() || c == '_' => Ok(self.word()),
            _ => Err(self.error(format!("unexpected character {c:?}"))),
        }
    }

    fn take_while(&mut self, keep: impl Fn(char) -> bool) -> String {
        let start = self.pos;
        while self.chars.get(self.pos).is_some_and(|&c| keep(c)) {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn word(&mut self) -> Tok {
        let word = self.take_while(|c| c.is_ascii_alphanumeric() || c == '_');
        if word.starts_with(|c: char| c.is_ascii_uppercase()) { Tok::Type(word) } else { Tok::Name(word) }
    }

    fn number(&mut self) -> Result<Tok, Error> {
        let mut text = self.take_while(|c| c == '-');
        text += &self.take_while(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E');
        if text.ends_with(['e', 'E']) && self.chars.get(self.pos).is_some_and(|&c| c == '+' || c == '-') {
            text += &self.take_while(|c| c == '+' || c == '-');
            text += &self.take_while(|c| c.is_ascii_digit());
        }
        let number = text.parse::<f64>().ok().and_then(|f| {
            let whole = f.fract() == 0.0 && f.abs() < 9_007_199_254_740_992.0;
            if whole { Some(Number::from(f as i64)) } else { Number::from_f64(f) }
        });
        number.map(Tok::Num).ok_or_else(|| self.error(format!("{text} is not a number")))
    }

    fn double_quoted(&mut self) -> Result<Tok, Error> {
        let start = self.pos;
        self.pos += 1;
        while let Some(&c) = self.chars.get(self.pos) {
            self.pos += if c == '\\' { 2 } else { 1 };
            if c == '"' {
                let raw: String = self.chars[start..self.pos].iter().collect();
                return serde_json::from_str(&raw)
                    .map(Tok::Str)
                    .map_err(|e| self.error(format!("bad string {raw}: {e}")));
            }
        }
        Err(self.error("unterminated string".to_owned()))
    }

    fn single_quoted(&mut self) -> Result<Tok, Error> {
        let mut out = String::new();
        self.pos += 1;
        while let Some(&c) = self.chars.get(self.pos) {
            self.pos += 1;
            match c {
                '\'' => return Ok(Tok::Str(out)),
                '\\' => {
                    let escaped = self.chars.get(self.pos).copied().unwrap_or('\\');
                    self.pos += 1;
                    out.push(match escaped {
                        'n' => '\n',
                        't' => '\t',
                        other => other,
                    });
                }
                '\n' => return Err(self.error("unterminated string".to_owned())),
                _ => out.push(c),
            }
        }
        Err(self.error("unterminated string".to_owned()))
    }
}
