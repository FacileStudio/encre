<script lang="ts">
  import { IconButton } from '@facile/muse';
  import type { Editor } from '@tiptap/core';
  import { labels as french, type Labels } from './labels.js';

  let { editor, version, labels = french }: { editor: Editor; version: number; labels?: Labels } = $props();

  type MarkName = 'bold' | 'italic' | 'strike' | 'highlight' | 'code';
  const marks: { name: MarkName; label: string; glyph: string; glyphClass: string }[] = $derived([
    { name: 'bold', label: labels.bold, glyph: 'B', glyphClass: 'font-bold' },
    { name: 'italic', label: labels.italic, glyph: 'I', glyphClass: 'italic font-serif' },
    { name: 'strike', label: labels.strike, glyph: 'S', glyphClass: 'line-through' },
    { name: 'highlight', label: labels.highlight, glyph: 'H', glyphClass: 'rounded-fc-xs bg-fc-warning/20 px-1' },
    { name: 'code', label: labels.code, glyph: '</>', glyphClass: 'font-mono text-fc-xs' }
  ]);

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
    href = active.link ? String(editor.getAttributes('link').href ?? '') : '';
    linking = true;
  }

  function removeLink() {
    linking = false;
    editor.chain().focus().extendMarkRange('link').unsetLink().run();
  }

  function applyLink(event: SubmitEvent) {
    event.preventDefault();
    const target = href.trim();
    if (target === '') return removeLink();
    linking = false;
    editor.chain().focus().extendMarkRange('link').setLink({ href: target }).run();
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
      onkeydown={(e) => e.key === 'Escape' && (linking = false)}
    />
    {#if active.link}
      <IconButton variant="ghost" aria-label={labels.linkRemove} onclick={removeLink}>
        <span class="line-through" aria-hidden="true">a</span>
      </IconButton>
    {/if}
  </form>
{:else}
  {#each marks as mark (mark.name)}
    <IconButton
      variant="ghost"
      aria-label={mark.label}
      aria-pressed={active[mark.name]}
      class={active[mark.name] ? 'bg-fc-surface text-fc-fg' : ''}
      onclick={() => toggle(mark.name)}
    >
      <span class={mark.glyphClass} aria-hidden="true">{mark.glyph}</span>
    </IconButton>
  {/each}
  <IconButton
    variant="ghost"
    aria-label={labels.link}
    aria-pressed={active.link}
    class={active.link ? 'bg-fc-surface text-fc-fg' : ''}
    onclick={startLink}
  >
    <span class="underline" aria-hidden="true">a</span>
  </IconButton>
{/if}
