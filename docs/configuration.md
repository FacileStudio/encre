# encre — Configuration

Every switch encre reads: Cargo features, render options, editor props, and the variables the
scripts and tests take. encre reads no environment variable at run time; the `encre` binary
reads only `NO_COLOR`.

## Cargo features

| Feature | Adds | Pulls in |
|---|---|---|
| none | model, validation, ids, migration, Facile Markdown, `ops::apply`, `typst::render` | pulldown-cmark, serde |
| `yjs` | `yjs` module and `ops::apply_yjs` | yrs 0.28 |
| `pdf` | `pdf::compile`, muse-typst and DejaVu embedded | typst, typst-pdf `=0.14.2`, include_dir |
| `openui` | `openui` module | nothing |

`encre-wasm` builds `encre` with `openui` only. `typst::render` needs no feature: rendering the
source is plain string work, and only compiling it pulls in the compiler.

## `typst::RenderOptions`

| Field | Default | What it does |
|---|---|---|
| `title` | `""` | PDF metadata title and the cover title |
| `subtitle` | `None` | Cover subtitle |
| `doc_type` | `"Document"` | The label in muse-typst's running header |
| `cover` | `false` | Starts with muse-typst's cover page |
| `footer` | `None` | Text at the left of every footer; `None` leaves only the page number |
| `image` | `None` | Resolves an image `src` to a path the compiler reads; `None` writes the alt text |

`pdf::Assets` holds the files a document may read, by absolute path (`/images/logo.png`). An
image resolver answers paths inside it.

## `Editor` props

| Prop | Default | What it does |
|---|---|---|
| `content` | empty | The initial document; ignored when `ydoc` is set |
| `editable` | `true` | Whether the user can edit |
| `placeholder` | none | Text shown in an empty document |
| `toolbar` | `true` | Shows the toolbar |
| `onchange` | none | Called with the document after each change |
| `class` | `""` | Classes on the wrapper |
| `ydoc` | none | The shared `Y.Doc`; when set it is the source of truth |
| `provider` | none | A y-websocket `WebsocketProvider`, or any object with `awareness`, `synced` and the `sync` event |
| `user` | none | `{ name, color?, avatar? }`, how other people see this user |
| `locale` | `fr` | `fr`, `en`, `es` or `de`: the language of every interface string |
| `labels` | none | Overrides of some strings of that locale (`LabelOverrides`) |
| `renderDiagram` | none | `(language, source) => Promise<string \| SVGElement>`; draws diagrams in the editor |

`encreExtensions({ placeholder, collaboration, labels, renderDiagram })` builds the same Tiptap extension set for an app
that drives Tiptap itself. `collaboration` is `{ document, provider?, user? }`; with it the
extension set drops undo and redo, which Yjs owns.

## Loading the WebAssembly

`loadEncre(source?)` loads encre-wasm once and returns the same promise on later calls. Without
`source` it fetches `encre_wasm_bg.wasm` next to `src/lib/encre-wasm/encre_wasm.js`, which Vite
resolves. Pass bytes, a URL, a `Response` or a compiled module to load it another way, as the
tests do with `readFileSync`.

## Script and test variables

| Variable | Read by | What it does |
|---|---|---|
| `ENCRE_REGEN` | Rust tests `typst`, `openui`, `yjs` | Rewrites the golden outputs instead of comparing |
| `ENCRE_BIN` | the Go module in `go/` and its tests | The `encre` binary to run, default `encre` on `PATH` (tests: `target/*/encre`) |
| `ENCRE_GEN` | `vite.config.ts` | Runs `scripts/gen-*.ts` instead of the test suites |
| `PROPTEST_CASES` | `crates/encre/tests/laws.rs` | Cases per round-trip law, default `10000` |
| `HOSTILE_CASE` | `crates/encre/tests/hostile.rs` | Runs only the named crafted input, such as `fence ladder` |
| `MUSE_TYPST` | `scripts/sync-muse-typst.sh` | The muse-typst checkout to copy, default `~/Code/Facile/muse-typst` |
| `ENCRE_CLI_TARGETS` | `scripts/build-cli.sh` | Rust targets to build, default the two linux musl ones |
| `ENCRE_CLI_OUT` | `scripts/build-cli.sh` | Output directory, default `target/cli` |
| `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS` | `scripts/build-wasm.sh` | Rust flags for the wasm build, default `-Ccodegen-units=1` plus `--remap-path-prefix` for the toolchain source and `CARGO_HOME` |

Back to the [documentation index](README.md).
