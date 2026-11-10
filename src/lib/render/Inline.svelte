<script lang="ts">
  import type { EncreNode } from '../schema.js';
  import { Badge, StatusDot } from '@facile/muse';
  import { BADGE_TONES } from '../labels.js';
  import type { BadgeTone } from '../extensions/custom.js';
  import { linkHref } from './href.js';

  let { nodes = [] }: { nodes?: EncreNode[] } = $props();

  type Mark = NonNullable<EncreNode['marks']>[number];

  const toneOf = (node: EncreNode) => (BADGE_TONES.includes(node.attrs?.tone as BadgeTone) ? node.attrs?.tone : 'neutral') as BadgeTone;
  const textOf = (node: EncreNode) => (node.content ?? []).map((t) => t.text ?? '').join('');
</script>

{#snippet wrap(marks: Mark[], text: string)}
  {@const [mark, ...rest] = marks}
  {#if !mark}{text}{:else if mark.type === 'link'}<a
      href={linkHref(mark.attrs?.href)}
      title={String(mark.attrs?.title ?? '') || undefined}
      rel="noopener noreferrer nofollow">{@render wrap(rest, text)}</a
    >{:else if mark.type === 'bold'}<strong>{@render wrap(rest, text)}</strong>{:else if mark.type === 'italic'}<em
      >{@render wrap(rest, text)}</em
    >{:else if mark.type === 'strike'}<s>{@render wrap(rest, text)}</s>{:else if mark.type === 'highlight'}<mark>{@render wrap(rest, text)}</mark>{:else if mark.type === 'code'}<code
      >{@render wrap(rest, text)}</code
    >{:else}{@render wrap(rest, text)}{/if}
{/snippet}

{#each nodes as node, i (i)}{#if node.type === 'hardBreak'}<br />{:else if node.type === 'badge' && node.attrs?.dot === true}<StatusDot
      tone={toneOf(node)}
      label={textOf(node)}
      class="encre-status"
    />{:else if node.type === 'badge'}<Badge tone={toneOf(node)} class="encre-badge">{textOf(node)}</Badge>{:else}{@render wrap(node.marks ?? [], node.text ?? '')}{/if}{/each}
