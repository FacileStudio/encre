//! `encre ops`: a sequence of agent operations applied to one document, all or nothing.

use crate::cli::{Format, OpsArgs};
use crate::error::CliError;
use crate::input::{detect, json_doc, markdown_doc};
use crate::io;
use crate::output::Output;
use encre::ops::{Op, Outcome, apply};
use encre::{Node, assign_ids, validate};
use serde_json::json;

fn load(args: &OpsArgs, max: u64) -> Result<Node, CliError> {
    let text = io::read_text(Some(&args.doc), max, "--doc")?;
    match detect(&text, args.from.from)? {
        Format::Json => json_doc(&text),
        Format::Markdown => {
            let mut doc = markdown_doc(&text)?;
            assign_ids(&mut doc);
            Ok(doc)
        }
    }
}

/// Applies the operations in `input` to `--doc`. Each write adds one to the version.
pub fn run(args: &OpsArgs, input: &str, max: u64) -> Result<Output, CliError> {
    let mut doc = load(args, max)?;
    let ops: Vec<Op> = serde_json::from_str(input)
        .map_err(|e| CliError::invalid(format!("ops must be a JSON array of operations: {e}")).at("ops"))?;
    let mut version = args.doc_version;
    let mut outcomes = Vec::with_capacity(ops.len());
    for (i, op) in ops.iter().enumerate() {
        let outcome =
            apply(&mut doc, version, op).map_err(|e| CliError::invalid(e.to_string()).at(format!("ops[{i}]")))?;
        if matches!(outcome, Outcome::Changed(_)) {
            version += 1;
        }
        outcomes.push(outcome);
    }
    validate(&doc)?;
    Ok(Output::Json(json!({"doc": doc, "version": version, "outcomes": outcomes})))
}
