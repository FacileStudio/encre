<script lang="ts">
  import { Menu } from '@facile/muse';
  import type { ColumnTone } from '../extensions/custom.js';
  import type { NodeViewProps } from '../extensions/node-view.svelte.js';
  import { COLUMN_TONES } from '../labels.js';
  import { columnTone } from '../render/tones.js';

  let { view, update, content, settings }: NodeViewProps = $props();

  const labels = $derived(settings.labels);
  const tone = $derived((COLUMN_TONES.includes(view.attrs.tone as ColumnTone) ? view.attrs.tone : 'neutral') as ColumnTone);
  const title = $derived(String(view.attrs.title ?? ''));
  const items = $derived(COLUMN_TONES.map((t) => ({ label: labels.tones[t], onselect: () => update({ tone: t }) })));

  function host(node: HTMLElement) {
    if (content) node.appendChild(content);
  }

  function rename(event: Event & { currentTarget: HTMLInputElement }) {
    update({ title: event.currentTarget.value.replace(/`|[^\P{Cc}\t]/gu, '') });
  }
</script>

<section class="flex min-w-0 flex-col gap-2 rounded-fc-md p-4 {columnTone[tone].box}">
  {#if view.editable}
    <div contenteditable="false" class="flex items-center gap-2">
      <input
        class="min-w-0 flex-1 rounded-fc-xs bg-transparent text-fc-sm font-semibold placeholder:text-fc-fg-muted focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring {columnTone[tone].title}"
        value={title}
        placeholder={labels.columnTitle}
        aria-label={labels.columnTitle}
        oninput={rename}
      />
      <Menu label={labels.tones[tone]} {items} align="end" />
    </div>
  {:else if title}
    <p class="text-fc-sm font-semibold {columnTone[tone].title}">{title}</p>
  {/if}
  <div use:host class="text-fc-fg"></div>
</section>
