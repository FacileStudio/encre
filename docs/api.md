# encre — API

The public surface of the Rust crate `encre`, the `encre-wasm` entry points, and the npm package
`@facile/encre`. encre is a library and serves no HTTP routes, so there is no `/docs` reference.
rustdoc (`cargo doc -p encre --all-features --open`) has the full signatures.

## Rust: crate root

| Item | What it does |
|---|---|
| `Node`, `Mark` | A ProseMirror JSON node and mark; `Node::new`, `Node::text`, `attr_str`, `id`, `without_ids` |
| `SCHEMA_VERSION` | `2` |
| `validate(&Node) -> Result<(), ValidationError>` | Schema v2 and the rules of `spec/FORMAT.md` section 1; the error names a path |
| `assign_ids(&mut Node)` | Gives every block without a valid, unique id a fresh one |
| `is_block_id(&str) -> bool` | Ten characters of lowercase Crockford base32 |
| `compact(&Node) -> Node` | The canonical stored form: no nulls, no defaults, marks in rank order |
| `migrate(Node, from: u32) -> Result<Node, MigrationError>` | Upgrades a stored document to `SCHEMA_VERSION`; from 1 it is the identity |
| `schema::schema() -> &Schema` | The parsed `spec/schema.json`, with `node`, `mark`, `mark_rank` |
| `schema::NODE_KINDS`, `schema::MARK_KINDS` | Every node and mark name |

## Rust: `markdown`

| Item | What it does |
|---|---|
| `parse(&str) -> Result<Node, ParseError>` | Facile Markdown to a document without ids; refuses only markdown past the nesting limits of `spec/FORMAT.md` section 2.6 |
| `validate(&str) -> Result<Node, SourceError>` | Parses, then validates; `SourceError` has `path`, `message` and the 1-based `line` and `column` of the block |
| `serialize(&Node) -> String` | Canonical Facile Markdown |
| `serialize_commonmark(&Node) -> String` | Portable CommonMark with GFM, lossy |
| `plain_text(&Node) -> String` | The text alone, for search and previews |

## Rust: `ops`

| Item | What it does |
|---|---|
| `Op` | The operations, serde-tagged by `op` with unknown fields refused |
| `apply(&mut Node, version: u64, &Op) -> Result<Outcome, OpError>` | Applies one operation; on error the tree is unchanged |
| `apply_yjs(&mut TransactionMut, &XmlFragmentRef, version, &Op)` | The same on an open document, in the caller's transaction (feature `yjs`) |
| `Outcome` | `Markdown(String)`, `Blocks(Vec<BlockView>)` or `Changed(Vec<String>)`, the inserted ids |
| `BlockView` | `{ id, type, markdown }` of one top-level block |
| `OpError`, `PatchMatch`, `LISTED` | Typed refusals; the message says how to narrow the request. `PatchMatches` carries the match `count` and the first `LISTED` (20) matches |

## Rust: `yjs` (feature `yjs`)

| Item | What it does |
|---|---|
| `FRAGMENT` | `"default"` |
| `new_doc() -> yrs::Doc` | A document with UTF-16 offsets, as JavaScript clients use |
| `read(&impl ReadTxn, &XmlFragmentRef) -> Result<Node, YjsError>` | The fragment as a compact document |
| `write(&mut TransactionMut, &XmlFragmentRef, &Node)` | Replaces the fragment content with a valid document |
| `from_update(&[u8])`, `to_update(&Node)` | A v1 update to a document and back |

## Rust: `typst` and `pdf`

| Item | What it does |
|---|---|
| `typst::render(&Node, &RenderOptions) -> String` | A complete Typst file over muse-typst; same input, same bytes |
| `typst::RenderOptions`, `typst::ImageResolver` | Layout options, see [configuration.md](configuration.md) |
| `typst::escape_string(&str) -> String` | A Typst string literal body, Ardoise's escaping |
| `pdf::compile(&str, &Assets) -> Result<Vec<u8>, PdfError>` | Typst source to PDF bytes, in memory (feature `pdf`) |
| `pdf::Assets` | Files the document may read: `new`, `insert(path, bytes)` |

## Rust: `openui` (feature `openui`)

| Item | What it does |
|---|---|
| `to_document(&str) -> Result<Node, Error>` | An OpenUI Lang program as a document without ids |
| `to_blocks(&str) -> Result<Vec<Node>, Error>` | The same blocks, to place in an existing document |
| `Error` | `Syntax { line, message }`, `NoRoot`, `Undefined`, `Cycle`, `UnknownComponent`, `Argument`, `TooLarge`, `TooDeep` (references resolve past 64 levels), `Empty` |

## encre-wasm

Documents cross the boundary as JSON strings, and errors are thrown as `Error` with the Rust
message. `src/lib/wasm.ts` wraps these; call those wrappers rather than the raw exports.

