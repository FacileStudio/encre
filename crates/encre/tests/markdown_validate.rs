use encre::markdown::{SourceError, parse, validate};

fn refused(markdown: &str) -> SourceError {
    validate(markdown).expect_err(markdown)
}

#[test]
fn an_error_names_the_line_and_column_of_its_block() {
    let error = refused("# Title\n\nfine\n\n- item\n  - nested \u{1} text\n");
    assert_eq!((error.line, error.column), (6, 5), "{error}");
    assert_eq!(error.path, "doc.content[2].content[0].content[1].content[0].content[0]");
    assert!(error.to_string().starts_with("6:5: doc.content[2]"), "{error}");
}

#[test]
fn positions_follow_nested_custom_blocks() {
    let error = refused("intro\n\n> quote\n>\n> ````callout\n> ok\n>\n> bad \u{7} char\n> ````\n");
    assert_eq!((error.line, error.column), (8, 3), "{error}");
    let error = refused("````columns\n```column\nok\n```\n\n```column\n| a |\n| - |\n| `x\\\\|y` |\n```\n````");
    assert_eq!((error.line, error.column), (7, 1), "{error}");
    assert!(error.message.contains("backslash before a pipe"), "{error}");
}

#[test]
fn what_markdown_validation_refuses() {
    assert!(refused("a \u{1b}[31m red").message.contains("forbidden character"));
    assert!(refused("nul \u{0} char").message.contains("forbidden character"));
    assert!(refused("```\nbell \u{7}\n```").message.contains("forbidden character"));
    assert!(refused("| h |\n| --- |\n| `a\\\\|b` |").message.contains("backslash before a pipe"));
    assert!(refused("see [`a]:b`](https://x.y)").message.contains("`]:`"));
}

#[test]
fn what_markdown_validation_normalizes() {
    let pipe = validate("| h |\n| --- |\n| `a\\|b` |").expect("an escaped pipe in a cell is a pipe");
    let code = &pipe.content[0].content[1].content[0].content[0].content[0];
    assert_eq!(code.text.as_deref(), Some("a|b"));
    let definition = validate("[`a]:b`](https://x.y)").expect("a reference definition");
    assert_eq!(definition, parse("").expect("empty"));
    for markdown in ["tab\there", "crlf\r\nline", "[badge:info \u{2003}x]", "==a=="] {
        assert!(validate(markdown).is_ok(), "{markdown:?}");
    }
}
