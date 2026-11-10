#[path = "support/spec.rs"]
mod spec;
mod support;

use serde_json::Value;
use support::encre;

fn markdown_vectors() -> Vec<Value> {
    let mut paths: Vec<_> = std::fs::read_dir(spec::spec("vectors/markdown"))
        .expect("vector dir")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| serde_json::from_str(&std::fs::read_to_string(path).expect("vector")).expect("JSON"))
        .collect()
}

#[test]
fn every_markdown_vector_round_trips_through_parse_then_serialize() {
    let vectors = markdown_vectors();
    assert!(vectors.len() >= 40, "found {} vectors", vectors.len());
    for vector in vectors {
        let name = vector["name"].as_str().expect("name");
        let markdown = vector["markdown"].as_str().expect("markdown");
        let canonical = vector.get("canonical").and_then(Value::as_str).unwrap_or(markdown);
        let parsed = encre(&["parse"], markdown);
        assert_eq!(parsed.code, 0, "{name}: {}", parsed.stderr);
        assert_eq!(parsed.json(), vector["doc"], "{name}: parse");
        let serialized = encre(&["serialize"], &parsed.text());
        assert_eq!(serialized.code, 0, "{name}: {}", serialized.stderr);
        assert_eq!(serialized.text(), canonical, "{name}: serialize");
    }
}