| Export | Rust |
|---|---|
| `parse(markdown)` | `markdown::parse` |
| `serialize(doc)`, `serialize_commonmark(doc)` | validate, then `markdown::serialize` or `serialize_commonmark` |
| `validate(doc)` | `validate` |
| `validateMarkdown(markdown)` | `markdown::validate`, as the JSON `{ valid, errors: [{ path, message, line, column }] }` that `encre validate --from markdown --json` prints |
| `openuiToDocument(source)` | `openui::to_document` |

## TypeScript: `@facile/encre`

| Export | What it does |
|---|---|
| `Editor` (props `EditorProps`) | The editor; methods `instance`, `getDocument`, `setDocument`, `importMarkdown`, `exportMarkdown`, `exportCommonMark`, `revealPath`, `revealBlock` |
| `Document` | Read-only renderer: `doc`, `class`, `locale`, `labels`, `renderDiagram`, `highlight` |
| `Toolbar`, `BubbleMenu`, `SlashMenu`, `Presence` | The editor's parts, for apps that assemble their own |
| `SlashState`, `slashExtension`, `BLOCK_COMMANDS`, `blockCommands(labels)`, `findBlocks` | The slash menu's state and the block commands, French or in the given labels |
| `encreExtensions`, `EncreExtensionOptions` | Every Tiptap extension of the schema, optional collaboration |
| `Callout`, `Chart`, `Stat`, `Diagram`, `Columns`, `Column`, `Badge`, `Table`, `TableRow`, `TableHeader`, `TableCell` | The custom Tiptap nodes |
| `Settings`, `settingsOf`, `EncreSettings`, `RenderDiagram` | The extension that carries labels and the diagram renderer to node views |
| `CATALOGUES`, `labelsFor`, `Labels`, `LabelOverrides`, `Locale` | The interface copy in `fr`, `en`, `es` and `de`, and overrides |
| `Highlight` | The type of `Document`'s `highlight` hook |
| `FRAGMENT`, `caretColor`, `peers`, `CollabUser`, `CollabOptions`, `AwarenessProvider`, `Peer` | Collaboration helpers and types; `caretColor(clientId)` is a `#rrggbb` muse chart color, the only form y-tiptap draws |
| `schema`, `SCHEMA_VERSION`, `BLOCK_ID_TYPES`, `EncreNode` and the spec types | `spec/schema.json` typed |
| `compactDocument` | The compact form, as `compact` in Rust |
| `newBlockId`, `isBlockId` | Block ids from `crypto.getRandomValues` |
| `linkHref`, `imageSrc` | A link or image source when its scheme is safe to follow, else `undefined` |
| `loadEncre`, `encreReady` | Load encre-wasm once, and whether it is loaded; after a failed load, the next call tries again |
| `parseMarkdown`, `serializeMarkdown`, `serializeCommonMark`, `validateDocument` | The markdown projection in the browser; the two serializers throw an `EncreDocumentError` on an invalid document |
| `validateMarkdown(markdown) -> MarkdownValidation` | `{ valid, errors: MarkdownProblem[] }`, each problem `{ path, message, line, column }`; the same verdict as `encre validate --from markdown --json`, except that the wasm has no size limit: the CLI and Go refuse past `--max-bytes` |
| `EncreMarkdownError` | What `parseMarkdown` throws: an `Error` with the Rust message unchanged, plus `line`, `column` and `path` when the source position is known |
| `EncreDocumentError` | What `serializeMarkdown` and `serializeCommonMark` throw: an `Error` with the Rust message unchanged, plus `path` (`doc.content[1]`) and `blockId`, the id of the failing node or of the nearest enclosing block that has one |
| `openuiToDocument` | An OpenUI program as a document without ids; throws with the reason |
| `looksLikeMarkdown` | The paste heuristic the editor uses |
| `IMAGE_TYPES`, `imageRefusal(file, max?, labels?)`, `ImageUpload`, `UploadedImage` | The image types the editor uploads, the check it runs before `onimage`, and their types |

The package also exports `@facile/encre/schema.json` and `@facile/encre/styles`.

### `Editor` props

