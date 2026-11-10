<script module lang="ts">
  import type * as Y from 'yjs';
  import type { AwarenessProvider, CollabUser } from './extensions/collaboration.js';
  import type { EncreNode } from './schema.js';

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
    /** A y-websocket `WebsocketProvider` (or any object with an `awareness`) for carets and presence. */
    provider?: AwarenessProvider;
    /** This user, as other people see them. */
    user?: CollabUser;
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
    user
  }: EditorProps = $props();

  let element: HTMLDivElement;
  let editor = $state<Editor | null>(null);
  let version = $state(0);
  const slash = new SlashState();

  onMount(() => {
    const instance = new Editor({
      element,
      extensions: [
        ...encreExtensions({ placeholder, collaboration: ydoc ? { document: ydoc, provider, user } : undefined }),
        slashExtension(slash)
      ],
      content: ydoc ? null : (content ?? null),
      editable,
      editorProps: {
        attributes: { class: 'encre-prose outline-none min-h-40' },
        handlePaste: (_view, event) => (editor ? pasteMarkdown(editor, event) : false)
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
  {#if provider}
    <Presence {provider} />
  {/if}
  {#if editor && toolbar && editable}
    <Toolbar {editor} {version} class="sticky top-0 z-10" />
  {/if}
  <div bind:this={element} class="text-fc-fg"></div>
  {#if editor && editable}
    <BubbleMenu {editor} {version} />
    <SlashMenu {slash} />
  {/if}
</div>
