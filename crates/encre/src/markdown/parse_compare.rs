use super::parse::{Reader, attr};
use super::parse_columns::columns;
use crate::model::Node;
use serde_json::{Map, Value};

const SUCCESS: &[&str] = &["advantage", "pro", "positive", "plus", "benefit"];
const DANGER: &[&str] = &["drawback", "con", "negative", "minus", "risk", "issue"];

fn tone(title: &str) -> &'static str {
    let lower = title.to_lowercase();
    if SUCCESS.iter().any(|word| lower.contains(word)) {
        "success"
    } else if DANGER.iter().any(|word| lower.contains(word)) {
        "danger"
    } else {
        "neutral"
    }
}

fn heading(line: &str) -> Option<&str> {
    line.strip_prefix("###").filter(|rest| rest.starts_with([' ', '\t'])).map(str::trim_start)
}

fn rule(line: &str) -> bool {
    line.trim().len() >= 3 && line.trim().chars().all(|c| c == '-')
}

fn sections(raw: &str, headed: bool) -> Vec<String> {
    let mut out: Vec<String> = vec![String::new()];
    for line in raw.lines() {
        let starts = if headed { heading(line).is_some() } else { rule(line) };
        if starts && !out.last().is_some_and(|s| s.trim().is_empty()) {
            out.push(String::new());
        }
        if !headed && starts {
            continue;
        }
        let current = out.last_mut().expect("out is never empty");
        current.push_str(heading(line).filter(|_| headed).unwrap_or(line));
        current.push('\n');
    }
    out.retain(|s| !s.trim().is_empty());
    out
}

fn label(title: &str) -> String {
    title.trim_start_matches('#').trim().replace('`', "'").chars().filter(|c| *c == '\t' || !c.is_control()).collect()
}

fn section(section: &str) -> Node {
    let (title, body) = section.trim().split_once('\n').unwrap_or((section.trim(), ""));
    let title = label(title);
    let lines: Vec<&str> = body.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    let mut attrs: Map<String, Value> = attr("title", title.as_str());
    attrs.insert("tone".to_owned(), Value::from(tone(&title)));
    Node::new("column", attrs, Reader { source: &lines.join("\n\n"), track: None }.doc().content)
}

impl Reader<'_> {
    /// A `compare` fence as `columns`: one column per `### ` section, else per `---` separated part.
    pub fn compare(&self) -> Option<Node> {
        let headed = sections(self.source, true);
        let parts = if headed.len() >= 2 { headed } else { sections(self.source, false) };
        columns(parts.iter().map(|part| section(part)).collect())
    }
}
