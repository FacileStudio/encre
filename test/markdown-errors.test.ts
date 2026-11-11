import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import { EncreMarkdownError, loadEncre, parseMarkdown, validateMarkdown } from '../src/lib/index.js';

const NESTING = 'custom blocks nest more than 8 levels deep';

function ladder(depth: number): string {
  const ticks = (d: number) => '`'.repeat(d + 3);
  const levels = [...Array(depth).keys()];
  const open = [...levels].reverse().map((d) => `${ticks(d)}callout\n`).join('');
  const close = levels.map((d) => `\n${ticks(d)}`).join('');
  return `# Notes\n\n${open}x${close}`;
}

beforeAll(async () => {
  await expect(loadEncre(new Uint8Array([0, 1, 2, 3]))).rejects.toThrow();
  await loadEncre(readFileSync(join(import.meta.dirname, '../src/lib/encre-wasm/encre_wasm_bg.wasm')));
});

describe('validateMarkdown', () => {
  it('accepts valid markdown', () => {
    expect(validateMarkdown('# Fine')).toEqual({ valid: true, errors: [] });
    expect(validateMarkdown(ladder(8))).toEqual({ valid: true, errors: [] });
  });

  it('places nine nested custom blocks where the CLI does', () => {
    expect(validateMarkdown(ladder(9))).toEqual({
      valid: false,
      errors: [{ path: 'doc', message: NESTING, line: 11, column: 1 }]
    });
  });

  it('places a schema error at its block', () => {
    expect(validateMarkdown('# Title\n\n- ok\n- bell \u0007 here\n').errors).toEqual([
      { path: 'doc.content[1].content[1].content[0]', message: expect.any(String), line: 4, column: 3 }
    ]);
  });
});

describe('parseMarkdown errors', () => {
  it('throws an EncreMarkdownError with the position and the old message', () => {
    let caught: unknown;
    try {
      parseMarkdown(ladder(9));
    } catch (error) {
      caught = error;
    }
    expect(caught).toBeInstanceOf(EncreMarkdownError);
    expect(caught).toBeInstanceOf(Error);
    expect(caught).toMatchObject({
      name: 'EncreMarkdownError',
      message: `markdown could not be read: ${NESTING}`,
      line: 11,
      column: 1,
      path: 'doc'
    });
  });

  it('places markdown nested too deep on its line', () => {
    expect(() => parseMarkdown('>'.repeat(100))).toThrow(expect.objectContaining({ line: 1, column: 65 }));
  });
});
