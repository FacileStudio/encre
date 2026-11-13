import { Editor } from '@tiptap/core';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import EditorView from '../src/lib/Editor.svelte';
import { compactDocument } from '../src/lib/compact.js';
import { embedKind, fileTag, frameSrc, humanSize } from '../src/lib/embed.js';
import { encreExtensions } from '../src/lib/extensions/index.js';
import { pasteEmbed } from '../src/lib/files.js';
import type { EncreNode } from '../src/lib/schema.js';
import { selectionMarkdown } from '../src/lib/selection.js';
import { loadEncre, parseMarkdown, serializeCommonMark, serializeMarkdown, validateDocument } from '../src/lib/wasm.js';

let editor: Editor | null = null;
const json = () => compactDocument(editor!.getJSON() as EncreNode);

beforeAll(async () => {
  const rect = () => new DOMRect(0, 0, 0, 0);
  Range.prototype.getClientRects = () => [rect()] as unknown as DOMRectList;
  Range.prototype.getBoundingClientRect = rect;
  await loadEncre(readFileSync(join(import.meta.dirname, '../src/lib/encre-wasm/encre_wasm_bg.wasm')));
});

afterEach(() => editor?.destroy());

function open(markdown: string): Editor {
  editor = new Editor({ element: document.createElement('div'), extensions: encreExtensions(), content: parseMarkdown(markdown) });
  return editor;
}

describe('embedKind', () => {
  it('maps the Facile apps by host and everything else to link', () => {
    expect(embedKind('https://toile.facile.studio/b/abc')).toBe('toile');
    expect(embedKind('https://feuilles.facile.studio/w/1')).toBe('feuilles');
    expect(embedKind('https://vision.facile.studio/s/x')).toBe('vision');
    expect(embedKind('https://docs.facile.studio/d/x')).toBe('docs');
    expect(embedKind('https://nuage.facile.studio/s/x')).toBe('nuage');
    expect(embedKind('https://facile.studio/')).toBe('link');
    expect(embedKind('https://toile.facile.studio.evil.com/b')).toBe('link');
    expect(embedKind('http://toile.facile.studio/b/abc')).toBe('link');
    expect(embedKind('not a url')).toBe('link');
    expect(embedKind('https://localhost:5173/b/1', { 'localhost:5173': 'toile' })).toBe('toile');
  });

  it('frames apps with embed=1 but Vision as it is', () => {
    expect(frameSrc('https://toile.facile.studio/b/abc?x=1', 'toile')).toBe('https://toile.facile.studio/b/abc?x=1&embed=1');
    expect(frameSrc('https://vision.facile.studio/s/x', 'vision')).toBe('https://vision.facile.studio/s/x');
  });

  it('formats sizes and tags', () => {
    expect(humanSize(0)).toBe('0 B');
    expect(humanSize(1536)).toBe('1.5 KB');
    expect(humanSize(25 * 1024 * 1024)).toBe('25 MB');
    expect(fileTag('report.pdf', '')).toBe('PDF');
    expect(fileTag('notes', 'text/plain')).toBe('PLAI');
  });
});

describe('embed and file blocks', () => {
  const embed = '```embed\n{"url": "https://toile.facile.studio/b/abc", "kind": "toile", "title": "Board"}\n```';
  const file = '```file\n{"src": "https://nuage.facile.studio/f/1", "name": "a.pdf", "size": 2048, "mime": "application/pdf"}\n```';

  it('round-trips through the editor', () => {
    open(`${embed}\n\n${file}`);
    expect(validateDocument(json())).toBeNull();
    expect(serializeMarkdown(json())).toBe(`${embed}\n\n${file}`);
    expect(serializeCommonMark(json())).toBe('[Board](https://toile.facile.studio/b/abc)\n\n[a.pdf](https://nuage.facile.studio/f/1)');
  });

  it('refuses an embed that is not https', () => {
    const doc = { type: 'doc', content: [{ type: 'embed', attrs: { url: 'http://a.b' } }] };
    expect(validateDocument(doc)).not.toBeNull();
  });
});

describe('selectionMarkdown', () => {
  it('gives the text range inside one block, with its marks', () => {
    const e = open('Hello **bold** world');
    e.commands.setTextSelection({ from: 7, to: 15 });
    expect(selectionMarkdown(e)).toBe('**bold** wor');
  });

  it('gives every block the selection touches', () => {
    const e = open('# One\n\ntwo\n\n- three');
    e.commands.setTextSelection({ from: 3, to: 9 });
    expect(selectionMarkdown(e)).toBe('# One\n\ntwo');
    e.commands.setTextSelection(8);
    expect(selectionMarkdown(e)).toBe('two');
  });
});

describe('paste-to-embed', () => {
  it('embeds a lone https URL pasted on an empty paragraph', () => {
    const e = open('\\');
    e.commands.setTextSelection(1);
    expect(pasteEmbed(e, ' https://feuilles.facile.studio/w/9 ')).toBe(true);
    expect(json().content?.[0]).toMatchObject({ type: 'embed', attrs: { url: 'https://feuilles.facile.studio/w/9', kind: 'feuilles' } });
  });

  it('leaves text, non-empty paragraphs and refused hosts alone', () => {
    const e = open('words');
    e.commands.setTextSelection(3);
    expect(pasteEmbed(e, 'https://facile.studio')).toBe(false);
    const empty = open('\\');
    empty.commands.setTextSelection(1);
    expect(pasteEmbed(empty, 'see https://facile.studio')).toBe(false);
    const onembed = vi.fn(() => false);
    expect(pasteEmbed(empty, 'https://facile.studio', onembed)).toBe(false);
    expect(onembed).toHaveBeenCalledWith('https://facile.studio');
    expect(json().content?.[0]?.type).toBe('paragraph');
  });
});

describe('the Editor component', () => {
  const paste = (component: ReturnType<typeof mount>, text: string) => {
    const event = new Event('paste', { bubbles: true, cancelable: true });
    const getData = (type: string) => (type === 'text/plain' ? text : '');
    Object.defineProperty(event, 'clipboardData', { value: { files: [], types: ['text/plain'], getData } });
    component.instance()!.view.dom.dispatchEvent(event);
  };

  const mountEditor = (props: Record<string, unknown>) => {
    const target = document.createElement('div');
    document.body.append(target);
    const component = mount(EditorView, { target, props: { toolbar: false, ...props } });
    flushSync();
    return component;
  };

  it('pastes a URL as an embed unless onembed refuses it', () => {
    const embeds = mountEditor({});
    paste(embeds, 'https://toile.facile.studio/b/1');
    expect(embeds.getDocument()?.content?.[0]).toMatchObject({ type: 'embed', attrs: { kind: 'toile' } });
    const refused = mountEditor({ onembed: () => false });
    paste(refused, 'https://toile.facile.studio/b/1');
    expect(refused.getDocument()?.content?.[0]?.type).toBe('paragraph');
    void unmount(embeds);
    void unmount(refused);
  });

  it('hands a right click to oncontextmenu instead of the browser', () => {
    const oncontextmenu = vi.fn();
    const component = mountEditor({ oncontextmenu, content: parseMarkdown('Hello') });
    const event = new MouseEvent('contextmenu', { bubbles: true, cancelable: true });
    component.instance()!.view.dom.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
    expect(oncontextmenu).toHaveBeenCalledWith(expect.objectContaining({ empty: true, markdown: 'Hello' }));
    expect(component.run('heading2')).toBe(true);
    expect(component.selectionMarkdown()).toBe('## Hello');
    void unmount(component);
  });
});
