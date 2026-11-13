# Changelog

All notable changes to `encre` are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). While on `v0`, a breaking change bumps
the minor version.

## [0.12.0] - 2026-11-13

Editor only. The document format, schema and Markdown are unchanged.

### Added

- Every `BlockCommand` has an `icon`: a muse `Icon` name (Solar linear glyphs, plus three
  heading glyphs drawn in the same stroke). The slash menu, the toolbar block menu and the bubble
  menu "Turn into" menu show it before the label.
- `BLOCK_ICONS` (command id to icon name) and `blockIcon(id)` are exported. Importing encre
  registers the glyphs with muse, so a host reuses them in its own menus, such as a right-click
  "Turn into" submenu: `blockCommands(labels).map((c) => ({ label: c.label, icon: c.icon, ... }))`.
- The slash menu offers one embed entry per Facile app after "Link / embed": Toile board,
  Feuilles spreadsheet, Vision dashboard, Docs page and Nuage file (French: "Tableau Toile",
  "Classeur Feuilles", "Tableau de bord Vision", "Page Docs", "Fichier Nuage"). Each shows the
  app's bold duotone brand logo, is found by the app name (`/toile`, `/feuilles`, `/vision`,
  `/docs`, `/nuage`), asks for the address and inserts an embed of that kind. New exports
  `embedCommands(onerror, labels)` and the optional `kind` argument of `embedCommand`; new labels
  `embedToile`, `embedFeuilles`, `embedVision`, `embedDocs`, `embedNuage` in the four catalogues.
- `EMBED_LOGOS` and `embedLogo(kind)`: the app logo of each embed kind (the glyph of muse's brand
  catalog, bundled, no network fetch), the link glyph for `link`. A framed embed's "Open" link and
  a Facile app's link card show it in place of the site favicon.

### Changed

- muse v0.21.2. Icon-only controls show a muse tooltip: the mark buttons, the link buttons and
  the toolbar undo and redo through `IconButton` (the extra `Tooltip` wrappers are gone, so each
  shows one tooltip), the embed resize handle, and each `Presence` avatar (was a native `title`).
- `embedCommand` moved from `files.ts` to `embed-commands.ts`; the package export is unchanged.
- `BlockCommand.icon` is required. A host that builds its own `BlockCommand` adds an `icon`.

### Fixed

- The slash menu scrolls the highlighted entry into view as the arrow keys move it. Before, the
  highlight left the visible part of the list and only the mouse wheel reached the entries below.
- The slash menu closes on a press anywhere outside its list (the editor included), when focus
  leaves the editor or the window, and when the page scrolls. The typed `/query` stays as plain
  text, Enter then splits the paragraph as usual, and typing `/` again reopens the menu. Escape
  still closes it. Closing exits the suggestion plugin, so no decoration is left behind.

## [0.11.0] - 2026-11-13

Schema 3. Strictly additive: every valid schema 1 or 2 document is a valid schema 3 document,
unchanged, and `migrate(doc, 2)` is the identity. Stores that record the schema version should
write 3 from now on.

### Added

- Schema 3 adds two atom blocks, marked `"since": 3`: `embed {id, url, kind, title}` and
  `file {id, src, name, size, mime}`. `url` is a new attribute type: an https address with a
  host. `kind` is `link`, `toile`, `feuilles`, `vision`, `docs` or `nuage`.
- Facile Markdown reads and writes them as data-only fences, like `stat`:
  ```` ```embed ```` holding `{"url": "…", "kind": "…", "title": "…"}` and ```` ```file ````
  holding `{"src": "…", "name": "…", "size": 0, "mime": "…"}`. The CommonMark export, plain text
  and Typst show each as one link, labelled by its title or name.
- Editor: `embed` and `file` node views. Toile, Feuilles, Vision and Docs embeds show the page in
  a lazy frame (`?embed=1` added, Vision as is) with an "Open" link and a drag handle for the
  height; other links show a card that `resolveEmbed` can enrich. A file is a row that downloads
  `src`. `Document` renders both.
