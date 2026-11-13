mod support;

use serde_json::json;
use support::encre;

#[test]
fn input_that_only_looks_like_json_needs_from() {
    let run = encre(&["validate", "--json"], "{not json} but a paragraph");
    assert_eq!(run.code, 2);
    assert!(run.stderr.contains("--from"), "{}", run.stderr);
    assert_eq!(encre(&["validate", "--from", "markdown"], "{not json} but a paragraph").code, 0);
}

fn ladder(depth: usize) -> String {
    let open: String = (0..depth).rev().map(|d| format!("{}callout\n", "`".repeat(d + 3))).collect();
    let close: String = (0..depth).map(|d| format!("\n{}", "`".repeat(d + 3))).collect();
    format!("# Notes\n\n{open}x{close}")
}

#[test]
fn the_wasm_validate_markdown_matches_the_cli_on_every_input() {
    let inputs = [
        "# Fine".to_owned(),
        "plain *markdown*".to_owned(),
        "{not json} but a paragraph".to_owned(),
        "# Title\n\n- ok\n- bell \u{7} here\n".to_owned(),
        "x\n\na \u{1} b".to_owned(),
        "# Hi\n\na\u{1}b\n".to_owned(),
        ">".repeat(100),
        ladder(8),
        ladder(9),
    ];
    for input in &inputs {
        let cli = encre(&["validate", "--from", "markdown", "--json"], input).json();
        let wasm: serde_json::Value = serde_json::from_str(&encre_wasm::validate_markdown(input)).expect("JSON");
        assert_eq!(wasm, cli, "{input:?}");
    }
    let nine = encre(&["validate", "--from", "markdown", "--json"], &ladder(9)).json();
    assert_eq!((nine["errors"][0]["line"].clone(), nine["errors"][0]["column"].clone()), (json!(11), json!(1)));
}
