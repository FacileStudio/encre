use super::Error;
use super::args::Args;
use super::data;
use super::resolve::Value;
use super::text::{attrs, inline, line, paragraph, paragraphs};
use crate::model::{Mark, Node};
use serde_json::{Map, Value as Json};

/// Every component of OpenUI's library, as Bulle defines it in `library.ts`.
pub const COMPONENTS: [&str; 12] =
    ["Card", "Stat", "Table", "List", "Badge", "Callout", "Chart", "Button", "Form", "Field", "Text", "Divider"];

/// The blocks `value` shows. `level` is the heading level a Card title takes.
pub fn blocks(value: &Value, level: u8) -> Result<Vec<Node>, Error> {
    match value {
        Value::Component(name, values) => component(name, values, level),
        other => Err(Error::Argument {
            component: "root".to_owned(),
            message: format!("must be a component, found {}", other.describe()),
        }),
    }
}

fn component(name: &str, values: &[Value], level: u8) -> Result<Vec<Node>, Error> {
    match name {
        "Card" => card(&Args::new(name, &["children", "title", "description"], values)?, level),
        "Stat" => data::stat(&Args::new(name, &["label", "value", "delta"], values)?).map(|n| vec![n]),
        "Table" => data::table(&Args::new(name, &["columns", "rows"], values)?),
        "List" => list(&Args::new(name, &["items", "ordered"], values)?),
        "Callout" => data::callout(&Args::new(name, &["text", "tone", "title"], values)?).map(|n| vec![n]),
        "Chart" => data::chart(&Args::new(name, &["kind", "labels", "series", "title"], values)?).map(|n| vec![n]),
        "Text" => Ok(paragraphs(&Args::new(name, &["content", "tone"], values)?.string(0, true)?)),
        "Badge" => Ok(vec![paragraph(&Args::new(name, &["text", "tone"], values)?.string(0, true)?, &[bold()])]),
        "Divider" => Args::new(name, &[], values).map(|_| vec![Node::new("horizontalRule", Map::new(), Vec::new())]),
        _ => Ok(Vec::new()),
    }
}

fn bold() -> Mark {
    Mark::new("bold", Map::new())
}

fn card(args: &Args, level: u8) -> Result<Vec<Node>, Error> {
    let mut out = Vec::new();
    let title = line(&args.opt_string(1, true)?.unwrap_or_default());
    if !title.trim().is_empty() {
        out.push(Node::new("heading", attrs(vec![("level", Json::from(level))]), inline(title.trim(), &[])));
    }
    if let Some(description) = args.opt_string(2, true)?.filter(|d| !d.trim().is_empty()) {
        out.extend(paragraphs(&description));
    }
    for child in args.array(0)? {
        let Value::Component(..) = child else {
            return Err(args.error(format!("argument 1 (children) holds {}, not a component", child.describe())));
        };
        out.extend(blocks(child, (level + 1).min(3))?);
    }
    Ok(out)
}

fn list(args: &Args) -> Result<Vec<Node>, Error> {
    let items = data::strings(args, 0)?;
    let ordered = match args.get(1) {
        None => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err(args.wrong(1, "a boolean", args.found(1))),
    };
    if items.is_empty() {
        return Ok(Vec::new());
    }
    let item = |text: &String| Node::new("listItem", Map::new(), vec![paragraph(text, &[])]);
    let kind = if ordered { "orderedList" } else { "bulletList" };
    Ok(vec![Node::new(kind, Map::new(), items.iter().map(item).collect())])
}
