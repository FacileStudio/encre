<script lang="ts">
  import type { EncreNode } from '../schema.js';
  import { linkHref } from './href.js';

  let { nodes = [] }: { nodes?: EncreNode[] } = $props();

  type Mark = NonNullable<EncreNode['marks']>[number];
</script>

{#snippet wrap(marks: Mark[], text: string)}
  {@const [mark, ...rest] = marks}
  {#if !mark}{text}{:else if mark.type === 'link'}<a
      href={linkHref(mark.attrs?.href)}
      title={String(mark.attrs?.title ?? '') || undefined}
      rel="noopener noreferrer nofollow">{@render wrap(rest, text)}</a
    >{:else if mark.type === 'bold'}<strong>{@render wrap(rest, text)}</strong>{:else if mark.type === 'italic'}<em
      >{@render wrap(rest, text)}</em
    >{:else if mark.type === 'strike'}<s>{@render wrap(rest, text)}</s>{:else if mark.type === 'code'}<code
      >{@render wrap(rest, text)}</code
    >{:else}{@render wrap(rest, text)}{/if}
{/snippet}

{#each nodes as node, i (i)}{#if node.type === 'hardBreak'}<br />{:else}{@render wrap(node.marks ?? [], node.text ?? '')}{/if}{/each}
