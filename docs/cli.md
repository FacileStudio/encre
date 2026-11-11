# encre — CLI

`encre` is the crate as a command-line tool, for services that are not written in Rust
(Mycelium, Bulle) and run it as a subprocess. It is offline: no network, no configuration,
no credentials, and it reads only the files its flags name.

## Build

```sh
cargo build --release -p encre-cli          # target/release/encre
mise run cli                                # static linux binaries in target/cli
```

`mise run cli` runs `scripts/build-cli.sh`. It builds `x86_64-unknown-linux-musl` and
`aarch64-unknown-linux-musl`, fully static and stripped, and writes:

| File | What it is |
|---|---|
| `target/cli/encre-x86_64-unknown-linux-musl` | The binary, for `COPY` into an image |
| `target/cli/encre-aarch64-unknown-linux-musl` | The same for arm64 |
| `target/cli/encre_<version>_linux_<arch>.tar.gz` | The binary at the archive root, suite release naming |
| `target/cli/checksums.txt` | `sha256sum` of the archives |

typst depends on the `psm` crate, which compiles a little C and assembly, so each target needs
a C compiler. The script takes `<arch>-linux-musl-gcc`, then `<arch>-linux-gnu-gcc`, then the
host `gcc` when the target matches the host. On Debian, `apt install gcc-aarch64-linux-gnu`
gives the arm64 one; the arm64 link itself uses the toolchain's `rust-lld`. A target with no
compiler is reported and skipped, and the script exits 1.
`ENCRE_CLI_TARGETS` picks other targets and `ENCRE_CLI_OUT` another directory.

In a Go service's Dockerfile, with a binary built on this machine:

```dockerfile
COPY encre-x86_64-unknown-linux-musl /usr/local/bin/encre
```

## Release binaries

Pushing a `v*` tag runs `.github/workflows/release.yml`. It checks that the tag equals the
crate version, builds both targets with `scripts/build-cli.sh` on `ubuntu-latest` (with
`musl-tools` and `gcc-aarch64-linux-gnu`), and creates the GitHub release with
`encre_<version>_linux_amd64.tar.gz`, `encre_<version>_linux_arm64.tar.gz` and
`checksums.txt`. The release notes are the version's `CHANGELOG.md` section.

A Dockerfile downloads the archive, checks it against `checksums.txt`, then extracts it:

```dockerfile
ARG ENCRE_VERSION=0.9.3
ARG TARGETARCH
RUN set -eu; base="https://github.com/FacileStudio/encre/releases/download/v${ENCRE_VERSION}"; \
    archive="encre_${ENCRE_VERSION}_linux_${TARGETARCH}.tar.gz"; cd /tmp; \
    curl -fsSLO "$base/$archive" && curl -fsSLO "$base/checksums.txt"; \
    grep " $archive\$" checksums.txt | sha256sum -c -; \
    tar -xzf "$archive" -C /usr/local/bin encre; rm "$archive" checksums.txt; encre --version
```

`TARGETARCH` is `amd64` or `arm64` under BuildKit, the same names as the archives.

## Commands

Every command reads stdin, or `--input <file>`, and writes stdout, or `--output <file>`.

| Command | Input | Output |
|---|---|---|
| `parse [--ids]` | Facile Markdown | encre JSON; `--ids` gives every block a fresh id |
| `serialize` | encre JSON | canonical Facile Markdown |
| `commonmark [--from]` | JSON or markdown | portable CommonMark with GFM, lossy |
| `plain-text [--from]` | JSON or markdown | the text alone |
| `validate [--from]` | JSON or markdown | nothing, or the verdict with `--json` |
| `compact` | encre JSON | encre JSON in the canonical stored form |
| `openui [--ids]` | an OpenUI Lang program | encre JSON |
| `ops --doc <file> [--from] [--doc-version N]` | a JSON array of operations | `{"doc","version","outcomes"}` |
| `typst [--from] [--title] [--footer] [--image-dir]` | JSON or markdown | Typst source over muse-typst |
| `pdf [--from] [--title] [--footer] [--image-dir]` | JSON or markdown | PDF bytes |

Text output is written exactly as encre produces it, with no trailing newline added, so
`encre parse | encre serialize` gives back the canonical markdown byte for byte. JSON output is
one line ending in a newline. JSON input is validated against schema v2 before any command uses
it.

### Global flags

| Flag | Default | What it does |
|---|---|---|
| `--input <file>` | stdin | Read the input from a file |
| `--output <file>` | stdout | Write the output to a file |
| `--json` | off | Machine-readable output and errors, see below |
| `--max-bytes <n>` | `8388608` (8 MiB) | Refuse an input, `--doc` or image file past this size, exit 1 |
| `--no-color` | off | No colour on stderr; colour is also off without a TTY or with `NO_COLOR` |
| `--version`, `--help` | | `encre <version>`; help with examples |

### Input format

`--from markdown` or `--from json` sets the format. Without it, `encre` decides:

1. Input whose first non-blank character is `{` and that parses as JSON is JSON.
2. Input whose first non-blank character is `{` and that does not parse as JSON is ambiguous:
   exit 2, pass `--from`.
3. Any other input is markdown.

`parse` always reads markdown, and `serialize` and `compact` always read JSON.

### `--json`

| Command | stdout with `--json` |
|---|---|
| `serialize`, `commonmark` | `{"markdown": "..."}` |
| `plain-text` | `{"text": "..."}` |
| `typst` | `{"typst": "..."}` |
| `validate` | `{"valid": true, "errors": []}` or `{"valid": false, "errors": [{"path", "message", "line", "column"}]}`; `line` and `column` only for markdown input |
| `pdf` | `{"output": "<file>", "bytes": n}`; needs `--output`, which gets the PDF |
| `parse`, `compact`, `openui`, `ops` | unchanged, already JSON |

