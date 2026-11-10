//! Typst output: the document as Typst source that imports muse-typst and wraps the body in its layout.
//!
//! Every string of the document reaches Typst as a string literal escaped by [`escape_string`], so no
//! document can inject Typst code. Node mapping is in `docs/architecture.md`.

mod blocks;
mod data;
mod inline;

use crate::escape::quoted;
use crate::model::Node;
use std::path::PathBuf;

pub use crate::escape::escape_string;

/// Resolves an image `src` to the path the Typst compiler reads it from.
pub type ImageResolver<'a> = &'a dyn Fn(&str) -> Option<PathBuf>;

/// How [`render`] lays out the document.
pub struct RenderOptions<'a> {
    /// Document title, for the PDF metadata and the cover.
    pub title: String,
    /// Subtitle on the cover.
    pub subtitle: Option<String>,
    /// The label in the running header, muse-typst's `doc-type`.
    pub doc_type: String,
    /// Whether to start with muse-typst's cover page.
    pub cover: bool,
    /// Text at the left of every footer, beside the page number; `None` writes none.
    pub footer: Option<String>,
    /// Resolves an image `src` to a path the compiler can read; `None` writes the alt text instead.
    pub image: Option<ImageResolver<'a>>,
}

impl Default for RenderOptions<'_> {
    fn default() -> Self {
        RenderOptions {
            title: String::new(),
            subtitle: None,
            doc_type: "Document".to_owned(),
            cover: false,
            footer: None,
            image: None,
        }
    }
}

fn preamble(options: &RenderOptions) -> String {
    let subtitle = options.subtitle.as_deref().map_or_else(|| "none".to_owned(), quoted);
    let footer = options.footer.as_deref().map_or_else(|| "none".to_owned(), quoted);
    format!(
        "#import \"/muse-typst/lib.typ\": *\n\
         #show: muse-doc.with(title: {}, subtitle: {subtitle}, doc-type: {}, cover: {}, footer: {footer})\n\
         #set text(font: (\"Satoshi\", \"DejaVu Sans\"))\n\
         #show raw: set text(font: (\"Menlo\", \"DejaVu Sans Mono\"))\n",
        quoted(&options.title),
        quoted(&options.doc_type),
        options.cover,
    )
}

/// The document as a complete Typst source file. The same tree and options always give the same bytes.
pub fn render(doc: &Node, options: &RenderOptions) -> String {
    format!("{}\n{}\n", preamble(options), blocks::Writer { options }.blocks(&doc.content))
}
