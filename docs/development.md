# encre — Development

Setup, the two gates, the shared test vectors, hostile-input costs, the WebAssembly build and the
muse-typst sync.

## Prerequisites

- Rust 1.98 with the `wasm32-unknown-unknown` target, and wasm-pack 0.15
- bun 1.4 and Node 24.22.0 (pinned in `mise.toml`)
- Go 1.24 or later, for the module in `go/`
- filet, for the code standard in `filet.yml`. It does not scan `go/`: filet 0.19.1 panics on Go
  1.27 export data; `gofmt`, `go vet` and the Go tests gate that module
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
`git diff --exit-code -- src/lib/encre-wasm`, `gofmt -l` and `go vet` on `go/`, and `filet check`. The wasm step fails when the
committed package does not match the Rust source, so commit `src/lib/encre-wasm` with the change
that moved it.

`mise run test` runs `cargo test --all-features --locked`, the hostile-input benchmark (below),
`vitest run`, then builds the debug `encre` binary and runs the Go module's tests against it
(Go 1.24 or later).

CI (`.github/workflows/ci.yml`) runs both gates on every push to `main` and every pull request,
after filet's own action: Rust 1.98.1 with rustfmt, clippy and `wasm32-unknown-unknown`,
wasm-pack 0.15.1, Node and bun from `mise.toml`, Go from `go/go.mod`. The round-trip laws
in `crates/encre/tests/laws.rs` run 10 000 cases each; set `PROPTEST_CASES=200` for a quick loop.

## Vectors

`spec/vectors/` holds the cases both test suites read, so Rust and TypeScript cannot drift.

| Directory | File | Written by |
|---|---|---|
| `markdown/` | `{markdown, doc, canonical?}` | Hand; `canonical` when the writer normalizes the input |
| `ops/` | `{markdown or doc, version?, op, want_markdown or want_blocks or want_error, want_kept?}` | Hand |
| `yjs/` | `<name>.bin`, `<name>.json`, `<name>.rust.bin` | `scripts/gen-yjs-vectors.ts`, then the Rust test; `v1-custom.bin` is kept as Tiptap wrote it under schema 1 |
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

## Hostile input

Servers validate every write, so no markdown may cost more than linear time.
`crates/encre/tests/hostile.rs` builds a crafted worst case for each path at 1 MiB and 8 MiB and
fails when the 8 MiB run costs more than 20 times the 1 MiB run (a linear path costs about 8
times, a quadratic one 64). `mise run test` runs it; for the numbers, run it in release:

```sh
cargo test --release -p encre --test hostile -- --ignored --test-threads=1 --nocapture
HOSTILE_CASE="fence ladder" cargo test --release -p encre --test hostile -- --ignored --nocapture
```

CPU time of `markdown::validate` (parse and validate) and of `serialize`, v0.8.0 against v0.9.0,
release build, measured on 2026-11-10 on the 12-core build machine at a load average of 50 to 120.
pulldown-cmark alone costs 0.4 to 0.9 s on the event-dense 8 MiB inputs, which is a floor for
every number here. `plain_text` stays under 0.35 s at 8 MiB in both versions.

| Input | validate 1 MiB, v0.8.0 | v0.9.0 | validate 8 MiB, v0.8.0 | v0.9.0 | serialize 8 MiB, v0.8.0 | v0.9.0 |
|---|---|---|---|---|---|---|
| open brackets | 419.2ms | 180.6ms | 3.6s | 1.2s | 7.5s | 2.2s |
| bracket text | 497.4ms | 112.9ms | 3.7s | 1.1s | 5.5s | 1.1s |
| unclosed badges | 287.1ms | 53.8ms | 2.2s | 398.8ms | 4.5s | 949.2ms |
| badges | 748.8ms | 391.9ms | 4.5s | 2.7s | 16.3s | 5.9s |
| backslashes | > 120 s | 100.0ms | not run | 839.4ms | not run | 884.4ms |
| equals | 179.2ms | 16.0ms | 1.6s | 299.1ms | 5.7s | 1.9s |
| highlight openers | 174.4ms | 20.0ms | 1.5s | 303.3ms | 4.6s | 1.1s |
| alerts | 347.9ms | 108.0ms | 3.0s | 1.3s | 3.3s | 1.2s |
| deep quote | stack overflow | 135.8ms | stack overflow | 1.0s | stack overflow | < 4 ms |
| deep list | < 4 ms | 4.0ms | 12.9ms | 16.0ms | < 4 ms | < 4 ms |
| links | 900.5ms | 497.0ms | 7.1s | 5.1s | 26.8s | 11.0s |
| paragraphs | 1.4s | 495.9ms | 11.3s | 4.2s | 15.0s | 4.9s |
| marks | 1.2s | 615.6ms | 7.7s | 4.8s | 24.0s | 13.3s |
| flanking | 824.0ms | 486.8ms | 6.5s | 4.3s | 19.1s | 9.9s |
| backslash runs | 219.5ms | 114.8ms | 1.8s | 1.0s | 3.4s | 1.1s |
| deep emphasis | stack overflow | 134.4ms | stack overflow | 834.3ms | stack overflow | < 4 ms |
| fence ladder | 930.3ms | 589.3ms | 8.8s | 5.0s | 3.5s | 163.6ms |
| compare | 370.1ms | 24.0ms | 2.7s | 184.0ms | 52.0ms | 48.0ms |
| columns | 311.5ms | 40.0ms | 2.6s | 370.9ms | 52.0ms | 44.0ms |
| digits then dots | > 120 s | 4.0ms | not run | 32.0ms | not run | 128.0ms |
| table padding | 1.1s | 527.3ms | 2.0s | 1.9s | 1.9s | 311.8ms |
| backtick runs | 200.0ms | < 4 ms | 1.8s | 28.0ms | 7.2s | 1.8s |
| patch matches | 1.4s | 16.0ms | 10.8s | 116.0ms | - | - |

