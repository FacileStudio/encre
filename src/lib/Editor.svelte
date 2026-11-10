<script module lang="ts">
  import type * as Y from 'yjs';
  import type { AwarenessProvider, CollabUser } from './extensions/collaboration.js';
  import type { EncreNode } from './schema.js';
  import type { UploadedImage } from './upload.js';

  /** Props of the encre editor. */
  export interface EditorProps {
    content?: EncreNode;
    editable?: boolean;
    placeholder?: string;
    toolbar?: boolean;
    onchange?: (doc: EncreNode) => void;
    class?: string;
    /** The shared document; when set, it is the source of truth and `content` is ignored. */
    ydoc?: Y.Doc;
    /** A y-websocket `WebsocketProvider` (or a stand-in with `awareness`, `synced` and the `sync` event) for carets and presence. */
    provider?: AwarenessProvider;
    /** This user, as other people see them. */
    user?: CollabUser;
    /** Where the editor shows who is here: `inline` above the toolbar, or `none` for an app that renders `Presence` itself. */
    presence?: 'inline' | 'none';
    /** Uploads a pasted, dropped or picked image and answers its address; without it images are not uploaded. */
    onimage?: (file: File) => Promise<UploadedImage>;
    /** Reports a refused image or a failed upload. */
    onerror?: (error: Error) => void;
    /** The largest image, in bytes, handed to `onimage`. */
    maxImageBytes?: number;
  }
</script>

<script lang="ts">
  import { Editor } from '@tiptap/core';
  import { onMount } from 'svelte';
  import BubbleMenu from './BubbleMenu.svelte';
  import Presence from './Presence.svelte';
  import { encreExtensions } from './extensions/index.js';
  import { SlashState, slashExtension } from './slash.svelte.js';
  import SlashMenu from './SlashMenu.svelte';
  import Toolbar from './Toolbar.svelte';
  import { pasteMarkdown } from './paste.js';
  import { BLOCK_COMMANDS } from './blocks.js';
  import { UploadPlaceholder } from './extensions/upload-placeholder.js';
  import { dropImages, imageCommand, pasteImages, type ImageUpload } from './upload.js';
  import { loadEncre, parseMarkdown, serializeCommonMark, serializeMarkdown } from './wasm.js';

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
    maxImageBytes
  }: EditorProps = $props();

  let element: HTMLDivElement;
  let editor = $state<Editor | null>(null);
  let version = $state(0);
  const slash = new SlashState();
  const upload = (): ImageUpload | undefined => (onimage ? { onimage, onerror, maxImageBytes } : undefined);
  const commands = $derived(onimage ? [...BLOCK_COMMANDS, imageCommand(upload)] : BLOCK_COMMANDS);

  onMount(() => {
    const instance = new Editor({
      element,
      extensions: [
        ...encreExtensions({ placeholder, collaboration: ydoc ? { document: ydoc, provider, user } : undefined }),
        slashExtension(slash, () => commands),
        UploadPlaceholder
      ],
      content: ydoc ? null : (content ?? null),
      editable,
      editorProps: {
        attributes: { class: 'encre-prose outline-none min-h-40' },
        handlePaste: (_view, event) => (editor ? pasteImages(editor, event, upload()) || pasteMarkdown(editor, event) : false),
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

  /** The document as canonical Facile Markdown. */
  export async function exportMarkdown(): Promise<string> {
    await loadEncre();
    return serializeMarkdown(getDocument() ?? { type: 'doc', content: [{ type: 'paragraph' }] });
  }

  /** The document as portable CommonMark. */
  export async function exportCommonMark(): Promise<string> {
    await loadEncre();
    return serializeCommonMark(getDocument() ?? { type: 'doc', content: [{ type: 'paragraph' }] });
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
    <Toolbar {editor} {version} {commands} class="sticky top-0 z-10" />
  {/if}
  <div bind:this={element} class="text-fc-fg"></div>
  {#if editor && editable}
    <BubbleMenu {editor} {version} />
    <SlashMenu {slash} />
  {/if}
</div>
