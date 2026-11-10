import { Editor } from '@tiptap/core';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import * as Y from 'yjs';
import { compactDocument } from '../src/lib/compact.js';
import { encreExtensions, FRAGMENT } from '../src/lib/extensions/index.js';
import type { EncreNode } from '../src/lib/schema.js';

const dir = join(import.meta.dirname, '../spec/vectors/yjs');
const names = readdirSync(dir)
  .filter((file) => file.endsWith('.json'))
  .map((file) => file.slice(0, -'.json'.length))
  .sort();

function load(file: string): { editor: Editor; ydoc: Y.Doc } {
  const ydoc = new Y.Doc();
  Y.applyUpdate(ydoc, readFileSync(join(dir, file)));
  const extensions = encreExtensions({ collaboration: { document: ydoc } });
  return { editor: new Editor({ element: document.createElement('div'), extensions }), ydoc };
}

function expected(name: string): EncreNode {
  return JSON.parse(readFileSync(join(dir, `${name}.json`), 'utf8')).doc as EncreNode;
}

describe('y-prosemirror vectors', () => {
  it('has the full vector set', () => {
    expect(names.length).toBeGreaterThanOrEqual(6);
  });

  it.each(names)('%s: Tiptap reads its own update back', (name) => {
    const { editor } = load(`${name}.bin`);
    expect(compactDocument(editor.getJSON() as EncreNode)).toEqual(expected(name));
    editor.destroy();
  });

  it.each(names)('%s: Tiptap shows the update Rust wrote, and keeps editing it', (name) => {
    const { editor, ydoc } = load(`${name}.rust.bin`);
    expect(compactDocument(editor.getJSON() as EncreNode)).toEqual(expected(name));
    editor.chain().setTextSelection(editor.state.doc.content.size).insertContent({ type: 'paragraph', content: [{ type: 'text', text: 'tail' }] }).run();
    const back = compactDocument(editor.getJSON() as EncreNode);
    expect(back.content?.at(-1)?.content).toEqual([{ type: 'text', text: 'tail' }]);
    expect(ydoc.getXmlFragment(FRAGMENT).length).toBe(back.content?.length);
    editor.destroy();
  });

  it('names the overlapping code mark the way encre-rs writes it', () => {
    const ydoc = new Y.Doc();
    Y.applyUpdate(ydoc, readFileSync(join(dir, 'inline.bin')));
    const keys = new Set<string>();
    for (const node of ydoc.getXmlFragment(FRAGMENT).createTreeWalker(() => true)) {
      if (node instanceof Y.XmlText) for (const op of node.toDelta()) Object.keys(op.attributes ?? {}).forEach((k) => keys.add(k));
    }
    expect([...keys].filter((k) => k.startsWith('code'))).toEqual(['code--LpaW+ak5']);
  });
});
