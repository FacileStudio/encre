#![cfg(feature = "yjs")]

use encre::markdown::{parse, serialize};
use encre::yjs::{from_update, to_update};
use encre::{Node, compact, validate};
use serde_json::Value;
use std::path::PathBuf;

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec/vectors/yjs")
}

fn vectors() -> Vec<(String, Vec<u8>, Value)> {
    let mut names: Vec<String> = std::fs::read_dir(dir())
        .expect("vector directory")
        .filter_map(|e| e.expect("entry").file_name().into_string().ok())
        .filter_map(|name| name.strip_suffix(".json").map(str::to_owned))
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let bin = std::fs::read(dir().join(format!("{name}.bin"))).expect("update bytes");
            let json = std::fs::read_to_string(dir().join(format!("{name}.json"))).expect("vector JSON");
            (name, bin, serde_json::from_str(&json).expect("vector JSON"))
        })
        .collect()
}

#[test]
fn rust_reads_what_tiptap_wrote() {
    let all = vectors();
    assert!(all.len() >= 6, "expected the full yjs vector set, found {}", all.len());
    for (name, bin, vector) in all {
        let want: Node = serde_json::from_value(vector["doc"].clone()).expect("doc");
        let got = from_update(&bin).expect("from_update");
        assert_eq!(got, want, "{name}: from_update");
        assert_eq!(validate(&got), Ok(()), "{name}: valid");
        assert_eq!(serialize(&got), vector["markdown"].as_str().expect("markdown"), "{name}: markdown");
        assert_eq!(
            compact(&parse(&serialize(&got)).expect("parse")),
            want.without_ids(),
            "{name}: markdown round trip"
        );
    }
}

#[test]
fn rust_written_updates_read_back_and_match_the_committed_ones() {
    for (name, _, vector) in vectors() {
        let want: Node = serde_json::from_value(vector["doc"].clone()).expect("doc");
        let fresh = to_update(&want).expect("to_update");
        assert_eq!(from_update(&fresh).expect("from_update"), want, "{name}: fresh update");
        let path = dir().join(format!("{name}.rust.bin"));
        if std::env::var_os("ENCRE_REGEN").is_some() {
            std::fs::write(&path, &fresh).expect("write rust update");
        }
        let committed = std::fs::read(&path).expect("committed rust update: run with ENCRE_REGEN=1");
        assert_eq!(from_update(&committed).expect("from_update"), want, "{name}: committed update");
    }
}

#[test]
fn bad_bytes_are_an_error() {
    assert!(from_update(&[0xff, 0xff, 0xff]).is_err());
}
