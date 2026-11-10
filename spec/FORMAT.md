# encre format, version 1

This document is normative. `schema.json` is the machine-readable half of section 1. When this file and an implementation disagree, the implementation is wrong. The words MUST, MUST NOT and MAY are used in their RFC 2119 sense.

## 1. Document model

A document is a ProseMirror JSON tree that validates against `spec/schema.json`.

- The **stored form** is that JSON. A store keeps the schema version next to it.
- The **collaborative form** is a Yjs `XmlFragment` named `default`, in the layout `y-prosemirror` writes.
- **Markdown is a projection.** Facile Markdown (section 2) is lossless. CommonMark export (section 3) is lossy.

### 1.1 Nodes and marks

`schema.json` lists every node with `name`, `group`, `content` (a ProseMirror content expression), `marks` (the marks its inline content may carry: `_` all, empty none) and `attrs`. Marks are listed in rank order: the order a text node's `marks` array MUST follow.

| Node | Group | Content | Attributes |
|---|---|---|---|
| `doc` | | `block+` | |
| `paragraph` | block | `inline*` | `id` |
| `heading` | block | `inline*` | `id`, `level` 1, 2 or 3 |
| `bulletList` | block list | `listItem+` | `id` |
| `orderedList` | block list | `listItem+` | `id`, `start` 0 to 999999999, default 1 |
| `taskList` | block list | `taskItem+` | `id` |
| `listItem` | | `paragraph block*` | `id` |
| `taskItem` | | `paragraph block*` | `id`, `checked` |
| `blockquote` | block | `block+` | `id` |
| `codeBlock` | block | `text*`, no marks | `id`, `language` (default empty) |
| `horizontalRule` | block | | `id` |
| `image` | block | | `id`, `src` (required), `alt`, `title` |
| `table` | block | `tableRow+` | `id` |
| `tableRow` | | `(tableHeader \| tableCell)+` | |
| `tableHeader`, `tableCell` | | `paragraph` | `align`: none, left, center or right |
| `callout` | block | `block+` | `id`, `tone`: neutral, info, success, warning or danger, `title` |
| `stat` | block | | `id`, `label`, `value`, `delta` |
| `chart` | block | | `id`, `kind` bar or line, `title`, `labels` (strings), `series` (`{name, data}` objects) |
| `text` | inline | | |
| `hardBreak` | inline | | |

Marks, in rank order: `link` (`href` required, `title`), `bold`, `italic`, `strike`, `code`. `code` combines with every other mark. Underline is not in v1.

### 1.2 Attribute types

| Type | Values |
|---|---|
| `id` | `null`, or a block id (section 1.4) |
| `integer` | a JSON integer, within `values` or `min`..`max` when given |
| `boolean` | `true` or `false` |
| `string` | one of `values` |
| `line` | a string without a line break or control character other than tab |
| `info` | a `line` without tab, leading or trailing space, or two spaces in a row |
| `label` | a `line` without a backtick |
| `lines` | an array of `line` |
| `series` | an array of `{"name": line, "data": [number, ...]}` |

An attribute that is absent takes its default. An attribute whose schema default is `null` and that is marked `required` MUST be present and non-null. A node or mark MUST NOT carry an attribute its type does not declare.

### 1.3 Rules beyond the content expressions

A valid document also follows these rules. They keep the markdown projection lossless.

1. Text is never empty. Text outside `codeBlock` contains no line break and no control character other than tab. Text inside `codeBlock` contains no control character other than tab and line feed.
2. Two adjacent text nodes never carry the same marks: they are one text node.
3. A text node's marks are in rank order, each type at most once.
4. `hardBreak` carries no marks. The editor strips them.
5. Every row of a table has the same number of cells. The first row holds only `tableHeader` cells and the other rows only `tableCell` cells. Every cell of a column has the same `align`.
6. In a table cell, text carrying `code` never holds a backslash directly before a pipe (`\|`): a GFM cell cannot write it.
7. Text carrying both `link` and `code` never holds `]:`: at the start of a paragraph it would read as a link reference definition.

