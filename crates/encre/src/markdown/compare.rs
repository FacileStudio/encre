use crate::model::{Mark, Node};
use crate::schema::{AttrSpec, schema};
use serde_json::{Map, Value};

fn value<'v>(specs: &'v [AttrSpec], attrs: &'v Map<String, Value>, key: &str) -> Option<&'v Value> {
    attrs.get(key).or_else(|| specs.iter().find(|s| s.kind != "id" && s.name == key).map(|s| &s.default))
}

fn same_attrs(specs: Option<&[AttrSpec]>, a: &Map<String, Value>, b: &Map<String, Value>) -> bool {
    let Some(specs) = specs else {
        return a == b;
    };
    let keys = specs.iter().filter(|s| s.kind != "id").map(|s| s.name.as_str());
    let keys = keys.chain(a.keys().chain(b.keys()).map(String::as_str).filter(|k| *k != "id"));
    keys.into_iter().all(|key| value(specs, a, key) == value(specs, b, key))
}

fn same_mark(a: &Mark, b: &Mark) -> bool {
    a.kind == b.kind && same_attrs(schema().mark(&a.kind).map(|t| &t.attrs[..]), &a.attrs, &b.attrs)
}

/// Whether two trees mean the same: equal once ids are dropped and absent attributes take their defaults.
pub fn same(a: &Node, b: &Node) -> bool {
    a.kind == b.kind
        && a.text == b.text
        && a.marks.len() == b.marks.len()
        && a.marks.iter().zip(&b.marks).all(|(x, y)| same_mark(x, y))
        && same_attrs(schema().node(&a.kind).map(|t| &t.attrs[..]), &a.attrs, &b.attrs)
        && same_inline(&a.content, &b.content)
}

/// Whether two sibling sequences are the same once compared by meaning.
pub fn same_inline(a: &[Node], b: &[Node]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| same(x, y))
}
