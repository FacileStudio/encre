import { flushSync, mount, unmount } from 'svelte';
import { describe, expect, it } from 'vitest';
import { Awareness, applyAwarenessUpdate, encodeAwarenessUpdate } from 'y-protocols/awareness';
import * as Y from 'yjs';
import Editor from '../src/lib/Editor.svelte';
import { peers } from '../src/lib/Presence.svelte';
import { compactDocument } from '../src/lib/compact.js';
import type { AwarenessProvider } from '../src/lib/extensions/collaboration.js';
import type { EncreNode } from '../src/lib/schema.js';

const noRects = () => Object.assign([], { item: () => null }) as unknown as DOMRectList;
const zeroRect = () => new DOMRect(0, 0, 0, 0);
for (const proto of [Range.prototype, Element.prototype, Text.prototype] as { getClientRects?: unknown; getBoundingClientRect?: unknown }[]) {
  proto.getClientRects ??= noRects;
  proto.getBoundingClientRect ??= zeroRect;
}

function pair(): [Y.Doc, Y.Doc, Awareness, Awareness] {
  const a = new Y.Doc();
  const b = new Y.Doc();
  a.on('update', (update: Uint8Array, origin: unknown) => origin !== b && Y.applyUpdate(b, update, a));
  b.on('update', (update: Uint8Array, origin: unknown) => origin !== a && Y.applyUpdate(a, update, b));
  const [wa, wb] = [new Awareness(a), new Awareness(b)];
  const relay = (from: Awareness, to: Awareness) =>
    from.on('update', ({ added, updated, removed }: Record<string, number[]>) =>
      applyAwarenessUpdate(to, encodeAwarenessUpdate(from, [...added, ...updated, ...removed]), 'relay')
    );
  relay(wa, wb);
  relay(wb, wa);
  return [a, b, wa, wb];
}

function provider(awareness: Awareness): AwarenessProvider {
  return { awareness, synced: true, on: () => undefined, off: () => undefined };
}

function open(ydoc: Y.Doc, awareness: Awareness, name: string) {
  const target = document.createElement('div');
  document.body.append(target);
  const editor = mount(Editor, { target, props: { ydoc, provider: provider(awareness), user: { name }, toolbar: false } });
  flushSync();
  return { editor, target };
}

describe('two collaborating editors', () => {
  it('converge on one document and see each other', async () => {
    const [a, b, wa, wb] = pair();
    const left = open(a, wa, 'Ana');
    const right = open(b, wb, 'Bo');
    left.editor.instance()?.chain().focus().insertContent('Hello from Ana').run();
    right.editor.instance()?.chain().setTextSelection(1).insertContent('Bo says hi. ').run();
    await new Promise((resolve) => setTimeout(resolve, 0));
    const one = compactDocument(left.editor.getDocument() as EncreNode);
    expect(compactDocument(right.editor.getDocument() as EncreNode)).toEqual(one);
    expect(JSON.stringify(one)).toContain('Hello from Ana');
    expect(JSON.stringify(one)).toContain('Bo says hi.');
    expect(peers(provider(wa)).map((p) => p.name)).toEqual(['Ana', 'Bo']);
    flushSync();
    expect(left.target.querySelectorAll('li[title]')).toHaveLength(2);
    expect(one.content?.every((block) => typeof block.attrs?.id === 'string')).toBe(true);
    void unmount(left.editor);
    void unmount(right.editor);
  });
});

describe('a provider that syncs after the editor opens', () => {
  it('gives the synced content block ids once the sync event fires', async () => {
    const ydoc = new Y.Doc();
    const listeners: ((synced: boolean) => void)[] = [];
    const late: AwarenessProvider = {
      awareness: new Awareness(ydoc),
      synced: false,
      on: (_event, listener) => listeners.push(listener),
      off: () => undefined
    };
    const target = document.createElement('div');
    const editor = mount(Editor, { target, props: { ydoc, provider: late, user: { name: 'Ana' }, toolbar: false } });
    flushSync();
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(listeners).toHaveLength(1);
    const remote = new Y.Doc();
    remote.getXmlFragment('default').insert(0, [new Y.XmlElement('paragraph')]);
    Y.applyUpdate(ydoc, Y.encodeStateAsUpdate(remote));
    listeners.forEach((listener) => listener(true));
    await new Promise((resolve) => setTimeout(resolve, 0));
    const doc = compactDocument(editor.getDocument() as EncreNode);
    expect(doc.content?.map((block) => typeof block.attrs?.id)).toEqual(['string']);
    void unmount(editor);
  });
});

describe('presence placement', () => {
  const mountWith = (presence?: 'inline' | 'none') => {
    const ydoc = new Y.Doc();
    const awareness = new Awareness(ydoc);
    awareness.setLocalStateField('user', { name: 'Ana' });
    const target = document.createElement('div');
    const props = { ydoc, provider: provider(awareness), user: { name: 'Ana' }, toolbar: false, presence };
    const editor = mount(Editor, { target, props });
    flushSync();
    return { editor, target };
  };

  it('renders avatars above the editor by default', () => {
    const { editor, target } = mountWith();
    expect(target.querySelectorAll('li[title]')).toHaveLength(1);
    void unmount(editor);
  });

  it('renders no avatars with presence none', () => {
    const { editor, target } = mountWith('none');
    expect(target.querySelector('[aria-label="Editing now"]')).toBeNull();
    expect(target.querySelectorAll('li[title]')).toHaveLength(0);
    void unmount(editor);
  });
});
