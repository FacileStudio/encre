use super::escape::Ctx;
use super::serialize::Writer;
use crate::model::Node;

fn delimiter(align: &str) -> &'static str {
    match align {
        "left" => ":--",
        "center" => ":-:",
        "right" => "--:",
        _ => "---",
    }
}

/// Writes a GFM table: header row, delimiter row, body rows.
pub fn table(writer: &Writer, node: &Node) -> String {
    let rows: Vec<String> = node
        .content
        .iter()
        .map(|row| {
            let cells: Vec<String> = row
                .content
                .iter()
                .map(|cell| writer.inline(cell.content.first().map_or(&[][..], |p| &p.content), Ctx::Cell))
                .collect();
            format!("| {} |", cells.join(" | "))
        })
        .collect();
    let header = node.content.first().map_or(&[][..], |row| &row.content);
    let aligns: Vec<&str> = header.iter().map(|cell| delimiter(cell.attr_str("align"))).collect();
    let mut lines = rows;
    lines.insert(1.min(lines.len()), format!("| {} |", aligns.join(" | ")));
    lines.join("\n")
}
