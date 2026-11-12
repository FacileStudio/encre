import { NodeSelection, TextSelection } from '@tiptap/pm/state';
import { flushSync, mount, unmount } from 'svelte';
import { afterEach, describe, expect, it } from 'vitest';
import Editor from '../src/lib/Editor.svelte';
import { blockCommands } from '../src/lib/blocks.js';
import { CATALOGUES, labelsFor, type Labels, type Locale } from '../src/lib/labels.js';
import type { EncreNode } from '../src/lib/schema.js';
import { imageRefusal } from '../src/lib/upload.js';
import { settingsOf } from '../src/lib/extensions/index.js';

const para = (text: string): EncreNode => ({ type: 'paragraph', content: [{ type: 'text', text }] });
const cell = (type: string): EncreNode => ({ type, content: [para('x')] });
const doc: EncreNode = {
  type: 'doc',
  content: [
    { type: 'paragraph', content: [{ type: 'text', text: 'a ' }, { type: 'badge', attrs: { tone: 'info' }, content: [{ type: 'text', text: 'b' }] }] },
    { type: 'callout', content: [para('c')] },
    { type: 'stat', attrs: { label: 'L', value: '1' } },
    { type: 'chart' },
    { type: 'diagram', content: [{ type: 'text', text: 'A --> B' }] },
    { type: 'columns', content: [{ type: 'column', content: [para('d')] }, { type: 'column', content: [para('e')] }] },
    { type: 'table', content: [{ type: 'tableRow', content: [cell('tableHeader')] }, { type: 'tableRow', content: [cell('tableCell')] }] }
  ]
};

function values(labels: Labels): string[] {
  const fixed = Object.values(labels).filter((v): v is string => typeof v === 'string');
  return [...fixed, ...Object.values(labels.tones), ...[1, 2, 3].map(labels.heading), labels.imageSize(2048)];
}

function shown(root: HTMLElement): Set<string> {
  const out = new Set<string>();
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  for (let node = walker.nextNode(); node; node = walker.nextNode()) out.add(node.textContent?.trim() ?? '');
  for (const element of root.querySelectorAll('*')) {
    for (const name of ['aria-label', 'placeholder', 'title', 'data-placeholder']) out.add(element.getAttribute(name)?.trim() ?? '');
  }
  return out;
}

const mounted: ReturnType<typeof mount>[] = [];
afterEach(() => mounted.splice(0).forEach((editor) => void unmount(editor)));

function everything(locale: Locale): Set<string> {
  const target = document.createElement('div');
  document.body.append(target);
  const editor = mount(Editor, { target, props: { content: doc, locale, placeholder: undefined } });
  mounted.push(editor);
  flushSync();
  const tiptap = editor.instance()!;
  const seen = shown(target);
  const visit = (select: (pos: number) => void) => {
    tiptap.state.doc.descendants((node, pos) => {
      if (['stat', 'chart', 'badge', 'columns', 'table', 'diagram'].includes(node.type.name)) {
        select(pos);
        flushSync();
        target.querySelectorAll<HTMLButtonElement>('button[aria-haspopup]').forEach((button) => button.click());
        flushSync();
        shown(document.body).forEach((s) => seen.add(s));
      }
    });
  };
  visit((pos) => tiptap.view.dispatch(tiptap.state.tr.setSelection(NodeSelection.create(tiptap.state.doc, pos))));
  visit((pos) => tiptap.view.dispatch(tiptap.state.tr.setSelection(TextSelection.near(tiptap.state.doc.resolve(pos + 2)))));
  return seen;
}

describe('editor locale', () => {
  it('shows no French string in English', () => {
    const english = new Set(values(CATALOGUES.en));
    const french = values(CATALOGUES.fr).filter((v) => !english.has(v));
    const seen = everything('en');
    expect(seen.has('Callout title')).toBe(true);
    for (const s of ['Column title', 'Subtitle', 'Add a row', 'Add a text column', 'Bars', 'Accent', 'Highlight']) expect(seen.has(s), s).toBe(true);
    expect(french.filter((v) => seen.has(v))).toEqual([]);
  });

  it('shows French by default, so the English test can fail', () => {
    const seen = everything('fr');
    expect(seen.has("Titre de l'encadré")).toBe(true);
  });

  it('updates mounted node views when the labels change', () => {
    const target = document.createElement('div');
    document.body.append(target);
    const editor = mount(Editor, { target, props: { content: doc, locale: 'fr' } });
    mounted.push(editor);
    flushSync();
    expect(shown(target).has("Titre de l'encadré")).toBe(true);
    settingsOf(editor.instance()!).labels = labelsFor('en');
    flushSync();
    expect(shown(target).has('Callout title')).toBe(true);
    expect(shown(target).has("Titre de l'encadré")).toBe(false);
  });

  it('names commands and refusals in the chosen locale, with overrides on top', () => {
    const labels = labelsFor('de', { bold: 'Stark', tones: { info: 'Hinweis' } });
    expect(labels.bold).toBe('Stark');
    expect(labels.tones.info).toBe('Hinweis');
    expect(labels.tones.danger).toBe('Gefahr');
    expect(blockCommands(labelsFor('es')).map((c) => c.label)).toContain('Diagrama');
    expect(imageRefusal(new File(['x'], 'a.svg', { type: 'image/svg+xml' }), undefined, labelsFor('en'))).toBe(CATALOGUES.en.imageType);
  });

  it('ships every key in every catalogue', () => {
    const keys = Object.keys(CATALOGUES.fr).sort();
    for (const locale of ['en', 'es', 'de'] as const) expect(Object.keys(CATALOGUES[locale]).sort()).toEqual(keys);
  });
});
