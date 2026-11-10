import { Editor, type Extensions } from '@tiptap/core';
import { yXmlFragmentToProsemirrorJSON } from '@tiptap/y-tiptap';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { expect, it } from 'vitest';
import * as Y from 'yjs';
import { compactDocument } from '../src/lib/compact.js';
import { encreExtensions, FRAGMENT } from '../src/lib/extensions/index.js';
import type { EncreNode } from '../src/lib/schema.js';
import { loadEncre, parseMarkdown, serializeMarkdown, validateDocument } from '../src/lib/wasm.js';

const OUT = join(import.meta.dirname, '../spec/vectors/yjs');

const SOURCES: Record<string, string> = {
  inline: [
    '# Title',
    '## Second',
    '### Third *with* `code`',
    'Plain **bold** *italic* ~~strike~~ `code` [link](https://facile.studio "Facile") **`bold code`** ' +
      '[**bold link**](https://facile.studio/a) é 漢字 😀\\\nafter a break',
    '\\'
  ].join('\n\n'),
  lists: [
    '- one\n- two\n  - nested\n  - nested two',
    '3. three\n4. four\n\n   ```sh\n   echo item\n   ```',
    '- [ ] open\n- [x] done\n  - [x] nested done'
  ].join('\n\n'),
  blocks: [
    '> quoted\n>\n> second',
    '```rust\nfn main() {}\n```',
    '```\nplain\n```',
    '---',
    '![alt text](https://facile.studio/a.png "A title")',
    '![](https://facile.studio/b.png)'
  ].join('\n\n'),
  table: '| none | left | center | right |\n| --- | :-- | :-: | --: |\n| a | **b** | `c` | d |\n| e | f | g | h |',
  custom: [
    '````callout tone="warning" title="Before you deploy"\nRun the gate **first**.\n\n- a list\n````',
    '```callout\nDefault tone.\n```',
    '```stat\n{"label": "Revenue", "value": "12 400 €", "delta": "+4%"}\n```',
    '```chart\n{"kind": "line", "title": "Signups", "labels": ["Mon", "Tue"], ' +
      '"series": [{"name": "web", "data": [3, 5.5]}, {"name": "app", "data": [-1, 0]}]}\n```'
  ].join('\n\n'),
  schema2: [
    '> [!WARNING]\n> Read ==this== first.',
    'Release [badge:success Shipped] and [status:pending], ==**bold** highlight==',
    '```mermaid\nflowchart LR\n  A --> B\n```',
    '````columns\n```column title="Pros" tone="success"\n- fast\n```\n\n```column\nplain [badge:accent x]\n```\n````',
    '```stat\n{"label": "Active", "value": "16", "delta": "", "sub": "In production"}\n```'
  ].join('\n\n')
};

function sequentialIds(extensions: Extensions): Extensions {
  let next = 0;
  const generateID = () => `y${String(next++).padStart(9, '0')}`;
  return extensions.map((ext) => (ext.name === 'uniqueID' ? ext.configure({ ...ext.options, generateID }) : ext));
}

function open(): { editor: Editor; ydoc: Y.Doc } {
  const ydoc = new Y.Doc();
  ydoc.clientID = 1;
  const extensions = sequentialIds(encreExtensions({ collaboration: { document: ydoc } }));
  return { editor: new Editor({ element: document.createElement('div'), extensions }), ydoc };
}

function edit(editor: Editor): void {
  editor.chain().focus().insertContent('Hello world').run();
  editor.chain().setTextSelection({ from: 7, to: 12 }).toggleBold().run();
  editor.chain().setTextSelection(12).insertContent(' and more').run();
  editor.chain().setTextSelection({ from: 1, to: 7 }).deleteSelection().run();
  editor.chain().setTextSelection(editor.state.doc.content.size).splitBlock().insertContent('Second line').run();
  editor.chain().setTextSelection({ from: 1, to: 6 }).toggleItalic().setLink({ href: 'https://facile.studio' }).run();
}

async function record(name: string, build: (editor: Editor) => void): Promise<void> {
  const { editor, ydoc } = open();
  build(editor);
  await new Promise((resolve) => setTimeout(resolve, 0));
  const doc = compactDocument(editor.getJSON() as EncreNode);
  expect(validateDocument(doc)).toBeNull();
  expect(compactDocument(yXmlFragmentToProsemirrorJSON(ydoc.getXmlFragment(FRAGMENT)) as EncreNode)).toEqual(doc);
  writeFileSync(join(OUT, `${name}.bin`), Y.encodeStateAsUpdate(ydoc));
  writeFileSync(join(OUT, `${name}.json`), `${JSON.stringify({ name, markdown: serializeMarkdown(doc), doc }, null, 2)}\n`);
  editor.destroy();
}

it('writes the y-prosemirror vectors from a real Tiptap editor', async () => {
  await loadEncre(readFileSync(join(import.meta.dirname, '../src/lib/encre-wasm/encre_wasm_bg.wasm')));
  mkdirSync(OUT, { recursive: true });
  for (const [name, markdown] of Object.entries(SOURCES)) {
    await record(name, (editor) => editor.commands.setContent(parseMarkdown(markdown)));
  }
  await record('edited', edit);
});
