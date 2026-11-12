# encre — Architecture

How a document moves between its stored, collaborative, markdown, Typst and OpenUI forms, and
which code owns each conversion.

## The model

```
                        encre schema v2 (spec/schema.json)
                                     │
   ┌───────────────┬─────────────────┼─────────────────┬──────────────┬──────────────┐
 Tiptap          Yjs XmlFragment   Facile Markdown    Typst / PDF    Agent ops      OpenUI
 extensions      "default"         (lossless) +       muse-typst,    guarded,       adapter
 Svelte, TS      yrs in Rust,      CommonMark export  in-memory      ops::apply,    one way in,
 (src/lib)       y-tiptap in TS    (markdown)         compiler       ops::apply_yjs (openui)
                 (yjs)                                (typst, pdf)
                                     │
                     encre-wasm: the same Rust in the browser
```

- The **stored form** is ProseMirror JSON that passes `validate`. A store keeps the schema
  version beside it and calls `migrate` on read when the version is older.
- The **collaborative form** is a Yjs `XmlFragment` named `default`, in the layout Tiptap's
  Collaboration extension writes through `@tiptap/y-tiptap`.
- **Markdown is a projection**, never the truth. Facile Markdown is lossless for every node and
  mark; CommonMark is a second, lossy export.
- **One implementation per conversion, in Rust.** `crates/encre-wasm` compiles the markdown
  projection, validation and the OpenUI adapter to WebAssembly, and `src/lib/wasm.ts` loads it
  once. `@tiptap/markdown` is not used: it has an open round-trip bug
  (https://github.com/ueberdosis/tiptap/issues/8519).

## Components

| Path | Role |
|---|---|
| `spec/schema.json` | Nodes, marks, attributes and the id alphabet; both languages load it |
| `crates/encre/src/model.rs`, `validate/`, `ids.rs`, `compact.rs`, `migrate.rs` | The tree, its rules, block ids, the compact form, upgrades |
| `crates/encre/src/markdown/` | Facile Markdown reader (pulldown-cmark 0.13) and canonical writer |
| `crates/encre/src/yjs/` | Reads and writes the y-tiptap layout with yrs (feature `yjs`) |
| `crates/encre/src/ops/` | Agent operations on a stored tree or an open Yjs fragment |
| `crates/encre/src/typst/`, `escape.rs`, `pdf/` | Typst source, string escaping, the in-memory compiler (feature `pdf`) |
| `crates/encre/src/openui/` | OpenUI Lang lexer, parser, reference resolver and block mapping (feature `openui`) |
| `crates/encre-cli/` | The `encre` binary: every conversion over stdin and stdout, for subprocess callers |
| `crates/encre/typst/` | Vendored muse-typst (`SOURCE` names the commit) and the embedded fonts (`FONTS.md`) |
| `src/lib/extensions/` | One Tiptap extension per custom node, StarterKit set-up, collaboration |
| `src/lib/nodes/`, `Document.svelte` | Node views and the read-only renderer |
| `src/lib/Editor.svelte` | The editor with toolbar, bubble menu, slash menu and optional collaboration |

## Conversions

| From | To | Rust | Browser |
|---|---|---|---|
| Facile Markdown | document | `markdown::parse` | `parseMarkdown` |
| document | Facile Markdown | `markdown::serialize` | `serializeMarkdown`, `Editor.exportMarkdown` |
| document | CommonMark | `markdown::serialize_commonmark` | `serializeCommonMark` |
| document | plain text | `markdown::plain_text` | |
| Yjs update | document | `yjs::from_update`, `yjs::read` | Tiptap Collaboration |
| document | Yjs | `yjs::to_update`, `yjs::write` | Tiptap Collaboration |
| any document | compact form | `compact` | `compactDocument` |
| stored version n | current version | `migrate` | |
| document | Typst source | `typst::render` | |
| Typst source | PDF bytes | `pdf::compile` | |
| OpenUI Lang | document | `openui::to_document`, `openui::to_blocks` | `openuiToDocument` |

Readers return documents without ids. Server-side writes call `assign_ids` before they store; the
editor gives each new block an id through Tiptap's UniqueID extension.

## Yjs layout

The facts a second implementation of the layout must copy. `spec/FORMAT.md` section 1.6 is the
normative statement.

- Tiptap 3 collaborates through `@tiptap/y-tiptap`, a fork of y-prosemirror. encre pins it at
  3.3.0 and pins `@tiptap/extension-collaboration` exactly; regenerate the yjs vectors on any bump.
- Every node except `text` is an `XmlElement` named after its type, with defaults filled in and
  nulls left out. A run of adjacent text nodes is one `XmlText`, each mark a formatting attribute.
- The `code` mark has `excludes: ''`, so y-tiptap names its attribute `code--LpaW+ak5`: the
  type, `--`, and the folded SHA-256 of `{"type":"code"}`. The Rust writer uses that exact key
  (`CODE_KEY` in `crates/encre/src/yjs/attrs.rs`); a reader strips `--` plus eight characters.
- `yjs::new_doc` sets `OffsetKind::Utf16`, so text offsets match JavaScript clients.
- `ops::apply_yjs` inserts, removes or replaces single top-level elements inside the caller's
  transaction, so other people's carets survive an agent edit. Only `replace_all` rewrites all.
- y-websocket 3 emits `sync`, never `synced`, while UniqueID waits for `synced`. The wrapper in
  `src/lib/extensions/collaboration.ts` translates, and fires at once when already synced.

## Agent operations

`ops::Op` is the JSON agents send (`{"op": "insert_after", "block_id": ..., "markdown": ...}`).
`ops::apply` plans the whole edit, validates the result, and only then changes the tree. Failures
are `OpError` values whose message says what matched and how to narrow the request. The rules
are `spec/FORMAT.md` section 4; the agent-facing summary is [agents.md](agents.md).

## Typst and PDF

`typst::render` writes a complete Typst file that imports `/muse-typst/lib.typ` and wraps the
body in `muse-doc` with the `RenderOptions` title, subtitle, `doc_type`, cover and footer. Every
string of the document reaches Typst as a string literal escaped by `escape_string`, a port of
Ardoise's `escapeTypstString`, so no document can inject Typst markup or code.

`pdf::compile` compiles that source with the `typst` crate (0.15.1), in memory. The compiler sees the vendored muse-typst, the fonts in
`crates/encre/typst/fonts`, and the caller's `Assets`; no system fonts, no packages, no
filesystem. The same source and assets always give the same bytes.

| Node | Typst |
|---|---|
| `heading` | `heading(level: n)` |
| `paragraph` | the inline content; an empty paragraph writes nothing |
| `bulletList`, `orderedList` | `list`, `enum(start: n)` |
| `taskList` | `list(marker: [])` with ☐ or ☑ before each item |
| `blockquote` | `quote(block: true)` |
| `codeBlock` | `raw(block: true, lang: ...)` |
| `horizontalRule` | muse-typst `divider()` |
| `image` | `figure(image(path, alt: ...), caption: title)` when the resolver answers a path; otherwise the alt text in emphasis |
| `table` | `table` with `table.header` and the header row's alignments |
| `callout` | muse-typst `alert(tone:, title:)` |
| `stat` | muse-typst `metric(value, label, sub: ...)`, `sub` being `delta` and `sub` joined by ` · ` |
| `chart` | a `figure` holding a table, one column per label and one row per series, captioned with the title; muse-typst has no chart component, so `kind` is not drawn |
| `diagram` | a block holding the language as a small label and the source as `raw(block: true)`; encre never draws it |
| `columns` | `grid(columns: n)`; a toned column is an `alert(tone:, title:)`, a neutral one its title in `strong` then its blocks |
| `badge` | muse-typst `badge(text, tone:)`; with `dot`, `status-dot(tone:)` then the text |
| marks | `strong`, `emph`, `strike`, `highlight`, `raw`, `link(href)`; a link title is not shown |
| `hardBreak` | `linebreak()` |

The body sets `font: ("Satoshi", "DejaVu Sans")` and raw text `("Menlo", "DejaVu Sans Mono")`.
A machine with muse-typst's fonts uses them; the embedded compiler falls back to DejaVu, because
Satoshi and Menlo do not allow redistribution (`crates/encre/typst/FONTS.md`). `muse-doc` gets
`footer: none` unless `RenderOptions.footer` is set, so a page footer holds only the page number.

## OpenUI adapter

Bulle's assistant answers with OpenUI Lang programs (`Bulle/apps/client/src/lib/openui`).
`openui::to_document` turns one into a document for "save as document". Nothing stores OpenUI.

- A program is `identifier = Expression` statements; a statement continues across line feeds
  inside brackets. Strings are double-quoted JSON or single-quoted with `\n`, `\t` escapes.
- `root` is the entry point. References may be used before their definition, a later statement
  of the same name replaces an earlier one, and statements `root` never reaches are dropped.
- Cycles, undefined references, unknown components, named arguments, wrong argument types and
  programs past 100 000 values or 64 nesting levels are errors. A program that shows only
  buttons and forms is an error, because the document would be empty.

| Component | Blocks |
|---|---|
| `Card(children, title?, description?)` | `heading` of the title (level 1, a nested card one deeper), the description as paragraphs, then the children |
| `Stat(label, value, delta?)` | `stat` |
| `Table(columns, rows)` | `table` with the columns as header row; short rows padded with empty cells |
| `List(items, ordered?)` | `bulletList`, or `orderedList` when `ordered` |
| `Callout(text, tone?, title?)` | `callout` holding the text's paragraphs |
| `Chart(kind, labels, series, title?)` | `chart` |
| `Text(content, tone?)` | one paragraph per blank-line-separated run, line feeds as hard breaks; tone dropped |
| `Badge(text, tone?)` | a paragraph of bold text; tone dropped |
| `Divider()` | `horizontalRule` |
| `Button`, `Form`, `Field` | dropped |

Strings that land in single-line attributes lose line breaks and control characters, and a
callout title's backticks become apostrophes, so the output always validates.

Back to the [documentation index](README.md).
