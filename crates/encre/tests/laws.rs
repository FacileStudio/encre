mod support;

use encre::markdown::{parse, serialize, serialize_commonmark};
use proptest::prelude::*;
use support::document;
use support::text::markdown_text;

fn config() -> ProptestConfig {
    let cases = std::env::var("PROPTEST_CASES").ok().and_then(|n| n.parse().ok()).unwrap_or(10_000);
    ProptestConfig { cases, failure_persistence: None, ..ProptestConfig::default() }
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn parse_inverts_serialize(doc in document()) {
        prop_assert_eq!(encre::validate(&doc), Ok(()));
        let written = serialize(&doc);
        let back = parse(&written).expect("parse never fails");
        prop_assert_eq!(back.without_ids(), doc.without_ids(), "markdown:\n{}", written);
    }

    #[test]
    fn serialize_parse_is_a_fixed_point(md in markdown_text()) {
        let doc = parse(&md).expect("parse never fails");
        prop_assert_eq!(encre::validate(&doc), Ok(()));
        let once = serialize(&doc);
        let twice = serialize(&parse(&once).expect("parse never fails"));
        prop_assert_eq!(&twice, &once, "input:\n{}", md);
    }

    #[test]
    fn commonmark_export_parses(doc in document()) {
        let written = serialize_commonmark(&doc);
        let fenced = commonmark_fences(&written);
        for block in doc.content.iter().filter(|b| ["callout", "stat", "chart"].contains(&b.kind.as_str())) {
            prop_assert!(fenced.iter().any(|info| info.starts_with(block.kind.as_str())), "{} is not fenced in:\n{}", block.kind, written);
        }
    }
}

fn commonmark_fences(markdown: &str) -> Vec<String> {
    let arena = comrak::Arena::new();
    let root = comrak::parse_document(&arena, markdown, &comrak::Options::default());
    let mut html = String::new();
    comrak::format_html(root, &comrak::Options::default(), &mut html).expect("comrak renders what it parsed");
    root.descendants()
        .filter_map(|node| match &node.data.borrow().value {
            comrak::nodes::NodeValue::CodeBlock(code) if code.fenced => Some(code.info.clone()),
            _ => None,
        })
        .collect()
}
