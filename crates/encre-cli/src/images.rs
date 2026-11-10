//! `--image-dir`: image sources resolved to files inside one directory, and nowhere else.

use crate::error::CliError;
use std::path::{Path, PathBuf};

/// The directory images may come from, canonicalized.
pub struct ImageDir {
    root: PathBuf,
}

/// One resolved image.
pub struct Image {
    /// The path the Typst source names it with, under `/images/`.
    pub typst_path: String,
    /// The file on disk.
    pub file: PathBuf,
}

fn relative_parts(src: &str) -> Option<Vec<&str>> {
    if src.starts_with('/') || src.contains([':', '\\', '\0']) {
        return None;
    }
    let parts: Vec<&str> = src.split('/').filter(|p| !p.is_empty() && *p != ".").collect();
    (!parts.is_empty() && !parts.contains(&"..")).then_some(parts)
}

impl ImageDir {
    /// Opens `path`, which must be a directory.
    pub fn open(path: &Path) -> Result<Self, CliError> {
        let root = path
            .canonicalize()
            .map_err(|e| CliError::usage(format!("cannot open --image-dir {}: {e}", path.display())))?;
        if !root.is_dir() {
            return Err(CliError::usage(format!("--image-dir {} is not a directory", path.display())));
        }
        Ok(ImageDir { root })
    }

    /// The file a relative `src` names, when it is a regular file inside the directory after
    /// following symlinks. URLs, absolute paths and `..` resolve to nothing.
    pub fn resolve(&self, src: &str) -> Option<Image> {
        let parts = relative_parts(src)?;
        let file = parts.iter().fold(self.root.clone(), |path, part| path.join(part)).canonicalize().ok()?;
        (file.starts_with(&self.root) && file.is_file())
            .then(|| Image { typst_path: format!("/images/{}", parts.join("/")), file })
    }
}