- Editor props `onfile`, `onembed`, `resolveEmbed`, `oncontextmenu` and `extras`, and the methods
  `selectionMarkdown()`, `run(id)`, `turnInto(id)`, `duplicateBlock()`, `deleteSelection()` and
  `copyBlockLink(base)`. A lone https URL pasted on an empty paragraph becomes an embed unless
  `onembed` answers false; non-image files pasted, dropped or picked go through `onfile`.
- The bubble menu shows for block and node selections too, with a "Turn into" menu, the mark
  buttons, the link button and the host's `extras`.
- The slash menu gains "Lien / intégration" and, with `onfile`, "Fichier".
- `embedKind(url, hosts?)` and the embed and file helpers, exported; copy for every new string in
  `fr`, `en`, `es` and `de`.

### Changed

- The link button edits an existing link instead of removing it; an empty address or the new
  remove button removes it.
- `Presence` renders nothing while this client is the only one connected.
- A fence named `embed` or `file` holding a valid JSON object now reads as that block. A code
  block whose language starts with `embed` or `file` is written with a tilde fence, as other
  custom block names are.

## [0.10.1] - 2026-11-13

No change to the library, the CLI or the Go module: consumers need not bump.

### Fixed

- `sh scripts/check.sh` with no mode stopped on nothing in its `check` half: it ran
  `check && test_suites`, and `sh` ignores `set -e` inside a function called on the left of
  `&&`, so a failing `cargo fmt`, clippy, `bun run check`, wasm diff or `go vet` was skipped
  past unless `filet check`, the last step, failed too.
- A clippy `dead_code` error in the CLI tests that the gate above let through.

### Changed

- `filet.yml` drops six settings no rule reads for `.rs`, `.ts`, `.svelte` or `.sh`
  (`returns`, `structFields`, `interfaceMethods`, `requireDocComments`, `banGlobalMutable`,
  `banInit`).
- The Yjs and OpenUI nesting tests move to `crates/encre/tests/nesting.rs` and the markdown
  validation tests to `crates/encre-cli/tests/validate_markdown.rs`, under filet's
  eight-functions-per-file limit.

## [0.10.0] - 2026-11-12

### Changed

- **Breaking:** the `pdf` feature compiles with typst 0.15.1 (was 0.14.2), so PDF bytes differ
  from 0.9.x. On the sample documents the page count and extracted text are unchanged, the files
  are about 10 % smaller and `Creator` reads `Typst 0.15.1`. The vendored muse-typst (e0bb711)
  compiles unchanged. The feature also depends on `typst-layout`.
- The workspace patches `citationberg` to upstream commit 06a591e (0.7.0 plus quick-xml 0.41), so
  quick-xml 0.38 and its advisories leave the build. A `[patch]` does not reach dependents: a
  workspace that uses encre with `pdf` needs the same entry, shown in `docs/development.md`.

### Fixed

- After a wasm trap (`memory access out of bounds`, `unreachable`, a stack overflow), the next
  encre function or `loadEncre` call runs on a fresh instance built from the compiled module.
  Before, every later call failed until the page reloaded. A trap inside `validateDocument` or
  `parseMarkdown` now reaches the caller instead of reading as a refusal.
- `Editor` node views follow a change of `locale`, `labels` or `renderDiagram`; they kept the copy
  from mount. `placeholder`, `ydoc`, `provider` and `user` stay read at mount, as `docs/api.md`
  now says.
- Callout, column, stat and chart title inputs and the chart labels and series drop every control
  character but tab, so a pasted U+0007 no longer makes exports refuse the document.
- An empty chart entry (`web: 1, , 2` or a trailing comma) no longer becomes a 0.
- Go: the tests fail instead of skipping when `CI` is set and no `encre` binary is found, or when
  `ENCRE_BIN` names a missing file.

## [0.9.4] - 2026-11-11

### Fixed

- `yjs::read`, `yjs::from_update` and `ops::apply_yjs` refuse an update nested past 48 levels with
  `YjsError::Invalid`, the message `validate` gives, before they recurse. A 115 KB update nested
  5 000 deep overflowed the stack and aborted the process.
