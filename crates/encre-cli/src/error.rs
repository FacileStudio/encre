//! Errors, their exit codes, and how they reach stderr.

use encre::ValidationError;
use serde_json::json;
use std::io::Write;

/// What kind of failure, which fixes the exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Exit 1: the input or the document is invalid.
    Invalid,
    /// Exit 2: the command line is wrong.
    Usage,
    /// Exit 3: something failed that the input did not cause.
    Internal,
}

/// A failure, with the message the user reads.
#[derive(Debug)]
pub struct CliError {
    /// What kind of failure.
    pub kind: Kind,
    /// Lowercase, names what failed and what to do.
    pub message: String,
    /// Where in the input, when known (`doc.content[2]`, `ops[1]`).
    pub path: Option<String>,
    /// Already written to stdout, so nothing more is printed.
    pub reported: bool,
}

impl CliError {
    fn new(kind: Kind, message: impl Into<String>) -> Self {
        CliError { kind, message: message.into(), path: None, reported: false }
    }

    /// Exit 1.
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(Kind::Invalid, message)
    }

    /// Exit 2.
    pub fn usage(message: impl Into<String>) -> Self {
        Self::new(Kind::Usage, message)
    }

    /// Exit 3.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(Kind::Internal, message)
    }

    /// The same error, located at `path`.
    pub fn at(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// The process exit code.
    pub fn code(&self) -> u8 {
        match self.kind {
            Kind::Invalid => 1,
            Kind::Usage => 2,
            Kind::Internal => 3,
        }
    }

    /// Writes the error to stderr: one JSON object with `json`, else a `✗` line.
    pub fn report(&self, json: bool, color: bool) {
        if self.reported {
            return;
        }
        let names = ["invalid", "usage", "internal"];
        let line = if json {
            let code = names[usize::from(self.code() - 1)];
            json!({"error": {"code": code, "exit": self.code(), "message": self.message, "path": self.path}})
                .to_string()
        } else {
            let text = self.path.as_ref().map_or_else(|| self.message.clone(), |p| format!("{p}: {}", self.message));
            if color { format!("\x1b[31m✗\x1b[0m {text}") } else { format!("✗ {text}") }
        };
        let _ = writeln!(std::io::stderr(), "{line}");
    }
}

impl From<ValidationError> for CliError {
    fn from(e: ValidationError) -> Self {
        CliError::invalid(e.message).at(e.path)
    }
}
