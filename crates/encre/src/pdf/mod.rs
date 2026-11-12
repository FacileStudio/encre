//! PDF output: compiles the Typst that [`crate::typst::render`] writes, with the `typst` crate in memory.
//!
//! The compiler sees muse-typst (embedded from `typst/muse-typst`, see its `SOURCE`), the fonts in
//! `typst/fonts` (see `typst/FONTS.md`) and the [`Assets`] the caller hands over. Nothing else: no system
//! fonts, no package downloads, no filesystem.

mod files;
mod world;

use std::collections::HashMap;
use typst::diag::SourceDiagnostic;
use typst::foundations::Smart;
use typst_layout::PagedDocument;
use typst_pdf::PdfOptions;

/// Files the document may read, by the absolute path it names them with (`/images/logo.png`).
#[derive(Debug, Clone, Default)]
pub struct Assets {
    files: HashMap<String, Vec<u8>>,
}

impl Assets {
    /// An empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes `bytes` readable at `path`. A leading `/` is added when missing.
    pub fn insert(&mut self, path: &str, bytes: Vec<u8>) {
        let key = if path.starts_with('/') { path.to_owned() } else { format!("/{path}") };
        self.files.insert(key, bytes);
    }

    fn get(&self, path: &str) -> Option<&[u8]> {
        self.files.get(path).map(Vec::as_slice)
    }
}

/// Why a document did not become a PDF.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PdfError {
    /// Typst refused the source; the messages are Typst's.
    #[error("typst compile failed: {}", .0.join("; "))]
    Compile(Vec<String>),
    /// Layout worked but the PDF writer failed.
    #[error("pdf export failed: {}", .0.join("; "))]
    Export(Vec<String>),
}

fn messages(diagnostics: &[SourceDiagnostic]) -> Vec<String> {
    diagnostics.iter().map(|d| d.message.to_string()).collect()
}

/// Compiles Typst `source` to PDF bytes. The same source and assets always give the same bytes.
pub fn compile(source: &str, assets: &Assets) -> Result<Vec<u8>, PdfError> {
    let world = world::MemoryWorld::new(source, assets);
    let document: PagedDocument = typst::compile(&world).output.map_err(|e| PdfError::Compile(messages(&e)))?;
    let options = PdfOptions { ident: Smart::Custom("encre".into()), ..PdfOptions::default() };
    typst_pdf::pdf(&document, &options).map_err(|e| PdfError::Export(messages(&e)))
}
