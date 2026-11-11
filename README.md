# encre

[![ci](https://github.com/FacileStudio/encre/actions/workflows/ci.yml/badge.svg)](https://github.com/FacileStudio/encre/actions/workflows/ci.yml)

The Facile document format: one validated tree that people edit in Tiptap, agents edit as
markdown, and every suite app exports to Typst and PDF.

A document is ProseMirror JSON checked against `spec/schema.json`. Each conversion has exactly
one implementation, in Rust, and the browser runs the same code compiled to WebAssembly.

## What it does

- Defines schema v2 (`spec/schema.json`) and validates documents against it in Rust and TypeScript;
  every schema 1 document is a valid schema 2 document
- Projects documents to Facile Markdown and back without loss, plus a lossy CommonMark export
- Reads and writes the Yjs layout Tiptap's Collaboration extension uses, so Rust edits live documents
- Gives agents guarded operations (`append`, `insert_after`, `patch`, ...) that refuse ambiguous edits
- Renders documents as Typst through muse-typst and compiles them to PDF in memory
- Converts OpenUI programs from Bulle's chat into documents, one way
- Ships an `encre` command-line tool, and a Go module that wraps it, so Go services run every
  conversion as a subprocess
- Ships a Svelte 5 Tiptap editor with collaboration, a read-only renderer and its menus, in
  French, English, Spanish or German

## Stack

| Layer | Tech |
|---|---|
| Runtime | Rust 2024 (1.98), pulldown-cmark 0.13, yrs 0.28, typst 0.14.2, wasm-bindgen 0.2 |
| Client | Svelte 5, Tiptap 3.33, `@tiptap/y-tiptap` 3.3.0, muse (`@facile/muse` v0.16.1 or later) |
| Go | Go 1.24+, standard library only (`go/`) |

## Install

Rust, from a git tag. Features: `yjs` (Yjs layout and live operations), `pdf` (Typst compiler),
`openui` (the OpenUI adapter).

```toml
[dependencies]
encre = { git = "https://github.com/FacileStudio/encre", tag = "v0.9.4", features = ["yjs", "pdf"] }
```

Svelte, from the same tag:

```sh
bun add "github:FacileStudio/encre#v0.9.4"
```

### CLI

Build the binary with `cargo build --release -p encre-cli`, or the static linux binaries with
`mise run cli` (`target/cli/encre-x86_64-unknown-linux-musl` and the aarch64 one).

```sh
encre parse --ids < note.md > note.json
encre validate --json < note.json
echo '[{"op":"append","markdown":"Done."}]' | encre ops --doc note.json
encre pdf --title Plan --image-dir ./images --input note.json --output plan.pdf
```

Exit codes are 0 ok, 1 invalid input, 2 usage, 3 internal. [docs/cli.md](docs/cli.md) has every
command and what markdown validation refuses.

### Go

```sh
go get github.com/FacileStudio/encre/go@v0.9.4
```

The Go module has its own tag, `go/vX.Y.Z`, on the same commit as `vX.Y.Z`.

```go
import "github.com/FacileStudio/encre/go"

verdict, err := encre.Validate(ctx, body, encre.Markdown)
markdown, err := encre.Serialize(ctx, doc)
pdf, err := encre.PDF(ctx, doc, encre.JSON, encre.PDFOptions{Title: "Plan"})
```

The module runs `ENCRE_BIN`, else `encre` on `PATH`. A refusal is a `*encre.Error` with the exit
code; `Invalid()` tells a bad input from a failure. The full API is in [docs/cli.md](docs/cli.md).

### Rust quick start

```rust
use encre::ops::{Op, apply};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = encre::markdown::parse("# Plan\n\nShip the **gate** first.")?;
    encre::assign_ids(&mut doc);
    encre::validate(&doc)?;
    apply(&mut doc, 1, &Op::Append { markdown: "- [ ] tag v0.9.4".to_owned() })?;
    println!("{}", encre::markdown::serialize(&doc));
    Ok(())
}
```

### Svelte quick start

```svelte
<script lang="ts">
  import '@facile/encre/styles';
  import { Editor, serializeMarkdown, type EncreNode } from '@facile/encre';

  let doc = $state<EncreNode>({ type: 'doc', content: [{ type: 'paragraph' }] });
</script>

<Editor content={doc} onchange={(next) => (doc = next)} />
<button onclick={() => console.log(serializeMarkdown(doc))}>Copy as markdown</button>
```

`Editor` and `Document` take `locale` (`fr` by default, `en`, `es`, `de`), `labels` overrides and
an optional `renderDiagram(language, source)`; `Document` also takes `highlight(code, language)`,
whose HTML the app must sanitize. muse is a peer dependency: v0.16.1 or later. The peer is the
untagged git spec, because bun looks a semver range up on npm, where muse is not published.

The editor loads encre-wasm on mount. Other code calls `await loadEncre()` before the first
`parseMarkdown`, `serializeMarkdown` or `openuiToDocument`. Like muse, the components are styled
with Tailwind classes, so the app's CSS needs `@source '../node_modules/@facile/encre/src/lib';`.

## Structure

```
spec/             schema.json, FORMAT.md (normative), vectors/ shared by both test suites
crates/encre/     the Rust crate: model, markdown, yjs, ops, typst, pdf, openui
crates/encre-wasm/  wasm-bindgen entry points for the browser
crates/encre-cli/   the encre binary
go/               the Go module that wraps the binary
src/lib/          the Svelte package: editor, extensions, node views, wasm loader
test/             vitest suites
scripts/          gates, the wasm build, vector generation, muse-typst sync
docs/             architecture, configuration, development, API, agents
```

## Documentation

| Doc | What's in it |
|---|---|
| [Format](spec/FORMAT.md) | The normative schema, Facile Markdown and agent operation rules |
| [Architecture](docs/architecture.md) | The model, every conversion, the Yjs layout, Typst and OpenUI mapping |
| [Configuration](docs/configuration.md) | Cargo features, render options, editor props, script variables |
| [Development](docs/development.md) | Setup, the gates, vectors, the wasm build, muse-typst sync |
| [API](docs/api.md) | The public Rust and TypeScript surface |
| [CLI](docs/cli.md) | The `encre` binary, its exit codes, and calling it from Go |
| [Agents](docs/agents.md) | The short form of the format and operations for MCP tool descriptions |

## License

Apache-2.0, see [LICENSE](LICENSE) and [NOTICE](NOTICE). The bundled DejaVu fonts keep their own
licence, in `crates/encre/typst/fonts/LICENSE`.

---

Part of the [Facile Suite](https://facile.studio) — self-hosted tools for creative studios
and freelancers. One login, zero cloud dependency.
