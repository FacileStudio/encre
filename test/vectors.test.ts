import { getSchema } from '@tiptap/core';
import { Node as PMNode } from '@tiptap/pm/model';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import { encreExtensions } from '../src/lib/extensions/index.js';
import { insertable, looksLikeMarkdown } from '../src/lib/paste.js';
import type { EncreNode } from '../src/lib/schema.js';
import { loadEncre, parseMarkdown, serializeCommonMark, serializeMarkdown, validateDocument } from '../src/lib/wasm.js';

interface Vector {
  markdown: string;
  doc: EncreNode;
  canonical?: string;
}

const dir = join(import.meta.dirname, '../spec/vectors/markdown');
const vectors = readdirSync(dir)
  .filter((file) => file.endsWith('.json'))
  .sort()
  .map((file) => [file, JSON.parse(readFileSync(join(dir, file), 'utf8')) as Vector] as const);
const pm = getSchema(encreExtensions());

beforeAll(async () => {
  await loadEncre(readFileSync(join(import.meta.dirname, '../src/lib/encre-wasm/encre_wasm_bg.wasm')));
});

describe('markdown vectors through encre-wasm', () => {
  it('has the full vector set', () => {
    expect(vectors.length).toBeGreaterThanOrEqual(40);
  });

  it.each(vectors)('%s', (_file, vector) => {
    const canonical = vector.canonical ?? vector.markdown;
    expect(parseMarkdown(vector.markdown)).toEqual(vector.doc);
    expect(serializeMarkdown(vector.doc)).toBe(canonical);
    expect(validateDocument(vector.doc)).toBeNull();
    const tiptap = PMNode.fromJSON(pm, vector.doc);
    tiptap.check();
    expect(serializeMarkdown(tiptap.toJSON() as EncreNode)).toBe(canonical);
  });
});

describe('wasm entry points', () => {
  it('reports why a document is invalid', () => {
    expect(validateDocument({ type: 'doc', content: [] })).toMatch(/does not match/);
    expect(() => serializeMarkdown({ type: 'doc', content: [{ type: 'paragraph', attrs: { colour: 'red' } }] })).toThrow(/unknown attribute/);
  });

  it('exports CommonMark without Facile-only syntax', () => {
    expect(serializeCommonMark(parseMarkdown('a\\\n\\\n\n\\'))).toBe('a');
  });
});

describe('markdown paste', () => {
  it('recognises markdown and inserts a lone paragraph inline', () => {
    expect(looksLikeMarkdown('# Title')).toBe(true);
    expect(looksLikeMarkdown('just words, nothing else')).toBe(false);
    expect(insertable(parseMarkdown('**bold**'))).toEqual([{ type: 'text', text: 'bold', marks: [{ type: 'bold' }] }]);
    expect(insertable(parseMarkdown('- a\n- b'))[0]?.type).toBe('bulletList');
  });
});
