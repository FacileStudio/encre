<script lang="ts">
  import type { RenderDiagram } from './extensions/settings.svelte.js';
  import { labelsFor, type LabelOverrides, type Locale } from './labels.js';
  import { provideRender, type Highlight } from './render/context.js';
  import type { EncreNode } from './schema.js';
  import Block from './render/Block.svelte';

  let {
    doc,
    class: className = '',
    locale = 'fr',
    labels,
    renderDiagram,
    highlight
  }: {
    doc: EncreNode;
    class?: string;
    /** The language of the copy the renderer shows. */
    locale?: Locale;
    /** Replaces some of that copy. */
    labels?: LabelOverrides;
    /** Draws a diagram; without it a diagram shows as its source. */
    renderDiagram?: RenderDiagram;
    /** Highlights a code block; it must return safe HTML. Without it, code is escaped text. */
    highlight?: Highlight;
  } = $props();

  const copy = $derived(labelsFor(locale, labels));
  provideRender(() => ({ labels: copy, renderDiagram, highlight }));
</script>

<article class="encre-prose text-fc-fg {className}">
  {#each doc.content ?? [] as node, i (i)}<Block {node} />{/each}
</article>
