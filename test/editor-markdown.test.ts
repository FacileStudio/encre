import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { flushSync, mount, unmount } from 'svelte';
import { beforeAll, describe, expect, it } from 'vitest';
import Editor from '../src/lib/Editor.svelte';
import { loadEncre } from '../src/lib/wasm.js';

beforeAll(async () => {
  await loadEncre(readFileSync(join(import.meta.dirname, '../src/lib/encre-wasm/encre_wasm_bg.wasm')));
});

describe('Editor markdown import and export', () => {
  it('imports Facile Markdown and exports the same canonical bytes', async () => {
    const markdown = '# Title\n\n- [x] done\n\n```stat\n{"label": "Revenue", "value": "12", "delta": "+4%"}\n```\n\nLast **word**.';
    const target = document.createElement('div');
    const editor = mount(Editor, { target, props: { toolbar: false } });
    flushSync();
    await editor.importMarkdown(markdown);
    expect(await editor.exportMarkdown()).toBe(markdown);
    expect(await editor.exportCommonMark()).toBe(markdown);
    void unmount(editor);
  });
});
