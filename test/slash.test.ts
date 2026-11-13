import { flushSync, mount, unmount } from 'svelte';
import { afterEach, describe, expect, it } from 'vitest';
import Editor from '../src/lib/Editor.svelte';
import type { EncreNode } from '../src/lib/schema.js';

const mounted: ReturnType<typeof mount>[] = [];
const doc: EncreNode = { type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'Bonjour' }] }] };
const tick = () => new Promise((resolve) => setTimeout(resolve, 0));
const rect = () => new DOMRect(0, 0, 0, 0);
const rects = () => Object.assign([rect()], { item: () => rect() }) as unknown as DOMRectList;

Range.prototype.getBoundingClientRect = rect;
Range.prototype.getClientRects = rects;
Element.prototype.scrollIntoView = () => undefined;

async function open() {
  const target = document.createElement('div');
  document.body.append(target);
  const app = mount(Editor, { target, props: { content: doc, toolbar: false } });
  mounted.push(app);
  flushSync();
  const editor = app.instance()!;
  editor.chain().focus('end').insertContent(' /ti').run();
  await tick();
  flushSync();
  return { editor, list: () => document.querySelector('[role="listbox"]') };
}

function enter(dom: HTMLElement) {
  dom.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }));
}

afterEach(() => mounted.splice(0).forEach((app) => void unmount(app)));

describe('slash menu dismissal', () => {
  it('opens on "/", closes on a press outside, keeps the text and lets Enter split', async () => {
    const { editor, list } = await open();
    expect(list()).not.toBeNull();
    document.body.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }));
    await tick();
    flushSync();
    expect(list()).toBeNull();
    expect(editor.getText()).toBe('Bonjour /ti');
    enter(editor.view.dom);
    expect(editor.getJSON().content?.map((block) => block.type)).toEqual(['paragraph', 'paragraph']);
    expect(editor.getText()).toBe('Bonjour /ti\n\n');
  });

  it('closes when the window loses focus or the page scrolls, and "/" reopens it', async () => {
    const { editor, list } = await open();
    window.dispatchEvent(new Event('blur'));
    flushSync();
    expect(list()).toBeNull();
    editor.chain().focus('end').insertContent(' /').run();
    await tick();
    flushSync();
    expect(list()).not.toBeNull();
    document.dispatchEvent(new Event('scroll'));
    flushSync();
    expect(list()).toBeNull();
  });

  it('keeps open on a press inside the list', async () => {
    const { list } = await open();
    list()!.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }));
    flushSync();
    expect(list()).not.toBeNull();
  });
});
