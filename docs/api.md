# encre — API

The public surface of the Rust crate `encre`, the `encre-wasm` entry points, and the npm package
`@facile/encre`. encre is a library and serves no HTTP routes, so there is no `/docs` reference.
rustdoc (`cargo doc -p encre --all-features --open`) has the full signatures.

## Rust: crate root

| Item | What it does |
|---|---|
| `Node`, `Mark` | A ProseMirror JSON node and mark; `Node::new`, `Node::text`, `attr_str`, `id`, `without_ids` |
| `SCHEMA_VERSION` | `1` |
| `validate(&Node) -> Result<(), ValidationError>` | Schema v1 and the rules of `spec/FORMAT.md` section 1; the error names a path |
| `assign_ids(&mut Node)` | Gives every block without a valid, unique id a fresh one |
| `is_block_id(&str) -> bool` | Ten characters of lowercase Crockford base32 |
| `compact(&Node) -> Node` | The canonical stored form: no nulls, no defaults, marks in rank order |
| `migrate(Node, from: u32) -> Result<Node, MigrationError>` | Upgrades a stored document to `SCHEMA_VERSION` |
| `schema::schema() -> &Schema` | The parsed `spec/schema.json`, with `node`, `mark`, `mark_rank` |
| `schema::NODE_KINDS`, `schema::MARK_KINDS` | Every node and mark name |

## Rust: `markdown`

| Item | What it does |
|---|---|
| `parse(&str) -> Result<Node, ParseError>` | Facile Markdown to a document without ids; accepts any input |
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
| `OpError`, `PatchMatch` | Typed refusals; the message says how to narrow the request |

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
| `Error` | `Syntax { line, message }`, `NoRoot`, `Undefined`, `Cycle`, `UnknownComponent`, `Argument`, `TooLarge`, `Empty` |

## encre-wasm

Documents cross the boundary as JSON strings, and errors are thrown as `Error` with the Rust
message. `src/lib/wasm.ts` wraps these; call those wrappers rather than the raw exports.

| Export | Rust |
|---|---|
| `parse(markdown)` | `markdown::parse` |
| `serialize(doc)`, `serialize_commonmark(doc)` | validate, then `markdown::serialize` or `serialize_commonmark` |
| `validate(doc)` | `validate` |
| `openuiToDocument(source)` | `openui::to_document` |

## TypeScript: `@facile/encre`

| Export | What it does |
|---|---|
| `Editor` (props `EditorProps`) | The editor; methods `instance`, `getDocument`, `setDocument`, `importMarkdown`, `exportMarkdown`, `exportCommonMark` |
| `Document` | Read-only renderer: `doc`, `class` |
| `Toolbar`, `BubbleMenu`, `SlashMenu`, `Presence` | The editor's parts, for apps that assemble their own |
| `SlashState`, `slashExtension`, `BLOCK_COMMANDS`, `findBlocks` | The slash menu's state and the block commands |
| `encreExtensions`, `EncreExtensionOptions` | Every Tiptap extension of the schema, optional collaboration |
| `Callout`, `Chart`, `Stat`, `Table`, `TableRow`, `TableHeader`, `TableCell` | The custom Tiptap nodes |
| `FRAGMENT`, `caretColor`, `peers`, `CollabUser`, `CollabOptions`, `AwarenessProvider`, `Peer` | Collaboration helpers and types |
| `schema`, `SCHEMA_VERSION`, `BLOCK_ID_TYPES`, `EncreNode` and the spec types | `spec/schema.json` typed |
| `compactDocument` | The compact form, as `compact` in Rust |
| `newBlockId`, `isBlockId` | Block ids from `crypto.getRandomValues` |
| `linkHref`, `imageSrc` | A link or image source when its scheme is safe to follow, else `undefined` |
| `loadEncre`, `encreReady` | Load encre-wasm once, and whether it is loaded |
| `parseMarkdown`, `serializeMarkdown`, `serializeCommonMark`, `validateDocument` | The markdown projection in the browser |
| `openuiToDocument` | An OpenUI program as a document without ids; throws with the reason |
| `looksLikeMarkdown` | The paste heuristic the editor uses |
| `IMAGE_TYPES`, `imageRefusal`, `ImageUpload`, `UploadedImage` | The image types the editor uploads, the check it runs before `onimage`, and their types |

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
| `onerror` | `(error: Error) => void` | none | A refused image or a failed upload |
| `maxImageBytes` | `number` | no limit | The largest file handed to `onimage` |

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

Back to the [documentation index](README.md).
