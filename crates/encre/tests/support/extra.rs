use super::blocks::line;
use super::inline::text;
use super::node;
use encre::Node;
use proptest::prelude::*;
use serde_json::{Map, Value, json};

const CODE: &[char] = &['a', ' ', '\t', '\n', '`', '~', '#', '\\', '<', '&', '>', '-', '*'];

pub fn code_text() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(CODE), 0..12).prop_map(|c| c.into_iter().collect::<String>())
}

pub fn columns(inner: BoxedStrategy<Node>) -> impl Strategy<Value = Node> {
    let tone = prop::sample::select(vec!["neutral", "success", "danger", "warning", "info"]);
    let title = prop::sample::select(vec!["", "Pros", "a \"b\" \\ c", "x=\"y\""]);
    let column = (title, tone, prop::collection::vec(inner, 1..3))
        .prop_map(|(t, tone, c)| node("column", &[("title", t.into()), ("tone", tone.into())], c));
    prop::collection::vec(column, 2..5).prop_map(|c| node("columns", &[], c))
}

pub fn badge() -> impl Strategy<Value = Node> {
    let tone = prop::sample::select(vec!["neutral", "accent", "info", "success", "warning", "danger"]);
    let words = prop::sample::select(vec!["", "done", "pending", "error", "success", "info"]);
    let text = prop_oneof![2 => text(), 1 => words.prop_map(str::to_owned)];
    (tone, any::<bool>(), text).prop_map(|(tone, dot, text)| {
        let mut attrs = Map::new();
        attrs.insert("tone".into(), Value::from(tone));
        attrs.insert("dot".into(), Value::from(dot));
        let content = if text.is_empty() { Vec::new() } else { vec![Node::text(&text, Vec::new())] };
        Node::new("badge", attrs, content)
    })
}

pub fn stat() -> impl Strategy<Value = Node> {
    (line(), line(), line(), prop::option::of(line())).prop_map(|(l, v, d, sub)| {
        let mut pairs = vec![("label", l.into()), ("value", v.into()), ("delta", d.into())];
        pairs.extend(sub.filter(|sub| !sub.is_empty()).map(|sub| ("sub", sub.into())));
        node("stat", &pairs, Vec::new())
    })
}

pub fn diagram() -> impl Strategy<Value = Node> {
    (prop::sample::select(vec!["mermaid", "d2", "dot"]), code_text()).prop_map(|(language, body)| {
        let content = if body.is_empty() { Vec::new() } else { vec![Node::text(&body, Vec::new())] };
        node("diagram", &[("language", Value::from(language))], content)
    })
}

pub fn chart() -> impl Strategy<Value = Node> {
    (any::<bool>(), line(), prop::collection::vec(line(), 0..3), -5i64..50, any::<bool>()).prop_map(
        |(bar, title, labels, n, float)| {
            let point = if float { json!(n as f64 / 4.0) } else { json!(n) };
            let series = json!([{"name": "web", "data": [point, 1]}]);
            let kind = if bar { "bar" } else { "line" };
            node(
                "chart",
                &[("kind", kind.into()), ("title", title.into()), ("labels", json!(labels)), ("series", series)],
                vec![],
            )
        },
    )
}

pub fn embed() -> impl Strategy<Value = Node> {
    let url =
        prop::sample::select(vec!["https://toile.facile.studio/b/x", "https://a.example/p?q=\"1\"&r=\\", "https://x"]);
    let kind = prop::sample::select(vec!["link", "toile", "feuilles", "vision", "docs", "nuage"]);
    (url, kind, line()).prop_map(|(url, kind, title)| {
        node("embed", &[("url", url.into()), ("kind", kind.into()), ("title", title.into())], Vec::new())
    })
}

pub fn file() -> impl Strategy<Value = Node> {
    let src = prop::sample::select(vec!["https://nuage.facile.studio/f/1", "a b.pdf", "", "x(1)"]);
    (src, line(), 0..9_007_199_254_740_991i64, line()).prop_map(|(src, name, size, mime)| {
        node(
            "file",
            &[("src", src.into()), ("name", name.into()), ("size", size.into()), ("mime", mime.into())],
            vec![],
        )
    })
}
