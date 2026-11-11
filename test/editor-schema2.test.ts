import { Editor } from '@tiptap/core';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { afterEach, beforeAll, describe, expect, it } from 'vitest';
import { compactDocument } from '../src/lib/compact.js';
import { encreExtensions } from '../src/lib/extensions/index.js';
import type { EncreNode } from '../src/lib/schema.js';
import { loadEncre, serializeMarkdown, validateDocument } from '../src/lib/wasm.js';

let editor: Editor | null = null;
const json = () => compactDocument(editor!.getJSON() as EncreNode);
const markdown = () => serializeMarkdown(json());

beforeAll(async () => {
  await loadEncre(readFileSync(join(import.meta.dirname, '../src/lib/encre-wasm/encre_wasm_bg.wasm')));
});

function open(): Editor {
  editor = new Editor({ element: document.createElement('div'), extensions: encreExtensions(), content: { type: 'doc', content: [{ type: 'paragraph' }] } });
  return editor;
}

afterEach(() => editor?.destroy());

describe('schema 2 commands', () => {
  it('inserts columns and keeps them between two and four', () => {
    const e = open();
    e.chain().insertColumns(2).run();
    const count = () => json().content?.find((n) => n.type === 'columns')?.content?.length;
    expect(count()).toBe(2);
    expect(e.commands.deleteSide()).toBe(false);
    e.commands.addSide();
    e.commands.addSide();
    expect(count()).toBe(4);
    expect(e.commands.addSide()).toBe(false);
    e.commands.deleteSide();
    expect(count()).toBe(3);
    expect(validateDocument(json())).toBeNull();
    expect(markdown()).toMatch(/^````columns\n```column\n\\\n```/);
  });

  it('inserts a badge, retones it, and highlights text', () => {
    const e = open();
    e.chain().insertContent('Ship ').insertBadge('Done', 'warning').run();
    e.chain().setTextSelection(7).setBadge({ tone: 'success', dot: true }).run();
    e.chain().setTextSelection({ from: 1, to: 5 }).toggleHighlight().run();
    expect(markdown()).toBe('==Ship== [status:success Done]');
  });

  it('inserts a diagram that serializes as its fence', () => {
    const e = open();
    e.chain().insertDiagram('d2').insertContent('x -> y').run();
    expect(markdown()).toContain('```d2\nx -> y\n```');
  });
});

describe('pasted HTML', () => {
  const html = [
    '<table><tr><td>a</td><td>b</td></tr><tr><td colspan="2">c</td></tr></table>',
    '<table><tr><th style="text-align: justify">a</th></tr><tr><td>b</td></tr></table>',
    '<table><tr><th align="center">a</th><th>b</th></tr><tr><td>c</td><td align="right">d</td></tr></table>',
    '<ol start="-2"><li><p>a</p></li></ol><ol start="5000000000"><li><p>b</p></li></ol>'
  ];

  it.each(html)('always validates: %s', (source) => {
    const e = open();
    e.commands.setContent(source);
    expect(validateDocument(json())).toBeNull();
  });

  it('keeps the header alignment of each column and clamps list starts', () => {
    const e = open();
    e.commands.setContent(html[2]!);
    const rows = json().content?.[0]?.content ?? [];
    expect(rows.map((row) => row.content?.map((cell) => cell.attrs?.align ?? 'none'))).toEqual([
      ['center', 'none'],
      ['center', 'none']
    ]);
    e.commands.setContent(html[3]!);
    expect(json().content?.filter((n) => n.type === 'orderedList').map((list) => list.attrs?.start ?? 1)).toEqual([0, 999_999_999]);
  });
});
