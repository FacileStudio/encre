#[path = "support/spec.rs"]
mod spec;
mod support;

use serde_json::{Value, json};
use support::encre;

const NOTE: &str = "# Plan\n\nShip the **gate** first.\n\n- [ ] tag";

fn doc_json() -> String {
    encre(&["parse"], NOTE).text()
}

#[test]
fn parse_turns_markdown_into_a_document_and_ids_are_opt_in() {
    let plain = encre(&["parse"], NOTE);
    assert_eq!(plain.code, 0, "{}", plain.stderr);
    assert_eq!(plain.json()["type"], "doc");
    assert!(plain.json()["content"][0]["attrs"].get("id").is_none());
    let with_ids = encre(&["parse", "--ids"], NOTE).json();
    let id = with_ids["content"][0]["attrs"]["id"].as_str().expect("an id");
    assert_eq!(id.len(), 10);
}

#[test]
fn serialize_writes_canonical_markdown_without_a_trailing_newline() {
    let run = encre(&["serialize"], &doc_json());
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.text(), NOTE);
    let wrapped = encre(&["serialize", "--json"], &doc_json()).json();
    assert_eq!(wrapped, json!({ "markdown": NOTE }));
}

#[test]
fn commonmark_reads_json_or_markdown() {
    let from_json = encre(&["commonmark"], &doc_json());
    let from_markdown = encre(&["commonmark", "--from", "markdown"], NOTE);
    assert_eq!(from_json.code, 0, "{}", from_json.stderr);
    assert_eq!(from_json.text(), from_markdown.text());
    assert!(from_json.text().contains("**gate**"));
}

#[test]
fn plain_text_drops_the_markup() {
    let run = encre(&["plain-text"], NOTE);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.text().contains("Ship the gate first."), "{}", run.text());
    assert!(!run.text().contains("**"));
    assert!(encre(&["plain-text", "--json"], NOTE).json()["text"].is_string());
}

#[test]
fn compact_drops_defaults() {
    let loose = r#"{"type":"doc","content":[{"type":"paragraph","marks":[],"content":[{"type":"text","text":"a"}]}]}"#;
    let run = encre(&["compact"], loose);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.json()["content"][0].get("marks").is_none(), "{}", run.text());
}

#[test]
fn openui_converts_a_program_like_the_shared_vector() {
    let path = spec::spec("vectors/openui/badge.json");
    let vector: Value = serde_json::from_str(&std::fs::read_to_string(path).expect("vector")).expect("JSON");
    let run = encre(&["openui"], vector["source"].as_str().expect("source"));
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.json(), vector["want_doc"]);
    assert_eq!(encre(&["openui"], "root = ").code, 1);
}
