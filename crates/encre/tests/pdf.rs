#![cfg(feature = "pdf")]

#[path = "support/typst.rs"]
mod typst_support;

use encre::Node;
use encre::pdf::{Assets, compile};
use encre::typst::escape_string;
use serde_json::Value;
use typst_support::{render_vector, typst_vectors};

const FLOW_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="gray"/></svg>"##;

fn strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) if !s.is_empty() => out.push(s.clone()),
        Value::Number(n) => out.push(n.to_string()),
        Value::Array(items) => items.iter().for_each(|v| strings(v, out)),
        Value::Object(map) => map
            .iter()
            .filter(|(k, _)| !matches!(k.as_str(), "kind" | "tone" | "src" | "href"))
            .for_each(|(_, v)| strings(v, out)),
        _ => {}
    }
}

fn source_strings(node: &Node, images: &Value, out: &mut Vec<String>) {
    out.extend(node.text.clone());
    let embedded = images.get(node.attr_str("src")).is_some();
    for (key, value) in &node.attrs {
        let shown = !matches!(
            key.as_str(),
            "id" | "level"
                | "start"
                | "checked"
                | "align"
                | "language"
                | "src"
                | "tone"
                | "kind"
                | "url"
                | "mime"
                | "size"
        );
        if shown && !(embedded && key == "alt") {
            strings(value, out);
        }
    }
    node.content.iter().for_each(|child| source_strings(child, images, out));
}

fn squash(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace() && *c != '-' && *c != '\u{ad}').collect()
}

#[test]
fn a_document_with_every_node_compiles_and_keeps_every_string() {
    let v = typst_vectors().into_iter().find(|v| v.name == "every-node").expect("every-node vector");
    let mut assets = Assets::new();
    assets.insert("/images/flow.svg", FLOW_SVG.as_bytes().to_vec());
    let pdf = compile(&render_vector(&v), &assets).expect("compile");
    assert!(pdf.starts_with(b"%PDF-"));
    let text = squash(&pdf_extract::extract_text_from_mem(&pdf).expect("extract text"));
    let mut wanted = Vec::new();
    source_strings(&v.doc, &v.raw["options"]["images"], &mut wanted);
    assert!(wanted.len() > 40, "the vector covers every node");
    let missing: Vec<&String> = wanted.iter().filter(|s| !text.contains(&squash(s))).collect();
    assert!(missing.is_empty(), "{}: missing from the PDF text: {missing:?}\n{text}", v.path.display());
}

#[test]
fn hostile_strings_reach_the_pdf_as_text() {
    let v = typst_vectors().into_iter().find(|v| v.name == "escapes").expect("escapes vector");
    let pdf = compile(&render_vector(&v), &Assets::new()).expect("compile");
    let text = squash(&pdf_extract::extract_text_from_mem(&pdf).expect("extract text"));
    assert!(text.contains(&squash("#text(size: 40pt)[INJECTED]")));
    assert!(text.contains(&squash("#set page(fill: red)")));
}

#[test]
fn escaped_strings_round_trip_through_the_compiler() {
    let hostile = [
        "Acme \"Corp\"",
        "back\\slash",
        "ends with backslash\\",
        "a\\\"; #text(size: 40pt)[INJECTED]; #let b = \"",
        "two\nlines\tand tab",
        "plain",
    ];
    for value in hostile {
        let mut assets = Assets::new();
        assets.insert("/orig.json", serde_json::to_vec(&[value]).expect("json"));
        let source = format!(
            "#let value = \"{}\"\n#let orig = json(\"/orig.json\")\n#assert.eq(value, orig.at(0))\n",
            escape_string(value)
        );
        assert!(compile(&source, &assets).is_ok(), "{value:?}");
    }
}

#[test]
fn the_compiler_reads_nothing_outside_the_assets() {
    let err = compile("#read(\"/etc/hostname\")", &Assets::new()).expect_err("no filesystem");
    assert!(err.to_string().contains("not found"), "{err}");
    let err = compile("#import \"@preview/cetz:0.3.0\": *", &Assets::new()).expect_err("no packages");
    assert!(err.to_string().contains("typst compile failed"), "{err}");
}

#[test]
fn the_same_source_gives_the_same_bytes() {
    let source = render_vector(&typst_vectors().into_iter().find(|v| v.name == "cover").expect("cover"));
    assert_eq!(compile(&source, &Assets::new()), compile(&source, &Assets::new()));
}
