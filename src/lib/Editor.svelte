<script module lang="ts">
  export type { ContextMenuInfo, EditorProps } from './editor-props.js';
</script>

<script lang="ts">
  import { Editor } from '@tiptap/core';
  import type { EditorProps } from './editor-props.js';
  import type { EncreNode } from './schema.js';
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
  import { embedCommand, fileCommand, pasteEmbed, transferFiles, type FileUpload } from './files.js';
  import * as select from './selection.js';

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
    onfile,
    onembed,
    resolveEmbed,
    oncontextmenu,
    extras,
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
  const files = (): FileUpload | undefined => (onfile ? { onfile, onerror } : undefined);
  const turnable = $derived(blockCommands(copy));
  const commands = $derived([
    ...turnable,
    ...(onimage ? [imageCommand(upload, copy)] : []),
    embedCommand(onerror, copy),
    ...(onfile ? [fileCommand(files, copy)] : [])
  ]);

  function paste(event: ClipboardEvent): boolean {
    if (!editor) return false;
    const text = event.clipboardData?.getData('text/plain') ?? '';
    return (
      pasteImages(editor, event, upload()) ||
      transferFiles(editor, event.clipboardData, files()) ||
      pasteEmbed(editor, text, onembed) ||
      pasteMarkdown(editor, event, onerror)
    );
  }

  function drop(event: DragEvent, moved: boolean): boolean {
    if (!editor || moved) return false;
    if (dropImages(editor, event, upload())) return true;
    const handled = transferFiles(editor, event.dataTransfer, files());
    if (handled) event.preventDefault();
    return handled;
  }

  function contextmenu(event: MouseEvent): boolean {
    if (!editor || !oncontextmenu) return false;
    event.preventDefault();
    oncontextmenu(select.contextInfo(editor, event));
    return true;
  }

  onMount(() => {
    const instance = new Editor({
      element,
      extensions: [
        ...encreExtensions({
          placeholder,
          labels: copy,
          renderDiagram,
          resolveEmbed,
          collaboration: ydoc ? { document: ydoc, provider, user } : undefined
        }),
        slashExtension(slash, () => commands),
        UploadPlaceholder
      ],
      content: ydoc ? null : (content ?? null),
      editable,
      editorProps: {
        attributes: { class: 'encre-prose outline-none min-h-40' },
        handlePaste: (_view, event) => paste(event),
        handleDrop: (_view, event, _slice, moved) => drop(event, moved),
        handleDOMEvents: { contextmenu: (_view, event) => contextmenu(event) }
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
    settings.resolveEmbed = resolveEmbed;
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

  /** The selection as Facile Markdown: the text range inside one block, or every block it touches. */
  export const selectionMarkdown = (): string => (editor ? select.selectionMarkdown(editor) : '');

  /** Runs the block command `commandId` (`heading2`, `embed`, `file`, ...); false when there is none. */
  export const run = (commandId: string): boolean => runFrom(commands, commandId);

  /** Turns the selected blocks into the block of `blockCommands` named `id`; false when there is none. */
  export const turnInto = (id: string): boolean => runFrom(turnable, id);

  /** Copies the top-level block under the selection right after it. */
  export const duplicateBlock = (): boolean => (editor ? select.duplicateBlock(editor) : false);

  /** Deletes the selection, or the top-level block under an empty one. */
  export const deleteSelection = (): boolean => (editor ? select.deleteSelection(editor) : false);

  /** Copies `<base>#<id>` of the top-level block under the selection and answers that block id. */
  export const copyBlockLink = (base: string): string | null => (editor ? select.copyBlockLink(editor, base) : null);

  const runFrom = (list: typeof commands, id: string): boolean => {
    const command = list.find((c) => c.id === id);
    if (!editor || !command) return false;
    command.run(editor);
    return true;
  };

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
    <BubbleMenu {editor} {version} labels={copy} commands={turnable} {extras} />
    <SlashMenu {slash} labels={copy} />
  {/if}
</div>
