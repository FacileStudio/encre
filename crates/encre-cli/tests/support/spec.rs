use std::path::PathBuf;

/// A file under `spec/`.
pub fn spec(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec").join(path)
}
