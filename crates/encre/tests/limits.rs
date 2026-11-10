use encre::markdown::{parse, validate as validate_markdown};
use encre::{Node, validate};
use serde_json::json;

fn doc(value: serde_json::Value) -> Node {
    serde_json::from_value(value).expect("node JSON")
}

fn ladder(depth: usize) -> String {
    let open: String = (0..depth).rev().map(|d| format!("{}callout\n", "`".repeat(d + 3))).collect();
    let close: String = (0..depth).map(|d| format!("\n{}", "`".repeat(d + 3))).collect();
    format!("{open}x{close}")
}

fn nested(depth: usize, kind: &str) -> serde_json::Value {
    let mut node = json!({"type": "paragraph", "content": [{"type": "text", "text": "x"}]});
    for _ in 0..depth {
        node = json!({"type": kind, "content": [node]});
    }
    json!({"type": "doc", "content": [node]})
}

#[test]
fn refuses_documents_nested_past_the_limits() {
    assert_eq!(validate(&doc(nested(45, "blockquote"))), Ok(()));
    let error = validate(&doc(nested(46, "blockquote"))).expect_err("49 levels");
    assert!(error.message.contains("more than 48 levels"), "{error}");
    assert_eq!(error.path.matches(".content[0]").count(), 48, "{error}");
    let callout = |depth| {
        let mut value = nested(depth, "callout");
        value["content"][0]["attrs"] = json!({"tone": "info", "title": ""});
        doc(value)
    };
    assert_eq!(validate(&callout(8)), Ok(()));
    let error = validate(&callout(9)).expect_err("nine callouts");
    assert!(error.message.contains("custom blocks nest more than 8"), "{error}");
}

#[test]
fn a_valid_document_at_the_depth_limit_reads_back_from_json() {
    let mut value = nested(45, "blockquote");
    let mut text = &mut value["content"][0];
    while text["type"] != "text" {
        text = &mut text["content"][0];
    }
    text["marks"] = json!([{"type": "link", "attrs": {"href": "u", "title": ""}}]);
    let written = serde_json::to_string(&value).expect("writes");
    let back: Node = serde_json::from_str(&written).expect("serde_json reads 48 levels");
    assert_eq!(validate(&back), Ok(()));
}

#[test]
fn markdown_past_the_limits_is_refused_with_a_position() {
    let error = validate_markdown(&">".repeat(100)).expect_err("too deep");
    assert_eq!((error.path.as_str(), error.line, error.column), ("doc", 1, 65), "{error}");
    assert!(error.message.contains("64 levels"), "{error}");
    assert!(parse(&">".repeat(64)).is_ok());
    assert!(parse(&">".repeat(65)).is_err());
    assert!(validate_markdown(&ladder(8)).is_ok());
    let error = validate_markdown(&ladder(9)).expect_err("nine callouts");
    assert!(error.message.contains("custom blocks nest more than 8"), "{error}");
    assert_eq!((error.line, error.column), (9, 1), "{error}");
    let started = std::time::Instant::now();
    assert!(parse(&ladder(2000)).is_err());
    assert!(started.elapsed() < std::time::Duration::from_secs(5), "{:?}", started.elapsed());
}

#[test]
fn an_operation_with_markdown_past_the_limits_changes_nothing() {
    let mut doc = parse("keep").expect("parses");
    encre::assign_ids(&mut doc);
    let before = doc.clone();
    let op = encre::ops::Op::Append { markdown: ">".repeat(100) };
    let error = encre::ops::apply(&mut doc, 1, &op).expect_err("too deep");
    assert!(error.to_string().contains("64 levels"), "{error}");
    assert_eq!(doc, before);
}
