mod support;

use serde_json::json;
use support::encre;

const BAD_LEVEL: &str = r#"{"type":"doc","content":[{"type":"heading","attrs":{"level":9}}]}"#;

#[test]
fn a_valid_document_exits_zero_quietly() {
    let doc = encre(&["parse"], "# Fine").text();
    let run = encre(&["validate"], &doc);
    assert_eq!((run.code, run.text().as_str(), run.stderr.as_str()), (0, "", ""));
    assert_eq!(encre(&["validate", "--json"], &doc).json(), json!({"valid": true, "errors": []}));
    assert_eq!(encre(&["validate"], "plain *markdown*").code, 0);
}

#[test]
fn an_invalid_document_reports_its_path_as_json_on_stdout() {
    let run = encre(&["validate", "--json"], BAD_LEVEL);
    assert_eq!(run.code, 1);
    assert_eq!(run.stderr, "");
    let errors = &run.json()["errors"];
    assert_eq!(run.json()["valid"], false);
    assert_eq!(errors[0]["path"], "doc.content[0]");
    assert!(errors[0]["message"].as_str().expect("message").contains("level"));
}

#[test]
fn without_json_the_error_goes_to_stderr() {
    let run = encre(&["validate"], BAD_LEVEL);
    assert_eq!(run.code, 1);
    assert!(run.text().is_empty());
    assert!(run.stderr.starts_with("✗ doc.content[0]: "), "{}", run.stderr);
}

#[test]
fn json_that_is_not_a_document_is_invalid_not_a_crash() {
    let run = encre(&["validate", "--json", "--from", "json"], "{\"type\": ");
    assert_eq!(run.code, 1);
    assert_eq!(run.json()["errors"][0]["path"], "doc");
}

#[test]
fn input_that_only_looks_like_json_needs_from() {
    let run = encre(&["validate", "--json"], "{not json} but a paragraph");
    assert_eq!(run.code, 2);
    assert!(run.stderr.contains("--from"), "{}", run.stderr);
    assert_eq!(encre(&["validate", "--from", "markdown"], "{not json} but a paragraph").code, 0);
}
