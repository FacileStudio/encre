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
        let c = if CUSTOM.contains(&first) || language.contains('`') { '~' } else { '`' };
        fenced(c, &info_escape(language), &text_of(node))
    }

    /// Writes a custom block: info string and children, its source, or its JSON body.
    pub fn custom(&self, node: &Node) -> String {
        let (head, inner) = match node.kind.as_str() {
            "callout" => (blocks::info(node), self.blocks(&node.content)),
            "columns" => (node.kind.clone(), self.columns(node)),
            "diagram" => return fenced('`', &diagram_language(node), &text_of(node)),
            _ => (node.kind.clone(), blocks::body(node)),
        };
        let fence = fence_for(&inner, '`');
        format!("{fence}{head}\n{inner}\n{fence}")
    }

    fn columns(&self, node: &Node) -> String {
        let columns: Vec<String> = node
            .content
            .iter()
            .map(|column| {
                let inner = self.blocks(&column.content);
                let fence = fence_for(&inner, '`');
                format!("{fence}{}\n{inner}\n{fence}", blocks::info(column))
            })
            .collect();
        columns.join("\n\n")
    }
}

/// The language of a diagram, `mermaid` when unset.
pub fn diagram_language(node: &Node) -> String {
    match node.attr_str("language") {
        "" => "mermaid".to_owned(),
        language => language.to_owned(),
    }
}

fn text_of(node: &Node) -> String {
    node.content.iter().filter_map(|t| t.text.as_deref()).collect()
}

fn fenced(c: char, info: &str, text: &str) -> String {
    let fence = fence_for(text, c);
    if text.is_empty() { format!("{fence}{info}\n{fence}") } else { format!("{fence}{info}\n{text}\n{fence}") }
}
