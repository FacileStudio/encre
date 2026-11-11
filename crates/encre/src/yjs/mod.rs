//! The collaborative form: a Yjs `XmlFragment` in the layout y-prosemirror (`@tiptap/y-tiptap`) writes.
//!
//! Each node is an `XmlElement` named after its type, attributes as element attributes (nulls left out).
//! A run of text nodes is one `XmlText` whose formatting attributes are the marks. Overlapping marks
//! (`code`, whose `excludes` is empty) carry a hash suffix, `code--LpaW+ak5`, as y-prosemirror names them.

mod attrs;
mod read;
mod write;

use crate::model::Node;
use crate::validate::{ValidationError, validate};
use yrs::updates::decoder::Decode;
use yrs::{
    Doc, OffsetKind, Options, ReadTxn, StateVector, Transact, TransactionMut, Update, XmlFragment, XmlFragmentRef,
};

pub(crate) use write::insert_blocks;

/// The fragment name Tiptap's Collaboration extension uses.
pub const FRAGMENT: &str = "default";

/// Why a Yjs document could not be read or written.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum YjsError {
    /// The bytes are not a Yjs v1 update.
    #[error("the update does not decode: {0}")]
    Decode(String),
    /// The update decodes but does not apply.
    #[error("the update does not apply: {0}")]
    Apply(String),
    /// An element or mark the schema does not know.
    #[error("the fragment holds an unknown {0} `{1}`")]
    Unknown(&'static str, String),
    /// A shape y-prosemirror never writes: a nested fragment, or an embed inside text.
    #[error("the fragment holds {0}, which the schema has no place for")]
    Shape(&'static str),
    /// The document to write does not validate.
    #[error("the document is invalid: {0}")]
    Invalid(#[from] ValidationError),
}

/// A Yjs document with the UTF-16 offsets JavaScript clients use.
pub fn new_doc() -> Doc {
    Doc::with_options(Options { offset_kind: OffsetKind::Utf16, ..Options::default() })
}

/// Reads the document held in `fragment`, in compact form: attributes equal to their default left out.
pub fn read<T: ReadTxn>(txn: &T, fragment: &XmlFragmentRef) -> Result<Node, YjsError> {
    Ok(Node::new("doc", serde_json::Map::new(), read::children(txn, fragment, &mut Vec::new())?))
}

/// Replaces the whole content of `fragment` with `doc`, inside the caller's transaction.
pub fn write(txn: &mut TransactionMut, fragment: &XmlFragmentRef, doc: &Node) -> Result<(), YjsError> {
    validate(doc)?;
    let len = fragment.len(txn);
    if len > 0 {
        fragment.remove_range(txn, 0, len);
    }
    insert_blocks(txn, fragment, 0, &doc.content);
    Ok(())
}

/// Reads the document from a full Yjs v1 update, as a server stores it.
pub fn from_update(update: &[u8]) -> Result<Node, YjsError> {
    let update = Update::decode_v1(update).map_err(|e| YjsError::Decode(e.to_string()))?;
    let doc = new_doc();
    let fragment = doc.get_or_insert_xml_fragment(FRAGMENT);
    let mut txn = doc.transact_mut();
    txn.apply_update(update).map_err(|e| YjsError::Apply(e.to_string()))?;
    read(&txn, &fragment)
}

/// A full Yjs v1 update holding `doc` in the `default` fragment.
pub fn to_update(doc: &Node) -> Result<Vec<u8>, YjsError> {
    let ydoc = new_doc();
    let fragment = ydoc.get_or_insert_xml_fragment(FRAGMENT);
    let mut txn = ydoc.transact_mut();
    write(&mut txn, &fragment, doc)?;
    Ok(txn.encode_state_as_update_v1(&StateVector::default()))
}
