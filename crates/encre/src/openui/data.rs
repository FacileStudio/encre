use super::Error;
use super::args::Args;
use super::resolve::Value;
use super::text::{attrs, label, line, paragraph, paragraphs};
use crate::model::Node;
use serde_json::{Map, Value as Json, json};

const TONES: [&str; 5] = ["neutral", "info", "success", "warning", "danger"];

/// Optional argument `i` that must be one of `allowed`; `fallback` when absent.
pub fn choice(args: &Args, i: usize, allowed: &[&str], fallback: &str) -> Result<String, Error> {
    let value = args.opt_string(i, false)?.unwrap_or_else(|| fallback.to_owned());
    if allowed.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(args.error(format!(
            "argument {} ({}) must be one of {}, found {value:?}",
            i + 1,
            args.names[i],
            allowed.join(", ")
        )))
    }
}

/// Required argument `i` as an array of strings.
pub fn strings(args: &Args, i: usize) -> Result<Vec<String>, Error> {
    args.array(i)?
        .iter()
        .map(|v| match v {
            Value::Str(s) => Ok(s.clone()),
            Value::Num(n) => Ok(n.to_string()),
            other => Err(args.wrong(i, "an array of strings", format!("{} in it", other.describe()))),
        })
        .collect()
}

/// `Stat(label, value, delta?)` as a `stat` block.
pub fn stat(args: &Args) -> Result<Node, Error> {
    let delta = args.opt_string(2, true)?.unwrap_or_default();
    let pairs = vec![
        ("label", Json::from(line(&args.string(0, false)?))),
        ("value", Json::from(line(&args.string(1, true)?))),
        ("delta", Json::from(line(&delta))),
    ];
    Ok(Node::new("stat", attrs(pairs), Vec::new()))
}

/// `Callout(text, tone?, title?)` as a `callout` block holding the text's paragraphs.
pub fn callout(args: &Args) -> Result<Node, Error> {
    let text = args.string(0, true)?;
    let tone = choice(args, 1, &TONES, "neutral")?;
    let title = label(&args.opt_string(2, true)?.unwrap_or_default());
    let tone = if tone == "neutral" { String::new() } else { tone };
    Ok(Node::new("callout", attrs(vec![("tone", Json::from(tone)), ("title", Json::from(title))]), paragraphs(&text)))
}

/// `Chart(kind, labels, series, title?)` as a `chart` block.
pub fn chart(args: &Args) -> Result<Node, Error> {
    let kind = choice(args, 0, &["bar", "line"], "")?;
    let labels: Vec<String> = strings(args, 1)?.iter().map(|l| line(l)).collect();
    let series = args.array(2)?.iter().map(|s| series(args, s)).collect::<Result<Vec<_>, _>>()?;
    let title = line(&args.opt_string(3, true)?.unwrap_or_default());
    let mut pairs = vec![("title", Json::from(title)), ("labels", json!(labels)), ("series", Json::Array(series))];
    if kind != "bar" {
        pairs.insert(0, ("kind", Json::from(kind)));
    }
    let mut out = attrs(pairs);
    out.retain(|_, v| v.as_array().is_none_or(|a| !a.is_empty()));
    Ok(Node::new("chart", out, Vec::new()))
}

fn series(args: &Args, value: &Value) -> Result<Json, Error> {
    let wrong = || {
        args.wrong(
            2,
            "an array of {name: string, data: [numbers]} objects",
            format!("{} that is not one", value.describe()),
        )
    };
    let Value::Object(fields) = value else { return Err(wrong()) };
    let field = |key: &str| fields.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v);
    let Some(Value::Str(name)) = field("name") else { return Err(wrong()) };
    let Some(Value::Array(data)) = field("data") else { return Err(wrong()) };
    let numbers = data.iter().map(|d| if let Value::Num(n) = d { Ok(Json::Number(n.clone())) } else { Err(wrong()) });
    Ok(json!({"name": line(name), "data": numbers.collect::<Result<Vec<_>, _>>()?}))
}

fn cell(args: &Args, value: &Value) -> Result<String, Error> {
    match value {
        Value::Str(s) => Ok(s.clone()),
        Value::Num(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok(String::new()),
        other => Err(args.wrong(1, "rows of strings, numbers or booleans", format!("{} in a row", other.describe()))),
    }
}

/// `Table(columns, rows)` as a `table` whose first row is the columns; short rows are padded.
pub fn table(args: &Args) -> Result<Vec<Node>, Error> {
    let mut grid = vec![strings(args, 0)?];
    for row in args.array(1)? {
        let Value::Array(cells) = row else {
            return Err(args.wrong(1, "an array of rows", format!("{} as a row", row.describe())));
        };
        grid.push(cells.iter().map(|c| cell(args, c)).collect::<Result<_, _>>()?);
    }
    let width = grid.iter().map(Vec::len).max().unwrap_or(0);
    if width == 0 {
        return Ok(Vec::new());
    }
    let rows = grid.iter().enumerate().map(|(i, row)| {
        let kind = if i == 0 { "tableHeader" } else { "tableCell" };
        let cells = (0..width).map(|c| row.get(c).map_or("", String::as_str));
        Node::new("tableRow", Map::new(), cells.map(|c| Node::new(kind, Map::new(), vec![paragraph(c, &[])])).collect())
    });
    Ok(vec![Node::new("table", Map::new(), rows.collect())])
}