### 1.4 Block ids

Every node whose attributes include `id` is a block. A block id is 10 characters from the lowercase Crockford base32 alphabet `0123456789abcdefghjkmnpqrstvwxyz`. Ids are unique within a document. An id MAY be `null` until assigned: the editor assigns one to each new block, and server-side writes call `assign_ids` before they store a document. Ids never appear in markdown.

### 1.5 Compact form

The compact form of a document leaves out every attribute that its type does not declare, that is null, or that equals its default, and orders marks by rank. Readers that rebuild a document from another representation (the Yjs reader, `compactDocument` in TypeScript) return it, so two implementations compare byte for byte. Validation treats the compact and the filled form alike.

### 1.6 Collaborative layout

The `default` fragment holds what y-prosemirror (`@tiptap/y-tiptap` 3.3.0, the binding Tiptap 3's Collaboration extension uses) writes:

- Every node except `text` is an `XmlElement` named after its type. Its attributes are the node's attributes with defaults filled in; null values are left out.
- A run of adjacent text nodes is one `XmlText`. Each mark is a formatting attribute whose value is the mark's attribute map (`{}` when it has none).
- `code` is the one mark whose `excludes` is empty, so y-prosemirror names its attribute `code--LpaW+ak5`: the type, `--`, and the folded SHA-256 of `{"type":"code"}`. A reader strips any `--` plus eight base64 characters from a mark name. A writer MUST use that exact name, or Tiptap sees two `code` marks.
- `spec/vectors/yjs/<name>.bin` is a v1 update written by a real Tiptap editor (`scripts/gen-yjs-vectors.ts`), `<name>.json` the compact document and its Facile Markdown, and `<name>.rust.bin` the update the Rust writer gives for the same document.

### 1.7 Changing the schema

A change bumps `version` in `schema.json` and adds a function to `crates/encre/src/migrate.rs` that upgrades a stored document from the previous version. No change reinterprets an existing document silently.

## 2. Facile Markdown v1

Facile Markdown is CommonMark 0.31 with the GFM extensions for tables, task list items and strikethrough (`~~a~~`, and `~a~` on input), plus the rules below. Autolinks are the CommonMark ones (`<https://…>`); bare URLs stay text. The reference reader is pulldown-cmark. A parser MUST accept any input and MUST NOT fail on malformed custom blocks: they become code blocks.

### 2.1 Custom blocks

A custom block is a fenced code block opened with backticks whose info string's first word is a custom block name: `callout`, `stat` or `chart`. A tilde fence is always a code block.

- **Attributes** follow the name as `key="value"`, separated by spaces. Double quotes are required. `\"` and `\\` are the only escapes, and the parser reads them from the raw source line, before CommonMark processes the info string. An attribute left out takes its default. A writer omits attributes equal to their default and writes the rest in schema order.
- **A block with children** (`callout`) holds Facile Markdown inside its fence. The fence is one backtick longer than the longest backtick run inside it, and at least three.
- **A block with data only** (`stat`, `chart`) holds one JSON object inside its fence. A writer writes every attribute but `id`, keys in schema order, with `", "` and `": "` separators. A reader accepts keys in any order and fills missing ones with defaults.
- An unknown attribute, a duplicate attribute, an attribute value outside its type, a key outside the schema or JSON that does not parse makes the block a code block whose language is the info string and whose text is the fence content.
- A code block whose language's first word is a custom block name is written with a tilde fence, so it stays a code block.

```````markdown
````callout tone="warning" title="Before you deploy"
Run the gate **first**.

```sh
sh scripts/check.sh
```
````

```stat
{"label": "Revenue", "value": "12 400 €", "delta": "+4%"}
```

```chart
{"kind": "bar", "title": "Signups", "labels": ["Mon", "Tue"], "series": [{"name": "web", "data": [3, 5]}]}
```
```````

### 2.2 Dialect rules

1. **Raw HTML is text.** An HTML block becomes a paragraph holding its lines as text, one hard break between lines. Inline HTML is kept as literal text, with one exception: `<br>`, `<br/>` and `<br />` are a hard break.
2. **Empty paragraph.** A lone backslash alone on the last line of a paragraph is removed. So a paragraph holding only `\` is an empty paragraph, and a paragraph ending in a hard break is written with a final `\` line. A literal backslash is always written `\\`.
3. **Hard breaks** are written as a backslash at the end of the line in paragraphs, and as `<br>` in headings and table cells, where a line ending cannot appear.
4. **Lists.** A GFM list whose items are all task items is a `taskList`. A list mixing task and plain items is split into runs. Adjacent lists are written with alternating markers (`-` then `*`, `1.` then `1)`) so they stay separate.
5. **Images.** A paragraph that holds only images and whitespace becomes image blocks. An image inside other inline content splits the paragraph around it.
6. **Mapped constructs.** Setext headings are headings. Headings of level 4 to 6 become level 3. Indented code is a code block. Reference links and images are resolved. A soft line break becomes a space. A task checkbox in an ordered list is dropped. An empty list item, blockquote or callout gets one empty paragraph. A list item that starts with another block gets an empty first paragraph.
7. Footnotes, math, front matter, GFM alerts and literal autolinks are not recognized.

### 2.3 Canonical serialization

The same tree always gives the same bytes.

- Bullets `-`, ordered markers `1.` counting up from `start`, emphasis `*`, strong `**`, strike `~~`, ATX headings, backtick fences of at least three, horizontal rule `---`.
- One blank line between blocks. Inside a list item, a block follows a paragraph on the next line when it can interrupt a paragraph (a list, a heading, a quote, a fence); otherwise a blank line separates them. A list whose items need such a blank line separates every item with one; other lists are tight.
- Tables always have a header row and a delimiter row, `| a | b |` with one space of padding.
- Code block content is written verbatim. The fence is one character longer than the longest run of its character in the content, and at least three.
- **Escaping.** A writer first escapes only what is likely to change the parse, then parses each inline container on its own and compares. When the parse differs, it writes that container in full form: every ASCII punctuation character backslash-escaped, and a character that would be lost or would break a delimiter written as a decimal character reference (`&#32;`). Finally it parses the whole document; when that differs, it writes every container in full form.

