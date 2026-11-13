use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A node of an encre document, in ProseMirror JSON form.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// The node type, `type` in JSON.
    #[serde(rename = "type")]
    pub kind: String,
    /// Attributes; empty when the type has none.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub attrs: Map<String, Value>,
    /// Child nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<Node>,
    /// The text of a `text` node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Marks of an inline node, in schema rank order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<Mark>,
}

/// A mark on an inline node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mark {
    /// The mark type, `type` in JSON.
    #[serde(rename = "type")]
    pub kind: String,
    /// Attributes; empty when the type has none.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub attrs: Map<String, Value>,
}

impl Node {
    /// A node of `kind` with `attrs` and `content`.
    pub fn new(kind: &str, attrs: Map<String, Value>, content: Vec<Node>) -> Self {
        Node { kind: kind.to_owned(), attrs, content, text: None, marks: Vec::new() }
    }

    /// A text node.
    pub fn text(text: &str, marks: Vec<Mark>) -> Self {
        Node { kind: "text".to_owned(), attrs: Map::new(), content: Vec::new(), text: Some(text.to_owned()), marks }
    }

    /// The string attribute `name`, or an empty string.
    pub fn attr_str(&self, name: &str) -> &str {
        self.attrs.get(name).and_then(Value::as_str).unwrap_or("")
    }

    /// The block id, when one is assigned.
    pub fn id(&self) -> Option<&str> {
        self.attrs.get("id").and_then(Value::as_str)
    }

    /// For an `embed` or `file` block, a paragraph holding one link to its target, labelled by its title or name,
    /// else by the target; None for any other node or an empty target.
    pub fn link_paragraph(&self) -> Option<Node> {
        let (href, label) = match self.kind.as_str() {
            "embed" => (self.attr_str("url"), self.attr_str("title")),
            "file" => (self.attr_str("src"), self.attr_str("name")),
            _ => return None,
        };
        let label = if label.is_empty() { href } else { label };
        let link = Mark::new("link", Map::from_iter([("href".to_owned(), Value::from(href))]));
        (!href.is_empty()).then(|| Node::new("paragraph", Map::new(), vec![Node::text(label, vec![link])]))
    }

    /// A copy of this tree with every `id` attribute removed.
    pub fn without_ids(&self) -> Node {
        let mut copy = self.clone();
        copy.attrs.remove("id");
        copy.content = self.content.iter().map(Node::without_ids).collect();
        copy
    }
}

impl Mark {
    /// A mark of `kind` with `attrs`.
    pub fn new(kind: &str, attrs: Map<String, Value>) -> Self {
        Mark { kind: kind.to_owned(), attrs }
    }
}
