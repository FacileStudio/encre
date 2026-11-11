#[path = "support/typst.rs"]
mod typst_support;

use encre::typst::{RenderOptions, escape_string, render};
use typst_support::{render_vector, typst_vectors};

#[test]
fn typst_golden_vectors_hold() {
    let all = typst_vectors();
    assert!(all.len() >= 4, "expected the full typst vector set, found {}", all.len());
    for v in all {
        let got = render_vector(&v);
        if std::env::var_os("ENCRE_REGEN").is_some() {
            let mut raw = v.raw.clone();
            raw["want_typst"] = got.clone().into();
            std::fs::write(&v.path, format!("{}\n", serde_json::to_string_pretty(&raw).expect("JSON"))).expect("write");
            continue;
        }
        assert_eq!(got, v.raw["want_typst"].as_str().expect("want_typst"), "{}: typst", v.name);
    }
}

#[test]
fn escape_matches_ardoise() {
    let cases = [
        ("Acme \"Corp\"", "Acme \\\"Corp\\\""),
        ("back\\slash", "back\\\\slash"),
        ("ends with backslash\\", "ends with backslash\\\\"),
        ("two\nlines\tand tab", "two\\nlines\\tand tab"),
        ("crlf\r\nline", "crlf\\nline"),
        ("lone\rreturn", "lone\\rreturn"),
        ("bell\u{7}del\u{7f}nul\u{0}", "belldelnul"),
        ("plain", "plain"),
    ];
    for (input, want) in cases {
        assert_eq!(escape_string(input), want, "{input:?}");
    }
}

#[test]
fn only_followable_links_reach_the_pdf() {
    let markdown =
        "[a](javascript:alert(1)) [b](data:text/html,x) [c](https://x.example) [d](/rel) [e](mailto:a@b.example)";
    let typst = render(&encre::markdown::parse(markdown).expect("parses"), &RenderOptions::default());
    assert!(!typst.contains("javascript") && !typst.contains("data:"), "{typst}");
    for href in ["https://x.example", "/rel", "mailto:a@b.example"] {
        assert!(typst.contains(&format!("#link(\"{href}\")")), "{href}: {typst}");
    }
}
