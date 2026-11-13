use super::RenderOptions;
use super::data::{args, cell, chart, code_block, stat, table};
use super::inline::inline;
use crate::escape::quoted;
use crate::model::Node;

/// Writes blocks as Typst markup.
pub struct Writer<'a, 'b> {
    pub options: &'a RenderOptions<'b>,
}

impl Writer<'_, '_> {
    /// Blocks separated by blank lines; empty paragraphs leave no trace.
    pub fn blocks(&self, nodes: &[Node]) -> String {
        nodes.iter().map(|node| self.block(node)).filter(|out| !out.is_empty()).collect::<Vec<_>>().join("\n\n")
    }

    fn block(&self, node: &Node) -> String {
        match node.kind.as_str() {
            "paragraph" => inline(&node.content),
            "heading" => format!(
                "#heading(level: {})[{}]",
                node.attrs.get("level").and_then(|v| v.as_u64()).unwrap_or(1),
                inline(&node.content)
            ),
            "bulletList" => format!("#list({})", args(node.content.iter().map(|item| self.item(item, "")))),
            "orderedList" => format!(
                "#enum(start: {}, {})",
                node.attrs.get("start").and_then(|v| v.as_u64()).unwrap_or(1),
                args(node.content.iter().map(|item| self.item(item, "")))
            ),
            "taskList" => {
                format!("#list(marker: [], {})", args(node.content.iter().map(|item| self.item(item, task_box(item)))))
            }
            "blockquote" => format!("#quote(block: true)[{}]", self.blocks(&node.content)),
            "codeBlock" => code_block(node),
            "horizontalRule" => "#divider()".to_owned(),
            "image" => self.image(node),
            "table" => table(node),
            "callout" => self.callout(node),
            "stat" => stat(node),
            "chart" => chart(node),
            "diagram" => diagram(node),
            "columns" => self.columns(node),
            "embed" | "file" => node.link_paragraph().map(|p| inline(&p.content)).unwrap_or_default(),
            _ => String::new(),
        }
    }

    fn item(&self, item: &Node, prefix: &str) -> String {
        format!("[{prefix}{}]", self.blocks(&item.content))
    }

    fn callout(&self, node: &Node) -> String {
        let title = node.attr_str("title");
        let title = if title.is_empty() { String::new() } else { format!(", title: {}", quoted(title)) };
        let tone = match node.attr_str("tone") {
            "" => "neutral",
            tone => tone,
        };
        format!("#alert([{}], tone: {}{title})", self.blocks(&node.content), quoted(tone))
    }

    fn columns(&self, node: &Node) -> String {
        let cells = node.content.iter().map(|column| {
            let (title, tone) = (column.attr_str("title"), column.attr_str("tone"));
            let body = self.blocks(&column.content);
            match (tone, title.is_empty()) {
                ("" | "neutral", true) => format!("[{body}]"),
                ("" | "neutral", false) => format!("[#strong[#{}]\n\n{body}]", quoted(title)),
                _ => {
                    let title = if title.is_empty() { String::new() } else { format!(", title: {}", quoted(title)) };
                    format!("[#alert([{body}], tone: {}{title})]", quoted(tone))
                }
            }
        });
        format!("#grid(columns: {}, gutter: 1em, {})", node.content.len(), args(cells))
    }

    fn image(&self, node: &Node) -> String {
        let (alt, title) = (node.attr_str("alt"), node.attr_str("title"));
        let Some(path) = self.options.image.and_then(|resolve| resolve(node.attr_str("src"))) else {
            return if alt.is_empty() { String::new() } else { format!("#emph[#{}]", quoted(alt)) };
        };
        let image = format!("image({}, alt: {})", quoted(&path.to_string_lossy()), quoted(alt));
        if title.is_empty() {
            format!("#figure({image})")
        } else {
            format!("#figure({image}, caption: {})", cell(title))
        }
    }
}

fn task_box(item: &Node) -> &'static str {
    if item.attrs.get("checked").and_then(|v| v.as_bool()).unwrap_or(false) {
        "#\"☑\"#h(0.4em)"
    } else {
        "#\"☐\"#h(0.4em)"
    }
}

/// A diagram as a raw block under a label naming its language; encre never draws it.
fn diagram(node: &Node) -> String {
    let language = match node.attr_str("language") {
        "" => "mermaid",
        language => language,
    };
    let source: String = node.content.iter().filter_map(|n| n.text.as_deref()).collect();
    format!(
        "#block(width: 100%)[#text(size: 0.8em, weight: \"medium\")[#{}]\n#raw(block: true, {})]",
        quoted(language),
        quoted(&source)
    )
}