- `openui::to_document` counts nesting through references, so `x0 = [x1]`, `x1 = [x2]` chains stop
  at 64 levels with the new `Error::TooDeep`. Long chains overflowed the stack in the CLI and broke
  the browser's wasm instance for every later call.
- `patch` gives ids only to the blocks it inserts, so its cost is linear in the number of matching
  blocks. 4 000 matching blocks took 113 s in a debug build.
- Pasted or `setContent` HTML always validates: the editor rewrites a table with plain `td` header
  cells, `colspan` rows or mixed column alignment into a rectangular GFM table aligned like its
  header row, maps any `text-align` other than left, center and right to none, and clamps an
  ordered list `start` to 0 to 999 999 999.
- `caretColor` returns a `#rrggbb` color from muse's chart palette. y-tiptap draws only that form,
  so carets and selections without a `user.color` were transparent.
- `loadEncre` forgets a failed load, so the next call tries again.
- PDF export writes a link only for http, https, mailto, tel and relative targets, as `Document`
  does; other links become plain text.
- Go: `PDF` passes `--title=`, `--footer=` and `--image-dir=` values, so a title starting with `-`
  is not read as a flag.
- Go: `Validate` returns an invalid `Verdict` for input larger than the size limit instead of an
  `*Error`. The new package variable `MaxBytes` sets `--max-bytes` on every call.

## [0.9.3] - 2026-11-11

### Fixed

- `Editor` methods `revealPath` and `revealBlock` scrolled the node to the top of the scroll
  container, under the editor's sticky toolbar. They now keep it below the toolbar's measured
  height.

## [0.9.2] - 2026-11-11

### Added

- `serializeMarkdown` and `serializeCommonMark` throw an `EncreDocumentError`, exported, when the
  document is invalid. It extends `Error` with the same message as before, and carries `path`, the
  tree path of the failing node, and `blockId`, the id of that node or of the nearest block around
  it that has one.
- `Editor` methods `revealPath(path)` and `revealBlock(blockId)` select a node and scroll to it, so
  an app can jump to the spot an `EncreDocumentError` names. A textblock's text is selected.
- `Editor.exportMarkdown` and `exportCommonMark` hand an `EncreDocumentError` to `onerror` before
  they reject with it.
- A markdown paste that `parseMarkdown` refuses still lands as literal text, and the editor now
  hands the `EncreMarkdownError` (`line`, `column`, `path`) to `onerror`.

## [0.9.1] - 2026-11-11

### Added

- encre-wasm `validateMarkdown(markdown)` and the TypeScript `validateMarkdown`, backed by
  `markdown::validate`: `{ valid, errors: [{ path, message, line, column }] }`, the same verdict as
  `encre validate --from markdown --json`. A CLI test compares both on the CLI's inputs.
- `parseMarkdown` throws an `EncreMarkdownError`, exported, which extends `Error` and carries
  `line`, `column` and `path` when the source position is known. The message is the same text as
  before, so code that catches and shows it keeps working. Nine nested custom blocks after a
  heading now report line 11, column 1, as the CLI does.

## [0.9.0] - 2026-11-10

### Security

- Every markdown path now runs in linear time on hostile input. Reading, validating, writing and
  `plain_text` were quadratic or worse on crafted text: runs of backslashes (the bracket and `=`
  scan counted backslashes back from every character), digits followed by dots (the escaper
  re-read the line start for every dot), every `[` building a string, flanking fixes re-sweeping
  every token, mark runs re-measured at every node, and `patch` storing a context for every match.
  `docs/development.md` records the cost of each crafted input at 1 MiB and 8 MiB before and after.
- Deeply nested markdown no longer overflows the stack. `>` repeated or `*` runs aborted the
  process, and so the WebAssembly module.

### Changed

