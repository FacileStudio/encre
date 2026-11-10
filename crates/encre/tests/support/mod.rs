#![allow(dead_code)]

pub mod blocks;
pub mod extra;
pub mod inline;
pub mod text;

use encre::Node;
use proptest::prelude::*;
use serde_json::{Map, Value};

pub fn attrs(pairs: &[(&str, Value)]) -> Map<String, Value> {
    pairs.iter().map(|(k, v)| ((*k).to_owned(), v.clone())).collect()
}

pub fn node(kind: &str, pairs: &[(&str, Value)], content: Vec<Node>) -> Node {
    Node::new(kind, attrs(pairs), content)
}

pub fn doc(content: Vec<Node>) -> Node {
    let mut doc = Node::new("doc", Map::new(), content);
    encre::assign_ids(&mut doc);
    doc
}

/// Any valid document, with ids.
pub fn document() -> impl Strategy<Value = Node> {
    prop::collection::vec(blocks::block(), 1..5).prop_map(doc)
}
