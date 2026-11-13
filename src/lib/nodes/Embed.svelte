<script lang="ts">
  import { EMBED_KINDS, type EmbedKind } from '../embed.js';
  import type { NodeViewProps } from '../extensions/node-view.svelte.js';
  import EmbedCard from '../render/EmbedCard.svelte';

  let { view, settings }: NodeViewProps = $props();

  const kind = $derived((EMBED_KINDS.includes(view.attrs.kind as EmbedKind) ? view.attrs.kind : 'link') as EmbedKind);
</script>

<div contenteditable="false" class="rounded-fc-md {view.selected ? 'outline-2 outline-offset-2 outline-fc-ring' : ''}">
  <EmbedCard
    url={String(view.attrs.url ?? '')}
    {kind}
    title={String(view.attrs.title ?? '')}
    labels={settings.labels}
    resolveEmbed={settings.resolveEmbed}
  />
</div>
