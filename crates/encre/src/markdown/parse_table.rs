use super::md_tree::Md;
use super::parse::{Reader, attr};
use crate::model::Node;
use pulldown_cmark::{Alignment, Tag};
use serde_json::Map;

fn align_name(align: &Alignment) -> &'static str {
    match align {
        Alignment::Left => "left",
        Alignment::Right => "right",
        Alignment::Center => "center",
        Alignment::None => "none",
    }
}

fn row(reader: &Reader, kind: &str, aligns: &[&str], cells: &[Md]) -> Node {
    let cells: Vec<&Md> = cells.iter().filter(|c| matches!(c, Md::Block { tag: Tag::TableCell, .. })).collect();
    let built = aligns.iter().enumerate().map(|(c, align)| {
        let content = cells.get(c).map_or(Vec::new(), |cell| reader.inline(cell.children()));
        Node::new(kind, attr("align", *align), vec![Node::new("paragraph", Map::new(), content)])
    });
    Node::new("tableRow", Map::new(), built.collect())
}

/// Converts a GFM table: header row first, rows padded or cut to the header's width.
pub fn table(reader: &Reader, aligns: &[Alignment], children: &[Md]) -> Node {
    let aligns: Vec<&str> = aligns.iter().map(align_name).collect();
    let rows = children.iter().filter_map(|child| match child {
        Md::Block { tag: Tag::TableHead, children, .. } => Some(row(reader, "tableHeader", &aligns, children)),
        Md::Block { tag: Tag::TableRow, children, .. } => Some(row(reader, "tableCell", &aligns, children)),
        _ => None,
    });
    Node::new("table", Map::new(), rows.collect())
}