- New normative limits (`spec/FORMAT.md` rules 8 and 9 and section 2.6). A document nests at most
  48 levels and holds at most 8 `callout`, `columns` and `column` nodes on one path; `validate`
  refuses beyond. Markdown whose blocks and inline spans nest more than 64 levels, or whose custom
  fences open more than 8 custom blocks on one path, is refused by `parse` (a `ParseError`) and by
  `markdown::validate` (path `doc`, at the line and column of the syntax that crosses the limit).
  Agent operations refuse such markdown as `OpError::Invalid` and change nothing.
- `OpError::PatchMatches` gains `count`, and `found` lists at most the first 20 matches
  (`ops::LISTED`); the message ends with "and N more".
- A validation message no longer starts with the child's relative path (`content[0]: `): `path`
  alone says where the problem is.
- `markdown::validate` reads the source once and tracks positions only when it has an error to
  place.
- `scripts/build-wasm.sh` remaps local paths out of the module, so every machine builds the same
  `src/lib/encre-wasm`.

### Added

- `crates/encre/tests/hostile.rs`, run by `mise run test`: every crafted input at 1 MiB and 8 MiB,
  failing when 8 MiB costs more than 20 times 1 MiB.
- CI on pushes to `main` and pull requests (`.github/workflows/ci.yml`), and bun 1.4.2 in
  `mise.toml`.
- The Go module is tagged `go/vX.Y.Z` with every release.

## [0.8.0] - 2026-11-10

### Schema 2

Strictly additive: every valid schema 1 document is a valid schema 2 document, unchanged, and
`migrate(doc, 1)` is the identity (a test checks it on every vector document). `SCHEMA_VERSION`
is 2. What schema 2 adds carries `"since": 2` in `spec/schema.json`.

- `highlight` mark, ranked between `strike` and `code`. Markdown `==text==`.
- `badge` inline node: `tone` (neutral, accent, info, success, warning, danger), `dot`, and text
  content without marks. Markdown `[badge:<tone> <text>]` (`primary` reads as `accent`) and
  `[status:<state>]`, a dot badge whose states map to tones (active, done, ready, success:
  success; error, failed, stopped, dead: danger; pending, wip, running, loading: warning).
  `[status:<tone> <text>]` writes a dot badge whose text is not a state, so every badge
  round-trips. An unknown tone or state stays text, and code never holds badges.
- `diagram` block: `language` (mermaid, d2, dot) and the source as text. encre never runs it.
- `columns` block of two to four `column` children, each with `title`, `tone` (neutral,
  success, danger, warning, info) and block content. Markdown: a `columns` fence holding
  `column title="..." tone="..."` fences.
- `stat.sub`, a subtitle separate from `delta`. A writer leaves it out of the JSON body when
  empty, so schema 1 stats serialize as before.

### Added

- GFM alerts parse natively: `> [!NOTE]` and `[!IMPORTANT]` become an `info` callout,
  `[!TIP]`, `[!PROS]` and `[!ADVANTAGE]` `success`, `[!WARNING]` `warning`, `[!CAUTION]`,
  `[!CONS]` and `[!DRAWBACK]` `danger`, any case, with an empty title for the app to localize.
  Serialize keeps writing callout fences.
- `compare`, `pros-cons` and `vs` fences read as `columns`: `### ` sections (or `---` separated
  parts), titles matching advantage, pro, positive, plus or benefit are `success`, drawback, con,
  negative, minus, risk or issue `danger`. Serialize writes `columns`.
- `markdown::validate`: parse and validate, with a `SourceError` holding the tree path and the
  1-based line and column of the offending block. `encre validate --json` on markdown adds
  `line` and `column` to each error. `docs/cli.md` lists what markdown validation refuses.
- `Document`: diagrams through an optional `renderDiagram(language, source)`, else their source
  under a language label; columns that stack on narrow screens; badges and status dots; `<mark>`;
  stat subtitles; per-line tones for `diff` code blocks; an optional `highlight(code, language)`
  hook whose HTML the app must sanitize (encre escapes the code without it).
- `Editor`: `locale` (`fr` default, `en`, `es`, `de`), `labels` overrides and `renderDiagram`.
  Every string of the editor, toolbar, slash and bubble menus, node views and the upload
  placeholder comes from the catalogue. New slash commands: diagram, columns, badge; a highlight
  button (Mod-Shift-H); toolbar menus to add or remove a column and to retone a badge.
