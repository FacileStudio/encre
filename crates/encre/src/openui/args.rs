use super::Error;
use super::resolve::Value;

/// The positional arguments of one component call.
pub struct Args<'a> {
    component: &'a str,
    pub names: &'a [&'a str],
    values: &'a [Value],
}

impl<'a> Args<'a> {
    /// The arguments of `component`, whose parameters are `names`. More arguments than names fail.
    pub fn new(component: &'a str, names: &'a [&'a str], values: &'a [Value]) -> Result<Self, Error> {
        let args = Args { component, names, values };
        if values.len() > names.len() {
            return Err(args.error(format!("takes at most {} arguments, found {}", names.len(), values.len())));
        }
        Ok(args)
    }

    /// An argument error on this component.
    pub fn error(&self, message: String) -> Error {
        Error::Argument { component: self.component.to_owned(), message }
    }

    /// What argument `i` is, for an error message.
    pub fn found(&self, i: usize) -> String {
        self.get(i).map_or_else(|| "nothing".to_owned(), Value::describe)
    }

    /// Argument `i`, or `None` when it is absent or null.
    pub fn get(&self, i: usize) -> Option<&'a Value> {
        self.values.get(i).filter(|v| **v != Value::Null)
    }

    /// The error for argument `i` not being `want`; `found` says what it is.
    pub fn wrong(&self, i: usize, want: &str, found: String) -> Error {
        self.error(format!("argument {} ({}) must be {want}, found {found}", i + 1, self.names[i]))
    }

    /// Optional string argument `i`; with `numbers`, a number is read as its decimal form.
    pub fn opt_string(&self, i: usize, numbers: bool) -> Result<Option<String>, Error> {
        match self.get(i) {
            None => Ok(None),
            Some(Value::Str(s)) => Ok(Some(s.clone())),
            Some(Value::Num(n)) if numbers => Ok(Some(n.to_string())),
            Some(_) => Err(self.wrong(i, if numbers { "a string or a number" } else { "a string" }, self.found(i))),
        }
    }

    /// Required string argument `i`.
    pub fn string(&self, i: usize, numbers: bool) -> Result<String, Error> {
        self.opt_string(i, numbers)?.ok_or_else(|| {
            self.wrong(i, if numbers { "a string or a number" } else { "a string" }, "nothing".to_owned())
        })
    }

    /// Required array argument `i`.
    pub fn array(&self, i: usize) -> Result<&'a [Value], Error> {
        match self.get(i) {
            Some(Value::Array(items)) => Ok(items),
            _ => Err(self.wrong(i, "an array", self.found(i))),
        }
    }
}
