use encre::{Node, validate};
use serde_json::json;

fn doc(value: serde_json::Value) -> Node {
    serde_json::from_value(value).expect("node JSON")
}

fn para(content: serde_json::Value) -> serde_json::Value {
    json!({"type": "doc", "content": [{"type": "paragraph", "content": content}]})
}

fn column(text: &str) -> serde_json::Value {
    json!({"type": "column", "attrs": {"title": "T", "tone": "success"}, "content": [
        {"type": "paragraph", "content": [{"type": "text", "text": text}]}
    ]})
}

#[test]
fn accepts_the_schema_2_nodes() {
    let value = json!({"type": "doc", "content": [
        {"type": "paragraph", "content": [
            {"type": "text", "text": "a", "marks": [{"type": "bold"}, {"type": "highlight"}, {"type": "code"}]},
            {"type": "badge", "attrs": {"tone": "accent", "dot": true}, "content": [{"type": "text", "text": "done"}]},
            {"type": "badge"}
        ]},
        {"type": "diagram", "attrs": {"language": "dot"}, "content": [{"type": "text", "text": "a\nb"}]},
        {"type": "columns", "content": [column("a"), column("b"), column("c"), column("d")]},
        {"type": "stat", "attrs": {"label": "Active", "sub": "In production"}}
    ]});
    assert_eq!(validate(&doc(value)), Ok(()));
}

#[test]
fn rejects_what_schema_2_forbids() {
    let badge = |extra: serde_json::Value| para(json!([extra]));
    let cases = [
        json!({"type": "doc", "content": [{"type": "columns", "content": [column("a")]}]}),
        json!({"type": "doc", "content": [{"type": "columns", "content": [column("a"), column("b"), column("c"), column("d"), column("e")]}]}),
        json!({"type": "doc", "content": [column("a")]}),
        json!({"type": "doc", "content": [{"type": "diagram", "attrs": {"language": "plantuml"}}]}),
        badge(json!({"type": "badge", "attrs": {"tone": "pink"}})),
        badge(json!({"type": "badge", "marks": [{"type": "bold"}]})),
        badge(json!({"type": "badge", "content": [{"type": "text", "text": "x", "marks": [{"type": "bold"}]}]})),
        badge(json!({"type": "badge", "content": [{"type": "text", "text": "a\nb"}]})),
        para(json!([{"type": "text", "text": "a", "marks": [{"type": "highlight"}, {"type": "strike"}]}])),
        json!({"type": "doc", "content": [{"type": "stat", "attrs": {"hint": "x"}}]}),
    ];
    for case in cases {
        assert!(validate(&doc(case.clone())).is_err(), "accepted {case}");
    }
}
