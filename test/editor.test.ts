import { Editor } from '@tiptap/core';
import { afterEach, describe, expect, it } from 'vitest';
import { encreExtensions } from '../src/lib/extensions/index.js';
import { isBlockId } from '../src/lib/ids.js';
import type { EncreNode } from '../src/lib/schema.js';

let editor: Editor | null = null;

function open(content: EncreNode): Editor {
  editor = new Editor({ element: document.createElement('div'), extensions: encreExtensions(), content });
  return editor;
}

function walk(node: EncreNode, visit: (node: EncreNode) => void): void {
  visit(node);
  node.content?.forEach((child) => walk(child, visit));
}

afterEach(() => editor?.destroy());

const doc = (...content: EncreNode[]): EncreNode => ({ type: 'doc', content });
const para = (text: string): EncreNode => ({ type: 'paragraph', content: [{ type: 'text', text }] });

describe('encre editor', () => {
  it('gives every block a unique ten-character base32 id', async () => {
    const e = open(doc(para('a'), { type: 'callout', content: [para('b')] }, { type: 'stat' }));
    await new Promise((resolve) => setTimeout(resolve, 0));
    e.commands.insertTable({ rows: 2, cols: 2 });
    const ids: unknown[] = [];
    walk(e.getJSON() as EncreNode, (node) => {
      if (node.attrs && 'id' in node.attrs) ids.push(node.attrs.id);
    });
    expect(ids.length).toBeGreaterThan(5);
    expect(ids.filter((id) => !isBlockId(id))).toEqual([]);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it('strips marks from hard breaks', () => {
    const e = open(doc({ type: 'paragraph', content: [{ type: 'text', text: 'a', marks: [{ type: 'bold' }] }] }));
    e.chain().selectAll().setBold().setTextSelection(2).setHardBreak().run();
    const inline = (e.getJSON() as EncreNode).content?.[0]?.content ?? [];
    expect(inline.find((n) => n.type === 'hardBreak')?.marks ?? []).toEqual([]);
  });

  it('keeps a table rectangular with a header row and one alignment per column', () => {
    const e = open(doc(para('x')));
    e.chain().insertTable({ rows: 2, cols: 2 }).addColumnAfter().setColumnAlign('center').addRowAfter().deleteRow().run();
    const table = (e.getJSON() as EncreNode).content?.find((n) => n.type === 'table');
    const rows = table?.content ?? [];
    expect(rows).toHaveLength(2);
    expect(rows.map((r) => r.content?.map((c) => c.type))).toEqual([
      ['tableHeader', 'tableHeader', 'tableHeader'],
      ['tableCell', 'tableCell', 'tableCell']
    ]);
    expect(rows.map((r) => r.content?.[1]?.attrs?.align)).toEqual(['center', 'center']);
  });

  it('keeps the table id when the table is rebuilt', () => {
    const e = open(doc(para('x')));
    e.commands.insertTable({ rows: 2, cols: 2 });
    const idOf = () => (e.getJSON() as EncreNode).content?.find((n) => n.type === 'table')?.attrs?.id;
    const before = idOf();
    e.commands.addColumnAfter();
    expect(isBlockId(before)).toBe(true);
    expect(idOf()).toBe(before);
  });
});
