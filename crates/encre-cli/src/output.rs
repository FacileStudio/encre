//! What a subcommand produces, and how it reaches stdout or `--output`.

use crate::cli::{Cli, Command};
use crate::error::CliError;
use crate::io;
use encre::Node;
use serde_json::{Value, json};
use std::io::IsTerminal;

/// The result of one subcommand.
pub enum Output {
    /// Text, written as is; under `--json`, the object `{key: text}`.
    Text(&'static str, String),
    /// A JSON document, one line.
    Json(Value),
    /// PDF bytes; under `--json`, written to `--output` with a summary on stdout.
    Pdf(Vec<u8>),
    /// The `validate` verdict: `None` when the document is valid.
    Validation(Option<Verdict>),
}

/// Why `validate` refused its input, and where: a tree path, plus the source line and column for markdown.
pub struct Verdict {
    /// The tree path of the problem.
    pub path: String,
    /// What is wrong.
    pub message: String,
    /// The 1-based line and column of the markdown block that holds it.
    pub at: Option<(usize, usize)>,
}

impl Verdict {
    fn json(&self) -> Value {
        let mut value = json!({"path": self.path, "message": self.message});
        if let Some((line, column)) = self.at {
            value["line"] = json!(line);
            value["column"] = json!(column);
        }
        value
    }

    fn error(&self) -> CliError {
        let message = match self.at {
            Some((line, column)) => format!("line {line}, column {column}: {}", self.message),
            None => self.message.clone(),
        };
        CliError::invalid(message).at(self.path.clone())
    }
}

impl Output {
    /// A document as JSON output.
    pub fn doc(doc: &Node) -> Result<Output, CliError> {
        serde_json::to_value(doc).map(Output::Json).map_err(|e| CliError::internal(format!("cannot encode JSON: {e}")))
    }
}

fn line(value: &Value) -> Vec<u8> {
    let mut bytes = value.to_string().into_bytes();
    bytes.push(b'\n');
    bytes
}

/// Refuses a PDF with nowhere sensible to go, before any work is done.
pub fn check_destination(cli: &Cli) -> Result<(), CliError> {
    if !matches!(cli.command, Command::Pdf(_)) || cli.output.is_some() {
        return Ok(());
    }
    if cli.json {
        return Err(CliError::usage("pdf --json needs --output; the PDF goes there and the summary to stdout"));
    }
    if std::io::stdout().is_terminal() {
        return Err(CliError::usage("refusing to write a PDF to a terminal; pass --output or redirect stdout"));
    }
    Ok(())
}

fn validation(verdict: Option<Verdict>, cli: &Cli) -> Result<(), CliError> {
    if !cli.json {
        return verdict.map_or(Ok(()), |v| Err(v.error()));
    }
    let errors: Vec<Value> = verdict.iter().map(Verdict::json).collect();
    io::write(cli.output.as_deref(), &line(&json!({"valid": errors.is_empty(), "errors": errors})))?;
    match verdict {
        None => Ok(()),
        Some(v) => Err(CliError { reported: true, ..v.error() }),
    }
}

/// Writes `output` where the flags say.
pub fn emit(output: Output, cli: &Cli) -> Result<(), CliError> {
    let target = cli.output.as_deref();
    match output {
        Output::Text(key, text) if cli.json => io::write(target, &line(&json!({ key: text }))),
        Output::Text(_, text) => io::write(target, text.as_bytes()),
        Output::Json(value) => io::write(target, &line(&value)),
        Output::Pdf(bytes) if cli.json => {
            io::write(target, &bytes)?;
            let path = target.map(|p| p.display().to_string());
            io::write(None, &line(&json!({"output": path, "bytes": bytes.len()})))
        }
        Output::Pdf(bytes) => io::write(target, &bytes),
        Output::Validation(error) => validation(error, cli),
    }
}
