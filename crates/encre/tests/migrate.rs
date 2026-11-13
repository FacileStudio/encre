use encre::{MigrationError, Node, SCHEMA_VERSION, migrate, validate};
use serde_json::Value;
use std::path::PathBuf;

fn docs(dir: &str, key: &str) -> Vec<(String, Node)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec/vectors").join(dir);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir).expect("vectors").map(|e| e.expect("entry").path()).collect();
    files.sort();
    files
        .into_iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .filter_map(|path| {
            let raw: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("file")).expect("JSON");
            let doc = serde_json::from_value(raw.get(key)?.clone()).expect("doc");
            Some((path.display().to_string(), doc))
        })
        .collect()
}

#[test]
fn migrating_from_versions_1_and_2_changes_no_vector_document() {
    let all: Vec<(String, Node)> =
        [("markdown", "doc"), ("typst", "doc"), ("yjs", "doc"), ("openui", "want_doc"), ("ops", "doc")]
            .into_iter()
            .flat_map(|(dir, key)| docs(dir, key))
            .collect();
    assert!(all.len() > 80, "expected every vector document, found {}", all.len());
    for (name, doc) in all {
        assert_eq!(validate(&doc), Ok(()), "{name}: valid");
        assert_eq!(migrate(doc.clone(), 1), Ok(doc.clone()), "{name}: from 1");
        assert_eq!(migrate(doc.clone(), 2), Ok(doc.clone()), "{name}: from 2");
        assert_eq!(migrate(doc.clone(), SCHEMA_VERSION), Ok(doc), "{name}: from the current version");
    }
}

#[test]
fn unknown_versions_are_refused() {
    let doc: Node = serde_json::from_str(r#"{"type":"doc","content":[{"type":"paragraph"}]}"#).expect("doc");
    assert_eq!(migrate(doc.clone(), 0), Err(MigrationError::UnknownVersion(0)));
    assert_eq!(migrate(doc, SCHEMA_VERSION + 1), Err(MigrationError::UnknownVersion(SCHEMA_VERSION + 1)));
}
