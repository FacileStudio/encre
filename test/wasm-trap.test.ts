import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import { initSync } from '../src/lib/encre-wasm/encre_wasm.js';
import { loadEncre, parseMarkdown, serializeMarkdown, validateDocument } from '../src/lib/wasm.js';
import type { EncreNode } from '../src/lib/schema.js';

const bytes = readFileSync(join(import.meta.dirname, '../src/lib/encre-wasm/encre_wasm_bg.wasm'));
const doc: EncreNode = { type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'still here' }] }] };

function poison(): void {
  new Uint8Array(initSync({ module: bytes }).memory.buffer).fill(0xff);
}

beforeAll(async () => {
  await loadEncre(bytes);
});

describe('wasm trap recovery', () => {
  it('runs the next call on a fresh instance after a trap', () => {
    poison();
    expect(() => serializeMarkdown(doc)).toThrow(WebAssembly.RuntimeError);
    expect(serializeMarkdown(doc)).toBe('still here');
    expect(validateDocument(doc)).toBeNull();
  });

  it('lets loadEncre replace a trapped instance', async () => {
    poison();
    expect(() => parseMarkdown('# a')).toThrow(WebAssembly.RuntimeError);
    await loadEncre(bytes);
    expect(parseMarkdown('# a').type).toBe('doc');
  });

  it('reports a trap inside validateDocument instead of calling the document invalid', () => {
    poison();
    expect(() => validateDocument(doc)).toThrow(WebAssembly.RuntimeError);
    expect(validateDocument(doc)).toBeNull();
  });
});
