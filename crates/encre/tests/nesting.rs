//! Hostile nesting for the Yjs and OpenUI readers: each refuses past its depth limit on a 2 MiB stack, the
//! size of a tokio worker, instead of overflowing it.

#[cfg(feature = "yjs")]
fn nested_update(depth: usize) -> Vec<u8> {
    use yrs::{ReadTxn, StateVector, Transact, XmlElementPrelim, XmlFragment};
    let doc = encre::yjs::new_doc();
    let fragment = doc.get_or_insert_xml_fragment(encre::yjs::FRAGMENT);
    let mut txn = doc.transact_mut();
    let mut parent = fragment.insert(&mut txn, 0, XmlElementPrelim::empty("blockquote"));
    for _ in 1..depth {
        parent = parent.insert(&mut txn, 0, XmlElementPrelim::empty("blockquote"));
    }
    txn.encode_state_as_update_v1(&StateVector::default())
}

#[cfg(feature = "yjs")]
#[test]
fn deep_yjs_updates_are_refused_on_a_small_stack() {
    let run = || {
        assert!(encre::yjs::from_update(&nested_update(47)).is_ok());
        for depth in [48, 10_000] {
            let error = encre::yjs::from_update(&nested_update(depth)).expect_err("too deep").to_string();
            assert!(error.contains("nests more than 48 levels deep"), "{depth}: {error}");
        }
    };
    let worker = std::thread::Builder::new().stack_size(2 << 20).spawn(run).expect("thread");
    worker.join().expect("refused without overflowing the stack");
}

#[cfg(feature = "openui")]
fn chain(links: usize, wrap: bool) -> String {
    let link = |n: usize| if wrap { format!("[x{n}]") } else { format!("x{n}") };
    let defs: String = (0..links).map(|i| format!("x{i} = {}\n", link(i + 1))).collect();
    format!("root = Card([x0])\n{defs}x{links} = Text(\"a\")\n")
}

#[cfg(feature = "openui")]
#[test]
fn openui_reference_chains_obey_the_nesting_limit() {
    let run = || {
        encre::openui::to_document(&chain(8, false)).expect("a short chain resolves");
        for (links, wrap) in [(40, true), (50_000, true), (70, false), (50_000, false)] {
            let error = encre::openui::to_document(&chain(links, wrap)).expect_err("too deep").to_string();
            assert!(error.contains("nest deeper than 64 levels"), "{links}: {error}");
        }
    };
    let worker = std::thread::Builder::new().stack_size(2 << 20).spawn(run).expect("thread");
    worker.join().expect("refused without overflowing the stack");
}
