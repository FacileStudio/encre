<script lang="ts">
  import type { RenderDiagram } from '../extensions/settings.svelte.js';

  let { render, language, source }: { render: RenderDiagram; language: string; source: string } = $props();

  let drawn = $state<string | SVGElement | null>(null);
  let failed = $state(false);

  $effect(() => {
    let live = true;
    failed = false;
    render(language, source).then(
      (out) => live && (drawn = out),
      () => live && ((failed = true), (drawn = null))
    );
    return () => (live = false);
  });

  function place(node: HTMLElement, element: SVGElement) {
    node.replaceChildren(element);
    return { update: (next: SVGElement) => node.replaceChildren(next) };
  }
</script>

{#if typeof drawn === 'string'}
  <div class="encre-diagram overflow-x-auto" data-language={language}>{@html drawn}</div>
{:else if drawn}
  <div class="encre-diagram overflow-x-auto" data-language={language} use:place={drawn}></div>
{:else if failed}
  <pre class="overflow-x-auto" data-language={language}><code>{source}</code></pre>
{/if}
