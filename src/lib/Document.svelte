<script lang="ts">
  import type { EncreNode } from './schema.js';
  import Block from './render/Block.svelte';

  let { doc, class: className = '' }: { doc: EncreNode; class?: string } = $props();
</script>

<article class="encre-prose text-fc-fg {className}">
  {#each doc.content ?? [] as node, i (i)}<Block {node} />{/each}
</article>
