import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import Editor from '../src/lib/Editor.svelte';
import type { EncreNode } from '../src/lib/schema.js';
import { EncreDocumentError, EncreMarkdownError, loadEncre, serializeMarkdown } from '../src/lib/index.js';

const FORBIDDEN = 'text is empty or holds a forbidden character';
const para = (text: string): EncreNode => ({ type: 'paragraph', content: [{ type: 'text', text }] });
const invalid: EncreNode = { type: 'doc', content: [para('Title'), para('bell \u0007 here')] };
const mounted: ReturnType<typeof mount>[] = [];
const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

function ladder(depth: number): string {
  const ticks = (d: number) => '`'.repeat(d + 3);
  const levels = [...Array(depth).keys()];
  const open = [...levels].reverse().map((d) => `${ticks(d)}callout\n`).join('');
  const close = levels.map((d) => `\n${ticks(d)}`).join('');
  return `# Notes\n\n${open}x${close}`;
}

function open(props: Record<string, unknown>) {
  const target = document.createElement('div');
  document.body.append(target);
  const editor = mount(Editor, { target, props: { toolbar: false, ...props } });
  mounted.push(editor);
  flushSync();
  return editor;
}

function caught(run: () => unknown): unknown {
  try {
    run();
  } catch (error) {
    return error;
  }
  return undefined;
}

beforeAll(async () => {
  const rect = () => new DOMRect(0, 0, 0, 0);
  Range.prototype.getClientRects = () => [rect()] as unknown as DOMRectList;
  Range.prototype.getBoundingClientRect = rect;
  await loadEncre(readFileSync(join(import.meta.dirname, '../src/lib/encre-wasm/encre_wasm_bg.wasm')));
});

afterEach(() => mounted.splice(0).forEach((editor) => void unmount(editor)));

describe('placed export refusals', () => {
  it('throws an EncreDocumentError with the path and the id of the enclosing block', () => {
    const doc: EncreNode = {
      type: 'doc',
      content: [para('Title'), { type: 'paragraph', attrs: { id: 'abcdefghjk' }, content: [{ type: 'text', text: 'a\u0007' }] }]
    };
    const error = caught(() => serializeMarkdown(doc));
    expect(error).toBeInstanceOf(EncreDocumentError);
    expect(error).toMatchObject({
      name: 'EncreDocumentError',
      message: `doc.content[1]: ${FORBIDDEN}`,
      path: 'doc.content[1]',
      blockId: 'abcdefghjk'
    });
  });

  it('leaves blockId unset when no block on the path has an id', () => {
    expect(caught(() => serializeMarkdown(invalid))).toMatchObject({ path: 'doc.content[1]', blockId: undefined });
  });

  it('hands the refusal to onerror and reveals the node it names', async () => {
    const onerror = vi.fn();
    const editor = open({ content: invalid, onerror });
    await tick();
    const error = await editor.exportMarkdown().catch((e: unknown) => e);
    expect(error).toBeInstanceOf(EncreDocumentError);
    expect(onerror).toHaveBeenCalledWith(error);
    const { path, blockId } = error as EncreDocumentError;
    expect(blockId).toBe(editor.getDocument()?.content?.[1]?.attrs?.id);
    const tiptap = editor.instance()!;
    expect(editor.revealPath(path)).toBe(true);
    const { from, to } = tiptap.state.selection;
    expect(tiptap.state.doc.textBetween(from, to)).toBe('bell \u0007 here');
    expect(editor.revealPath('doc.content[0].content[0]')).toBe(true);
    expect(tiptap.state.doc.textBetween(tiptap.state.selection.from, tiptap.state.selection.to)).toBe('Title');
    expect(editor.revealBlock(blockId!)).toBe(true);
    expect(tiptap.state.selection.$from.parent.textContent).toBe('bell \u0007 here');
    expect(editor.revealPath('doc.content[7]')).toBe(false);
    expect(editor.revealBlock('zzzzzzzzzz')).toBe(false);
  });

  it('keeps a revealed node below the sticky toolbar', async () => {
    const editor = open({ content: invalid, toolbar: true });
    await tick();
    const bar = document.querySelector('[role="toolbar"]')!.parentElement!;
    Object.defineProperty(bar, 'offsetHeight', { value: 48 });
    const view = editor.instance()!.view;
    expect(editor.revealPath('doc.content[1]')).toBe(true);
    expect(view.someProp('scrollThreshold')).toMatchObject({ top: 48 });
    expect(view.someProp('scrollMargin')).toMatchObject({ top: 53 });
    expect(editor.revealBlock(editor.getDocument()!.content![0]!.attrs!.id as string)).toBe(true);
    expect(view.someProp('scrollMargin')).toMatchObject({ top: 53 });
  });
});

describe('refused paste', () => {
  it('pastes nine nested callouts as literal text and reports the parse error', () => {
    const onerror = vi.fn();
    const editor = open({ content: { type: 'doc', content: [{ type: 'paragraph' }] }, onerror });
    const text = ladder(9);
    const event = new Event('paste', { bubbles: true, cancelable: true });
    const getData = (type: string) => (type === 'text/plain' ? text : '');
    Object.defineProperty(event, 'clipboardData', { value: { files: [], types: ['text/plain'], getData } });
    editor.instance()!.view.dom.dispatchEvent(event);
    expect(onerror).toHaveBeenCalledTimes(1);
    expect(onerror.mock.calls[0]?.[0]).toBeInstanceOf(EncreMarkdownError);
    expect(onerror.mock.calls[0]?.[0]).toMatchObject({ line: 11, column: 1, path: 'doc' });
    const blocks = editor.getDocument()?.content ?? [];
    expect(blocks.some((block) => block.type === 'callout')).toBe(false);
    expect(editor.instance()!.state.doc.textContent).toContain('```callout');
  });
});
