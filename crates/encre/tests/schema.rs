use encre::schema::{MARK_KINDS, NODE_KINDS, schema};

const ATTR_TYPES: &[&str] = &["id", "integer", "boolean", "string", "line", "info", "label", "lines", "series", "url"];

#[test]
fn rust_types_match_schema_json() {
    let nodes: Vec<&str> = schema().nodes.iter().map(|n| n.name.as_str()).collect();
    let marks: Vec<&str> = schema().marks.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(nodes, NODE_KINDS);
    assert_eq!(marks, MARK_KINDS);
    assert_eq!(schema().version, encre::SCHEMA_VERSION);
}

#[test]
fn every_attribute_has_a_known_type_and_a_valid_default() {
    for (owner, attrs) in
        schema().nodes.iter().map(|n| (&n.name, &n.attrs)).chain(schema().marks.iter().map(|m| (&m.name, &m.attrs)))
    {
        for attr in attrs {
            assert!(ATTR_TYPES.contains(&attr.kind.as_str()), "{owner}.{} has type {}", attr.name, attr.kind);
            let in_values = attr.values.as_ref().is_none_or(|values| values.contains(&attr.default));
            assert!(in_values, "{owner}.{} default is outside its values", attr.name);
        }
    }
}

#[test]
fn every_block_but_rows_cells_and_inline_nodes_carries_an_id() {
    for node in &schema().nodes {
        let structural =
            ["doc", "text", "hardBreak", "badge", "tableRow", "tableHeader", "tableCell"].contains(&node.name.as_str());
        assert_eq!(node.has_id(), !structural, "{}", node.name);
    }
}
