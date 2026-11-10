use serde_json::Value;

/// Writes `value` as one-line JSON with `", "` and `": "` separators, keys in the map's order.
pub fn write(value: &Value) -> String {
    match value {
        Value::Array(items) => format!("[{}]", items.iter().map(write).collect::<Vec<_>>().join(", ")),
        Value::Object(map) => {
            let pairs: Vec<String> =
                map.iter().map(|(k, v)| format!("{}: {}", Value::from(k.as_str()), write(v))).collect();
            format!("{{{}}}", pairs.join(", "))
        }
        scalar => scalar.to_string(),
    }
}

/// Writes the pairs as a JSON object in the given order.
pub fn object(pairs: &[(&str, String)]) -> String {
    let body: Vec<String> = pairs.iter().map(|(k, v)| format!("{}: {v}", Value::from(*k))).collect();
    format!("{{{}}}", body.join(", "))
}
