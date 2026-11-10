import { flushSync, mount, unmount } from 'svelte';
import { describe, expect, it } from 'vitest';
import Document from '../src/lib/Document.svelte';
import { diffLines } from '../src/lib/render/context.js';
import type { EncreNode } from '../src/lib/schema.js';

const text = (value: string, marks?: string[]): EncreNode => ({ type: 'text', text: value, ...(marks ? { marks: marks.map((type) => ({ type })) } : {}) });
const code = (language: string, value: string): EncreNode => ({ type: 'codeBlock', attrs: { language }, content: [text(value)] });
const diagram: EncreNode = { type: 'diagram', attrs: { language: 'mermaid' }, content: [text('A --> B')] };
const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

function render(content: EncreNode[], props: Record<string, unknown> = {}) {
  const target = document.createElement('div');
  const app = mount(Document, { target, props: { doc: { type: 'doc', content }, ...props } });
  flushSync();
  return { target, html: () => target.innerHTML.replaceAll('<!---->', ''), close: () => void unmount(app) };
}

describe('Document, schema 2', () => {
  it('renders highlights, badges, status dots, stat subtitles and columns', () => {
    const { target, html, close } = render([
      { type: 'paragraph', content: [text('lit', ['highlight']), { type: 'badge', attrs: { tone: 'success' }, content: [text('Shipped')] }, { type: 'badge', attrs: { tone: 'danger', dot: true }, content: [text('failed')] }] },
      { type: 'stat', attrs: { label: 'Active', value: '16', sub: 'In production' } },
      { type: 'columns', content: [
        { type: 'column', attrs: { title: 'Pros', tone: 'success' }, content: [{ type: 'paragraph', content: [text('fast')] }] },
        { type: 'column', content: [{ type: 'paragraph', content: [text('plain')] }] }
      ] }
    ]);
    expect(html()).toContain('<mark>lit</mark>');
    for (const s of ['Shipped', 'failed', 'In production', 'Pros', 'fast', 'plain']) expect(target.textContent).toContain(s);
    expect(target.querySelectorAll('.encre-columns > section')).toHaveLength(2);
    close();
  });

  it('shows a diagram as labelled source without a renderer, and draws it with one', async () => {
    const plain = render([diagram]);
    expect(plain.target.querySelector('figcaption')?.textContent).toBe('mermaid');
    expect(plain.target.querySelector('pre code')?.textContent).toBe('A --> B');
    plain.close();
    const calls: string[] = [];
    const drawn = render([diagram], { renderDiagram: async (language: string, source: string) => (calls.push(`${language}:${source}`), '<svg data-drawn="1"></svg>') });
    await tick();
    flushSync();
    expect(calls).toEqual(['mermaid:A --> B']);
    expect(drawn.target.querySelector('svg[data-drawn]')).not.toBeNull();
    drawn.close();
  });

  it('tones diff lines and escapes code unless the app highlights it', () => {
    const { target, html, close } = render([code('diff', '+ add\n- del\n@@ hunk\n  ctx\n--- a/file'), code('js', '<b>x</b>')]);
    expect([...target.querySelectorAll('[data-language="diff"] span')].map((s) => s.className)).toEqual(['encre-diff-add', 'encre-diff-del', 'encre-diff-hunk', 'encre-diff-ctx', 'encre-diff-ctx']);
    expect(html()).toContain('&lt;b&gt;x&lt;/b&gt;');
    close();
    const hooked = render([code('js', 'let x')], { highlight: (value: string, language: string) => `<span class="hl-${language}">${value}</span>` });
    expect(hooked.target.querySelector('.hl-js')?.textContent).toBe('let x');
    hooked.close();
  });

  it('classifies diff lines like Mycelium does', () => {
    expect(diffLines('+++ b\n+a').map((l) => l.tone)).toEqual(['ctx', 'add']);
  });
});
