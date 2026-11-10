use super::inline::inline;
use crate::escape::quoted;
use crate::model::Node;
use serde_json::Value;

/// A content block holding `text` as a string literal.
pub fn cell(text: &str) -> String {
    format!("[#{}]", quoted(text))
}

fn number(value: &Value) -> String {
    value.as_number().map(ToString::to_string).unwrap_or_default()
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value.and_then(Value::as_array).into_iter().flatten().map(|v| v.as_str().unwrap_or_default().to_owned()).collect()
}

/// A muse-typst metric card.
pub fn stat(node: &Node) -> String {
    let sub: Vec<&str> = [node.attr_str("delta"), node.attr_str("sub")].into_iter().filter(|s| !s.is_empty()).collect();
    let sub = if sub.is_empty() { String::new() } else { format!(", sub: {}", quoted(&sub.join(" · "))) };
    format!("#metric({}, {}{sub})", quoted(node.attr_str("value")), quoted(node.attr_str("label")))
}

/// A chart as a table: muse-typst has no chart component. One row per series, one column per label.
pub fn chart(node: &Node) -> String {
    let labels = strings(node.attrs.get("labels"));
    let series: Vec<&Value> = node.attrs.get("series").and_then(Value::as_array).into_iter().flatten().collect();
    let data = |s: &Value| s.get("data").and_then(Value::as_array).cloned().unwrap_or_default();
    let width = series.iter().map(|s| data(s).len()).chain([labels.len()]).max().unwrap_or(0);
    let header: Vec<String> = std::iter::once("[]".to_owned())
        .chain((0..width).map(|i| cell(labels.get(i).map_or("", String::as_str))))
        .collect();
    let mut cells = Vec::new();
    for s in &series {
        let values = data(s);
        cells.push(cell(s.get("name").and_then(Value::as_str).unwrap_or_default()));
        cells.extend((0..width).map(|i| cell(&values.get(i).map(number).unwrap_or_default())));
    }
    let title = node.attr_str("title");
    let caption = if title.is_empty() { String::new() } else { format!(", caption: {}", cell(title)) };
    format!(
        "#figure(table(columns: {}, table.header({}), {}){caption})",
        width + 1,
        header.join(", "),
        cells.join(", ")
    )
}

/// Items joined as call arguments.
pub fn args(items: impl Iterator<Item = String>) -> String {
    items.collect::<Vec<_>>().join(", ")
}

/// A code block as a raw block.
pub fn code_block(node: &Node) -> String {
    let language = node.attr_str("language");
    let lang = if language.is_empty() { String::new() } else { format!("lang: {}, ", quoted(language)) };
    format!(
        "#raw(block: true, {lang}{})",
        quoted(&node.content.iter().filter_map(|n| n.text.as_deref()).collect::<String>())
    )
}

/// A table, with one alignment per column taken from the header row.
pub fn table(node: &Node) -> String {
    let rows = &node.content;
    let columns = rows.first().map_or(0, |row| row.content.len());
    let align = rows.first().into_iter().flat_map(|row| &row.content).map(|c| match c.attr_str("align") {
        "left" | "center" | "right" => c.attr_str("align"),
        _ => "auto",
    });
    let cells = |row: &Node| {
        args(row.content.iter().map(|c| format!("[{}]", inline(c.content.first().map_or(&[][..], |p| &p.content)))))
    };
    let header = rows.first().map(|row| format!("table.header({})", cells(row))).into_iter();
    let body = rows.iter().skip(1).map(cells).filter(|row| !row.is_empty());
    format!("#table(columns: {columns}, align: ({},), {})", args(align.map(str::to_owned)), args(header.chain(body)))
}
