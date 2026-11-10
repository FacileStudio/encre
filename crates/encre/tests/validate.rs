use encre::{Node, validate};
use serde_json::json;

fn doc(value: serde_json::Value) -> Node {
    serde_json::from_value(value).expect("node JSON")
}

fn para(content: serde_json::Value) -> serde_json::Value {
    json!({"type": "doc", "content": [{"type": "paragraph", "content": content}]})
}

#[test]
fn accepts_editor_json() {
    let value = json!({"type": "doc", "content": [
        {"type": "heading", "attrs": {"id": "a1b2c3d4e5", "level": 2}, "content": [{"type": "text", "text": "Hi"}]},
        {"type": "paragraph", "attrs": {"id": null}, "content": [
            {"type": "text", "text": "x", "marks": [{"type": "link", "attrs": {"href": "https://x.y", "title": ""}}, {"type": "bold"}]},
            {"type": "hardBreak"}
        ]},
        {"type": "chart", "attrs": {"kind": "line", "title": "", "labels": ["a"], "series": [{"name": "s", "data": [1, 2.5]}]}}
    ]});
    assert_eq!(validate(&doc(value)), Ok(()));
}

#[test]
fn rejects_what_the_schema_forbids() {
    let cases = [
        json!({"type": "doc", "content": []}),
        json!({"type": "doc", "content": [{"type": "video"}]}),
        json!({"type": "doc", "content": [{"type": "heading", "attrs": {"level": 4}}]}),
        json!({"type": "doc", "content": [{"type": "paragraph", "attrs": {"colour": "red"}}]}),
        json!({"type": "doc", "content": [{"type": "image", "attrs": {"alt": "x"}}]}),
        json!({"type": "doc", "content": [{"type": "callout", "attrs": {"title": "a`b"}, "content": [{"type": "paragraph"}]}]}),
        json!({"type": "doc", "content": [{"type": "bulletList", "content": [{"type": "listItem", "content": [{"type": "heading"}]}]}]}),
        json!({"type": "doc", "content": [{"type": "codeBlock", "attrs": {"language": " rust"}}]}),
    ];
    for case in cases {
        assert!(validate(&doc(case.clone())).is_err(), "accepted {case}");
    }
}

#[test]
fn rejects_inline_content_markdown_cannot_hold() {
    let cases = [
        para(json!([{"type": "text", "text": ""}])),
        para(json!([{"type": "text", "text": "a\nb"}])),
        para(
            json!([{"type": "text", "text": "a", "marks": [{"type": "bold"}, {"type": "link", "attrs": {"href": "x"}}]}]),
        ),
        para(json!([{"type": "text", "text": "a"}, {"type": "text", "text": "b"}])),
        para(json!([{"type": "hardBreak", "marks": [{"type": "bold"}]}])),
        para(json!([{"type": "text", "text": "a", "marks": [{"type": "underline"}]}])),
    ];
    for case in cases {
        assert!(validate(&doc(case.clone())).is_err(), "accepted {case}");
    }
}

#[test]
fn rejects_duplicate_ids_and_irregular_tables() {
    let duplicate = json!({"type": "doc", "content": [
        {"type": "paragraph", "attrs": {"id": "a1b2c3d4e5"}}, {"type": "paragraph", "attrs": {"id": "a1b2c3d4e5"}}
    ]});
    let cell = |kind: &str, align: &str, text: &str| {
        json!({"type": kind, "attrs": {"align": align}, "content": [
            {"type": "paragraph", "content": [{"type": "text", "text": text, "marks": [{"type": "code"}]}]}
        ]})
    };
    let table = |rows: serde_json::Value| json!({"type": "doc", "content": [{"type": "table", "content": rows}]});
    let header = json!({"type": "tableRow", "content": [cell("tableHeader", "left", "a")]});
    let cases = [
        duplicate,
        table(json!([header, {"type": "tableRow", "content": [cell("tableCell", "right", "b")]}])),
        table(json!([{"type": "tableRow", "content": [cell("tableCell", "none", "b")]}])),
        table(json!([{"type": "tableRow", "content": [cell("tableHeader", "none", "a\\|b")]}])),
    ];
    for case in cases {
        assert!(validate(&doc(case.clone())).is_err(), "accepted {case}");
    }
}
