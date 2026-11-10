use encre::Node;
use encre::typst::{RenderOptions, render};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// One typst golden vector.
pub struct TypstVector {
    pub name: String,
    pub path: PathBuf,
    pub doc: Node,
    pub raw: Value,
}

/// Every vector in `spec/vectors/typst`, sorted by file name.
pub fn typst_vectors() -> Vec<TypstVector> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec/vectors/typst");
    let mut files: Vec<PathBuf> =
        std::fs::read_dir(&dir).expect("typst vectors").map(|e| e.expect("entry").path()).collect();
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let raw: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("file")).expect("JSON");
            let doc = serde_json::from_value(raw["doc"].clone()).expect("doc");
            TypstVector { name: raw["name"].as_str().expect("name").to_owned(), path, doc, raw }
        })
        .collect()
}

/// Renders a vector with the options it declares; `images` maps an image `src` to the path Typst reads.
pub fn render_vector(v: &TypstVector) -> String {
    let opts = &v.raw["options"];
    let images: BTreeMap<String, String> = serde_json::from_value(opts["images"].clone()).unwrap_or_default();
    let resolve = |src: &str| images.get(src).map(PathBuf::from);
    let options = RenderOptions {
        title: opts["title"].as_str().unwrap_or_default().to_owned(),
        subtitle: opts["subtitle"].as_str().map(str::to_owned),
        doc_type: opts["doc_type"].as_str().unwrap_or("Document").to_owned(),
        cover: opts["cover"].as_bool().unwrap_or(false),
        image: Some(&resolve),
    };
    render(&v.doc, &options)
}
