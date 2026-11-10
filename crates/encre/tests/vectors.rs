use encre::markdown::{parse, serialize};
use encre::{Node, validate};
use serde_json::Value;
use std::path::PathBuf;

fn vectors() -> Vec<(String, Value)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec/vectors/markdown");
    let mut files: Vec<PathBuf> =
        std::fs::read_dir(&dir).expect("vector directory").map(|e| e.expect("entry").path()).collect();
    files.sort();
    files
        .into_iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| {
            let text = std::fs::read_to_string(&path).expect("vector file");
            (path.display().to_string(), serde_json::from_str(&text).expect("vector JSON"))
        })
        .collect()
}

#[test]
fn markdown_vectors_hold_in_rust() {
    let all = vectors();
    assert!(all.len() >= 40, "expected the full vector set, found {}", all.len());
    for (name, vector) in all {
        let markdown = vector["markdown"].as_str().expect("markdown");
        let doc: Node = serde_json::from_value(vector["doc"].clone()).expect("doc");
        let canonical = vector.get("canonical").and_then(Value::as_str).unwrap_or(markdown);
        assert_eq!(validate(&doc), Ok(()), "{name}: doc is invalid");
        assert_eq!(parse(markdown).expect("parse"), doc, "{name}: parse");
        assert_eq!(serialize(&doc), canonical, "{name}: serialize");
        assert_eq!(parse(canonical).expect("parse"), doc, "{name}: parse of canonical");
    }
}
