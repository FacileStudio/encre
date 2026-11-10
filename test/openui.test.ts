import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import type { EncreNode } from '../src/lib/schema.js';
import { loadEncre, openuiToDocument, validateDocument } from '../src/lib/wasm.js';

interface Vector {
  source: string;
  want_doc?: EncreNode;
  want_error?: string;
}

const dir = join(import.meta.dirname, '../spec/vectors/openui');
const vectors = readdirSync(dir)
  .filter((file) => file.endsWith('.json'))
  .sort()
  .map((file) => [file, JSON.parse(readFileSync(join(dir, file), 'utf8')) as Vector] as const);

beforeAll(async () => {
  await loadEncre(readFileSync(join(import.meta.dirname, '../src/lib/encre-wasm/encre_wasm_bg.wasm')));
});

describe('openui vectors through encre-wasm', () => {
  it('has the full vector set', () => {
    expect(vectors.length).toBeGreaterThanOrEqual(40);
  });

  it.each(vectors)('%s', (_file, vector) => {
    if (vector.want_doc) {
      const doc = openuiToDocument(vector.source);
      expect(doc).toEqual(vector.want_doc);
      expect(validateDocument(doc)).toBeNull();
    } else {
      expect(() => openuiToDocument(vector.source)).toThrow(new Error(vector.want_error));
    }
  });
});
