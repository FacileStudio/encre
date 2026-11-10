//! The command-line surface: global flags, subcommands and their help.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// The default `--max-bytes`: 8 MiB.
pub const DEFAULT_MAX_BYTES: u64 = 8 * 1024 * 1024;

const EXAMPLES: &str = "\
Examples:
  encre parse --ids < note.md > note.json
  encre serialize --input note.json
  encre validate --json < note.json
  encre commonmark --from markdown < note.md
  echo '[{\"op\":\"append\",\"markdown\":\"Done.\"}]' | encre ops --doc note.json
  encre pdf --title Plan --image-dir ./images --input note.json --output plan.pdf

Input is read from stdin, or --input. Output goes to stdout, or --output.
Without --from, input whose first non-blank character is { is JSON, and any other
input is markdown. Input that starts with { but is not JSON needs --from.

Exit codes: 0 ok, 1 invalid input or document, 2 usage, 3 internal error.";

const OPS_EXAMPLES: &str = "\
Examples:
  echo '[{\"op\":\"read_blocks\"}]' | encre ops --doc note.json
  encre ops --doc note.json --doc-version 4 --input ops.json --output result.json

Prints {\"doc\", \"version\", \"outcomes\"}. Each write adds one to the version. When an
operation is refused, nothing is printed on stdout and the error names ops[i].";

/// Convert, validate, edit and render encre documents
#[derive(Parser)]
#[command(name = "encre", version, after_help = EXAMPLES)]
pub struct Cli {
    /// The subcommand to run
    #[command(subcommand)]
    pub command: Command,
    /// Read the input from this file instead of stdin
    #[arg(long, global = true, value_name = "FILE")]
    pub input: Option<PathBuf>,
    /// Write the output to this file instead of stdout
    #[arg(long, global = true, value_name = "FILE")]
    pub output: Option<PathBuf>,
    /// Print JSON on stdout, and errors as JSON on stderr
    #[arg(long, global = true)]
    pub json: bool,
    /// Refuse an input or image larger than this many bytes
    #[arg(long, global = true, value_name = "BYTES", default_value_t = DEFAULT_MAX_BYTES)]
    pub max_bytes: u64,
    /// Never colour the output
    #[arg(long, global = true)]
    pub no_color: bool,
}

/// One subcommand.
#[derive(Subcommand)]
pub enum Command {
    /// Parse Facile Markdown into encre JSON
    Parse(IdArgs),
    /// Write encre JSON as canonical Facile Markdown
    Serialize,
    /// Write a document as portable CommonMark with GFM, lossy
    Commonmark(FromArgs),
    /// Write the text of a document alone
    PlainText(FromArgs),
    /// Check a document against schema v1
    Validate(FromArgs),
    /// Write encre JSON in its canonical stored form
    Compact,
    /// Convert an OpenUI Lang program into encre JSON
    Openui(IdArgs),
    /// Apply a JSON array of agent operations to a document
    #[command(after_help = OPS_EXAMPLES)]
    Ops(OpsArgs),
    /// Render a document as Typst source over muse-typst
    Typst(RenderArgs),
    /// Render a document as a PDF
    Pdf(RenderArgs),
}

/// The input format, for inputs that can be either.
#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Facile Markdown
    Markdown,
    /// encre JSON
    Json,
}

/// `--from`.
#[derive(Args)]
pub struct FromArgs {
    /// Input format; detected from the first character when omitted
    #[arg(long, value_enum)]
    pub from: Option<Format>,
}

/// `--ids`.
#[derive(Args)]
pub struct IdArgs {
    /// Give every block a fresh block id
    #[arg(long)]
    pub ids: bool,
}

/// The `ops` flags.
#[derive(Args)]
pub struct OpsArgs {
    /// The document to edit, JSON or markdown; markdown blocks get fresh ids
    #[arg(long, value_name = "FILE")]
    pub doc: PathBuf,
    /// The format of --doc
    #[command(flatten)]
    pub from: FromArgs,
    /// The stored version of the document, checked by replace_all
    #[arg(long, value_name = "N", default_value_t = 0)]
    pub doc_version: u64,
}

/// The `typst` and `pdf` flags.
#[derive(Args)]
pub struct RenderArgs {
    /// The input format
    #[command(flatten)]
    pub from: FromArgs,
    /// Document title, for the PDF metadata
    #[arg(long)]
    pub title: Option<String>,
    /// Text at the left of every page footer
    #[arg(long)]
    pub footer: Option<String>,
    /// Embed images whose relative src names a file inside this directory
    #[arg(long, value_name = "DIR")]
    pub image_dir: Option<PathBuf>,
}
