use encre::markdown::{parse, plain_text, serialize_commonmark};
use encre::{Node, validate};
use serde_json::json;

fn doc(value: serde_json::Value) -> Node {
    serde_json::from_value(value).expect("node JSON")
}

fn one(block: serde_json::Value) -> Node {
    doc(json!({"type": "doc", "content": [block]}))
}

#[test]
fn accepts_the_schema_3_blocks() {
    let value = json!({"type": "doc", "content": [
        {"type": "embed", "attrs": {"url": "https://toile.facile.studio/b/abc", "kind": "toile", "title": "Board"}},
        {"type": "embed", "attrs": {"url": "https://example.com"}},
        {"type": "file", "attrs": {"src": "https://nuage.facile.studio/f/1", "name": "a.pdf", "size": 12, "mime": "application/pdf"}},
        {"type": "file", "attrs": {"src": "a.pdf"}}
    ]});
    assert_eq!(validate(&doc(value)), Ok(()));
}

#[test]
fn rejects_what_schema_3_forbids() {
    let cases = [
        json!({"type": "embed"}),
        json!({"type": "embed", "attrs": {"url": "http://example.com"}}),
        json!({"type": "embed", "attrs": {"url": "https://"}}),
        json!({"type": "embed", "attrs": {"url": "https:///path"}}),
        json!({"type": "embed", "attrs": {"url": "https://a b.example"}}),
        json!({"type": "embed", "attrs": {"url": "javascript:alert(1)"}}),
        json!({"type": "embed", "attrs": {"url": "https://a.example", "kind": "figma"}}),
        json!({"type": "embed", "attrs": {"url": "https://a.example", "title": "a\nb"}}),
        json!({"type": "file", "attrs": {"name": "a.pdf"}}),
        json!({"type": "file", "attrs": {"src": "a.pdf", "size": -1}}),
        json!({"type": "file", "attrs": {"src": "a.pdf", "size": 1.5}}),
        json!({"type": "file", "attrs": {"src": "a.pdf", "mime": "x\u{0}"}}),
        json!({"type": "file", "attrs": {"src": "a.pdf"}, "content": [{"type": "paragraph"}]}),
    ];
    for case in cases {
        assert!(validate(&one(case.clone())).is_err(), "accepted {case}");
    }
}

#[test]
fn commonmark_and_plain_text_show_a_link() {
    let markdown = "```embed\n{\"url\": \"https://a.example/x\", \"kind\": \"link\", \"title\": \"\"}\n```\n\n- ```file\n  {\"src\": \"r.pdf\", \"name\": \"Report [Q3]\", \"size\": 1, \"mime\": \"\"}\n  ```";
    let parsed = parse(markdown).expect("parses");
    assert_eq!(
        serialize_commonmark(&parsed),
        "[https://a.example/x](https://a.example/x)\n\n- [Report \\[Q3\\]](r.pdf)"
    );
    assert_eq!(plain_text(&parsed), "https://a.example/x\nReport [Q3]");
}
