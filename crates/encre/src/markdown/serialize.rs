use super::compare::same_inline;
use super::escape::{Ctx, destination, escape};
use super::inline::{self, Style};
use super::parse::parse_doc;
use crate::model::{Mark, Node};
use serde_json::Map;

/// Writes blocks; `full` forces the full escaping form, `commonmark` the portable export.
#[derive(Debug, Clone, Copy)]
pub struct Writer {
    pub full: bool,
    pub commonmark: bool,
}

/// Prefixes every line of `text`: `first` on the first line, `rest` on the others; blank lines stay blank.
pub fn indent(text: &str, first: &str, rest: &str) -> String {
    let lines: Vec<String> = text
        .split('\n')
        .enumerate()
        .map(|(i, line)| {
            let prefix = if i == 0 { first } else { rest };
            if line.is_empty() { prefix.trim_end().to_owned() } else { format!("{prefix}{line}") }
        })
        .collect();
    lines.join("\n")
}

fn first_inline(doc: &Node, ctx: Ctx) -> Option<&Node> {
    let top = doc.content.first().filter(|_| doc.content.len() == 1)?;
    match ctx {
        Ctx::Paragraph => Some(top).filter(|b| b.kind == "paragraph"),
        Ctx::Heading => Some(top).filter(|b| b.kind == "heading"),
        Ctx::Cell => top.content.first()?.content.first()?.content.first().filter(|_| top.kind == "table"),
    }
}

fn inline_trial(nodes: &[Node], written: &str, ctx: Ctx) -> bool {
    let source = match ctx {
        Ctx::Paragraph => written.to_owned(),
        Ctx::Heading => format!("# {written}"),
        Ctx::Cell => format!("| {written} |\n| --- |"),
    };
    parse_doc(&source).is_ok_and(|doc| first_inline(&doc, ctx).is_some_and(|b| same_inline(&b.content, nodes)))
}

impl Writer {
    /// Writes inline content, falling back to the full form when the parse would differ.
    pub fn inline(&self, nodes: &[Node], ctx: Ctx) -> String {
        let kept = nodes.iter().rposition(|n| n.kind != "hardBreak").map_or(0, |i| i + 1);
        let nodes = if self.commonmark { &nodes[..kept] } else { nodes };
        let style = |full| Style { ctx, full, commonmark: self.commonmark };
        let paragraph_end = |mut written: String| {
            let trailing = nodes.last().is_none_or(|n| n.kind == "hardBreak");
            if ctx == Ctx::Paragraph && trailing && !self.commonmark {
                written.push('\\');
            }
            written
        };
        let minimal = paragraph_end(inline::write(nodes, style(self.full)));
        if self.full || inline_trial(nodes, &minimal, ctx) {
            return minimal;
        }
        paragraph_end(inline::write(nodes, style(true)))
    }

    /// Writes one block.
    pub fn block(&self, node: &Node) -> String {
        match node.kind.as_str() {
            "paragraph" => self.inline(&node.content, Ctx::Paragraph),
            "heading" => {
                let level = node.attrs.get("level").and_then(serde_json::Value::as_u64).unwrap_or(1).clamp(1, 3);
                let text = self.inline(&node.content, Ctx::Heading);
                let hashes = "#".repeat(level as usize);
                if text.is_empty() { hashes } else { format!("{hashes} {text}") }
            }
            "horizontalRule" => "---".to_owned(),
            "blockquote" => indent(&self.blocks(&node.content), "> ", "> "),
            "codeBlock" => self.code(node),
            "image" => {
                let alt = escape(node.attr_str("alt"), Ctx::Paragraph, true, false);
                format!("![{alt}]{}", destination(node.attr_str("src"), node.attr_str("title"), self.full))
            }
            "table" => super::write_table::table(self, node),
            "columns" if self.commonmark => self.flat_columns(node),
            "callout" | "stat" | "chart" | "diagram" | "columns" => self.custom(node),
            _ => super::write_list::list(self, node, false),
        }
    }

    /// Columns for CommonMark readers: each column's title as a bold paragraph, then its blocks.
    fn flat_columns(&self, node: &Node) -> String {
        let blocks: Vec<Node> = node
            .content
            .iter()
            .flat_map(|column| {
                let title = column.attr_str("title");
                let bold = vec![Mark::new("bold", Map::new())];
                let heading =
                    (!title.is_empty()).then(|| Node::new("paragraph", Map::new(), vec![Node::text(title, bold)]));
                heading.into_iter().chain(column.content.iter().cloned())
            })
            .collect();
        self.blocks(&blocks)
    }
}
