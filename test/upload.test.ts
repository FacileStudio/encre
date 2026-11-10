import { flushSync, mount, unmount } from 'svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import Editor from '../src/lib/Editor.svelte';
import type { EncreNode } from '../src/lib/schema.js';
import { imageCommand, imageRefusal, type UploadedImage } from '../src/lib/upload.js';

type Props = Record<string, unknown>;
const mounted: ReturnType<typeof mount>[] = [];
const doc: EncreNode = { type: 'doc', content: [{ type: 'paragraph', content: [{ type: 'text', text: 'Hello world' }] }] };
const png = (name = 'a.png', bytes = 4) => new File([new Uint8Array(bytes)], name, { type: 'image/png' });
const svg = () => new File(['<svg/>'], 'a.svg', { type: 'image/svg+xml' });
const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

function open(props: Props) {
  const target = document.createElement('div');
  document.body.append(target);
  const editor = mount(Editor, { target, props: { content: doc, toolbar: false, ...props } });
  mounted.push(editor);
  flushSync();
  return { editor, target, view: editor.instance()!.view };
}

function transfer(files: File[]) {
  return { files, types: ['Files'], getData: () => '' };
}

function fire(dom: HTMLElement, type: 'paste' | 'drop', files: File[]): void {
  const event = new Event(type, { bubbles: true, cancelable: true });
  Object.defineProperty(event, type === 'paste' ? 'clipboardData' : 'dataTransfer', { value: transfer(files) });
  dom.dispatchEvent(event);
}

function deferred() {
  let settle!: { resolve: (image: UploadedImage) => void; reject: (error: Error) => void };
  const promise = new Promise<UploadedImage>((resolve, reject) => (settle = { resolve, reject }));
  return { promise, ...settle };
}

const blocks = (editor: { getDocument(): EncreNode | null }) => editor.getDocument()?.content ?? [];

afterEach(() => mounted.splice(0).forEach((editor) => void unmount(editor)));

describe('image upload hook', () => {
  it('uploads a pasted image and inserts it at the caret with the returned src', async () => {
    const onimage = vi.fn(async () => ({ src: '/api/attachments/1', alt: 'Logo' }));
    const { editor, view } = open({ onimage });
    editor.instance()!.commands.setTextSelection(6);
    fire(view.dom, 'paste', [png()]);
    await tick();
    expect(onimage).toHaveBeenCalledTimes(1);
    expect(blocks(editor).map((b) => b.type)).toEqual(['paragraph', 'image', 'paragraph']);
    expect(blocks(editor)[1]?.attrs).toMatchObject({ src: '/api/attachments/1', alt: 'Logo' });
  });

  it('uploads a dropped image at the drop point', async () => {
    const onimage = vi.fn(async () => ({ src: '/img/drop.webp' }));
    const { editor, view } = open({ onimage });
    editor.instance()!.commands.setTextSelection(12);
    view.posAtCoords = () => ({ pos: 0, inside: -1 });
    fire(view.dom, 'drop', [new File(['w'], 'd.webp', { type: 'image/webp' })]);
    await tick();
    expect(onimage).toHaveBeenCalledTimes(1);
    expect(blocks(editor).map((b) => b.type)).toEqual(['image', 'paragraph']);
    expect(blocks(editor)[0]?.attrs?.src).toBe('/img/drop.webp');
  });

  it('shows a placeholder while uploading and removes it on failure', async () => {
    const pending = deferred();
    const onerror = vi.fn();
    const { editor, target, view } = open({ onimage: () => pending.promise, onerror });
    fire(view.dom, 'paste', [png()]);
    expect(target.querySelector('.encre-uploading')).not.toBeNull();
    pending.reject(new Error('disk full'));
    await tick();
    expect(target.querySelector('.encre-uploading')).toBeNull();
    expect(onerror).toHaveBeenCalledWith(new Error('disk full'));
    expect(blocks(editor).map((b) => b.type)).toEqual(['paragraph']);
  });

  it('refuses svg and oversized images before calling the hook', async () => {
    const onimage = vi.fn(async () => ({ src: '/x' }));
    const onerror = vi.fn();
    const { editor, view } = open({ onimage, onerror, maxImageBytes: 10 });
    fire(view.dom, 'paste', [svg()]);
    fire(view.dom, 'paste', [png('big.png', 11)]);
    await tick();
    expect(onimage).not.toHaveBeenCalled();
    expect(onerror).toHaveBeenCalledTimes(2);
    expect(blocks(editor).map((b) => b.type)).toEqual(['paragraph']);
    expect(imageRefusal(svg())).not.toBeNull();
    expect(imageRefusal(png('ok.png', 10), 10)).toBeNull();
  });

  it('opens a file picker from the image command and uploads the choice', async () => {
    const onimage = vi.fn(async () => ({ src: '/picked.gif' }));
    const { editor } = open({ onimage });
    let input: HTMLInputElement | undefined;
    const click = vi.spyOn(HTMLInputElement.prototype, 'click').mockImplementation(function (this: HTMLInputElement) {
      input = this;
    });
    imageCommand(() => ({ onimage })).run(editor.instance()!);
    click.mockRestore();
    expect(input?.accept).toBe('image/png,image/jpeg,image/gif,image/webp');
    Object.defineProperty(input, 'files', { value: [new File(['g'], 'a.gif', { type: 'image/gif' })] });
    input!.dispatchEvent(new Event('change'));
    await tick();
    expect(blocks(editor).some((b) => b.type === 'image' && b.attrs?.src === '/picked.gif')).toBe(true);
  });

  it('leaves pasted files to the editor without a hook', async () => {
    const { editor, view } = open({});
    fire(view.dom, 'paste', [png()]);
    await tick();
    expect(blocks(editor).map((b) => b.type)).toEqual(['paragraph']);
  });
});
