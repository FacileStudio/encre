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

#[test]
fn markdown_errors_carry_the_source_line_and_column() {
    let run = encre(&["validate", "--json"], "# Title\n\n- ok\n- bell \u{7} here\n");
    assert_eq!(run.code, 1);
    let error = &run.json()["errors"][0];
    assert_eq!((error["line"].clone(), error["column"].clone()), (json!(4), json!(3)));
    assert_eq!(error["path"], "doc.content[1].content[1].content[0]");
    let text = encre(&["validate", "--from", "markdown"], "x\n\na \u{1} b");
    assert!(text.stderr.contains("line 3, column 1: "), "{}", text.stderr);
    let json = encre(&["validate", "--json"], BAD_LEVEL).json();
    assert!(json["errors"][0].get("line").is_none(), "{json}");
}

#[test]
fn a_markdown_error_message_carries_no_path_of_its_own() {
    let run = encre(&["validate", "--from", "markdown", "--json"], "# Hi\n\na\u{1}b\n");
    assert_eq!(run.code, 1);
    let error = &run.json()["errors"][0];
    assert_eq!(error["path"], "doc.content[1]");
    assert_eq!((error["line"].clone(), error["column"].clone()), (json!(3), json!(1)));
    let message = error["message"].as_str().expect("message");
    assert_eq!(message, "text is empty or holds a forbidden character");
}
