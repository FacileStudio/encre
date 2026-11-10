#[path = "support/scratch.rs"]
mod scratch;
mod support;

use support::encre;

fn doc_file(name: &str, markdown: &str) -> String {
    let path = scratch::scratch(name).join("doc.md");
    std::fs::write(&path, markdown).expect("doc file");
    path.to_str().expect("path").to_owned()
}

#[test]
fn ops_apply_in_order_and_count_versions() {
    let doc = doc_file("ops-order", "# Plan\n\nIntro.");
    let ops = r#"[{"op":"append","markdown":"End."},{"op":"read"},{"op":"prepend","markdown":"Start."}]"#;
    let run = encre(&["ops", "--doc", &doc, "--doc-version", "4"], ops);
    assert_eq!(run.code, 0, "{}", run.stderr);
    let out = run.json();
    assert_eq!(out["version"], 6);
    assert_eq!(out["outcomes"][1]["markdown"], "# Plan\n\nIntro.\n\nEnd.");
    assert_eq!(out["outcomes"][2]["changed"].as_array().expect("ids").len(), 1);
    let markdown = encre(&["serialize"], &out["doc"].to_string()).text();
    assert_eq!(markdown, "Start.\n\n# Plan\n\nIntro.\n\nEnd.");
}

#[test]
fn a_markdown_doc_gets_ids_that_later_ops_can_address() {
    let doc = doc_file("ops-ids", "# Plan\n\nIntro.");
    let blocks = encre(&["ops", "--doc", &doc], r#"[{"op":"read_blocks"}]"#).json();
    let id = blocks["doc"]["content"][1]["attrs"]["id"].as_str().expect("id").to_owned();
    let saved = scratch::scratch("ops-ids-json").join("doc.json");
    std::fs::write(&saved, blocks["doc"].to_string()).expect("json doc");
    let op = format!(r#"[{{"op":"replace_block","block_id":"{id}","markdown":"Outro."}}]"#);
    let run = encre(&["ops", "--doc", saved.to_str().expect("path"), "--from", "json"], &op);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.json()["doc"]["content"][1]["content"][0]["text"], "Outro.");
}

#[test]
fn a_refused_op_prints_nothing_and_names_its_index() {
    let doc = doc_file("ops-refused", "# Plan");
    let ops = r#"[{"op":"append","markdown":"Fine."},{"op":"delete_block","block_id":"zzzzzzzzzz"}]"#;
    let run = encre(&["ops", "--doc", &doc, "--json"], ops);
    assert_eq!(run.code, 1);
    assert!(run.text().is_empty());
    let error: serde_json::Value = serde_json::from_str(&run.stderr).expect("JSON error");
    assert_eq!(error["error"]["path"], "ops[1]");
    assert!(error["error"]["message"].as_str().expect("message").contains("read_blocks"));
}

#[test]
fn replace_all_checks_the_doc_version_and_bad_ops_are_invalid() {
    let doc = doc_file("ops-version", "# Plan");
    let op = r#"[{"op":"replace_all","markdown":"New.","expect_version":3}]"#;
    assert_eq!(encre(&["ops", "--doc", &doc, "--doc-version", "2"], op).code, 1);
    assert_eq!(encre(&["ops", "--doc", &doc, "--doc-version", "3"], op).code, 0);
    assert_eq!(encre(&["ops", "--doc", &doc], r#"{"op":"read"}"#).code, 1);
    assert_eq!(encre(&["ops", "--doc", &doc], r#"[{"op":"explode"}]"#).code, 1);
}
