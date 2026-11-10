use super::blocks::{self, CUSTOM};
use super::serialize::Writer;
use crate::model::Node;

fn fence_for(content: &str, c: char) -> String {
    let longest = content.split(|x| x != c).map(str::len).max().unwrap_or(0);
    c.to_string().repeat((longest + 1).max(3))
}

fn info_escape(language: &str) -> String {
    language.chars().map(|c| if "\\&`~".contains(c) { format!("\\{c}") } else { c.to_string() }).collect()
}

impl Writer {
    /// Writes a code block, fenced with tildes when its language names a custom block.
    pub fn code(&self, node: &Node) -> String {
        let language = node.attr_str("language");
        let first = language.split_whitespace().next().unwrap_or("");
        let text: String = node.content.iter().filter_map(|t| t.text.as_deref()).collect();
        let c = if CUSTOM.contains(&first) || language.contains('`') { '~' } else { '`' };
        let fence = fence_for(&text, c);
        let info = info_escape(language);
        if text.is_empty() { format!("{fence}{info}\n{fence}") } else { format!("{fence}{info}\n{text}\n{fence}") }
    }

    /// Writes a custom block: info string and children, or its JSON body.
    pub fn custom(&self, node: &Node) -> String {
        let (head, inner) = if node.kind == "callout" {
            (blocks::info(node), self.blocks(&node.content))
        } else {
            (node.kind.clone(), blocks::body(node))
        };
        let fence = fence_for(&inner, '`');
        format!("{fence}{head}\n{inner}\n{fence}")
    }
}