`patch matches` is `apply` of a `patch` whose `find` matches every byte of the document.
`patch_matching_is_linear` also applies a patch that changes every one of 500 and of 4 000 blocks;
before v0.9.4 each replacement re-assigned the ids of the whole document, and 4 000 blocks took
113 s in a debug build. The same file refuses a Yjs update nested 10 000 levels deep and OpenUI
reference chains of 50 000 links on a 2 MiB thread, the stack of a tokio worker.

What changed, path by path:

- **Badge and `==` scan.** The scan counted the backslashes before every character to decide
  whether it was written bare. It now does so only for `[`, `]` and `=`, whose backslash runs are
  disjoint, builds a badge's text only once its `]` is found, and skips the per-character pass
  for text with no `[` or `=`. Contiguous literal text events are one tree leaf.
- **Escaping.** The digit-prefix test before `.` and `)` re-read the line start for every dot;
  the prefix length is now counted once. Escapes and tokens are written into one buffer.
- **Flanking fixes.** The fix loop swept every token until nothing changed and rendered a whole
  text to read its first character; it now re-checks only the neighbours of a token that
  changed, and renders a token's edges once, when asked.
- **Mark runs.** How far each mark extends is computed once, backwards, instead of rescanning
  from every node.
- **Escaping by trying the parse.** A container whose minimal and full forms are the same bytes
  skips its trial parse; the whole-document comparison no longer clones both trees.
- **Columns and compare fences.** They count their columns before reading any column, so a
  fence of a million sections costs one scan.
- **Validation.** A path string is built only for the failing node, node types are found by hash,
  content expressions are compiled once, and `markdown::validate` tracks source positions only
  when it has an error to place.
- **`patch`.** It counts every match but keeps the text of the first 20 only.
- **Nesting.** Deep block quotes, lists and emphasis overflowed the stack; nested custom fences
  re-read their content once per level. Both are now bounded by the limits of `spec/FORMAT.md`
  rules 8 and 9 and section 2.6, so the fence ladder (8 nested callouts around 8 MiB of lines)
  reads the content at most 9 times.

Linear is not the same as fast: the event-dense inputs (links, marks, paragraphs, the fence
ladder) still take 4 to 5 s at 8 MiB on that machine, because each markdown event becomes a
`Node` with its own allocations. A server that must answer within 1 s should cap the size of a
write it validates; 1 MiB costs at most 0.6 s for every input above.

## The WebAssembly build

```sh
mise run wasm
```

`scripts/build-wasm.sh` runs wasm-pack for the web target into `src/lib/encre-wasm` and removes
the `.gitignore` wasm-pack writes. It sets `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS` to
`-Ccodegen-units=1` unless the variable is already set. That override matters on machines whose
`~/.cargo/config.toml` sets `build.rustflags` for the host, for example `-Wl,-rpath` for GTK:
the target variable replaces those flags, and without it the wasm link fails. The default also
remaps the toolchain's library source to `/rustc/<commit>` and `CARGO_HOME` to `/cargo`, so the
panic locations inside the module name no local path and CI builds the same bytes as a laptop. wasm-opt is off in
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
`Cargo.lock`, rebuild `src/lib/encre-wasm`, add the `CHANGELOG.md` entry, pass both gates, commit
`chore: release x.y.z`, then tag `vx.y.z` and `go/vx.y.z` together on that commit. Consumers pin
`vx.y.z` in Cargo and in `bun add`; `go get github.com/FacileStudio/encre/go@vx.y.z` resolves
only through the `go/` tag. The release workflow runs on `v*` tags only, so the `go/` tag starts
no second release.

Back to the [documentation index](README.md).
