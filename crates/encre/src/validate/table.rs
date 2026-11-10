use crate::model::Node;

fn code_pipe(node: &Node) -> bool {
    node.marks.iter().any(|m| m.kind == "code") && node.text.as_deref().is_some_and(|t| t.contains("\\|"))
}

fn cell_ok(header: &Node, cell: &Node, want: &str) -> Result<(), String> {
    if cell.kind != want {
        return Err(format!("is `{}`, want `{want}`", cell.kind));
    }
    if cell.content.iter().flat_map(|p| &p.content).any(code_pipe) {
        return Err("has code text with a backslash before a pipe".to_owned());
    }
    if cell.attr_str("align") != header.attr_str("align") {
        return Err("is aligned unlike its column".to_owned());
    }
    Ok(())
}

/// Checks rules 5 and 6 of section 1.3: rectangular, header row first, one alignment per column, no code pipe.
pub fn check(table: &Node) -> Result<(), String> {
    let header = table.content.first().map_or(&[][..], |row| &row.content[..]);
    for (r, row) in table.content.iter().enumerate() {
        if row.content.len() != header.len() {
            return Err(format!("row {r} has {} cells, the header row {}", row.content.len(), header.len()));
        }
        let want = if r == 0 { "tableHeader" } else { "tableCell" };
        for (c, cell) in row.content.iter().enumerate() {
            cell_ok(&header[c], cell, want).map_err(|e| format!("row {r} cell {c} {e}"))?;
        }
    }
    Ok(())
}
