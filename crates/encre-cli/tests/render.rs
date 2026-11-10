#[path = "support/scratch.rs"]
mod scratch;
mod support;

use support::encre;

const SVG: &str =
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20"/></svg>"#;
const DOC: &str = "# Report\n\n![inside](a.svg)\n\n![outside](../secret.svg)\n\n![absolute](/etc/hostname)";

fn image_dir(name: &str) -> String {
    let root = scratch::scratch(name);
    std::fs::create_dir_all(root.join("images")).expect("images dir");
    std::fs::write(root.join("images/a.svg"), SVG).expect("svg");
    std::fs::write(root.join("secret.svg"), SVG).expect("svg");
    root.join("images").to_str().expect("path").to_owned()
}

#[test]
fn typst_takes_title_and_footer() {
    let run = encre(&["typst", "--title", "Q3 \"plan\"", "--footer", "Facile Studio"], "# Hi");
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.text().contains(r#"title: "Q3 \"plan\"""#), "{}", run.text());
    assert!(run.text().contains(r#"footer: "Facile Studio""#));
    assert!(encre(&["typst", "--json"], "# Hi").json()["typst"].is_string());
}

#[test]
fn images_resolve_only_inside_the_image_dir() {
    let dir = image_dir("render-typst");
    let run = encre(&["typst", "--image-dir", &dir], DOC);
    assert_eq!(run.code, 0, "{}", run.stderr);
    let source = run.text();
    assert!(source.contains(r#"image("/images/a.svg", alt: "inside")"#), "{source}");
    assert!(!source.contains("secret") && !source.contains("hostname"), "{source}");
    assert_eq!(source.matches("image(").count(), 1);
    assert_eq!(encre(&["typst", "--image-dir", "/nonexistent/images"], DOC).code, 2);
}

#[test]
fn pdf_embeds_images_and_writes_pdf_bytes() {
    let dir = image_dir("render-pdf");
    let run = encre(&["pdf", "--image-dir", &dir, "--title", "Report"], DOC);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout.starts_with(b"%PDF-"));
    assert!(run.stderr.is_empty());
    let without = encre(&["pdf", "--title", "Report"], DOC);
    assert_eq!(without.code, 0, "{}", without.stderr);
    assert!(run.stdout.len() > without.stdout.len(), "the embedded image adds bytes");
}

#[test]
fn pdf_json_needs_an_output_file_and_summarises_it() {
    assert_eq!(encre(&["pdf", "--json"], "# Hi").code, 2);
    let out = scratch::scratch("render-pdf-json").join("out.pdf");
    let run = encre(&["pdf", "--json", "--output", out.to_str().expect("path")], "# Hi");
    assert_eq!(run.code, 0, "{}", run.stderr);
    let size = std::fs::metadata(&out).expect("pdf file").len();
    assert_eq!(run.json()["bytes"], size);
}
