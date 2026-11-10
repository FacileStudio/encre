#![cfg(feature = "pdf")]

#[path = "support/typst.rs"]
mod typst_support;

use encre::pdf::{Assets, compile};
use typst_support::{render_vector, typst_vectors};

fn pdf_text(name: &str) -> String {
    let v = typst_vectors().into_iter().find(|v| v.name == name).expect(name);
    let pdf = compile(&render_vector(&v), &Assets::new()).unwrap_or_else(|e| panic!("{}: {e}", v.path.display()));
    pdf_extract::extract_text_from_mem(&pdf).expect("text").chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn the_footer_holds_only_the_text_the_caller_gives() {
    let plain = pdf_text("empty-paragraphs");
    assert!(!plain.contains("confidentiel"), "{plain}");
    assert!(plain.contains("Page1sur1"), "{plain}");
    let footer = pdf_text("footer");
    assert!(footer.contains("FacileDocs·\"public\"page"), "{footer}");
    assert!(!footer.contains("confidentiel"), "{footer}");
}
