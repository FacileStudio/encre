use super::inline::inline;
use super::node;
use encre::Node;
use proptest::prelude::*;
use serde_json::Value;

pub fn line() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(super::inline::TEXT_LINE), 0..6).prop_map(|c| c.into_iter().collect())
}

fn paragraph() -> impl Strategy<Value = Node> {
    inline().prop_map(|content| node("paragraph", &[], content))
}

fn code_block() -> impl Strategy<Value = Node> {
    let language = prop::sample::select(vec!["", "rust", "stat", "callout x", "a`b", "c++", "x\\y", "a&b"]);
    (language, super::extra::code_text()).prop_map(|(language, body)| {
        let content = if body.is_empty() { Vec::new() } else { vec![Node::text(&body, Vec::new())] };
        node("codeBlock", &[("language", Value::from(language))], content)
    })
}

fn row(r: usize, cells: &[Vec<Node>], aligns: &[&'static str]) -> Node {
    let kind = if r == 0 { "tableHeader" } else { "tableCell" };
    let built = cells
        .iter()
        .zip(aligns)
        .map(|(content, a)| node(kind, &[("align", Value::from(*a))], vec![node("paragraph", &[], content.clone())]));
    node("tableRow", &[], built.collect())
}

fn table() -> impl Strategy<Value = Node> {
    let align = prop::sample::select(vec!["none", "left", "center", "right"]);
    (prop::collection::vec(align, 1..4), 1..4usize).prop_flat_map(|(aligns, rows)| {
        let cells = prop::collection::vec(inline().prop_map(super::inline::without_code_pipes), aligns.len() * rows);
        cells.prop_map(move |cells| {
            let rows = cells.chunks(aligns.len()).enumerate().map(|(r, c)| row(r, c, &aligns)).collect();
            node("table", &[], rows)
        })
    })
}

fn leaf() -> impl Strategy<Value = Node> {
    let image =
        (line(), line(), prop::sample::select(vec!["a.png", "a b.png", "", "x(1)"])).prop_map(|(alt, title, src)| {
            node("image", &[("src", src.into()), ("alt", alt.into()), ("title", title.into())], Vec::new())
        });
    let heading = (1..4i64, inline()).prop_map(|(level, content)| node("heading", &[("level", level.into())], content));
    prop_oneof![
        6 => paragraph(),
        2 => heading,
        2 => code_block(),
        1 => Just(node("horizontalRule", &[], Vec::new())),
        1 => image,
        1 => table(),
        1 => super::extra::stat(),
        1 => super::extra::chart(),
        1 => super::extra::diagram(),
    ]
}

fn item(kind: &'static str, inner: BoxedStrategy<Node>) -> impl Strategy<Value = Node> {
    (paragraph(), prop::collection::vec(inner, 0..3), any::<bool>()).prop_map(move |(first, rest, checked)| {
        let pairs: Vec<(&str, Value)> = if kind == "taskItem" { vec![("checked", checked.into())] } else { vec![] };
        node(kind, &pairs, std::iter::once(first).chain(rest).collect())
    })
}

/// Any valid block, nested up to a few levels.
pub fn block() -> impl Strategy<Value = Node> {
    leaf().prop_recursive(3, 24, 3, |inner| {
        let tone = prop::sample::select(vec!["neutral", "info", "success", "warning", "danger"]);
        let title = prop::sample::select(vec!["", "Before you deploy", "a \"b\" \\ c", "x=\"y\""]);
        let children = prop::collection::vec(inner.clone(), 1..3);
        prop_oneof![
            children.clone().prop_map(|c| node("blockquote", &[], c)),
            (tone, title, children).prop_map(|(t, ti, c)| node(
                "callout",
                &[("tone", t.into()), ("title", ti.into())],
                c
            )),
            prop::collection::vec(item("listItem", inner.clone()), 1..3).prop_map(|c| node("bulletList", &[], c)),
            (0..12i64, prop::collection::vec(item("listItem", inner.clone()), 1..3)).prop_map(|(start, c)| node(
                "orderedList",
                &[("start", start.into())],
                c
            )),
            prop::collection::vec(item("taskItem", inner.clone()), 1..3).prop_map(|c| node("taskList", &[], c)),
            super::extra::columns(inner),
        ]
    })
}
