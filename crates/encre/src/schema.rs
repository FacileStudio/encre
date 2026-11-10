use serde::Deserialize;
use serde_json::Value;
use std::sync::OnceLock;

/// The schema version of documents this crate writes.
pub const SCHEMA_VERSION: u32 = 1;

/// Node types the Rust projections handle, in `spec/schema.json` order.
pub const NODE_KINDS: &[&str] = &[
    "doc",
    "paragraph",
    "heading",
    "bulletList",
    "orderedList",
    "taskList",
    "listItem",
    "taskItem",
    "blockquote",
    "codeBlock",
    "horizontalRule",
    "image",
    "table",
    "tableRow",
    "tableHeader",
    "tableCell",
    "callout",
    "stat",
    "chart",
    "text",
    "hardBreak",
];

/// Mark types the Rust projections handle, in rank order.
pub const MARK_KINDS: &[&str] = &["link", "bold", "italic", "strike", "code"];

/// The parsed `spec/schema.json`.
#[derive(Debug, Deserialize)]
pub struct Schema {
    /// Schema version.
    pub version: u32,
    /// Block id format.
    pub id: IdSpec,
    /// Node types.
    pub nodes: Vec<NodeType>,
    /// Mark types, in rank order.
    pub marks: Vec<MarkType>,
}

/// Block id format.
#[derive(Debug, Deserialize)]
pub struct IdSpec {
    /// Allowed characters.
    pub alphabet: String,
    /// Exact length.
    pub length: usize,
}

/// One node type.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeType {
    /// Type name.
    pub name: String,
    /// Space-separated groups.
    #[serde(default)]
    pub group: String,
    /// ProseMirror content expression; empty for a leaf.
    #[serde(default)]
    pub content: String,
    /// Marks the inline content may carry: `_` all, empty none.
    #[serde(default)]
    pub marks: String,
    /// Whether an inline node of this type may carry marks.
    #[serde(default = "yes")]
    pub takes_marks: bool,
    /// Attributes.
    pub attrs: Vec<AttrSpec>,
}

/// One mark type.
#[derive(Debug, Deserialize)]
pub struct MarkType {
    /// Type name.
    pub name: String,
    /// Mark types this one excludes, when set.
    pub excludes: Option<String>,
    /// Attributes.
    pub attrs: Vec<AttrSpec>,
}

/// One attribute.
#[derive(Debug, Deserialize)]
pub struct AttrSpec {
    /// Attribute name.
    pub name: String,
    /// Value type: `id`, `integer`, `boolean`, `string`, `line`, `info`, `label`, `lines` or `series`.
    #[serde(rename = "type")]
    pub kind: String,
    /// Default value.
    pub default: Value,
    /// Allowed values, when restricted.
    pub values: Option<Vec<Value>>,
    /// Whether the attribute must be present and non-null.
    #[serde(default)]
    pub required: bool,
    /// Smallest allowed integer.
    pub min: Option<i64>,
    /// Largest allowed integer.
    pub max: Option<i64>,
}

fn yes() -> bool {
    true
}

/// The encre schema, read once from the embedded `spec/schema.json`.
pub fn schema() -> &'static Schema {
    static SCHEMA: OnceLock<Schema> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        serde_json::from_str(include_str!("../../../spec/schema.json")).expect("spec/schema.json is valid")
    })
}

impl Schema {
    /// The node type `name`.
    pub fn node(&self, name: &str) -> Option<&NodeType> {
        self.nodes.iter().find(|node| node.name == name)
    }

    /// The mark type `name`.
    pub fn mark(&self, name: &str) -> Option<&MarkType> {
        self.marks.iter().find(|mark| mark.name == name)
    }

    /// The rank of mark `name`, its position in the mark list.
    pub fn mark_rank(&self, name: &str) -> usize {
        self.marks.iter().position(|mark| mark.name == name).unwrap_or(usize::MAX)
    }
}

impl NodeType {
    /// Whether this node is in `group`.
    pub fn in_group(&self, group: &str) -> bool {
        self.group.split_whitespace().any(|g| g == group)
    }

    /// Whether this node carries a block id.
    pub fn has_id(&self) -> bool {
        self.attrs.iter().any(|attr| attr.kind == "id")
    }
}
