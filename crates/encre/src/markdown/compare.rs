use crate::model::Node;
use crate::schema::{AttrSpec, schema};
use serde_json::{Map, Value};

fn filled(specs: &[AttrSpec], attrs: &Map<String, Value>) -> Map<String, Value> {
    let mut out = attrs.clone();
    out.remove("id");
    for spec in specs.iter().filter(|s| s.kind != "id") {
        out.entry(spec.name.clone()).or_insert_with(|| spec.default.clone());
    }
    out
}

/// `node` without ids and with every absent attribute set to its default, so two trees compare by meaning.
pub fn comparable(node: &Node) -> Node {
    let mut copy = node.clone();
    if let Some(node_type) = schema().node(&node.kind) {
        copy.attrs = filled(&node_type.attrs, &node.attrs);
    }
    for mark in &mut copy.marks {
        if let Some(mark_type) = schema().mark(&mark.kind) {
            mark.attrs = filled(&mark_type.attrs, &mark.attrs);
        }
    }
    copy.content = node.content.iter().map(comparable).collect();
    copy
}

/// Whether two inline sequences are the same once compared by meaning.
pub fn same_inline(a: &[Node], b: &[Node]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| comparable(x) == comparable(y))
}
