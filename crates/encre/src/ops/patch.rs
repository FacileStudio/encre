use super::edit::Replace;
use super::error::{LISTED, OpError, PatchMatch};
use super::plan::{block_markdown, blocks};
use crate::model::Node;

const CONTEXT: usize = 16;

fn context(markdown: &str, at: usize, len: usize) -> String {
    let before: String = markdown[..at].chars().rev().take(CONTEXT).collect::<Vec<_>>().into_iter().rev().collect();
    let after: String = markdown[at + len..].chars().take(CONTEXT).collect();
    let open = if before.len() < at { "…" } else { "" };
    let close = if at + len + after.len() < markdown.len() { "…" } else { "" };
    format!("{open}{before}{}{after}{close}", &markdown[at..at + len]).replace('\n', "\\n").replace('"', "\\\"")
}

/// The block replacements a `patch` makes, after checking it matches exactly `expect` times.
pub fn replaces(doc: &Node, find: &str, replace: &str, expect: usize) -> Result<Vec<Replace>, OpError> {
    if find.is_empty() {
        return Err(OpError::BadPatch("find is empty; give the exact text to replace"));
    }
    if expect == 0 {
        return Err(OpError::BadPatch("expect must be at least 1"));
    }
    let (found, count, hits) = matches(doc, find);
    if count != expect {
        return Err(OpError::PatchMatches { expect, count, found });
    }
    hits.into_iter().map(|(i, markdown)| replacement(i, &markdown.replace(find, replace))).collect()
}

/// The first [`LISTED`] matches of `find`, how many there are, and the markdown of each block that holds one.
fn matches(doc: &Node, find: &str) -> (Vec<PatchMatch>, usize, Vec<(usize, String)>) {
    let (mut found, mut count, mut hits) = (Vec::new(), 0, Vec::new());
    for (i, block) in doc.content.iter().enumerate() {
        let markdown = block_markdown(block);
        let before = count;
        for (at, _) in markdown.match_indices(find) {
            count += 1;
            if found.len() < LISTED {
                let block = block.id().map_or_else(|| format!("#{i}"), str::to_owned);
                found.push(PatchMatch { block, context: context(&markdown, at, find.len()) });
            }
        }
        if count > before {
            hits.push((i, markdown));
        }
    }
    (found, count, hits)
}

fn replacement(at: usize, markdown: &str) -> Result<Replace, OpError> {
    let nodes = if markdown.trim().is_empty() { Vec::new() } else { blocks("patch", markdown)? };
    Ok(Replace::one(at, nodes))
}
