<script lang="ts">
  import { Menu } from '@facile/muse';
  import { BubbleMenuPlugin } from '@tiptap/extension-bubble-menu';
  import type { Editor } from '@tiptap/core';
  import { onMount, type Snippet } from 'svelte';
  import { blockCommands, type BlockCommand } from './blocks.js';
  import { labels as french, type Labels } from './labels.js';
  import MarkButtons from './MarkButtons.svelte';

  let {
    editor,
    version,
    labels = french,
    commands,
    extras
  }: {
    editor: Editor;
    version: number;
    labels?: Labels;
    /** The "Turn into" entries; every block command by default. */
    commands?: BlockCommand[];
    /** Rendered after the built-in buttons, such as an "Ask the assistant" button. */
    extras?: Snippet;
  } = $props();

  let element: HTMLDivElement;
  const key = `encreBubble${Math.random().toString(36).slice(2)}`;
  const items = $derived((commands ?? blockCommands(labels)).map((command) => ({ label: command.label, onselect: () => command.run(editor) })));

  onMount(() => {
    editor.registerPlugin(
      BubbleMenuPlugin({
        editor,
        element,
        pluginKey: key,
        shouldShow: ({ state }) => !state.selection.empty && editor.isEditable
      })
    );
    return () => editor.unregisterPlugin(key);
  });
</script>

<div bind:this={element} class="z-50 flex items-center gap-0.5 rounded-fc-md bg-fc-bg p-1 shadow-lg" style="visibility: hidden">
  <Menu label={labels.turnInto} {items} class="shrink-0" />
  <MarkButtons {editor} {version} {labels} />
  {@render extras?.()}
</div>
