//! OpenUI adapter: one way, from an OpenUI Lang program (the language Bulle's chat renders) to encre blocks.
//!
//! A program is one `identifier = Expression` statement per line, with `root` as its entry point.
//! References may be used before they are defined; statements `root` never reaches are dropped.
//! Card becomes a heading of its title followed by its children, Stat `stat`, Table `table`, List a bullet
//! or ordered list, Callout `callout`, Chart `chart`, Text paragraphs, Divider `horizontalRule` and Badge a
//! bold paragraph. Button, Form and Field are dropped. Nothing stores OpenUI.

mod args;
mod cursor;
mod data;
mod lex;
mod map;
mod parse;
mod resolve;
mod text;

use crate::model::Node;
use serde_json::Map;

/// Why an OpenUI program could not become blocks.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The source is not OpenUI Lang.
    #[error("line {line}: {message}")]
    Syntax {
        /// The 1-based line where the problem starts.
        line: usize,
        /// What is wrong.
        message: String,
    },
    /// No `root` statement.
    #[error("the program has no root statement; write root = Card(...)")]
    NoRoot,
    /// A reference names no statement.
    #[error("{0} is used but never defined")]
    Undefined(String),
    /// References that lead back to themselves, in the order followed.
    #[error("reference cycle: {}", .0.join(" -> "))]
    Cycle(Vec<String>),
    /// A component outside OpenUI's library.
    #[error("unknown component {0}")]
    UnknownComponent(String),
    /// A component argument of the wrong shape.
    #[error("{component}: {message}")]
    Argument {
        /// The component.
        component: String,
        /// What is wrong.
        message: String,
    },
    /// The program expands past the adapter's size limit.
    #[error("the program expands to more than {0} values")]
    TooLarge(usize),
    /// References resolve to values nested past the parser's depth limit.
    #[error("values nest deeper than {0} levels once references are resolved")]
    TooDeep(usize),
    /// Nothing the program shows has a place in a document.
    #[error("the program holds nothing a document can keep: only buttons and forms")]
    Empty,
}

/// The blocks an OpenUI program shows, in order, without ids. They validate once placed in a `doc`.
pub fn to_blocks(source: &str) -> Result<Vec<Node>, Error> {
    let statements = parse::statements(&lex::tokens(source)?)?;
    map::blocks(&resolve::root(&statements)?, 1)
}

/// The program as a document without ids; [`crate::assign_ids`] makes it ready to store.
pub fn to_document(source: &str) -> Result<Node, Error> {
    let blocks = to_blocks(source)?;
    if blocks.is_empty() {
        return Err(Error::Empty);
    }
    Ok(Node::new("doc", Map::new(), blocks))
}
