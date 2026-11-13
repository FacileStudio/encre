<script lang="ts">
  import { Icon } from '@facile/muse';
  import { labels as french, type Labels } from './labels.js';
  import type { SlashState } from './slash.svelte.js';

  let { slash, labels = french }: { slash: SlashState; labels?: Labels } = $props();

  let options: HTMLButtonElement[] = $state([]);

  const style = $derived(slash.rect ? `top: ${slash.rect.bottom + 6}px; left: ${slash.rect.left}px;` : '');

  $effect(() => {
    if (slash.open) options[slash.index]?.scrollIntoView({ block: 'nearest' });
  });
</script>

{#if slash.open && slash.rect}
  <div
    bind:this={slash.list}
    role="listbox"
    aria-label={labels.insert}
    class="fixed z-50 flex max-h-80 w-64 flex-col gap-0.5 overflow-y-auto overscroll-contain rounded-fc-md bg-fc-bg p-1 text-fc-fg shadow-lg"
    {style}
  >
    {#each slash.items as item, i (item.id)}
      <button
        bind:this={options[i]}
        type="button"
        role="option"
        aria-selected={i === slash.index}
        class="flex min-h-11 items-center gap-2.5 rounded-fc-sm px-3 text-left text-fc-sm hover:bg-fc-surface focus-visible:outline-2 focus-visible:outline-fc-ring {i === slash.index ? 'bg-fc-surface' : ''}"
        onmousedown={(e) => e.preventDefault()}
        onclick={() => slash.choose(item)}
      >
        <Icon icon={item.icon} size={16} class="text-fc-fg-muted" />
        <span class="min-w-0 flex-1 truncate">{item.label}</span>
      </button>
    {:else}
      <p class="px-3 py-2 text-fc-sm text-fc-fg-muted">{labels.slashEmpty}</p>
    {/each}
  </div>
{/if}
