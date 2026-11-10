<script lang="ts">
  import { Alert, Menu } from '@facile/muse';
  import type { CalloutTone } from '../extensions/custom.js';
  import type { NodeViewProps } from '../extensions/node-view.svelte.js';
  import { TONES, labels } from '../labels.js';

  let { view, update, content }: NodeViewProps = $props();

  const tone = $derived((TONES.includes(view.attrs.tone as CalloutTone) ? view.attrs.tone : 'neutral') as CalloutTone);
  const title = $derived(String(view.attrs.title ?? ''));
  const items = TONES.map((t) => ({ label: labels.tones[t], onselect: () => update({ tone: t }) }));

  function host(node: HTMLElement) {
    if (content) node.appendChild(content);
  }

  function rename(event: Event & { currentTarget: HTMLInputElement }) {
    update({ title: event.currentTarget.value.replace(/[`\r\n]/g, '') });
  }
</script>

<Alert {tone} class="my-3 flex flex-col gap-2">
  {#if view.editable}
    <div contenteditable="false" class="flex items-center gap-2">
      <input
        class="min-w-0 flex-1 bg-transparent font-medium placeholder:text-fc-fg-muted focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring rounded-fc-xs"
        value={title}
        placeholder={labels.calloutTitle}
        aria-label={labels.calloutTitle}
        oninput={rename}
      />
      <Menu label={labels.tones[tone]} {items} align="end" />
    </div>
  {:else if title}
    <p class="font-medium">{title}</p>
  {/if}
  <div use:host class="text-fc-fg"></div>
</Alert>