Errors always go to stderr. With `--json` an error is one JSON line:

```json
{"error":{"code":"invalid","exit":1,"message":"delete_block: no top-level block has id zzz; read_blocks lists the block ids","path":"ops[0]"}}
```

`code` is `invalid`, `usage` or `internal`, and `path` is null when the error has no place in the
input. The one exception is `validate --json`, which writes an invalid verdict to stdout and
nothing to stderr.

### Exit codes

| Code | Meaning | Examples |
|---|---|---|
| 0 | Done | |
| 1 | Invalid input or document | Not schema v2, malformed JSON, too large, a refused operation, OpenUI syntax, a Typst compile failure |
| 2 | Usage | Unknown flag or command, unreadable `--input`, `--doc` or `--image-dir`, ambiguous input, `pdf --json` without `--output` |
| 3 | Internal | Output not writable, PDF export failure, a panic |

### `ops`

`--doc` names the document, JSON or markdown. A markdown document gets fresh block ids first,
so later operations in the same call can address its blocks. The operations come from stdin or
`--input`, as a JSON array of the objects `ops::Op` deserializes (see [agents.md](agents.md)).

They apply in order, all or nothing. The version starts at `--doc-version` (default 0) and goes
up by one after each write, which is what `replace_all` checks `expect_version` against. When
one operation is refused, `encre` prints nothing on stdout, exits 1, and the error's `path` is
`ops[i]`. `outcomes` holds one entry per operation: `{"markdown"}` for `read`, `{"blocks"}` for
`read_blocks`, `{"changed": [ids]}` for a write.

### Images in `typst` and `pdf`

Without `--image-dir`, every image renders as its alt text. With it, an image `src` is embedded
when all of these hold, and renders as alt text otherwise:

- it is a relative path: no scheme, no leading `/`, no `\`, no `..` segment;
- after following symlinks it names a regular file inside the directory;
- the file is at most `--max-bytes`.

The Typst source names an embedded image `/images/<src>`. `pdf` reads only those files.

### What `validate` refuses in markdown

Markdown input is parsed, and parsing fails only past the two nesting limits, so
`validate --from markdown` refuses only those, what the parsed tree cannot hold
(`spec/FORMAT.md` section 2.6) and what the CLI itself checks. Each
error names the tree `path` and the 1-based `line` and `column` where the block holding the problem
starts. Without `--json`, the message starts with `line L, column C:`.

- Bytes that are not UTF-8, and input larger than `--max-bytes`. These are errors on stderr
  (exit 1), not a verdict on stdout, and carry no position.
- A control character other than tab and line feed, NUL and escape included, in text or code
  (rule 1).
- A code span holding a backslash, a backslash and a pipe inside a table cell: GFM unescapes the
  last pair, so the code text keeps a backslash before a pipe, which a cell cannot write back
  (rule 6).
- Linked code holding `]:`, as in ``see [`a]:b`](u)``: at a paragraph start it would read as a
  link reference definition (rule 7).
- Blocks and inline spans nested more than 64 levels deep, and custom block fences that open more
  than 8 `callout`, `columns` and `column` nodes on one path. The path is `doc` and the position is
  the syntax that crosses the limit; `encre parse` refuses the same input with exit code 1.
- A document nested more than 48 levels deep, or holding more than 8 custom blocks on one path
  through GFM alerts (rules 8 and 9).

Everything else is valid. `\|` in code in a cell is a pipe. ``[`a]:b`](u)`` at the very start of a
paragraph is a link reference definition and leaves nothing. Malformed custom blocks become code
blocks, unknown badges and alerts stay text, and HTML is text.

## Calling it from Go

The Go module `github.com/FacileStudio/encre/go` (package `encre`, in this repository's `go/`)
wraps the binary. It runs `ENCRE_BIN` when set, else `encre` on `PATH`, always with `--json`.

```sh
go get github.com/FacileStudio/encre/go@v0.9.3
```

| Function | What it does |
|---|---|
| `Run(ctx, stdin, args...) ([]byte, error)` | Any command; stdout on success, a `*Error` on a refusal |
| `Validate(ctx, input, format) (Verdict, error)` | `Verdict{Valid, Errors []Problem{Path, Message, Line, Column}}`; an invalid document is not an error |
| `Parse(ctx, markdown, ids) (json.RawMessage, error)` | Facile Markdown to encre JSON |
| `Serialize(ctx, doc) (string, error)` | encre JSON to canonical Facile Markdown |
| `CommonMark(ctx, input, format) (string, error)` | Portable CommonMark |
| `PlainText(ctx, input, format) (string, error)` | The text alone |
| `PDF(ctx, input, format, PDFOptions{Title, Footer, ImageDir}) ([]byte, error)` | PDF bytes, through a temporary file |

`format` is `encre.Markdown` or `encre.JSON`. `*Error` carries `Exit`, `Code`, `Message` and `Path`;
`Invalid()` is true for exit 1, so the caller answers 4xx, not 5xx. A call whose context has no
deadline gets `DefaultTimeout` (30 s); give a long PDF its own deadline and limit how many run at
once.

```go
verdict, err := encre.Validate(ctx, body, encre.Markdown)
if err != nil {
	return err
}
if !verdict.Valid {
	p := verdict.Errors[0]
	return fmt.Errorf("line %d, column %d: %s", p.Line, p.Column, p.Message)
}
markdown, err := encre.Serialize(ctx, doc)
var refused *encre.Error
if errors.As(err, &refused) && refused.Invalid() {
	return badRequest(refused.Message)
}
```

The module's tests run against `ENCRE_BIN`, else `target/release/encre` or `target/debug/encre`,
and skip with a message when no binary is built.

Back to the [documentation index](README.md).
