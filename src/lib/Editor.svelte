<script module lang="ts">
  import type * as Y from 'yjs';
  import type { AwarenessProvider, CollabUser } from './extensions/collaboration.js';
  import type { EncreNode } from './schema.js';
  import type { UploadedImage } from './upload.js';
  import type { RenderDiagram } from './extensions/settings.svelte.js';
  import type { LabelOverrides, Locale } from './labels.js';

  /** Props of the encre editor. */
  export interface EditorProps {
    content?: EncreNode;
    editable?: boolean;
    /** Text shown in an empty document, read once at mount. */
    placeholder?: string;
    toolbar?: boolean;
    onchange?: (doc: EncreNode) => void;
    class?: string;
    /** The shared document; when set, it is the source of truth and `content` is ignored. Read once at mount, with `provider` and `user`: wrap the editor in `{#key ydoc}` to switch documents. */
    ydoc?: Y.Doc;
    /** A y-websocket `WebsocketProvider` (or a stand-in with `awareness`, `synced` and the `sync` event) for carets and presence. */
    provider?: AwarenessProvider;
    /** This user, as other people see them. */
    user?: CollabUser;
    /** Where the editor shows who is here: `inline` above the toolbar, or `none` for an app that renders `Presence` itself. */
    presence?: 'inline' | 'none';
    /** Uploads a pasted, dropped or picked image and answers its address; without it images are not uploaded. */
    onimage?: (file: File) => Promise<UploadedImage>;
    /** Reports a refused image, a failed upload, a paste encre could not read as blocks, or a refused export. */
    onerror?: (error: Error) => void;
    /** The largest image, in bytes, handed to `onimage`. */
    maxImageBytes?: number;
    /** The language of the editor's copy: menus, node views and placeholders. */
    locale?: Locale;
    /** Replaces some of that copy. */
    labels?: LabelOverrides;
    /** Draws a diagram from its language and source; without it a diagram shows its source. */
    renderDiagram?: RenderDiagram;
  }
</script>

<script lang="ts">
  import { Editor } from '@tiptap/core';
  import { onMount } from 'svelte';
  import BubbleMenu from './BubbleMenu.svelte';
  import Presence from './Presence.svelte';
  import { encreExtensions, settingsOf } from './extensions/index.js';
  import { SlashState, slashExtension } from './slash.svelte.js';
  import SlashMenu from './SlashMenu.svelte';
  import Toolbar from './Toolbar.svelte';
  import { pasteMarkdown } from './paste.js';
  import { blockCommands } from './blocks.js';
  import { labelsFor } from './labels.js';
  import { UploadPlaceholder } from './extensions/upload-placeholder.js';
  import { dropImages, imageCommand, pasteImages, type ImageUpload } from './upload.js';
  import { loadEncre, parseMarkdown, serializeCommonMark, serializeMarkdown } from './wasm.js';
  import { blockPosition, positionOf, reveal } from './reveal.js';
  import { reported } from './refusal.js';

  let {
    content,
    editable = true,
    placeholder,
    toolbar = true,
    onchange,
    class: className = '',
    ydoc,
    provider,
    user,
    presence = 'inline',
    onimage,
    onerror,
    maxImageBytes,
    locale = 'fr',
    labels: overrides,
    renderDiagram
  }: EditorProps = $props();

  let element: HTMLDivElement;
  let bar = $state<HTMLDivElement | null>(null);
  let editor = $state<Editor | null>(null);
  let version = $state(0);
  const slash = new SlashState();
  const upload = (): ImageUpload | undefined => (onimage ? { onimage, onerror, maxImageBytes } : undefined);
  const copy = $derived(labelsFor(locale, overrides));
  const commands = $derived(onimage ? [...blockCommands(copy), imageCommand(upload, copy)] : blockCommands(copy));

  onMount(() => {
    const instance = new Editor({
      element,
      extensions: [
        ...encreExtensions({
          placeholder,
          labels: copy,
          renderDiagram,
          collaboration: ydoc ? { document: ydoc, provider, user } : undefined
        }),
        slashExtension(slash, () => commands),
        UploadPlaceholder
      ],
      content: ydoc ? null : (content ?? null),
      editable,
      editorProps: {
        attributes: { class: 'encre-prose outline-none min-h-40' },
        handlePaste: (_view, event) => (editor ? pasteImages(editor, event, upload()) || pasteMarkdown(editor, event, onerror) : false),
        handleDrop: (_view, event, _slice, moved) => (editor && !moved ? dropImages(editor, event, upload()) : false)
      },
      onTransaction: () => (version += 1),
      onUpdate: ({ editor: changed }) => onchange?.(changed.getJSON() as EncreNode)
    });
    editor = instance;
    loadEncre().catch(() => undefined);
    return () => instance.destroy();
  });

  $effect(() => {
    editor?.setEditable(editable);
  });

  $effect(() => {
    if (!editor) return;
    const settings = settingsOf(editor);
    settings.labels = copy;
    settings.renderDiagram = renderDiagram;
  });

  /** The Tiptap editor, once mounted. */
  export function instance(): Editor | null {
    return editor;
  }

  /** The current document as ProseMirror JSON. */
  export function getDocument(): EncreNode | null {
    return (editor?.getJSON() as EncreNode | undefined) ?? null;
  }

  /** Replaces the whole document with Facile Markdown. */
  export async function importMarkdown(markdown: string): Promise<void> {
    await loadEncre();
    setDocument(parseMarkdown(markdown));
  }

  const empty: EncreNode = { type: 'doc', content: [{ type: 'paragraph' }] };

  /** The document as canonical Facile Markdown; a refusal is an `EncreDocumentError`, also handed to `onerror`. */
  export async function exportMarkdown(): Promise<string> {
    await loadEncre();
    return reported(() => serializeMarkdown(getDocument() ?? empty), onerror);
  }

  /** The document as portable CommonMark; a refusal is an `EncreDocumentError`, also handed to `onerror`. */
  export async function exportCommonMark(): Promise<string> {
    await loadEncre();
    return reported(() => serializeCommonMark(getDocument() ?? empty), onerror);
  }

  /** Selects and scrolls to the node at a JSON path such as an `EncreDocumentError`'s `path`, below the sticky toolbar; false when it is not there. */
  export function revealPath(path: string): boolean {
    return editor ? reveal(editor, positionOf(editor.state.doc, path), bar?.offsetHeight) : false;
  }

  /** Selects and scrolls to the block whose id is `blockId`, below the sticky toolbar; false when no block has it. */
  export function revealBlock(blockId: string): boolean {
    return editor ? reveal(editor, blockPosition(editor.state.doc, blockId), bar?.offsetHeight) : false;
  }

  /** Replaces the whole document. */
  export function setDocument(doc: EncreNode): void {
    editor?.commands.setContent(doc, { emitUpdate: true });
  }
</script>

<div class="flex flex-col gap-3 {className}">
  {#if provider && presence === 'inline'}
    <Presence {provider} />
  {/if}
  {#if editor && toolbar && editable}
    <div bind:this={bar} class="sticky top-0 z-10">
      <Toolbar {editor} {version} {commands} labels={copy} />
    </div>
  {/if}
  <div bind:this={element} class="text-fc-fg"></div>
  {#if editor && editable}
    <BubbleMenu {editor} {version} labels={copy} />
    <SlashMenu {slash} labels={copy} />
  {/if}
</div>