- `blockCommands(labels)`, `labelsFor`, `CATALOGUES`, `Settings`, the `Diagram`, `Columns`,
  `Column` and `Badge` Tiptap nodes, and the matching types.
- Typst: `highlight`, muse-typst `badge` and `status-dot`, diagrams as a labelled raw block,
  columns as a `grid`, `stat.sub` in the metric's subtitle.
- CommonMark export: a diagram stays its fenced block, columns become a bold title and the
  blocks of each column, a badge is its text in brackets, a highlight is plain text.
- `go/`: the Go module `github.com/FacileStudio/encre/go` (package `encre`) with `Run`, a typed
  `*Error`, `Validate`, `Parse`, `Serialize`, `CommonMark`, `PlainText` and `PDF`, tested against
  the built binary. `mise run check` runs gofmt and `go vet`, `mise run test` the Go tests.
- Vectors: markdown vectors for every new construct, ops vectors over columns and badges, the
  Typst every-node vector with the new nodes, and a `schema2` yjs vector from Tiptap. The schema 1
  `custom` update is kept as `v1-custom.bin`, which Rust and Tiptap still read.

### Changed

- Fences with the info string `mermaid`, `d2` or `dot` now parse as `diagram`, and `compare`,
  `pros-cons` and `vs` fences as `columns`. A code block whose language starts with one of those
  words, or with `columns`, is written with a tilde fence so it stays a code block.
- A literal `==` in text is written escaped (`\=\=`), since `==` now delimits a highlight.
- The `@facile/muse` peer is the untagged `github:FacileStudio/muse`, and the README states
  v0.16.1 or later. A semver range such as `>=0.16.1` makes bun look muse up on npm, where it is
  not published, so every consumer install failed with a 404.
- `Document` renders `codeBlock` through a new `Code` component; output for other languages is
  unchanged without a `highlight` hook.

### Not accepted

- List status markers (`[+]`, `[-]`, `[!]`, `[?]`, `✅`, `❌`, `[pro]`, `[con]`, ...) and the
  `[+ Feature]` / `[- Missing]` forms stay literal text.

## [0.7.0] - 2026-11-10

### Added

- `Editor` image upload: an optional `onimage(file)` hook, called on pasted and dropped image
  files and from a new "Image" slash and toolbar command that opens a file picker. The returned
  `src` and `alt` become an `image` node at the caret or drop point. A local placeholder shows
  while the hook runs. PNG, JPEG, GIF and WebP only (never SVG), capped by `maxImageBytes`;
  refusals and failures reach `onerror`. Without `onimage` nothing changes.
- `Editor` `presence` prop: `'inline'` (default, as before) or `'none'`, so an app can render
  `Presence` itself, for example in its page header.
- `IMAGE_TYPES`, `imageRefusal`, `ImageUpload` and `UploadedImage` exports.
- `findBlocks`, `slashExtension` and `Toolbar` take an optional command list.
- `.github/workflows/release.yml`: a `v*` tag builds the static linux amd64 and arm64 `encre`
  binaries with `scripts/build-cli.sh` and publishes them with `checksums.txt` on the GitHub
  release. `docs/cli.md` shows the Dockerfile download.

## [0.6.0] - 2026-11-10

### Added

- `encre`, a command-line tool in the new `crates/encre-cli`, so Go services can run encre as a
  subprocess: `parse`, `serialize`, `commonmark`, `plain-text`, `validate`, `compact`, `openui`,
  `ops`, `typst` and `pdf`. Stable exit codes (0 ok, 1 invalid input, 2 usage, 3 internal),
  `--json` output and errors, a `--max-bytes` input cap (8 MiB by default), and `--image-dir`,
  which embeds only files inside that directory. See `docs/cli.md`.
- `scripts/build-cli.sh` (`mise run cli`): static, stripped musl binaries for linux amd64 and
  arm64, with release archives and `checksums.txt`.
- `LICENSE` (Apache-2.0) and `NOTICE`.

