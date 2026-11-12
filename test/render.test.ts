import { flushSync, mount, unmount } from 'svelte';
import { describe, expect, it } from 'vitest';
import Document from '../src/lib/Document.svelte';
import { textToLabels, textToSeries, seriesToText, toSeries } from '../src/lib/chart-data.js';
import { linkHref } from '../src/lib/render/href.js';
import type { EncreNode } from '../src/lib/schema.js';

const sample: EncreNode = {
  type: 'doc',
  content: [
    { type: 'heading', attrs: { level: 2 }, content: [{ type: 'text', text: 'Title' }] },
    {
      type: 'paragraph',
      content: [
        { type: 'text', text: 'bold', marks: [{ type: 'bold' }] },
        { type: 'hardBreak' },
        { type: 'text', text: 'evil', marks: [{ type: 'link', attrs: { href: 'javascript:alert(1)' } }] }
      ]
    },
    { type: 'callout', attrs: { tone: 'warning', title: 'Careful' }, content: [{ type: 'paragraph', content: [{ type: 'text', text: 'inside' }] }] },
    { type: 'stat', attrs: { label: 'Revenue', value: '12 400 €', delta: '+4%' } },
    { type: 'codeBlock', attrs: { language: 'sh' }, content: [{ type: 'text', text: 'echo <hi>' }] }
  ]
};

describe('Document', () => {
  it('renders every node and never a script link', () => {
    const target = document.createElement('div');
    const app = mount(Document, { target, props: { doc: sample } });
    flushSync();
    const body = target.innerHTML.replaceAll('<!---->', '');
    void unmount(app);
    for (const text of ['Title', '<strong>bold</strong>', 'Careful', 'inside', 'Revenue', '12 400 €', 'echo &lt;hi&gt;']) {
      expect(body).toContain(text);
    }
    expect(body).toContain('<h2');
    expect(body).not.toContain('javascript:');
  });
});

describe('chart data', () => {
  it('round-trips series through their text form and drops junk', () => {
    const series = [{ name: 'web', data: [3, 5.5] }];
    expect(textToSeries(seriesToText(series))).toEqual(series);
    expect(toSeries([{ name: 1, data: [1, 'x', Number.NaN] }, null])).toEqual([{ name: '', data: [1] }]);
    expect(textToSeries('w\u0007eb: 1, , 2,\r')).toEqual([{ name: 'web', data: [1, 2] }]);
    expect(textToLabels('Q\u00011, , Q2')).toEqual(['Q1', 'Q2']);
  });
});

describe('linkHref', () => {
  it('keeps web, mail and relative links and refuses script schemes', () => {
    expect(['https://a.b', 'mailto:x@y.z', '/p', '#h'].map(linkHref)).toEqual(['https://a.b', 'mailto:x@y.z', '/p', '#h']);
    expect(['javascript:x', ' data:text/html,x', 'vbscript:x'].map(linkHref)).toEqual([undefined, undefined, undefined]);
  });
});
