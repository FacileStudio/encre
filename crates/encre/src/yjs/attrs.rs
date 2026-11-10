use crate::schema::AttrSpec;
use serde_json::{Map, Value};
use yrs::Any;

/// The name y-prosemirror gives the overlapping `code` mark: its type plus the hash of `{"type":"code"}`.
pub const CODE_KEY: &str = "code--LpaW+ak5";

/// Every attribute `specs` declares, defaults filled in and nulls left out, as y-prosemirror stores them.
pub fn filled(specs: &[AttrSpec], attrs: &Map<String, Value>) -> Vec<(String, Any)> {
    specs
        .iter()
        .filter_map(|spec| {
            let value = attrs.get(&spec.name).unwrap_or(&spec.default);
            (!value.is_null()).then(|| (spec.name.clone(), to_any(value)))
        })
        .collect()
}

/// A JSON value as a Yjs `Any`.
pub fn to_any(value: &Value) -> Any {
    serde_json::from_value(value.clone()).unwrap_or(Any::Null)
}

/// A Yjs `Any` as a JSON value.
pub fn to_json(any: &Any) -> Value {
    serde_json::to_value(any).unwrap_or(Value::Null)
}

/// The mark type behind a y-prosemirror attribute name, without its `--hash` suffix.
pub fn mark_name(key: &str) -> &str {
    match key.rsplit_once("--") {
        Some((name, hash))
            if hash.len() == 8 && hash.chars().all(|c| c.is_ascii_alphanumeric() || "+/=".contains(c)) =>
        {
            name
        }
        _ => key,
    }
}
