<script lang="ts">
  import { Icon, icons, Tooltip } from '@facile/muse';
  import { FRAMED_KINDS, frameSrc, httpsUrl, type EmbedKind, type EmbedPreview, type ResolveEmbed } from '../embed.js';
  import { embedLogo } from '../block-icons.js';
  import type { Labels } from '../labels.js';

  let {
    url,
    kind,
    title = '',
    labels,
    resolveEmbed,
    id
  }: { url: string; kind: EmbedKind; title?: string; labels: Labels; resolveEmbed?: ResolveEmbed; id?: string } = $props();

  const parsed = $derived(httpsUrl(url));
  const framed = $derived(FRAMED_KINDS.includes(kind) && parsed !== null);
  let height = $state(420);
  let preview = $state<EmbedPreview>({});

  $effect(() => {
    preview = {};
    if (framed || !parsed || !resolveEmbed) return;
    let live = true;
    resolveEmbed(parsed.toString())
      .then((found) => live && (preview = found ?? {}))
      .catch(() => undefined);
    return () => (live = false);
  });

  function resize(event: PointerEvent) {
    event.preventDefault();
    const start = { y: event.clientY, height };
    const move = (e: PointerEvent) => (height = Math.min(1200, Math.max(160, start.height + e.clientY - start.y)));
    const stop = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', stop);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', stop);
  }

  function nudge(event: KeyboardEvent) {
    const step = { ArrowDown: 40, ArrowUp: -40 }[event.key];
    if (step === undefined) return;
    event.preventDefault();
    height = Math.min(1200, Math.max(160, height + step));
  }
</script>

{#if framed && parsed}
  <figure class="relative my-3 overflow-hidden rounded-fc-md bg-fc-component" data-id={id}>
    <iframe
      class="block w-full"
      style:height="{height}px"
      src={frameSrc(parsed.toString(), kind)}
      title={title || labels.embed}
      loading="lazy"
      referrerpolicy="strict-origin-when-cross-origin"
      allow="clipboard-write; fullscreen"
    ></iframe>
    <a
      class="absolute top-2 right-2 inline-flex min-h-11 items-center gap-1 rounded-fc-pill bg-fc-bg/90 px-3 text-fc-xs text-fc-fg shadow-sm focus-visible:outline-2 focus-visible:outline-fc-ring"
      href={parsed.toString()}
      target="_blank"
      rel="noopener noreferrer"
    >
      <Icon icon={embedLogo(kind)} size={16} class="text-fc-accent" />{labels.embedOpen}<Icon icon={icons.external} size={14} />
    </a>
    <Tooltip text={labels.embedResize} describe={false} class="contents">
      <div
        role="slider"
        tabindex="0"
        aria-label={labels.embedResize}
        aria-valuemin={160}
        aria-valuemax={1200}
        aria-valuenow={height}
        class="flex h-4 cursor-ns-resize items-center justify-center focus-visible:outline-2 focus-visible:outline-fc-ring"
        onpointerdown={resize}
        onkeydown={nudge}
      >
        <span class="h-1 w-10 rounded-fc-pill bg-fc-fg-muted/40"></span>
      </div>
    </Tooltip>
  </figure>
{:else if parsed}
  <a
    class="my-3 flex min-h-11 items-center gap-3 rounded-fc-md bg-fc-component p-3 text-fc-fg no-underline hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-fc-ring"
    href={parsed.toString()}
    target="_blank"
    rel="noopener noreferrer"
    data-id={id}
  >
    {#if kind === 'link'}
      <img class="size-5 shrink-0 rounded-fc-xs" src="https://{parsed.host}/favicon.ico" alt="" loading="lazy" />
    {:else}
      <Icon icon={embedLogo(kind)} size={20} class="text-fc-accent" />
    {/if}
    <span class="flex min-w-0 flex-1 flex-col">
      <span class="truncate text-fc-sm font-medium">{title || preview.title || preview.site || parsed.host}</span>
      {#if preview.description}<span class="truncate text-fc-xs text-fc-fg-muted">{preview.description}</span>{/if}
      <span class="truncate text-fc-xs text-fc-fg-muted">{parsed.toString()}</span>
    </span>
    {#if preview.image && httpsUrl(preview.image)}
      <img class="h-12 w-20 shrink-0 rounded-fc-sm object-cover" src={preview.image} alt="" loading="lazy" />
    {/if}
  </a>
{/if}
