<script lang="ts">
  import { renderOptions } from './context.js';
  import DiagramPreview from './DiagramPreview.svelte';

  let { language, source, id }: { language: string; source: string; id?: string } = $props();

  const read = renderOptions();
  const options = $derived(read());
</script>

<figure class="my-3" data-id={id} data-language={language}>
  {#if options.renderDiagram && source.trim()}
    <DiagramPreview render={options.renderDiagram} {language} {source} />
  {:else}
    <figcaption class="mb-1 text-fc-xs font-medium text-fc-fg-muted">{language}</figcaption>
    <pre class="!mt-0"><code>{source}</code></pre>
  {/if}
</figure>
