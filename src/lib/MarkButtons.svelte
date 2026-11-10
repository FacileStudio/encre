<script lang="ts">
  import { IconButton, Tooltip } from '@facile/muse';
  import type { Editor } from '@tiptap/core';
  import { labels } from './labels.js';

  let { editor, version }: { editor: Editor; version: number } = $props();

  type MarkName = 'bold' | 'italic' | 'strike' | 'code';
  const marks: { name: MarkName; label: string; glyph: string; glyphClass: string }[] = [
    { name: 'bold', label: labels.bold, glyph: 'B', glyphClass: 'font-bold' },
    { name: 'italic', label: labels.italic, glyph: 'I', glyphClass: 'italic font-serif' },
    { name: 'strike', label: labels.strike, glyph: 'S', glyphClass: 'line-through' },
    { name: 'code', label: labels.code, glyph: '</>', glyphClass: 'font-mono text-fc-xs' }
  ];

  let linking = $state(false);
  let href = $state('');

  const active = $derived.by(() => {
    void version;
    return Object.fromEntries([...marks.map((m) => m.name), 'link'].map((name) => [name, editor.isActive(name)]));
  });

  function toggle(name: MarkName) {
    editor.chain().focus().toggleMark(name).run();
  }

  function startLink() {
    if (active.link) {
      editor.chain().focus().extendMarkRange('link').unsetLink().run();
      return;
    }
    href = '';
    linking = true;
  }

  function applyLink(event: SubmitEvent) {
    event.preventDefault();
    const target = href.trim();
    linking = false;
    if (target !== '') editor.chain().focus().extendMarkRange('link').setLink({ href: target }).run();
  }
</script>

{#if linking}
  <form class="flex items-center gap-1" onsubmit={applyLink}>
    <input
      class="h-9 w-56 rounded-fc-sm bg-fc-field px-2 text-fc-sm text-fc-fg placeholder:text-fc-fg-muted focus-visible:outline-2 focus-visible:outline-fc-ring"
      type="url"
      placeholder="https://"
      aria-label={labels.linkPrompt}
      bind:value={href}
    />
  </form>
{:else}
  {#each marks as mark (mark.name)}
    <Tooltip text={mark.label}>
      <IconButton
        variant="ghost"
        aria-label={mark.label}
        aria-pressed={active[mark.name]}
        class={active[mark.name] ? 'bg-fc-surface text-fc-fg' : ''}
        onclick={() => toggle(mark.name)}
      >
        <span class={mark.glyphClass} aria-hidden="true">{mark.glyph}</span>
      </IconButton>
    </Tooltip>
  {/each}
  <Tooltip text={labels.link}>
    <IconButton
      variant="ghost"
      aria-label={labels.link}
      aria-pressed={active.link}
      class={active.link ? 'bg-fc-surface text-fc-fg' : ''}
      onclick={startLink}
    >
      <span class="underline" aria-hidden="true">a</span>
    </IconButton>
  </Tooltip>
{/if}
