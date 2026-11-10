use super::attrs::{self, line_char};
use crate::model::Node;
use crate::schema::{NodeType, Schema};

fn text_ok(text: &str, in_code: bool) -> bool {
    !text.is_empty() && text.chars().all(|c| line_char(c) || (in_code && c == '\n'))
}

fn marks_ok(schema: &Schema, parent: &NodeType, child: &Node) -> Result<(), String> {
    let takes = schema.node(&child.kind).is_some_and(|t| t.takes_marks);
    if !child.marks.is_empty() && (!takes || parent.marks.is_empty()) {
        return Err(format!("`{}` here takes no marks", child.kind));
    }
    let mut last = None;
    for mark in &child.marks {
        let mark_type = schema.mark(&mark.kind).ok_or_else(|| format!("unknown mark `{}`", mark.kind))?;
        attrs::check(&mark_type.attrs, &mark.attrs).map_err(|e| format!("mark `{}`: {e}", mark.kind))?;
        let rank = schema.mark_rank(&mark.kind);
        if last.is_some_and(|previous| previous >= rank) {
            return Err("marks are out of rank order or repeated".to_owned());
        }
        last = Some(rank);
    }
    Ok(())
}

fn child_ok(node: &Node, i: usize, in_code: bool) -> Result<(), String> {
    let child = &node.content[i];
    let linked_code = ["link", "code"].iter().all(|k| child.marks.iter().any(|m| m.kind == *k));
    if linked_code && child.text.as_deref().is_some_and(|t| t.contains("]:")) {
        return Err("linked code text holds `]:`".to_owned());
    }
    if child.kind == "text" && !child.text.as_deref().is_some_and(|t| text_ok(t, in_code)) {
        return Err("text is empty or holds a forbidden character".to_owned());
    }
    let previous = i.checked_sub(1).map(|p| &node.content[p]);
    if previous.is_some_and(|p| p.kind == "text" && child.kind == "text" && p.marks == child.marks) {
        return Err("adjacent text nodes with the same marks".to_owned());
    }
    Ok(())
}

/// Checks the inline rules of section 1.3 on the children of `node`.
pub fn check(schema: &Schema, node: &Node, node_type: &NodeType) -> Result<(), String> {
    let in_code = node.kind == "codeBlock" || node.kind == "diagram";
    for (i, child) in node.content.iter().enumerate() {
        marks_ok(schema, node_type, child)
            .and_then(|()| child_ok(node, i, in_code))
            .map_err(|e| format!("content[{i}]: {e}"))?;
    }
    Ok(())
}
