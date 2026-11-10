use super::md_tree::Md;
use super::parse::{Reader, attr};
use crate::model::Node;
use pulldown_cmark::Tag;
use serde_json::{Map, Value};

/// The callout tone of a GFM alert marker such as `[!NOTE]`, case ignored.
fn tone(marker: &str) -> Option<&'static str> {
    let kind = marker.strip_prefix("[!")?.strip_suffix(']')?.to_ascii_uppercase();
    Some(match kind.as_str() {
        "NOTE" | "IMPORTANT" => "info",
        "TIP" | "PROS" | "ADVANTAGE" => "success",
        "WARNING" => "warning",
        "CAUTION" | "CONS" | "DRAWBACK" => "danger",
        _ => return None,
    })
}

impl Reader<'_> {
    fn alert(&self, children: &[Md]) -> Option<(&'static str, Vec<Node>)> {
        let Some(Md::Block { tag: Tag::Paragraph, range, children: inline, .. }) = children.first() else {
            return None;
        };
        let rest = self.source.get(range.start..)?;
        let end = range.start + rest.find('\n').unwrap_or(rest.len());
        let tone = tone(self.source[range.start..end].trim_end())?;
        let start = inline.iter().position(|n| n.range().start > end).unwrap_or(inline.len());
        let mut blocks = Vec::new();
        if start < inline.len() {
            self.paragraph(&inline[start..], &mut blocks);
        }
        self.blocks_into(&children[1..], &mut blocks);
        Some((tone, blocks))
    }

    /// Converts a blockquote, or the callout a GFM alert marker on its first line opens.
    pub fn quote(&self, children: &[Md]) -> Node {
        match self.alert(children) {
            Some((tone, blocks)) => {
                let mut attrs: Map<String, Value> = attr("tone", tone);
                attrs.insert("title".to_owned(), Value::from(""));
                let content = if blocks.is_empty() { vec![super::parse::empty_paragraph()] } else { blocks };
                Node::new("callout", attrs, content)
            }
            None => Node::new("blockquote", Map::new(), self.filled(children)),
        }
    }
}
