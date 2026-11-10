use encre::markdown::{parse, serialize};
use encre::ops::{BlockView, Op, Outcome, apply};
use encre::{Node, validate};
use serde_json::Value;
use std::path::PathBuf;

struct Vector {
    name: String,
    doc: Node,
    version: u64,
    op: Op,
    raw: Value,
}

fn start(raw: &Value) -> Node {
    if let Some(doc) = raw.get("doc") {
        return serde_json::from_value(doc.clone()).expect("doc");
    }
    let mut doc = parse(raw["markdown"].as_str().expect("markdown")).expect("parse");
    for (i, block) in doc.content.iter_mut().enumerate() {
        block.attrs.insert("id".to_owned(), Value::from(format!("b{i:09}")));
    }
    doc
}

fn vectors() -> Vec<Vector> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec/vectors/ops");
    let mut files: Vec<PathBuf> =
        std::fs::read_dir(&dir).expect("ops vectors").map(|e| e.expect("entry").path()).collect();
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let raw: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("file")).expect("JSON");
            let op = serde_json::from_value(raw["op"].clone()).expect("op");
            let version = raw.get("version").and_then(Value::as_u64).unwrap_or(1);
            Vector { name: raw["name"].as_str().expect("name").to_owned(), doc: start(&raw), version, op, raw }
        })
        .collect()
}

fn ids(doc: &Node) -> Vec<String> {
    doc.content.iter().filter_map(|b| b.id().map(str::to_owned)).collect()
}

fn check(v: &Vector, before: &Node, result: Result<Outcome, encre::ops::OpError>, after: &Node) {
    let name = &v.name;
    if let Some(want) = v.raw.get("want_error").and_then(Value::as_str) {
        assert_eq!(result.expect_err(name).to_string(), want, "{name}: error");
        assert_eq!(after, before, "{name}: a refused operation changes nothing");
        return;
    }
    let outcome = result.unwrap_or_else(|e| panic!("{name}: {e}"));
    if let Some(want) = v.raw.get("want_blocks") {
        let want: Vec<BlockView> = serde_json::from_value(want.clone()).expect("want_blocks");
        assert_eq!(outcome, Outcome::Blocks(want), "{name}: blocks");
        return;
    }
    let want = v.raw["want_markdown"].as_str().expect("want_markdown");
    assert_eq!(serialize(after), want, "{name}: markdown");
    assert_eq!(validate(after), Ok(()), "{name}: valid");
    if let Outcome::Markdown(text) = &outcome {
        assert_eq!(text, want, "{name}: read");
    }
    for kept in v.raw.get("want_kept").and_then(Value::as_array).into_iter().flatten() {
        assert!(ids(after).contains(&kept.as_str().expect("id").to_owned()), "{name}: keeps id {kept}");
    }
    if let Outcome::Changed(new) = &outcome {
        assert!(new.iter().all(|id| ids(after).contains(id)), "{name}: reports inserted ids");
    }
}

#[test]
fn ops_vectors_hold_on_a_stored_document() {
    let all = vectors();
    assert!(all.len() >= 20, "expected the full ops vector set, found {}", all.len());
    for v in &all {
        let mut doc = v.doc.clone();
        let result = apply(&mut doc, v.version, &v.op);
        check(v, &v.doc, result, &doc);
    }
}

#[cfg(feature = "yjs")]
#[test]
fn ops_vectors_hold_on_an_open_yjs_document() {
    use encre::yjs::{FRAGMENT, new_doc, read, write};
    use yrs::Transact;
    for v in &vectors() {
        let ydoc = new_doc();
        let fragment = ydoc.get_or_insert_xml_fragment(FRAGMENT);
        write(&mut ydoc.transact_mut(), &fragment, &v.doc).expect("write");
        let before = read(&ydoc.transact(), &fragment).expect("read");
        let result = encre::ops::apply_yjs(&mut ydoc.transact_mut(), &fragment, v.version, &v.op);
        let after = read(&ydoc.transact(), &fragment).expect("read");
        check(v, &before, result, &after);
    }
}

#[test]
fn unknown_fields_in_an_op_are_refused() {
    let raw = serde_json::json!({"op": "delete_block", "block_id": "b000000000", "force": true});
    assert!(serde_json::from_value::<Op>(raw).is_err());
}
