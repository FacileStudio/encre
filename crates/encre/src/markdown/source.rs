use crate::validate::ValidationError;

/// A validation error in a document read from markdown, located in the source.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{line}:{column}: {path}: {message}")]
pub struct SourceError {
    /// Where the problem is in the tree (`doc.content[2]`).
    pub path: String,
    /// What is wrong.
    pub message: String,
    /// The 1-based source line of the block that holds the problem.
    pub line: usize,
    /// The 1-based column, in characters, where that block starts.
    pub column: usize,
}

/// Places `error` at the deepest block on its path whose source position is known.
pub fn locate(error: ValidationError, positions: &[(String, (usize, usize))]) -> SourceError {
    let mut path = error.path.as_str();
    let (line, column) = loop {
        if let Some((_, at)) = positions.iter().find(|(p, _)| p == path) {
            break *at;
        }
        match path.rfind(".content[") {
            Some(cut) => path = &path[..cut],
            None => break (1, 1),
        }
    };
    SourceError { path: error.path, message: error.message, line, column }
}
