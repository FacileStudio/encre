use super::attrs::{CODE_KEY, filled};
use crate::model::Node;
use crate::schema::schema;
use std::sync::Arc;
use yrs::types::Attrs;
use yrs::{Any, Text, TransactionMut, Xml, XmlElementPrelim, XmlFragment, XmlTextPrelim};

/// Inserts `nodes` into `parent` at `index`, one element each, in y-prosemirror's layout.
pub fn insert_blocks<P: XmlFragment>(txn: &mut TransactionMut, parent: &P, index: u32, nodes: &[Node]) {
    let mut at = index;
    let mut rest = nodes;
    while let Some(first) = rest.first() {
        let run = if first.kind == "text" { rest.iter().take_while(|n| n.kind == "text").count() } else { 1 };
        if first.kind == "text" {
            insert_text(txn, parent, at, &rest[..run]);
        } else {
            insert_element(txn, parent, at, first);
        }
        at += 1;
        rest = &rest[run..];
    }
}

fn insert_element<P: XmlFragment>(txn: &mut TransactionMut, parent: &P, index: u32, node: &Node) {
    let element = parent.insert(txn, index, XmlElementPrelim::empty(node.kind.as_str()));
    let specs = schema().node(&node.kind).map(|t| t.attrs.as_slice()).unwrap_or_default();
    for (key, value) in filled(specs, &node.attrs) {
        element.insert_attribute(txn, key, value);
    }
    insert_blocks(txn, &element, 0, &node.content);
}

fn insert_text<P: XmlFragment>(txn: &mut TransactionMut, parent: &P, index: u32, runs: &[Node]) {
    let text = parent.insert(txn, index, XmlTextPrelim::new(""));
    for run in runs {
        let offset = text.len(txn);
        text.insert_with_attributes(txn, offset, run.text.as_deref().unwrap_or_default(), marks(run));
    }
}

fn marks(node: &Node) -> Attrs {
    node.marks
        .iter()
        .map(|mark| {
            let specs = schema().mark(&mark.kind).map(|t| t.attrs.as_slice()).unwrap_or_default();
            let attrs: std::collections::HashMap<String, Any> = filled(specs, &mark.attrs).into_iter().collect();
            let key = if mark.kind == "code" { CODE_KEY } else { mark.kind.as_str() };
            (Arc::from(key), Any::Map(Arc::new(attrs)))
        })
        .collect()
}
