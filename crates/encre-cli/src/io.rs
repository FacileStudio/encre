//! Reading the named input with a size cap, and writing the output.

use crate::error::CliError;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

fn read_limited(reader: impl Read, max: u64, what: &str) -> Result<Vec<u8>, CliError> {
    let mut bytes = Vec::new();
    reader
        .take(max.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| CliError::internal(format!("cannot read {what}: {e}")))?;
    if bytes.len() as u64 > max {
        return Err(CliError::invalid(format!("{what} is larger than {max} bytes; send less or raise --max-bytes")));
    }
    Ok(bytes)
}

/// The bytes of `path`, or of stdin when `path` is `None`, refused past `max` bytes.
pub fn read_bytes(path: Option<&Path>, max: u64, what: &str) -> Result<Vec<u8>, CliError> {
    match path {
        None => read_limited(std::io::stdin().lock(), max, what),
        Some(path) => {
            let file =
                File::open(path).map_err(|e| CliError::usage(format!("cannot open {what} {}: {e}", path.display())))?;
            read_limited(file, max, what)
        }
    }
}

/// The text of `path`, or of stdin, which must be UTF-8.
pub fn read_text(path: Option<&Path>, max: u64, what: &str) -> Result<String, CliError> {
    String::from_utf8(read_bytes(path, max, what)?).map_err(|_| CliError::invalid(format!("{what} is not UTF-8")))
}

/// Writes `bytes` to `path`, or to stdout when `path` is `None`.
pub fn write(path: Option<&Path>, bytes: &[u8]) -> Result<(), CliError> {
    let result = match path {
        None => {
            let mut out = std::io::stdout().lock();
            out.write_all(bytes).and_then(|()| out.flush())
        }
        Some(path) => std::fs::write(path, bytes),
    };
    let target = path.map_or_else(|| "stdout".to_owned(), |p| p.display().to_string());
    result.map_err(|e| CliError::internal(format!("cannot write {target}: {e}")))
}
