<script lang="ts">
  import { IconButton, Menu, Tooltip } from '@facile/muse';
  import type { Editor } from '@tiptap/core';
  import { BLOCK_COMMANDS } from './blocks.js';
  import { labels } from './labels.js';
  import MarkButtons from './MarkButtons.svelte';

  let { editor, version, class: className = '' }: { editor: Editor; version: number; class?: string } = $props();

  const blockItems = BLOCK_COMMANDS.map((block) => ({ label: block.label, onselect: () => block.run(editor) }));
  const tableItems = [
    { label: labels.addRow, onselect: () => editor.chain().focus().addRowAfter().run() },
    { label: labels.addColumn, onselect: () => editor.chain().focus().addColumnAfter().run() },
    { separator: true as const },
    { label: labels.deleteRow, onselect: () => editor.chain().focus().deleteRow().run(), tone: 'danger' as const },
    { label: labels.deleteColumn, onselect: () => editor.chain().focus().deleteColumn().run(), tone: 'danger' as const },
    { label: labels.deleteTable, onselect: () => editor.chain().focus().deleteTable().run(), tone: 'danger' as const }
  ];

  const state = $derived.by(() => {
    void version;
    return {
      current: currentBlock(),
      inTable: editor.isActive('table'),
      canUndo: editor.can().undo(),
      canRedo: editor.can().redo()
    };
  });

  function currentBlock(): string {
    for (const level of [1, 2, 3]) if (editor.isActive('heading', { level })) return labels.heading(level);
    const names = ['bulletList', 'orderedList', 'taskList', 'blockquote', 'codeBlock', 'callout'] as const;
    const found = names.find((name) => editor.isActive(name));
    return found ? labels[found] : labels.paragraph;
  }
</script>

<div role="toolbar" aria-label={labels.toolbar} class="flex flex-wrap items-center gap-1 rounded-fc-md bg-fc-component p-1 {className}">
  <Tooltip text={labels.undo}>
    <IconButton variant="ghost" aria-label={labels.undo} disabled={!state.canUndo} onclick={() => editor.chain().focus().undo().run()}>
      <span aria-hidden="true">↶</span>
    </IconButton>
  </Tooltip>
  <Tooltip text={labels.redo}>
    <IconButton variant="ghost" aria-label={labels.redo} disabled={!state.canRedo} onclick={() => editor.chain().focus().redo().run()}>
      <span aria-hidden="true">↷</span>
    </IconButton>
  </Tooltip>
  <Menu label={state.current} items={blockItems} />
  <MarkButtons {editor} {version} />
  {#if state.inTable}
    <Menu label={labels.table} items={tableItems} />
  {/if}
</div>
