<script lang="ts">
  import { Field, Input, StatCard } from '@facile/muse';
  import type { NodeViewProps } from '../extensions/node-view.svelte.js';
  import { labels } from '../labels.js';

  let { view, update }: NodeViewProps = $props();

  const keys = ['label', 'value', 'delta'] as const;
  const names = { label: labels.statLabel, value: labels.statValue, delta: labels.statDelta };
  const text = (key: (typeof keys)[number]) => String(view.attrs[key] ?? '');
</script>

<div contenteditable="false" class="my-3 flex flex-col gap-3">
  <StatCard label={text('label') || labels.statLabel} value={text('value') || '—'} delta={text('delta') || undefined} />
  {#if view.editable && view.selected}
    <div class="grid gap-3 sm:grid-cols-3">
      {#each keys as key (key)}
        <Field label={names[key]}>
          {#snippet children({ id })}
            <Input {id} value={text(key)} oninput={(e) => update({ [key]: e.currentTarget.value.replace(/[\r\n]/g, '') })} />
          {/snippet}
        </Field>
      {/each}
    </div>
  {/if}
</div>