### 2.4 Round-trip laws

Each is a property test over generated input with 10 000 cases.

1. `parse(serialize(doc)) == doc`, ids ignored, for every valid document.
2. `serialize(parse(md))` is a fixed point: applying it twice gives the same bytes as once.
3. `serialize_commonmark(doc)` parses with a CommonMark parser without error.

## 3. CommonMark export

`serialize_commonmark` writes Facile Markdown minus what only Facile Markdown readers understand: empty paragraphs and trailing hard breaks are dropped, hard breaks in headings become a space, and custom blocks stay fenced code blocks, which is how any other reader shows them.

## 4. Agent operations

Agents read markdown and write through operations. Each operation is validated completely before anything changes, and applied as one Yjs transaction when the document is open.

| Operation | Arguments | Effect |
|---|---|---|
| `read` | | The document as Facile Markdown |
| `read_blocks` | `range` (optional, block index start and end) | `[{id, type, markdown}]` for top-level blocks |
| `append` | `markdown` | Adds blocks at the end |
| `prepend` | `markdown` | Adds blocks at the start |
| `insert_after` | `block_id`, `markdown` | Adds blocks after that block |
| `replace_block` | `block_id`, `markdown` | Replaces that one block. Replacing a heading replaces the heading only, never its section |
| `delete_block` | `block_id` | Removes that one block |
| `patch` | `find` (exact markdown text), `replace`, `expect` (default 1) | Replaces text inside blocks. Fails unless exactly `expect` matches exist |
| `replace_all` | `markdown`, `expect_version` | Replaces the whole document only when the document version still matches |