### Changed

- encre is licensed under Apache-2.0 instead of MIT, in every `Cargo.toml` and `package.json`.
- The vendored muse-typst moves to `e0bb711` on muse-typst's `main`, which carries the same
  templates as `a98e881` plus its Apache-2.0 `LICENSE`, now copied by
  `scripts/sync-muse-typst.sh`. The Typst golden vectors do not change.

## [0.5.0] - 2026-11-10

### Added

- `openui` feature: `openui::to_document` and `openui::to_blocks` turn an OpenUI Lang program
  from Bulle's chat into encre blocks, with hoisted references, unreachable statements dropped,
  and typed errors for cycles, unknown components and bad arguments.
- `openuiToDocument` in encre-wasm and in the package, for "save as document" in the browser.
- `spec/vectors/openui/`, read by the Rust and the TypeScript suites.
- `RenderOptions.footer`: the text at the left of every PDF footer.
- `README.md`, this changelog, and `docs/` (architecture, configuration, development, API,
  agents).

### Changed

- PDFs no longer print muse-typst's "Document de cadrage confidentiel" footer. encre passes
  `footer: none` unless `RenderOptions.footer` is set. The vendored muse-typst moves to
  `a98e881`, which adds the `footer:` parameter, and every `want_typst` golden vector changes.
- `spec/FORMAT.md` covers the format only. The Typst mapping moved to `docs/architecture.md`;
  the ops, yjs and typst vector layouts moved to `docs/development.md`.
- The `Editor` `provider` prop documents what a provider must offer.

## [0.4.0] - 2026-11-10

### Added

- `typst::render`: a document as Typst source over muse-typst, every string escaped by
  `escape_string`, a port of Ardoise's escaping.
- `pdf` feature: `pdf::compile` with the `typst` crate 0.14.2 in an in-memory world, embedding
  muse-typst and the DejaVu fonts, and reading nothing but the caller's `Assets`.
- `spec/vectors/typst/` golden files.

## [0.3.0] - 2026-11-10

### Added

- `yjs` feature: read and write the `@tiptap/y-tiptap` layout with yrs, including the hashed
  `code--LpaW+ak5` mark key.
- `spec/vectors/yjs/`, generated by a real Tiptap editor (`scripts/gen-yjs-vectors.ts`).
- `ops`: guarded agent operations on a stored document, and `ops::apply_yjs` on an open one.
- The editor's collaboration path: Tiptap Collaboration and CollaborationCaret over a
  y-websocket provider, with presence.

## [0.2.0] - 2026-11-10

### Added

- The Rust crate: model, validation, block ids, migration and Facile Markdown in both modes,
  with the round-trip property tests.
- `spec/vectors/markdown/` and the pinned Facile Markdown rules.
- encre-wasm, and its use in the editor for markdown paste, import and export.

## [0.1.0] - 2026-11-10

### Added

- Schema v1 (`spec/schema.json`) and the normative `spec/FORMAT.md`.
- Tiptap extensions and node views for every node of schema v1.
- `Editor.svelte` with its toolbar, bubble menu and slash menu, and the read-only
  `Document.svelte`.
- The Svelte package scaffold and its gates.

[0.12.0]: https://github.com/FacileStudio/encre/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/FacileStudio/encre/compare/v0.10.1...v0.11.0
[0.10.1]: https://github.com/FacileStudio/encre/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/FacileStudio/encre/compare/v0.9.4...v0.10.0
[0.9.4]: https://github.com/FacileStudio/encre/compare/v0.9.3...v0.9.4
[0.9.3]: https://github.com/FacileStudio/encre/compare/v0.9.2...v0.9.3
[0.9.2]: https://github.com/FacileStudio/encre/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/FacileStudio/encre/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/FacileStudio/encre/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/FacileStudio/encre/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/FacileStudio/encre/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/FacileStudio/encre/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/FacileStudio/encre/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/FacileStudio/encre/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/FacileStudio/encre/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/FacileStudio/encre/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/FacileStudio/encre/releases/tag/v0.1.0
