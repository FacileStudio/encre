#[path = "support/scratch.rs"]
mod scratch;
mod support;

use support::encre;

#[test]
fn version_prints_one_line() {
    let run = encre(&["--version"], "");
    assert_eq!(run.code, 0);
    assert_eq!(run.text(), format!("encre {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_shows_examples_and_exit_codes() {
    let run = encre(&["--help"], "");
    assert_eq!(run.code, 0);
    assert!(run.text().contains("Examples:"));
    assert!(run.text().contains("Exit codes: 0 ok, 1 invalid input or document, 2 usage, 3 internal error."));
}

#[test]
fn usage_errors_exit_two_and_speak_json_when_asked() {
    assert_eq!(encre(&["bogus"], "").code, 2);
    assert_eq!(encre(&["commonmark", "--from", "yaml"], "").code, 2);
    assert_eq!(encre(&["parse", "--input", "/nonexistent/encre-input.md"], "").code, 2);
    let run = encre(&["bogus", "--json"], "");
    assert_eq!(run.code, 2);
    let error: serde_json::Value = serde_json::from_str(&run.stderr).expect("JSON on stderr");
    assert_eq!((error["error"]["code"].as_str(), error["error"]["exit"].as_u64()), (Some("usage"), Some(2)));
}

#[test]
fn invalid_input_exits_one() {
    assert_eq!(encre(&["serialize"], "{\"type\":\"nope\"}").code, 1);
    let run = encre(&["parse", "--max-bytes", "10", "--json"], "# far more than ten bytes");
    assert_eq!(run.code, 1);
    assert!(run.stderr.contains("--max-bytes"), "{}", run.stderr);
    assert!(run.text().is_empty());
}

#[test]
fn an_unwritable_output_is_internal_and_exits_three() {
    let run = encre(&["parse", "--output", "/nonexistent/dir/out.json"], "# a");
    assert_eq!(run.code, 3, "{}", run.stderr);
    assert!(run.stderr.starts_with("✗ cannot write"), "{}", run.stderr);
}

#[test]
fn output_goes_to_the_named_file() {
    let path = scratch::scratch("output").join("out.json");
    let run = encre(&["parse", "--output", path.to_str().expect("path")], "# a");
    let written = std::fs::read_to_string(&path).expect("output file");
    assert_eq!((run.code, run.text().as_str()), (0, ""));
    assert!(written.ends_with("\n") && written.contains("\"heading\""));
    assert_eq!(encre(&["parse"], "# a").json(), serde_json::from_str::<serde_json::Value>(&written).expect("JSON"));
}
