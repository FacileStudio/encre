//! `encre typst` and `encre pdf`.

use crate::cli::RenderArgs;
use crate::error::CliError;
use crate::images::ImageDir;
use crate::input::load;
use crate::io;
use crate::output::Output;
use encre::Node;
use encre::pdf::{Assets, PdfError, compile};
use encre::typst::{RenderOptions, render};
use std::collections::BTreeSet;
use std::path::PathBuf;

fn image_dir(args: &RenderArgs) -> Result<Option<ImageDir>, CliError> {
    args.image_dir.as_deref().map(ImageDir::open).transpose()
}

fn source(doc: &Node, args: &RenderArgs, dir: Option<&ImageDir>) -> String {
    let resolve = |src: &str| dir.and_then(|d| d.resolve(src)).map(|image| PathBuf::from(image.typst_path));
    let options = RenderOptions {
        title: args.title.clone().unwrap_or_default(),
        footer: args.footer.clone(),
        image: Some(&resolve),
        ..RenderOptions::default()
    };
    render(doc, &options)
}

fn image_sources(node: &Node, out: &mut BTreeSet<String>) {
    if node.kind == "image" {
        out.insert(node.attr_str("src").to_owned());
    }
    node.content.iter().for_each(|child| image_sources(child, out));
}

fn assets(doc: &Node, dir: Option<&ImageDir>, max: u64) -> Result<Assets, CliError> {
    let mut assets = Assets::new();
    let Some(dir) = dir else { return Ok(assets) };
    let mut sources = BTreeSet::new();
    image_sources(doc, &mut sources);
    for image in sources.iter().filter_map(|src| dir.resolve(src)) {
        let bytes = io::read_bytes(Some(&image.file), max, "image")?;
        assets.insert(&image.typst_path, bytes);
    }
    Ok(assets)
}

/// The document as Typst source.
pub fn typst(args: &RenderArgs, input: &str) -> Result<Output, CliError> {
    let doc = load(input, args.from.from)?;
    Ok(Output::Text("typst", source(&doc, args, image_dir(args)?.as_ref())))
}

/// The document as PDF bytes. A compile failure is the input's fault (exit 1), an export failure is not.
pub fn pdf(args: &RenderArgs, input: &str, max: u64) -> Result<Output, CliError> {
    let doc = load(input, args.from.from)?;
    let dir = image_dir(args)?;
    let assets = assets(&doc, dir.as_ref(), max)?;
    match compile(&source(&doc, args, dir.as_ref()), &assets) {
        Ok(bytes) => Ok(Output::Pdf(bytes)),
        Err(e @ PdfError::Compile(_)) => Err(CliError::invalid(e.to_string())),
        Err(e) => Err(CliError::internal(e.to_string())),
    }
}
