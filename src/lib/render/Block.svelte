<script lang="ts">
  import { Alert, BarChart, LineChart, StatCard } from '@facile/muse';
  import { toLabels, toSeries } from '../chart-data.js';
  import type { CalloutTone } from '../extensions/custom.js';
  import { TONES } from '../labels.js';
  import type { EncreNode } from '../schema.js';
  import Block from './Block.svelte';
  import Code from './Code.svelte';
  import Columns from './Columns.svelte';
  import { renderOptions } from './context.js';
  import Diagram from './Diagram.svelte';
  import EmbedCard from './EmbedCard.svelte';
  import FileRow from './FileRow.svelte';
  import { EMBED_KINDS, type EmbedKind } from '../embed.js';
  import { imageSrc } from './href.js';
  import Inline from './Inline.svelte';
  import Table from './Table.svelte';

  let { node }: { node: EncreNode } = $props();

  const attrs = $derived(node.attrs ?? {});
  const children = $derived(node.content ?? []);
  const text = (key: string) => String(attrs[key] ?? '');
  const tone = $derived((TONES.includes(attrs.tone as CalloutTone) ? attrs.tone : 'neutral') as CalloutTone);
  const level = $derived(Math.min(3, Math.max(1, Number(attrs.level) || 1)));
  const id = $derived(typeof attrs.id === 'string' ? attrs.id : undefined);
  const source = $derived(children.map((c) => c.text ?? '').join(''));
  const read = renderOptions();
  const options = $derived(read());
</script>

{#snippet blocks(list: EncreNode[])}
  {#each list as child, i (i)}<Block node={child} />{/each}
{/snippet}

{#if node.type === 'paragraph'}
  <p data-id={id}><Inline nodes={children} /></p>
{:else if node.type === 'heading'}
  <svelte:element this={`h${level}`} data-id={id}><Inline nodes={children} /></svelte:element>
{:else if node.type === 'bulletList'}
  <ul data-id={id}>{@render blocks(children)}</ul>
{:else if node.type === 'orderedList'}
  <ol data-id={id} start={Number(attrs.start ?? 1)}>{@render blocks(children)}</ol>
{:else if node.type === 'taskList'}
  <ul data-id={id} data-type="taskList">{@render blocks(children)}</ul>
{:else if node.type === 'listItem'}
  <li data-id={id}>{@render blocks(children)}</li>
{:else if node.type === 'taskItem'}
  <li data-id={id} data-checked={attrs.checked === true}>
    <input type="checkbox" checked={attrs.checked === true} disabled aria-label={attrs.checked === true ? '✓' : '○'} />
    <div>{@render blocks(children)}</div>
  </li>
{:else if node.type === 'blockquote'}
  <blockquote data-id={id}>{@render blocks(children)}</blockquote>
{:else if node.type === 'codeBlock'}
  <Code code={source} language={text('language')} {id} />
{:else if node.type === 'diagram'}
  <Diagram {source} language={text('language') || 'mermaid'} {id} />
{:else if node.type === 'columns'}
  <Columns {node} />
{:else if node.type === 'horizontalRule'}
  <hr data-id={id} />
{:else if node.type === 'image'}
  {#if imageSrc(attrs.src)}<img data-id={id} src={imageSrc(attrs.src)} alt={text('alt')} title={text('title') || undefined} />{/if}
{:else if node.type === 'embed'}
  <EmbedCard
    url={text('url')}
    kind={(EMBED_KINDS.includes(attrs.kind as EmbedKind) ? attrs.kind : 'link') as EmbedKind}
    title={text('title')}
    labels={options.labels}
    resolveEmbed={options.resolveEmbed}
    {id}
  />
{:else if node.type === 'file'}
  <FileRow src={text('src')} name={text('name')} size={Number(attrs.size ?? 0)} mime={text('mime')} labels={options.labels} {id} />
{:else if node.type === 'table'}
  <Table {node} />
{:else if node.type === 'callout'}
  <Alert {tone} title={text('title') || undefined} class="my-3" data-id={id}>
    <div class="text-fc-fg">{@render blocks(children)}</div>
  </Alert>
{:else if node.type === 'stat'}
  <StatCard class="my-3" label={text('label')} value={text('value')} delta={text('delta') || undefined} data-id={id}>
    {#if text('sub')}<span class="text-fc-xs text-fc-fg-muted">{text('sub')}</span>{/if}
  </StatCard>
{:else if node.type === 'chart'}
  <figure class="my-3 flex flex-col gap-3 rounded-fc-md bg-fc-component p-5" data-id={id}>
    {#if text('title')}<figcaption class="text-fc-sm font-medium">{text('title')}</figcaption>{/if}
    {#if attrs.kind === 'line'}
      <LineChart series={toSeries(attrs.series)} labels={toLabels(attrs.labels)} emptyLabel={options.labels.emptyChart} />
    {:else}
      <BarChart series={toSeries(attrs.series)} labels={toLabels(attrs.labels)} emptyLabel={options.labels.emptyChart} />
    {/if}
  </figure>
{/if}
