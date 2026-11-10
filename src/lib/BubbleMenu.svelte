<script lang="ts">
  import { BubbleMenuPlugin } from '@tiptap/extension-bubble-menu';
  import type { Editor } from '@tiptap/core';
  import { NodeSelection } from '@tiptap/pm/state';
  import { onMount } from 'svelte';
  import type { Labels } from './labels.js';
  import MarkButtons from './MarkButtons.svelte';

  let { editor, version, labels }: { editor: Editor; version: number; labels?: Labels } = $props();

  let element: HTMLDivElement;
  const key = `encreBubble${Math.random().toString(36).slice(2)}`;

  onMount(() => {
    editor.registerPlugin(
      BubbleMenuPlugin({
        editor,
        element,
        pluginKey: key,
        shouldShow: ({ state, from, to }) => from !== to && !state.selection.empty && editor.isEditable && !(state.selection instanceof NodeSelection && state.selection.node.isAtom)
      })
    );
    return () => editor.unregisterPlugin(key);
  });
</script>

<div bind:this={element} class="z-50 flex items-center gap-0.5 rounded-fc-md bg-fc-bg p-1 shadow-lg" style="visibility: hidden">
  <MarkButtons {editor} {version} {labels} />
</div>
