use crate::model::{Mark, Node};
use crate::schema::{AttrSpec, schema};
use serde_json::{Map, Value};

/// `raw` restricted to the attributes `specs` declares, without nulls and defaults.
pub(crate) fn compact_attrs(specs: &[AttrSpec], raw: &Map<String, Value>) -> Map<String, Value> {
    specs
        .iter()
        .filter_map(|spec| {
            raw.get(&spec.name).filter(|v| !v.is_null() && **v != spec.default).map(|v| (spec.name.clone(), v.clone()))
        })
        .collect()
}

fn compact_mark(mark: &Mark) -> Mark {
    let specs = schema().mark(&mark.kind).map(|t| t.attrs.as_slice()).unwrap_or_default();
    Mark::new(&mark.kind, compact_attrs(specs, &mark.attrs))
}

/// The canonical stored form of `doc`: attributes outside the schema, null or equal to their default are
/// left out, and marks follow rank order. [`crate::yjs::read`] and the TypeScript `compactDocument` give this form.
pub fn compact(doc: &Node) -> Node {
    let mut out = doc.clone();
    let specs = schema().node(&doc.kind).map(|t| t.attrs.as_slice()).unwrap_or_default();
    out.attrs = compact_attrs(specs, &doc.attrs);
    out.marks = doc.marks.iter().map(compact_mark).collect();
    out.marks.sort_by_key(|m| schema().mark_rank(&m.kind));
    out.content = doc.content.iter().map(compact).collect();
    out
}
