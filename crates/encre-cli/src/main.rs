//! `encre`: the encre document library as a command-line tool, for services that run it as a subprocess.
#![deny(unused, dead_code)]

mod cli;
mod commands;
mod edit;
mod error;
mod images;
mod input;
mod io;
mod output;
mod render;

use clap::Parser;
use clap::error::ErrorKind;
use cli::Cli;
use error::CliError;
use std::io::IsTerminal;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::ExitCode;

fn run(cli: &Cli) -> Result<(), CliError> {
    output::check_destination(cli)?;
    let input = io::read_text(cli.input.as_deref(), cli.max_bytes, "input")?;
    let result = commands::run(&cli.command, &input, cli.max_bytes)?;
    output::emit(result, cli)
}

fn usage(e: &clap::Error) -> ExitCode {
    if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
        let _ = e.print();
        return ExitCode::SUCCESS;
    }
    if !std::env::args().any(|arg| arg == "--json") {
        let _ = e.print();
        return ExitCode::from(2);
    }
    let rendered = e.render().to_string();
    let first = rendered.lines().next().unwrap_or_default();
    let error = CliError::usage(first.trim_start_matches("error: "));
    error.report(true, false);
    ExitCode::from(error.code())
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => return usage(&e),
    };
    let color = !cli.json && !cli.no_color && std::env::var_os("NO_COLOR").is_none() && std::io::stderr().is_terminal();
    let result = catch_unwind(AssertUnwindSafe(|| run(&cli)))
        .unwrap_or_else(|_| Err(CliError::internal("encre panicked; this is a bug, please report it")));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            e.report(cli.json, color);
            ExitCode::from(e.code())
        }
    }
}
