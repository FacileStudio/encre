<script lang="ts">
  import type { NodeViewProps } from '../extensions/node-view.svelte.js';
  import FileRow from '../render/FileRow.svelte';

  let { view, settings }: NodeViewProps = $props();
</script>

<div contenteditable="false" class="rounded-fc-md {view.selected ? 'outline-2 outline-offset-2 outline-fc-ring' : ''}">
  <FileRow
    src={String(view.attrs.src ?? '')}
    name={String(view.attrs.name ?? '')}
    size={Number(view.attrs.size ?? 0)}
    mime={String(view.attrs.mime ?? '')}
    labels={settings.labels}
  />
</div>
