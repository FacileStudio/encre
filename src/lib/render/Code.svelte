<script lang="ts">
  import { diffLines, renderOptions } from './context.js';

  let { code, language, id }: { code: string; language: string; id?: string } = $props();

  const read = renderOptions();
  const options = $derived(read());
  const lines = $derived(language === 'diff' ? diffLines(code) : []);
</script>

{#if language === 'diff'}
  <pre data-id={id} data-language="diff"><code
      >{#each lines as { tone, line }, i (i)}<span class="encre-diff-{tone}">{line}</span>{'\n'}{/each}</code
    ></pre>
{:else if options.highlight}
  <pre data-id={id} data-language={language || undefined}><code>{@html options.highlight(code, language)}</code></pre>
{:else}
  <pre data-id={id} data-language={language || undefined}><code>{code}</code></pre>
{/if}
