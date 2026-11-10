<script module lang="ts">
  import type { EncreNode } from './schema.js';

  /** Props of the encre editor. */
  export interface EditorProps {
    content?: EncreNode;
    editable?: boolean;
    placeholder?: string;
    toolbar?: boolean;
    onchange?: (doc: EncreNode) => void;
    class?: string;
  }
</script>

<script lang="ts">
  import { Editor } from '@tiptap/core';
  import { onMount } from 'svelte';
  import BubbleMenu from './BubbleMenu.svelte';
  import { encreExtensions } from './extensions/index.js';
  import { SlashState, slashExtension } from './slash.svelte.js';
  import SlashMenu from './SlashMenu.svelte';
  import Toolbar from './Toolbar.svelte';

  let { content, editable = true, placeholder, toolbar = true, onchange, class: className = '' }: EditorProps = $props();

  let element: HTMLDivElement;
  let editor = $state<Editor | null>(null);
  let version = $state(0);
  const slash = new SlashState();

  onMount(() => {
    const instance = new Editor({
      element,
      extensions: [...encreExtensions({ placeholder }), slashExtension(slash)],
      content: content ?? null,
      editable,
      editorProps: { attributes: { class: 'encre-prose outline-none min-h-40' } },
      onTransaction: () => (version += 1),
      onUpdate: ({ editor: changed }) => onchange?.(changed.getJSON() as EncreNode)
    });
    editor = instance;
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

  /** Replaces the whole document. */
  export function setDocument(doc: EncreNode): void {
    editor?.commands.setContent(doc, { emitUpdate: true });
  }
</script>

<div class="flex flex-col gap-3 {className}">
  {#if editor && toolbar && editable}
    <Toolbar {editor} {version} class="sticky top-0 z-10" />
  {/if}
  <div bind:this={element} class="text-fc-fg"></div>
  {#if editor && editable}
    <BubbleMenu {editor} {version} />
    <SlashMenu {slash} />
  {/if}
</div>
