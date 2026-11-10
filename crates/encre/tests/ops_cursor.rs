#![cfg(feature = "yjs")]

use encre::ops::{Op, apply_yjs};
use encre::yjs::{FRAGMENT, new_doc, read};
use std::path::PathBuf;
use yrs::branch::Branch;
use yrs::types::xml::XmlOut;
use yrs::updates::decoder::Decode;
use yrs::{Assoc, BranchID, Doc, IndexedSequence, ReadTxn, StickyIndex, Transact, Update, XmlFragment};

fn tiptap_update() -> Vec<u8> {
    std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec/vectors/yjs/inline.bin")).expect("vector")
}

fn sync(from: &Doc, to: &Doc) {
    let diff = from.transact().encode_diff_v1(&to.transact().state_vector());
    to.transact_mut().apply_update(Update::decode_v1(&diff).expect("decode")).expect("apply");
}

fn element_ids(doc: &Doc) -> Vec<BranchID> {
    let fragment = doc.get_or_insert_xml_fragment(FRAGMENT);
    let txn = doc.transact();
    fragment
        .children(&txn)
        .map(|child| match child {
            XmlOut::Element(element) => AsRef::<Branch>::as_ref(&element).id(),
            other => panic!("unexpected top-level node {other:?}"),
        })
        .collect()
}

fn insert_after_first(agent: &Doc) {
    let fragment = agent.get_or_insert_xml_fragment(FRAGMENT);
    let first = read(&agent.transact(), &fragment).expect("read").content[0].id().expect("id").to_owned();
    let op = Op::InsertAfter { block_id: first, markdown: "Inserted by an agent.".to_owned() };
    apply_yjs(&mut agent.transact_mut(), &fragment, 1, &op).expect("insert_after");
}

#[test]
fn insert_after_keeps_a_concurrent_cursor_and_every_other_element() {
    let browser = Doc::new();
    browser.transact_mut().apply_update(Update::decode_v1(&tiptap_update()).expect("decode")).expect("apply");
    let agent = new_doc();
    sync(&browser, &agent);

    let fragment = browser.get_or_insert_xml_fragment(FRAGMENT);
    let (paragraph, cursor) = {
        let txn = browser.transact();
        let Some(XmlOut::Element(paragraph)) = fragment.get(&txn, 3) else { panic!("paragraph") };
        let Some(XmlOut::Text(text)) = paragraph.first_child() else { panic!("text") };
        let cursor = text.sticky_index(&txn, 3, Assoc::After).expect("cursor");
        (AsRef::<Branch>::as_ref(&paragraph).id(), (cursor, text))
    };
    let before = element_ids(&browser);

    insert_after_first(&agent);
    sync(&agent, &browser);

    let after = element_ids(&browser);
    assert_eq!(after.len(), before.len() + 1);
    assert_eq!([&after[..1], &after[2..]].concat(), before, "every existing element survives the insert");
    assert_eq!(after[4], paragraph);
    let (cursor, text): (StickyIndex, _) = cursor;
    let txn = browser.transact();
    let offset = cursor.get_offset(&txn).expect("cursor resolves");
    assert_eq!(offset.index, 3, "the cursor stays at the same character");
    assert_eq!(
        offset.branch,
        yrs::branch::BranchPtr::from(AsRef::<Branch>::as_ref(&text)),
        "the cursor stays in the same text"
    );
    let doc = read(&txn, &fragment).expect("read");
    assert_eq!(encre::markdown::plain_text(&doc).lines().nth(1), Some("Inserted by an agent."));
}

#[test]
fn replace_all_is_the_only_op_that_rewrites_every_element() {
    let doc = new_doc();
    doc.transact_mut().apply_update(Update::decode_v1(&tiptap_update()).expect("decode")).expect("apply");
    let fragment = doc.get_or_insert_xml_fragment(FRAGMENT);
    let before = element_ids(&doc);
    let op = Op::ReplaceAll { markdown: "New.".to_owned(), expect_version: 7 };
    apply_yjs(&mut doc.transact_mut(), &fragment, 7, &op).expect("replace_all");
    let after = element_ids(&doc);
    assert_eq!(after.len(), 1);
    assert!(!before.contains(&after[0]));
}
