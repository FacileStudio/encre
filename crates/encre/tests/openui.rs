#![cfg(feature = "openui")]

use encre::{Node, assign_ids, compact, markdown, openui, validate};
use proptest::prelude::*;
use serde_json::Value;
use std::path::PathBuf;

fn vectors() -> Vec<(PathBuf, Value)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec/vectors/openui");
    let mut files: Vec<PathBuf> =
        std::fs::read_dir(&dir).expect("openui vectors").map(|e| e.expect("entry").path()).collect();
    files.sort();
    files
        .into_iter()
        .map(|p| (p.clone(), serde_json::from_str(&std::fs::read_to_string(&p).expect("file")).expect("JSON")))
        .collect()
}

fn outcome(source: &str) -> Value {
    match openui::to_document(source) {
        Ok(doc) => serde_json::json!({ "want_doc": doc }),
        Err(e) => serde_json::json!({ "want_error": e.to_string() }),
    }
}

fn storable(mut doc: Node) {
    assign_ids(&mut doc);
    validate(&doc).expect("the adapter output validates");
    let again = markdown::parse(&markdown::serialize(&doc)).expect("markdown");
    assert_eq!(compact(&again), compact(&doc.without_ids()), "the output survives the markdown projection");
}

#[test]
fn openui_vectors_hold() {
    let all = vectors();
    assert!(all.len() >= 40, "expected the full openui vector set, found {}", all.len());
    for (path, mut raw) in all {
        let got = outcome(raw["source"].as_str().expect("source"));
        if std::env::var_os("ENCRE_REGEN").is_some() {
            raw.as_object_mut().expect("object").retain(|k, _| k == "name" || k == "source");
            raw.as_object_mut().expect("object").extend(got.as_object().expect("object").clone());
            std::fs::write(&path, format!("{}\n", serde_json::to_string_pretty(&raw).expect("JSON"))).expect("write");
            continue;
        }
        let want = raw.get("want_doc").map_or_else(
            || serde_json::json!({ "want_error": raw["want_error"] }),
            |d| serde_json::json!({ "want_doc": d }),
        );
        assert_eq!(got, want, "{}", path.display());
        if let Some(doc) = raw.get("want_doc") {
            storable(serde_json::from_value(doc.clone()).expect("doc"));
        }
    }
}

fn quoted() -> impl Strategy<Value = String> {
    any::<String>().prop_map(|s| serde_json::to_string(&s).expect("JSON"))
}

fn component() -> impl Strategy<Value = String> {
    let num = any::<i32>().prop_map(|n| (f64::from(n) / 8.0).to_string());
    prop_oneof![
        (quoted(), quoted(), quoted()).prop_map(|(a, b, c)| format!("Stat({a}, {b}, {c})")),
        (quoted(), num.clone()).prop_map(|(a, n)| format!("Stat({a}, {n})")),
        (quoted(), quoted(), any::<bool>()).prop_map(|(a, b, o)| format!("List([{a}, {b}], {o})")),
        (quoted(), prop::sample::select(vec!["info", "danger", "neutral"]), quoted())
            .prop_map(|(t, tone, title)| format!("Callout({t}, \"{tone}\", {title})")),
        (quoted(), quoted(), num.clone()).prop_map(|(c, r, n)| format!("Table([{c}], [[{r}, {n}], [true], []])")),
        (quoted(), quoted(), num)
            .prop_map(|(l, s, n)| format!("Chart(\"line\", [{l}], [{{name: {s}, data: [{n}]}}], {l})")),
        quoted().prop_map(|t| format!("Text({t})")),
        quoted().prop_map(|t| format!("Badge({t})")),
        Just("Divider()".to_owned()),
        quoted().prop_map(|t| format!("Button({t}, \"go\")")),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn any_card_of_components_gives_a_storable_document(
        children in prop::collection::vec(component(), 1..6),
        title in quoted(),
    ) {
        let names: Vec<String> = (0..children.len()).map(|i| format!("c{i}")).collect();
        let defs: Vec<String> = names.iter().zip(&children).map(|(n, c)| format!("{n} = {c}")).collect();
        let source = format!("root = Card([{}], {title}, {title})\n{}", names.join(", "), defs.join("\n"));
        match openui::to_document(&source) {
            Ok(doc) => storable(doc),
            Err(e) => prop_assert_eq!(e, openui::Error::Empty, "{}", source),
        }
    }

    #[test]
    fn any_source_either_fails_or_gives_a_storable_document(
        parts in prop::collection::vec(prop::sample::select(vec![
            "root = ", "Card(", "Stat(", "Table(", "List(", "Callout(", "Chart(", "Text(", "Badge(", "Divider(",
            "Button(", "[", "]", ")", "{name: ", "data: ", "}", ", ", "\n", "a", "a = ", "\"x\"", "\"`\\n\"", "12",
            "-1.5", "true", "null", "\"bar\"", "\"warning\"", "''", "\"\"",
        ]), 0..40)
    ) {
        if let Ok(doc) = openui::to_document(&parts.concat()) {
            storable(doc);
        }
    }
}
