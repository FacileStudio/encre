<script lang="ts">
  import { Field, Input, StatCard } from '@facile/muse';
  import type { NodeViewProps } from '../extensions/node-view.svelte.js';

  let { view, update, settings }: NodeViewProps = $props();

  const labels = $derived(settings.labels);
  const keys = ['label', 'value', 'delta', 'sub'] as const;
  const names = $derived({ label: labels.statLabel, value: labels.statValue, delta: labels.statDelta, sub: labels.statSub });
  const text = (key: (typeof keys)[number]) => String(view.attrs[key] ?? '');
</script>

<div contenteditable="false" class="my-3 flex flex-col gap-3">
  <StatCard label={text('label') || labels.statLabel} value={text('value') || '—'} delta={text('delta') || undefined}>
    {#if text('sub')}<span class="text-fc-xs text-fc-fg-muted">{text('sub')}</span>{/if}
  </StatCard>
  {#if view.editable && view.selected}
    <div class="grid gap-3 sm:grid-cols-2">
      {#each keys as key (key)}
        <Field label={names[key]}>
          {#snippet children({ id })}
            <Input {id} value={text(key)} oninput={(e) => update({ [key]: e.currentTarget.value.replace(/[^\P{Cc}\t]/gu, '') })} />
          {/snippet}
        </Field>
      {/each}
    </div>
  {/if}
</div>
