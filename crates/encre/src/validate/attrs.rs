use crate::ids::is_block_id;
use crate::schema::AttrSpec;
use serde_json::{Map, Value};

/// Whether `c` may appear in single-line text: anything but a control character other than tab.
pub fn line_char(c: char) -> bool {
    c == '\t' || !c.is_control()
}

fn is_line(value: &Value) -> bool {
    value.as_str().is_some_and(|s| s.chars().all(line_char))
}

fn is_info(value: &Value) -> bool {
    let s = value.as_str().unwrap_or("");
    is_line(value) && s.split_ascii_whitespace().collect::<Vec<_>>().join(" ") == s
}

fn is_url(value: &Value) -> bool {
    let host = value.as_str().and_then(|s| s.strip_prefix("https://")).unwrap_or("");
    is_line(value) && host.starts_with(|c: char| !"/?#".contains(c)) && !host.contains(char::is_whitespace)
}

fn is_series(value: &Value) -> bool {
    let Some(items) = value.as_array() else {
        return false;
    };
    items.iter().all(|item| {
        let Some(object) = item.as_object() else {
            return false;
        };
        let data_ok = object.get("data").and_then(Value::as_array).is_some_and(|d| d.iter().all(Value::is_number));
        object.len() == 2 && object.get("name").is_some_and(is_line) && data_ok
    })
}

fn integer_ok(spec: &AttrSpec, value: &Value) -> bool {
    value.as_i64().is_some_and(|n| spec.min.is_none_or(|min| n >= min) && spec.max.is_none_or(|max| n <= max))
}

fn value_ok(spec: &AttrSpec, value: &Value) -> bool {
    if value.is_null() {
        return !spec.required && spec.default.is_null();
    }
    let typed = match spec.kind.as_str() {
        "id" => value.as_str().is_some_and(is_block_id),
        "integer" => integer_ok(spec, value),
        "boolean" => value.is_boolean(),
        "string" => value.is_string(),
        "line" => is_line(value),
        "info" => is_info(value),
        "label" => is_line(value) && !value.as_str().unwrap_or("").contains('`'),
        "lines" => value.as_array().is_some_and(|items| items.iter().all(is_line)),
        "series" => is_series(value),
        "url" => is_url(value),
        _ => false,
    };
    typed && spec.values.as_ref().is_none_or(|values| values.contains(value))
}

/// Checks `attrs` against `specs`, returning the first problem.
pub fn check(specs: &[AttrSpec], attrs: &Map<String, Value>) -> Result<(), String> {
    if let Some(unknown) = attrs.keys().find(|key| !specs.iter().any(|spec| &spec.name == *key)) {
        return Err(format!("unknown attribute `{unknown}`"));
    }
    for spec in specs {
        match attrs.get(&spec.name) {
            None if spec.required => {
                return Err(format!("missing required attribute `{}`", spec.name));
            }
            Some(value) if !value_ok(spec, value) => {
                return Err(format!("invalid attribute `{}`: {value}", spec.name));
            }
            _ => {}
        }
    }
    Ok(())
}
