<script lang="ts">
  import { labels } from './labels.js';
  import type { SlashState } from './slash.svelte.js';

  let { slash }: { slash: SlashState } = $props();

  const style = $derived(slash.rect ? `top: ${slash.rect.bottom + 6}px; left: ${slash.rect.left}px;` : '');
</script>

{#if slash.open && slash.rect}
  <div
    role="listbox"
    aria-label={labels.insert}
    class="fixed z-50 flex max-h-80 w-64 flex-col gap-0.5 overflow-y-auto overscroll-contain rounded-fc-md bg-fc-bg p-1 text-fc-fg shadow-lg"
    {style}
  >
    {#each slash.items as item, i (item.id)}
      <button
        type="button"
        role="option"
        aria-selected={i === slash.index}
        class="flex min-h-11 items-center rounded-fc-sm px-3 text-left text-fc-sm hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-fc-ring {i === slash.index ? 'bg-fc-surface' : ''}"
        onmousedown={(e) => e.preventDefault()}
        onclick={() => slash.choose(item)}
      >
        {item.label}
      </button>
    {:else}
      <p class="px-3 py-2 text-fc-sm text-fc-fg-muted">{labels.slashEmpty}</p>
    {/each}
  </div>
{/if}
