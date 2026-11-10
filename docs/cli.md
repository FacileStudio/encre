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

In a Go service's Dockerfile:

```dockerfile
COPY encre-x86_64-unknown-linux-musl /usr/local/bin/encre
```

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
one line ending in a newline. JSON input is validated against schema v1 before any command uses
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
| `validate` | `{"valid": true, "errors": []}` or `{"valid": false, "errors": [{"path", "message"}]}` |
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
| 1 | Invalid input or document | Not schema v1, malformed JSON, too large, a refused operation, OpenUI syntax, a Typst compile failure |
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

## Calling it from Go

```go
package encre

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os/exec"
	"time"
)

// Error is an error encre reported, with its exit code.
type Error struct {
	Exit    int
	Code    string `json:"code"`
	Message string `json:"message"`
	Path    string `json:"path"`
}

func (e *Error) Error() string { return fmt.Sprintf("encre: %s (exit %d)", e.Message, e.Exit) }

// Invalid reports whether the input was at fault, so the caller answers 4xx, not 5xx.
func (e *Error) Invalid() bool { return e.Exit == 1 }

// Run runs encre with args, feeds it stdin, and returns stdout.
func Run(ctx context.Context, stdin []byte, args ...string) ([]byte, error) {
	ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
	defer cancel()
	cmd := exec.CommandContext(ctx, "encre", append(args, "--json")...)
	cmd.Stdin = bytes.NewReader(stdin)
	var stdout, stderr bytes.Buffer
	cmd.Stdout, cmd.Stderr = &stdout, &stderr
	err := cmd.Run()
	if ctx.Err() != nil {
		return nil, fmt.Errorf("encre %s: %w", args[0], ctx.Err())
	}
	var exit *exec.ExitError
	if errors.As(err, &exit) {
		var body struct {
			Error Error `json:"error"`
		}
		if json.Unmarshal(stderr.Bytes(), &body) != nil {
			return nil, fmt.Errorf("encre %s: exit %d: %s", args[0], exit.ExitCode(), stderr.String())
		}
		body.Error.Exit = exit.ExitCode()
		return nil, &body.Error
	}
	if err != nil {
		return nil, fmt.Errorf("encre %s: %w", args[0], err)
	}
	return stdout.Bytes(), nil
}

// Markdown turns an encre JSON document into Facile Markdown.
func Markdown(ctx context.Context, doc []byte) (string, error) {
	out, err := Run(ctx, doc, "serialize")
	if err != nil {
		return "", err
	}
	var body struct {
		Markdown string `json:"markdown"`
	}
	return body.Markdown, json.Unmarshal(out, &body)
}
```

- Pass `--json` on every call, so stderr is one JSON line you can decode.
- `validate --json` exits 1 with its verdict on stdout, not stderr: read `stdout` before
  treating exit 1 as an error, or call it outside `Run`.
- `exec.CommandContext` kills the process when the context ends. A PDF of a long document takes
  seconds, so give `pdf` its own timeout and limit how many run at once.
- For `pdf`, pass `--output` with a temporary file and read it back, or drop `--json` and take
  the PDF bytes from stdout.

Back to the [documentation index](README.md).
