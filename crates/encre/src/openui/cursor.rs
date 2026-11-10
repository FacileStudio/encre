use super::Error;
use super::lex::{Tok, Token};

/// A position in a token list.
pub struct Cursor<'a> {
    toks: &'a [Token],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(toks: &'a [Token]) -> Self {
        Cursor { toks, pos: 0 }
    }

    /// The token at the position, if any.
    pub fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.tok)
    }

    /// Takes the token at the position.
    pub fn next(&mut self) -> Option<Tok> {
        let tok = self.peek().cloned();
        self.pos += 1;
        tok
    }

    /// Skips line feeds.
    pub fn skip_newlines(&mut self) {
        while self.peek() == Some(&Tok::Newline) {
            self.pos += 1;
        }
    }

    /// Takes `c`, or fails with `message`.
    pub fn expect(&mut self, c: char, message: impl Into<String>) -> Result<(), Error> {
        if self.peek() == Some(&Tok::Punct(c)) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    /// A syntax error at the line of the last token read.
    pub fn error(&self, message: impl Into<String>) -> Error {
        let at = self.pos.min(self.toks.len()).saturating_sub(1);
        let line = self.toks.get(at).map_or(1, |t| t.line);
        Error::Syntax { line, message: message.into() }
    }
}