| Prop | Type | Default | What it does |
|---|---|---|---|
| `content` | `EncreNode` | empty | The initial document, ignored when `ydoc` is set |
| `editable` | `boolean` | `true` | Read-only when false |
| `placeholder` | `string` | none | Text shown in an empty document |
| `toolbar` | `boolean` | `true` | Shows the toolbar |
| `onchange` | `(doc: EncreNode) => void` | none | Called after every change |
| `class` | `string` | `''` | Classes on the wrapper |
| `ydoc`, `provider`, `user` | | none | Collaboration, see `CollabOptions` |
| `presence` | `'inline' \| 'none'` | `'inline'` | `inline` shows who is here above the toolbar; `none` leaves it to the app |
| `onimage` | `(file: File) => Promise<{ src: string; alt?: string }>` | none | Uploads an image and answers its address |
| `onerror` | `(error: Error) => void` | none | A refused image, a failed upload, an `EncreMarkdownError` from a markdown paste encre could not read (the text lands literally), or an `EncreDocumentError` from `exportMarkdown` or `exportCommonMark` |
| `maxImageBytes` | `number` | no limit | The largest file handed to `onimage` |
| `locale` | `'fr' \| 'en' \| 'es' \| 'de'` | `'fr'` | The language of every string in the toolbar, menus, node views and placeholders |
| `labels` | `LabelOverrides` | none | Replaces some of those strings; `tones` may name only some tones |
| `renderDiagram` | `(language, source) => Promise<string \| SVGElement>` | none | Draws a diagram; without it the diagram shows its source |

### Image upload

Without `onimage` the editor uploads nothing and behaves as before. With it:

- Pasting or dropping image files calls `onimage` once per file. A pasted image lands at the
  caret, a dropped one at the drop point.
- The slash menu and the toolbar block menu gain an "Image" entry that opens a file picker.
- While the hook runs, a local "Envoi de l'image…" placeholder marks the spot. It is a
  ProseMirror decoration, so it never enters the document or reaches collaborators. When the
  hook resolves, an `image` node with the returned `src` and `alt` replaces it.
- Before calling the hook the editor refuses anything but PNG, JPEG, GIF and WebP (never SVG),
  and files above `maxImageBytes`. Refusals, a rejected promise and an empty `src` all reach
  `onerror`, and the placeholder goes away.

```svelte
<Editor
  {content}
  maxImageBytes={10 * 1024 * 1024}
  onimage={async (file) => {
    const body = new FormData();
    body.append('file', file);
    const res = await fetch('/api/attachments', { method: 'POST', body });
    if (!res.ok) throw new Error(`upload failed: ${res.status}`);
    const { url } = await res.json();
    return { src: url, alt: file.name };
  }}
  onerror={(error) => toast.error(error.message)}
/>
```

### Export refusals

A document can hold a node the schema refuses, for example a control character in a text node.
`exportMarkdown` and `exportCommonMark` then hand an `EncreDocumentError` to `onerror` and reject
with it. Its `path` names the failing node and `blockId` the block around it, when that block has
an id. `revealPath(path)` and `revealBlock(blockId)` select that node and scroll to it, below the
sticky toolbar; each returns false when the node is not in the document.

```svelte
<script lang="ts">
  import { Editor, EncreDocumentError } from '@facile/encre';
  let editor: ReturnType<typeof Editor>;
</script>

<Editor
  bind:this={editor}
  onerror={(error) => {
    toast.error(error.message);
    if (error instanceof EncreDocumentError) editor.revealPath(error.path);
  }}
/>
```

### Presence in the page header

`presence="none"` stops the editor from drawing avatars. The app renders the exported
`Presence` with the same provider wherever it wants, for example in its page header:

```svelte
<script lang="ts">
  import { Editor, Presence } from '@facile/encre';
  let { ydoc, provider, user } = $props();
</script>

<header class="flex items-center justify-between">
  <h1>Plan</h1>
  <Presence {provider} />
</header>
<Editor {ydoc} {provider} {user} presence="none" />
```

For a custom display, `peers(provider)` returns the announced users, this client first; read
it again on the awareness `change` event, as `Presence` does.

### `Document` props

| Prop | Type | Default | What it does |
|---|---|---|---|
| `doc` | `EncreNode` | required | The document |
| `class` | `string` | `''` | Classes on the `article` |
| `locale`, `labels` | as `Editor` | `'fr'` | The copy the renderer shows (the empty chart label) |
| `renderDiagram` | `(language, source) => Promise<string \| SVGElement>` | none | Draws a `diagram`. Without it, or when it rejects, the diagram is a code block under its language |
| `highlight` | `(code, language) => string` | none | Returns HTML for a code block. **The app must sanitize it**: encre inserts it as is. Without it, encre writes the code as escaped text |

A `codeBlock` with language `diff` always gets line tones: `+` lines (not `+++`) are `encre-diff-add`,
`-` lines (not `---`) `encre-diff-del`, `@@` lines `encre-diff-hunk`, the rest `encre-diff-ctx`. It
does not go through `highlight`. An SVG string from `renderDiagram` is inserted as is too, so the
renderer must be safe; mermaid's `securityLevel: 'strict'` is.

```svelte
<Document
  {doc}
  locale="en"
  renderDiagram={async (language, source) => (await mermaid.render(newId(), source)).svg}
  highlight={(code, language) => DOMPurify.sanitize(hljs.highlight(code, { language, ignoreIllegals: true }).value)}
/>
```

Back to the [documentation index](README.md).
