# encre — Development

Setup, the two gates, the shared test vectors, the WebAssembly build and the muse-typst sync.

## Prerequisites

- Rust 1.98 with the `wasm32-unknown-unknown` target, and wasm-pack 0.15
- bun 1.4 and Node 24.22.0 (pinned in `mise.toml`)
- filet, for the code standard in `filet.yml`
- `typst` is not needed: the `pdf` feature compiles in memory

```sh
mise install
bun install --frozen-lockfile
```

## Gates

Both must pass before a commit.

```sh
mise run check
mise run test
```

`mise run check` runs `scripts/check.sh check`: `cargo fmt --check`, clippy on every target and
feature with `-D warnings`, `svelte-check --fail-on-warnings`, a fresh wasm build followed by
`git diff --exit-code -- src/lib/encre-wasm`, and `filet check`. The wasm step fails when the
committed package does not match the Rust source, so commit `src/lib/encre-wasm` with the change
that moved it.

`mise run test` runs `cargo test --all-features --locked` and `vitest run`. The round-trip laws
in `crates/encre/tests/laws.rs` run 10 000 cases each; set `PROPTEST_CASES=200` for a quick loop.

## Vectors

`spec/vectors/` holds the cases both test suites read, so Rust and TypeScript cannot drift.

| Directory | File | Written by |
|---|---|---|
| `markdown/` | `{markdown, doc, canonical?}` | Hand; `canonical` when the writer normalizes the input |
| `ops/` | `{markdown or doc, version?, op, want_markdown or want_blocks or want_error, want_kept?}` | Hand |
| `yjs/` | `<name>.bin`, `<name>.json`, `<name>.rust.bin` | `scripts/gen-yjs-vectors.ts`, then the Rust test |
| `typst/` | `{options, doc, want_typst}` | Hand, `want_typst` regenerated |
| `openui/` | `{name, source, want_doc or want_error}` | Hand, the outputs regenerated |

- **ops.** `op` is the JSON form agents send, unknown fields refused. `want_error` is the exact
  message and `want_kept` lists ids that must survive. When the vector gives `markdown`, the
  runner gives top-level block *i* the id `b` followed by *i* in nine digits (`b000000000`) and
  leaves nested blocks without ids.
- **yjs.** `<name>.bin` is a v1 update written by a real Tiptap editor, `<name>.json` the compact
  document and its Facile Markdown, `<name>.rust.bin` the update the Rust writer gives for the
  same document. It is the only proof that Rust matches y-tiptap's layout.
- **typst.** `options` takes `title`, `subtitle`, `doc_type`, `cover`, `footer` and `images`, a
  map from image `src` to the path the resolver answers.
- **openui.** The sources include every program from Bulle's `OpenUIBlock.test.ts`,
  `inspect.test.ts` and its system prompt examples. Every `want_doc` must also validate after
  `assign_ids` and survive the markdown projection; the Rust test checks both.

Regenerate golden outputs only on purpose, then read the diff before committing it:

```sh
mise run vectors:yjs
ENCRE_REGEN=1 cargo test -p encre --all-features --test typst --test openui
```

`vectors:yjs` runs the Tiptap generator (`ENCRE_GEN=1 bunx vitest run`) and then the Rust test
with `ENCRE_REGEN=1` to rewrite the `.rust.bin` files. Rerun it on every bump of Tiptap,
`@tiptap/y-tiptap` or yrs.

## The WebAssembly build

```sh
mise run wasm
```

`scripts/build-wasm.sh` runs wasm-pack for the web target into `src/lib/encre-wasm` and removes
the `.gitignore` wasm-pack writes. It sets `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS` to
`-Ccodegen-units=1` unless the variable is already set. That override matters on machines whose
`~/.cargo/config.toml` sets `build.rustflags` for the host, for example `-Wl,-rpath` for GTK:
the target variable replaces those flags, and without it the wasm link fails. wasm-opt is off in
`crates/encre-wasm/Cargo.toml`, so the build needs no binaryen.

## The CLI build

```sh
mise run cli
```

`scripts/build-cli.sh` builds static, stripped `encre` binaries for `x86_64-unknown-linux-musl`
and `aarch64-unknown-linux-musl` into `target/cli`, with release archives and `checksums.txt`.
The aarch64 link uses the toolchain's `rust-lld`. See [cli.md](cli.md).

## muse-typst sync

`crates/encre/typst/muse-typst` is a copy of muse-typst's `lib.typ`, `tokens.typ`,
`components.typ`, `template.typ` and `assets/*.svg`, so the PDF compiler embeds it.

```sh
sh scripts/sync-muse-typst.sh
MUSE_TYPST=~/Code/Facile/muse-typst-wt-topic sh scripts/sync-muse-typst.sh
```

The script refuses a source checkout with uncommitted changes, then writes
`crates/encre/typst/muse-typst/SOURCE` with the commit, its date and the typst version from
muse-typst's `mise.toml`. After a sync:

1. Check that `typst` and `typst-pdf` in `crates/encre/Cargo.toml` still equal that version.
2. Run the typst vectors and the PDF tests; regenerate `want_typst` if the preamble moved.
3. Commit the copy, `SOURCE` and the vectors together.

## Releasing

Bump `version` in the root `Cargo.toml` and in `package.json`, run `cargo check` to update
`Cargo.lock`, add the `CHANGELOG.md` entry, pass both gates, commit `chore: release x.y.z` and
tag `vx.y.z`. Consumers pin the tag in Cargo and in `bun add`.

Back to the [documentation index](README.md).
