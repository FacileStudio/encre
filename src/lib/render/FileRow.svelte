<script lang="ts">
  import { Icon, icons } from '@facile/muse';
  import { fileTag, humanSize } from '../embed.js';
  import type { Labels } from '../labels.js';
  import { linkHref } from './href.js';

  let { src, name = '', size = 0, mime = '', labels, id }: { src: string; name?: string; size?: number; mime?: string; labels: Labels; id?: string } =
    $props();

  const href = $derived(linkHref(src));
  const shown = $derived(name || labels.file);
</script>

<a
  class="my-3 flex min-h-11 items-center gap-3 rounded-fc-md bg-fc-component p-3 text-fc-fg no-underline hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-fc-ring"
  href={href}
  download={name}
  aria-label="{labels.fileDownload} {shown}"
  data-id={id}
>
  <span class="flex size-9 shrink-0 items-center justify-center rounded-fc-sm bg-fc-surface font-mono text-fc-xs font-semibold text-fc-fg-muted">
    {fileTag(name, mime)}
  </span>
  <span class="flex min-w-0 flex-1 flex-col">
    <span class="truncate text-fc-sm font-medium">{shown}</span>
    <span class="text-fc-xs text-fc-fg-muted">{humanSize(size)}</span>
  </span>
  <Icon icon={icons.download} size={16} class="text-fc-fg-muted" />
</a>
