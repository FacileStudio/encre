use super::YjsError;
use super::attrs::{mark_name, to_json};
use crate::compact::compact_attrs as compact;
use crate::model::{Mark, Node};
use crate::schema::schema;
use serde_json::{Map, Value};
use yrs::types::text::{Diff, YChange};
use yrs::{Any, Out, ReadTxn, Text, Xml, XmlElementRef, XmlFragment, XmlOut, XmlTextRef};

/// The nodes held by `parent`, text runs split into text nodes.
pub fn children<T: ReadTxn, P: XmlFragment>(txn: &T, parent: &P) -> Result<Vec<Node>, YjsError> {
    let mut nodes = Vec::new();
    for child in parent.children(txn) {
        match child {
            XmlOut::Element(element) => nodes.push(self::element(txn, &element)?),
            XmlOut::Text(text) => nodes.extend(self::text(txn, &text)?),
            XmlOut::Fragment(_) => return Err(YjsError::Shape("a nested fragment")),
        }
    }
    Ok(nodes)
}

fn element<T: ReadTxn>(txn: &T, element: &XmlElementRef) -> Result<Node, YjsError> {
    let kind = element.tag().to_string();
    let node_type = schema().node(&kind).ok_or_else(|| YjsError::Unknown("node", kind.clone()))?;
    let raw: Map<String, Value> = element
        .attributes(txn)
        .filter_map(|(key, value)| match value {
            Out::Any(any) => Some((key.to_owned(), to_json(&any))),
            _ => None,
        })
        .collect();
    Ok(Node::new(&kind, compact(&node_type.attrs, &raw), children(txn, element)?))
}

fn text<T: ReadTxn>(txn: &T, text: &XmlTextRef) -> Result<Vec<Node>, YjsError> {
    text.diff(txn, YChange::identity).into_iter().map(|chunk| run(&chunk)).collect()
}

fn run(chunk: &Diff<YChange>) -> Result<Node, YjsError> {
    let Out::Any(Any::String(text)) = &chunk.insert else {
        return Err(YjsError::Shape("an embed inside text"));
    };
    let mut marks = Vec::new();
    for (key, value) in chunk.attributes.iter().flat_map(|attrs| attrs.iter()) {
        if !value_is_null(value) {
            marks.push(mark(mark_name(key), value)?);
        }
    }
    marks.sort_by_key(|m| schema().mark_rank(&m.kind));
    Ok(Node::text(text, marks))
}

fn value_is_null(value: &Any) -> bool {
    matches!(value, Any::Null | Any::Undefined)
}

fn mark(name: &str, value: &Any) -> Result<Mark, YjsError> {
    let mark_type = schema().mark(name).ok_or_else(|| YjsError::Unknown("mark", name.to_owned()))?;
    let raw = match to_json(value) {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    Ok(Mark::new(name, compact(&mark_type.attrs, &raw)))
}
