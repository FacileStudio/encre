<script lang="ts">
  import { IconButton, Menu, Tooltip } from '@facile/muse';
  import type { Editor } from '@tiptap/core';
  import { blockCommands, type BlockCommand } from './blocks.js';
  import { BADGE_TONES, labels as french, type Labels } from './labels.js';
  import MarkButtons from './MarkButtons.svelte';

  let {
    editor,
    version,
    labels = french,
    commands,
    class: className = ''
  }: { editor: Editor; version: number; labels?: Labels; commands?: BlockCommand[]; class?: string } = $props();

  const blockItems = $derived((commands ?? blockCommands(labels)).map((block) => ({ label: block.label, onselect: () => block.run(editor) })));
  const run = (go: (c: ReturnType<Editor['chain']>) => ReturnType<Editor['chain']>) => () => go(editor.chain().focus()).run();
  const tableItems = $derived([
    { label: labels.addRow, onselect: run((c) => c.addRowAfter()) },
    { label: labels.addColumn, onselect: run((c) => c.addColumnAfter()) },
    { separator: true as const },
    { label: labels.deleteRow, onselect: run((c) => c.deleteRow()), tone: 'danger' as const },
    { label: labels.deleteColumn, onselect: run((c) => c.deleteColumn()), tone: 'danger' as const },
    { label: labels.deleteTable, onselect: run((c) => c.deleteTable()), tone: 'danger' as const }
  ]);
  const columnItems = $derived([
    { label: labels.addSide, onselect: run((c) => c.addSide()) },
    { label: labels.deleteSide, onselect: run((c) => c.deleteSide()), tone: 'danger' as const }
  ]);
  const badgeItems = $derived(BADGE_TONES.map((tone) => ({ label: labels.tones[tone], onselect: run((c) => c.setBadge({ tone })) })));

  const state = $derived.by(() => {
    void version;
    return {
      current: currentBlock(),
      inTable: editor.isActive('table'),
      inColumns: editor.isActive('columns'),
      inBadge: editor.isActive('badge'),
      canUndo: editor.can().undo(),
      canRedo: editor.can().redo()
    };
  });

  function currentBlock(): string {
    for (const level of [1, 2, 3]) if (editor.isActive('heading', { level })) return labels.heading(level);
    const names = ['bulletList', 'orderedList', 'taskList', 'blockquote', 'codeBlock', 'diagram', 'callout'] as const;
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
  <MarkButtons {editor} {version} {labels} />
  {#if state.inTable}
    <Menu label={labels.table} items={tableItems} />
  {/if}
  {#if state.inColumns}
    <Menu label={labels.columns} items={columnItems} />
  {/if}
  {#if state.inBadge}
    <Menu label={labels.badge} items={badgeItems} />
  {/if}
</div>
