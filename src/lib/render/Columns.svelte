<script lang="ts">
  import type { ColumnTone } from '../extensions/custom.js';
  import { COLUMN_TONES } from '../labels.js';
  import type { EncreNode } from '../schema.js';
  import Block from './Block.svelte';
  import { columnTone } from './tones.js';

  let { node }: { node: EncreNode } = $props();

  const id = (n: EncreNode) => (typeof n.attrs?.id === 'string' ? n.attrs.id : undefined);
  const tone = (n: EncreNode) => (COLUMN_TONES.includes(n.attrs?.tone as ColumnTone) ? n.attrs?.tone : 'neutral') as ColumnTone;
</script>

<div class="encre-columns my-3" data-id={id(node)}>
  {#each node.content ?? [] as column, i (i)}
    <section class="flex min-w-0 flex-col gap-2 rounded-fc-md p-4 {columnTone[tone(column)].box}" data-id={id(column)}>
      {#if column.attrs?.title}<p class="text-fc-sm font-semibold {columnTone[tone(column)].title}">{column.attrs.title}</p>{/if}
      <div class="text-fc-fg">
        {#each column.content ?? [] as child, c (c)}<Block node={child} />{/each}
      </div>
    </section>
  {/each}
</div>
