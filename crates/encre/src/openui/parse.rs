use super::Error;
use super::cursor::Cursor;
use super::lex::{Tok, Token};
use serde_json::Number;

pub const MAX_DEPTH: usize = 64;

/// An OpenUI Lang expression, before references are resolved.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Str(String),
    Num(Number),
    Bool(bool),
    Null,
    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>),
    Ref(String),
    Call(String, Vec<Expr>),
}

/// One `identifier = Expression` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub name: String,
    pub expr: Expr,
}

struct Parser<'a> {
    cur: Cursor<'a>,
    depth: usize,
}

/// Every statement of the program, in source order. Line feeds inside brackets continue a statement.
pub fn statements(toks: &[Token]) -> Result<Vec<Statement>, Error> {
    let mut parser = Parser { cur: Cursor::new(toks), depth: 0 };
    let mut out = Vec::new();
    parser.cur.skip_newlines();
    while parser.cur.peek().is_some() {
        out.push(parser.statement()?);
        parser.cur.skip_newlines();
    }
    Ok(out)
}

impl Parser<'_> {
    fn statement(&mut self) -> Result<Statement, Error> {
        let Some(Tok::Name(name) | Tok::Type(name)) = self.cur.next() else {
            return Err(self.cur.error("a statement starts with identifier ="));
        };
        self.cur.expect('=', format!("expected = after {name}"))?;
        let expr = self.expr()?;
        match self.cur.next() {
            None | Some(Tok::Newline) => Ok(Statement { name, expr }),
            Some(_) => {
                Err(self.cur.error(format!("unexpected text after the value of {name}: one statement per line")))
            }
        }
    }

    fn expr(&mut self) -> Result<Expr, Error> {
        if self.depth >= MAX_DEPTH {
            return Err(self.cur.error(format!("values nest deeper than {MAX_DEPTH} levels")));
        }
        self.depth += 1;
        let expr = match self.cur.next() {
            Some(Tok::Str(s)) => Ok(Expr::Str(s)),
            Some(Tok::Num(n)) => Ok(Expr::Num(n)),
            Some(Tok::Name(name)) => Ok(word(name)),
            Some(Tok::Type(name)) => self.call(name),
            Some(Tok::Punct('[')) => self.items(']', Self::expr).map(Expr::Array),
            Some(Tok::Punct('{')) => self.items('}', Self::field).map(Expr::Object),
            _ => Err(self.cur.error("expected a value")),
        };
        self.depth -= 1;
        expr
    }

    fn call(&mut self, name: String) -> Result<Expr, Error> {
        self.cur.expect('(', format!("expected ( after {name}"))?;
        self.items(')', Self::expr).map(|args| Expr::Call(name, args))
    }

    fn field(&mut self) -> Result<(String, Expr), Error> {
        let Some(Tok::Name(key) | Tok::Type(key) | Tok::Str(key)) = self.cur.next() else {
            return Err(self.cur.error("expected an object key"));
        };
        self.cur.expect(':', format!("expected : after {key}"))?;
        Ok((key, self.expr()?))
    }

    fn items<T>(&mut self, close: char, item: fn(&mut Self) -> Result<T, Error>) -> Result<Vec<T>, Error> {
        let mut out = Vec::new();
        loop {
            self.cur.skip_newlines();
            if self.cur.expect(close, "").is_ok() {
                break;
            }
            out.push(item(self)?);
            self.cur.skip_newlines();
            match self.cur.next() {
                Some(Tok::Punct(',')) => {}
                Some(Tok::Punct(c)) if c == close => break,
                found => return Err(self.separator_error(close, found)),
            }
        }
        Ok(out)
    }

    fn separator_error(&self, close: char, found: Option<Tok>) -> Error {
        if close == ')' && found == Some(Tok::Punct(':')) {
            self.cur.error("arguments are positional: write the value without name:")
        } else {
            self.cur.error(format!("expected , or {close}"))
        }
    }
}

fn word(name: String) -> Expr {
    match name.as_str() {
        "true" => Expr::Bool(true),
        "false" => Expr::Bool(false),
        "null" => Expr::Null,
        _ => Expr::Ref(name),
    }
}
