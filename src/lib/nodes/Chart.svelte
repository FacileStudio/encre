<script lang="ts">
  import { BarChart, Button, Field, Input, LineChart, Textarea } from '@facile/muse';
  import { seriesToText, textToLabels, textToSeries, toLabels, toSeries } from '../chart-data.js';
  import type { NodeViewProps } from '../extensions/node-view.svelte.js';

  let { view, update, settings }: NodeViewProps = $props();
  const copy = $derived(settings.labels);

  const kind = $derived(view.attrs.kind === 'line' ? 'line' : 'bar');
  const title = $derived(String(view.attrs.title ?? ''));
  const chartLabels = $derived(toLabels(view.attrs.labels));
  const series = $derived(toSeries(view.attrs.series));
</script>

<figure contenteditable="false" class="my-3 flex flex-col gap-3 rounded-fc-md bg-fc-component p-5">
  {#if title}<figcaption class="text-fc-sm font-medium text-fc-fg">{title}</figcaption>{/if}
  {#if kind === 'line'}
    <LineChart {series} labels={chartLabels} emptyLabel={copy.emptyChart} />
  {:else}
    <BarChart {series} labels={chartLabels} emptyLabel={copy.emptyChart} />
  {/if}
  {#if view.editable && view.selected}
    <div class="grid gap-3 sm:grid-cols-2">
      <Field label={copy.chartTitle}>
        {#snippet children({ id })}
          <Input {id} value={title} oninput={(e) => update({ title: e.currentTarget.value.replace(/[\r\n]/g, '') })} />
        {/snippet}
      </Field>
      <div class="flex flex-col gap-1.5">
        <span class="text-fc-sm font-medium text-fc-fg">{copy.chartKind}</span>
        <div class="flex gap-2" role="group" aria-label={copy.chartKind}>
          {#each [['bar', copy.chartBar], ['line', copy.chartLine]] as [value, name] (value)}
            <Button variant={kind === value ? 'primary' : 'secondary'} aria-pressed={kind === value} onclick={() => update({ kind: value })}>
              {name}
            </Button>
          {/each}
        </div>
      </div>
      <Field label={copy.chartLabels} class="sm:col-span-2">
        {#snippet children({ id })}
          <Input {id} value={chartLabels.join(', ')} onchange={(e) => update({ labels: textToLabels(e.currentTarget.value) })} />
        {/snippet}
      </Field>
      <Field label={copy.chartSeries} class="sm:col-span-2">
        {#snippet children({ id })}
          <Textarea {id} autoGrow value={seriesToText(series)} onchange={(e) => update({ series: textToSeries(e.currentTarget.value) })} />
        {/snippet}
      </Field>
    </div>
  {/if}
</figure>
