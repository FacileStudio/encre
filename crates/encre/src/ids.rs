use crate::model::Node;
use crate::schema::schema;
use serde_json::Value;
use std::collections::HashSet;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;

/// Whether `id` has the block id shape: ten characters of lowercase Crockford base32.
pub fn is_block_id(id: &str) -> bool {
    let spec = &schema().id;
    id.chars().count() == spec.length && id.chars().all(|c| spec.alphabet.contains(c))
}

fn fresh(taken: &HashSet<String>) -> String {
    let spec = &schema().id;
    let alphabet: Vec<char> = spec.alphabet.chars().collect();
    loop {
        let mut bits = RandomState::new().hash_one(taken.len());
        let id: String = (0..spec.length)
            .map(|_| {
                let c = alphabet[(bits % 32) as usize];
                bits /= 32;
                c
            })
            .collect();
        if !taken.contains(&id) {
            return id;
        }
    }
}

fn visit(node: &mut Node, taken: &mut HashSet<String>) {
    let carries_id = schema().node(&node.kind).is_some_and(|t| t.has_id());
    if carries_id {
        let keep = node.id().filter(|id| is_block_id(id) && !taken.contains(*id)).map(str::to_owned);
        let id = keep.unwrap_or_else(|| fresh(taken));
        taken.insert(id.clone());
        node.attrs.insert("id".to_owned(), Value::String(id));
    }
    node.content.iter_mut().for_each(|child| visit(child, taken));
}

/// Gives every block without a valid, unique id a fresh one. Existing valid ids are kept.
pub fn assign_ids(doc: &mut Node) {
    visit(doc, &mut HashSet::new());
}