Every failure is a typed error whose message lists what matched and how to narrow it. Example: `patch expected 1 match, found 2: block a1b2c3d4e5 "…", block f6g7h8j9k0 "…"; add more surrounding text to find`.

### 4.1 Rules

1. Operations address **top-level blocks**. An id that names a nested block is refused with the id of the top-level block that holds it.
2. A refused operation changes nothing. The result of an accepted one MUST validate; otherwise it is refused.
3. `replace_block` replaces one block with one or more blocks. The first new block keeps the old id when it has the same type. Empty markdown is refused: removing is `delete_block`'s job.
4. `patch` matches `find` against each top-level block's own markdown, as `read_blocks` shows it. A match never spans two blocks. Each matching block is replaced as a whole, keeping its id when its type stays; a block whose markdown becomes blank is removed. `find` MUST NOT be empty and `expect` MUST be at least 1.
5. An operation that would leave no block is refused.
6. New blocks get fresh ids. A write answers the ids of the top-level blocks it inserted.
7. On an open document every operation runs in the caller's Yjs transaction and inserts, removes or replaces single top-level elements, so other clients' relative positions stay valid. Only `replace_all` rewrites every element.

### 4.2 Vectors

`spec/vectors/ops/<name>.json` holds `markdown` (or a `doc` with ids), an optional `version` (default 1), the `op` in its JSON form (`{"op": "insert_after", "block_id": …, "markdown": …}`, unknown fields refused), and one of `want_markdown`, `want_blocks` or `want_error` (the exact message). `want_kept` lists ids that MUST survive. When the vector gives `markdown`, the runner gives top-level block *i* the id `b` followed by *i* in nine digits (`b000000000`) and leaves nested blocks without ids.

## 5. Typst and PDF

`typst::render` writes a complete Typst file that imports muse-typst (`/muse-typst/lib.typ`) and wraps the body in `muse-doc`. Every string of the document reaches Typst as a string literal escaped by `escape_string` (a port of Ardoise's `escapeTypstString`), so text is never read as Typst markup or code. `pdf::compile` (feature `pdf`) compiles it with the `typst` crate at the compiler version muse-typst pins (0.14.2), in memory: muse-typst and the fonts listed in `crates/encre/typst/FONTS.md` are embedded, and the only other readable files are the caller's `Assets`.

| Node | Typst |
|---|---|
| `heading` | `heading(level: n)` |
| `paragraph` | the inline content; an empty paragraph writes nothing |
| `bulletList`, `orderedList` | `list`, `enum(start: n)` |
| `taskList` | `list(marker: [])` with ☐ or ☑ before each item |
| `blockquote` | `quote(block: true)` |
| `codeBlock` | `raw(block: true, lang: …)` |
| `horizontalRule` | muse-typst `divider()` |
| `image` | `figure(image(path, alt: …), caption: title)` when the resolver answers a path; otherwise the alt text in emphasis |
| `table` | `table` with `table.header` and the header row's alignments |
| `callout` | muse-typst `alert(tone:, title:)` |
| `stat` | muse-typst `metric(value, label, sub: delta)` |
| `chart` | **a table fallback**: muse-typst has no chart component. A `figure` holding a table with one column per label and one row per series, captioned with the title. `kind` is not drawn |
| marks | `strong`, `emph`, `strike`, `raw`, `link(href)`; a link title is not shown |
| `hardBreak` | `linebreak()` |

The body sets `font: ("Satoshi", "DejaVu Sans")` and raw text `("Menlo", "DejaVu Sans Mono")`, so a compiler with muse-typst's own fonts installed uses them and the embedded compiler falls back to DejaVu.

`muse-doc` gets `footer: none` unless the caller gives a footer text, so a page footer holds only the page number by default.

`spec/vectors/typst/<name>.json` holds `options` (`title`, `subtitle`, `doc_type`, `cover`, `footer`, and `images` mapping an image `src` to the path the resolver answers), the `doc`, and the exact `want_typst`.
