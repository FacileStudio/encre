use encre::{MigrationError, Node, SCHEMA_VERSION, assign_ids, is_block_id, migrate};
use serde_json::json;

fn ids(node: &Node, out: &mut Vec<String>) {
    out.extend(node.id().map(str::to_owned));
    node.content.iter().for_each(|child| ids(child, out));
}

#[test]
fn assign_ids_fills_keeps_and_deduplicates() {
    let mut doc: Node = serde_json::from_value(json!({"type": "doc", "content": [
        {"type": "paragraph", "attrs": {"id": "a1b2c3d4e5"}},
        {"type": "paragraph", "attrs": {"id": "a1b2c3d4e5"}},
        {"type": "bulletList", "content": [{"type": "listItem", "content": [{"type": "paragraph", "attrs": {"id": "BAD"}}]}]},
        {"type": "table", "content": [{"type": "tableRow", "content": [
            {"type": "tableHeader", "attrs": {"align": "none"}, "content": [{"type": "paragraph"}]}
        ]}]}
    ]}))
    .expect("doc");
    assign_ids(&mut doc);
    let mut found = Vec::new();
    ids(&doc, &mut found);
    assert_eq!(found.len(), 7);
    assert_eq!(found[0], "a1b2c3d4e5");
    assert!(found.iter().all(|id| is_block_id(id)), "{found:?}");
    let unique: std::collections::HashSet<_> = found.iter().collect();
    assert_eq!(unique.len(), found.len());
    assert!(doc.content[3].content[0].id().is_none(), "rows carry no id");
    assert_eq!(encre::validate(&doc), Ok(()));
}

#[test]
fn block_id_shape_is_crockford_base32() {
    assert!(is_block_id("a1b2c3d4e5"));
    assert!(!is_block_id("a1b2c3d4e"));
    assert!(!is_block_id("a1b2c3d4eu"));
    assert!(!is_block_id("A1B2C3D4E5"));
}

#[test]
fn migrate_accepts_known_versions_only() {
    let doc: Node = serde_json::from_value(json!({"type": "doc", "content": [{"type": "paragraph"}]})).expect("doc");
    assert_eq!(migrate(doc.clone(), SCHEMA_VERSION), Ok(doc.clone()));
    assert_eq!(migrate(doc.clone(), 0), Err(MigrationError::UnknownVersion(0)));
    assert_eq!(migrate(doc, SCHEMA_VERSION + 1), Err(MigrationError::UnknownVersion(SCHEMA_VERSION + 1)));
}
