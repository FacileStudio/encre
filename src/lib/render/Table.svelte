<script lang="ts">
  import type { EncreNode } from '../schema.js';
  import Inline from './Inline.svelte';

  let { node }: { node: EncreNode } = $props();

  const rows = $derived(node.content ?? []);
  const alignOf = (cell: EncreNode) => {
    const align = cell.attrs?.align;
    return align === 'left' || align === 'center' || align === 'right' ? `text-align: ${align}` : undefined;
  };
</script>

<div class="my-3 overflow-x-auto">
  <table data-id={typeof node.attrs?.id === 'string' ? node.attrs.id : undefined}>
    <tbody>
      {#each rows as row, r (r)}
        <tr>
          {#each row.content ?? [] as cell, c (c)}
            <svelte:element this={cell.type === 'tableHeader' ? 'th' : 'td'} style={alignOf(cell)}>
              <Inline nodes={cell.content?.[0]?.content ?? []} />
            </svelte:element>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</div>
