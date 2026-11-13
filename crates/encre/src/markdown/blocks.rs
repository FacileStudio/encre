use super::json;
use crate::model::Node;
use crate::schema::{AttrSpec, schema};
use serde_json::{Map, Value};

/// The fence names that open a custom block: a code block whose language starts with one is written with tildes.
pub const CUSTOM: &[&str] =
    &["callout", "stat", "chart", "columns", "compare", "pros-cons", "vs", "mermaid", "d2", "dot", "embed", "file"];

/// The diagram languages, each its own fence name.
pub const DIAGRAMS: &[&str] = &["mermaid", "d2", "dot"];

fn data_attrs(kind: &str) -> impl Iterator<Item = &'static AttrSpec> {
    schema().node(kind).into_iter().flat_map(|t| t.attrs.iter()).filter(|a| a.kind != "id")
}

fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The info string of a custom block with children: its name and its non-default attributes.
pub fn info(node: &Node) -> String {
    let mut out = node.kind.clone();
    for spec in data_attrs(&node.kind) {
        let value = node.attrs.get(&spec.name).cloned().unwrap_or(Value::Null);
        if value != spec.default && !value.is_null() {
            out.push_str(&format!(" {}={}", spec.name, quote(value.as_str().unwrap_or(""))));
        }
    }
    out
}

fn series(value: &Value) -> String {
    let items: Vec<String> = value
        .as_array()
        .into_iter()
        .flatten()
        .map(|item| {
            let field = |key: &str| json::write(item.get(key).unwrap_or(&Value::Null));
            json::object(&[("name", field("name")), ("data", field("data"))])
        })
        .collect();
    format!("[{}]", items.join(", "))
}

/// The JSON body of a data-only custom block, keys in schema order; an attribute added after version 1
/// is left out when it holds its default.
pub fn body(node: &Node) -> String {
    let pairs: Vec<(&str, String)> = data_attrs(&node.kind)
        .filter(|spec| spec.since.is_none() || node.attrs.get(&spec.name).is_some_and(|v| *v != spec.default))
        .map(|spec| {
            let value = node.attrs.get(&spec.name).unwrap_or(&spec.default);
            (spec.name.as_str(), if spec.kind == "series" { series(value) } else { json::write(value) })
        })
        .collect();
    json::object(&pairs)
}

fn read_value(rest: &str) -> Option<(String, &str)> {
    let mut value = String::new();
    let mut chars = rest.strip_prefix('"')?.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Some((value, &rest[i + 2..])),
            '\\' => value.push(chars.next().map(|(_, e)| e).filter(|e| *e == '"' || *e == '\\')?),
            _ => value.push(c),
        }
    }
    None
}

/// Reads `key="value"` attributes from the raw info string after the block name.
pub fn read_info(mut rest: &str) -> Option<Vec<(String, String)>> {
    let mut pairs = Vec::new();
    loop {
        rest = rest.trim_start_matches([' ', '\t']);
        if rest.is_empty() {
            return Some(pairs);
        }
        let eq = rest.find('=')?;
        let key = &rest[..eq];
        if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric()) {
            return None;
        }
        let (value, after) = read_value(&rest[eq + 1..])?;
        if !after.is_empty() && !after.starts_with([' ', '\t']) {
            return None;
        }
        pairs.push((key.to_owned(), value));
        rest = after;
    }
}

/// Builds the attributes of `kind` from read pairs, defaults first (but for attributes added after version 1);
/// None when a key is unknown or repeated.
pub fn attrs_from(kind: &str, pairs: Vec<(String, Value)>) -> Option<Map<String, Value>> {
    let mut attrs: Map<String, Value> =
        data_attrs(kind).filter(|s| s.since.is_none()).map(|s| (s.name.clone(), s.default.clone())).collect();
    let mut seen = Vec::new();
    for (key, value) in pairs {
        if !data_attrs(kind).any(|s| s.name == key) || seen.contains(&key) {
            return None;
        }
        seen.push(key.clone());
        attrs.insert(key, value);
    }
    Some(attrs)
}
