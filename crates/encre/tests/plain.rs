use encre::markdown::{parse, plain_text, serialize_commonmark};

#[test]
fn plain_text_keeps_every_string_and_no_markup() {
    let doc = parse("# Title\n\n**bold** [link](https://x.y)\n\n| a | b |\n| - | - |\n| 1 | 2 |\n\n```stat\n{\"label\": \"Revenue\", \"value\": \"12\", \"delta\": \"+4%\"}\n```").expect("parse");
    assert_eq!(plain_text(&doc), "Title\nbold link\na\tb\n1\t2\nRevenue 12 +4%");
}

#[test]
fn commonmark_export_drops_facile_only_syntax() {
    let doc = parse("a\\\n\\\n\n\\\n\n# x<br>y").expect("parse");
    assert_eq!(serialize_commonmark(&doc), "a\n\n# x y");
}
