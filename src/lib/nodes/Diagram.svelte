<script lang="ts">
  import { Menu } from '@facile/muse';
  import type { NodeViewProps } from '../extensions/node-view.svelte.js';
  import DiagramPreview from '../render/DiagramPreview.svelte';
  import { DIAGRAM_LANGUAGES } from '../labels.js';

  let { view, update, content, settings }: NodeViewProps = $props();

  const labels = $derived(settings.labels);
  const language = $derived(DIAGRAM_LANGUAGES.includes(view.attrs.language as never) ? String(view.attrs.language) : 'mermaid');
  const items = $derived(DIAGRAM_LANGUAGES.map((value) => ({ label: value, onselect: () => update({ language: value }) })));
  let source = $state('');

  function host(node: HTMLElement) {
    if (!content) return;
    content.className = 'encre-diagram-source';
    content.setAttribute('aria-label', labels.diagramSource);
    node.appendChild(content);
    const read = () => (source = content.textContent ?? '');
    const observer = new MutationObserver(read);
    observer.observe(content, { characterData: true, childList: true, subtree: true });
    read();
    return { destroy: () => observer.disconnect() };
  }
</script>

<figure class="my-3 flex flex-col gap-3 rounded-fc-md bg-fc-component p-4">
  <div contenteditable="false" class="flex items-center justify-between gap-2">
    <span class="text-fc-xs font-medium text-fc-fg-muted">{labels.diagram} · {language}</span>
    {#if view.editable}<Menu label={labels.diagramLanguage} {items} align="end" />{/if}
  </div>
  {#if settings.renderDiagram && source.trim()}
    <div contenteditable="false"><DiagramPreview render={settings.renderDiagram} {language} {source} /></div>
  {/if}
  <pre class="m-0 overflow-x-auto rounded-fc-sm bg-fc-surface p-3 font-mono text-fc-sm" class:hidden={!view.editable && !!settings.renderDiagram}><div
      use:host></div></pre>
</figure>
