//! The conversions: one function per subcommand that reads a single input.

use crate::cli::{Command, Format, FromArgs, IdArgs};
use crate::error::{CliError, Kind};
use crate::input::{detect, json_doc, load, markdown_doc};
use crate::output::Output;
use crate::{edit, render};
use encre::markdown::{plain_text, serialize, serialize_commonmark};
use encre::{Node, ValidationError, assign_ids, compact};

fn with_ids(mut doc: Node, ids: bool) -> Result<Output, CliError> {
    if ids {
        assign_ids(&mut doc);
    }
    Output::doc(&doc)
}

fn openui(source: &str, args: &IdArgs) -> Result<Output, CliError> {
    let doc = encre::openui::to_document(source).map_err(|e| CliError::invalid(format!("openui: {e}")))?;
    with_ids(doc, args.ids)
}

fn validate(input: &str, args: &FromArgs) -> Result<Output, CliError> {
    let result = match detect(input, args.from)? {
        Format::Json => json_doc(input),
        Format::Markdown => {
            markdown_doc(input).and_then(|doc| encre::validate(&doc).map(|()| doc).map_err(CliError::from))
        }
    };
    match result {
        Ok(_) => Ok(Output::Validation(None)),
        Err(e) if e.kind == Kind::Invalid => {
            let path = e.path.unwrap_or_else(|| "doc".to_owned());
            Ok(Output::Validation(Some(ValidationError { path, message: e.message })))
        }
        Err(e) => Err(e),
    }
}

/// Runs `command` on `input`; `max` caps every further file it reads.
pub fn run(command: &Command, input: &str, max: u64) -> Result<Output, CliError> {
    match command {
        Command::Parse(args) => with_ids(markdown_doc(input)?, args.ids),
        Command::Serialize => Ok(Output::Text("markdown", serialize(&json_doc(input)?))),
        Command::Commonmark(args) => Ok(Output::Text("markdown", serialize_commonmark(&load(input, args.from)?))),
        Command::PlainText(args) => Ok(Output::Text("text", plain_text(&load(input, args.from)?))),
        Command::Validate(args) => validate(input, args),
        Command::Compact => Output::doc(&compact(&json_doc(input)?)),
        Command::Openui(args) => openui(input, args),
        Command::Ops(args) => edit::run(args, input, max),
        Command::Typst(args) => render::typst(args, input),
        Command::Pdf(args) => render::pdf(args, input, max),
    }
}
