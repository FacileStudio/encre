use super::Error;
use super::map::COMPONENTS;
use super::parse::{Expr, MAX_DEPTH, Statement};
use serde_json::Number;
use std::collections::HashMap;

const MAX_VALUES: usize = 100_000;

/// A value with every reference replaced by what it names.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Num(Number),
    Bool(bool),
    Null,
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    Component(String, Vec<Value>),
}

impl Value {
    /// What the value is, for error messages.
    pub fn describe(&self) -> String {
        match self {
            Value::Str(_) => "a string".to_owned(),
            Value::Num(_) => "a number".to_owned(),
            Value::Bool(_) => "a boolean".to_owned(),
            Value::Null => "null".to_owned(),
            Value::Array(_) => "an array".to_owned(),
            Value::Object(_) => "an object".to_owned(),
            Value::Component(name, _) => format!("a {name}"),
        }
    }
}

struct Resolver<'a> {
    defs: HashMap<&'a str, &'a Expr>,
    stack: Vec<&'a str>,
    left: usize,
    depth: usize,
}

/// The value of `root`, with references resolved. A later statement of the same name replaces an earlier one,
/// as in OpenUI's own parser. Statements `root` never reaches are not evaluated.
pub fn root(statements: &[Statement]) -> Result<Value, Error> {
    let defs: HashMap<&str, &Expr> = statements.iter().map(|s| (s.name.as_str(), &s.expr)).collect();
    let mut resolver = Resolver { defs, stack: Vec::new(), left: MAX_VALUES, depth: 0 };
    if !resolver.defs.contains_key("root") {
        return Err(Error::NoRoot);
    }
    resolver.reference("root")
}

impl<'a> Resolver<'a> {
    fn reference(&mut self, name: &'a str) -> Result<Value, Error> {
        if let Some(at) = self.stack.iter().position(|n| *n == name) {
            let mut cycle: Vec<String> = self.stack[at..].iter().map(|n| (*n).to_owned()).collect();
            cycle.push(name.to_owned());
            return Err(Error::Cycle(cycle));
        }
        let expr = *self.defs.get(name).ok_or_else(|| Error::Undefined(name.to_owned()))?;
        self.stack.push(name);
        let value = self.value(expr);
        self.stack.pop();
        value
    }

    fn all(&mut self, exprs: &'a [Expr]) -> Result<Vec<Value>, Error> {
        exprs.iter().map(|e| self.value(e)).collect()
    }

    fn value(&mut self, expr: &'a Expr) -> Result<Value, Error> {
        self.left = self.left.checked_sub(1).ok_or(Error::TooLarge(MAX_VALUES))?;
        if self.depth >= MAX_DEPTH {
            return Err(Error::TooDeep(MAX_DEPTH));
        }
        self.depth += 1;
        let value = self.build(expr);
        self.depth -= 1;
        value
    }

    fn build(&mut self, expr: &'a Expr) -> Result<Value, Error> {
        Ok(match expr {
            Expr::Str(s) => Value::Str(s.clone()),
            Expr::Num(n) => Value::Num(n.clone()),
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Null => Value::Null,
            Expr::Ref(name) => self.reference(name)?,
            Expr::Array(items) => Value::Array(self.all(items)?),
            Expr::Object(fields) => Value::Object(
                fields.iter().map(|(k, e)| Ok((k.clone(), self.value(e)?))).collect::<Result<_, Error>>()?,
            ),
            Expr::Call(name, args) if COMPONENTS.contains(&name.as_str()) => {
                Value::Component(name.clone(), self.all(args)?)
            }
            Expr::Call(name, _) => return Err(Error::UnknownComponent(name.clone())),
        })
    }
}
