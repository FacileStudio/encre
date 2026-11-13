import { Node, mergeAttributes } from '@tiptap/core';
import EmbedView from '../nodes/Embed.svelte';
import FileView from '../nodes/File.svelte';
import { embedKind, httpsUrl, type EmbedKind } from '../embed.js';
import { dataAttr } from './custom.js';
import { svelteNodeView } from './node-view.svelte.js';

/** What a file block holds, as the app's `onfile` hook answers it. */
export interface FileAttrs {
  src: string;
  name: string;
  size: number;
  mime: string;
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    encreEmbed: {
      insertEmbed: (url: string, kind?: EmbedKind) => ReturnType;
    };
    encreFile: {
      insertFile: (file: FileAttrs) => ReturnType;
    };
  }
}

/** An embed: a Facile app shown live in a frame, or a link card; https addresses only. */
export const Embed = Node.create({
  name: 'embed',
  group: 'block',
  atom: true,
  draggable: true,
  addAttributes: () => ({ url: dataAttr('url', null), kind: dataAttr('kind', 'link'), title: dataAttr('title', '') }),
  parseHTML: () => [{ tag: 'div[data-type="embed"]' }],
  renderHTML: ({ HTMLAttributes }) => ['div', mergeAttributes(HTMLAttributes, { 'data-type': 'embed' })],
  addNodeView: () => svelteNodeView(EmbedView),
  addCommands() {
    return {
      insertEmbed:
        (url, kind) =>
        ({ commands }) =>
          httpsUrl(url) !== null && commands.insertContent({ type: this.name, attrs: { url, kind: kind ?? embedKind(url) } })
    };
  }
});

/** A file: an uploaded attachment shown as a row that downloads it. */
export const FileBlock = Node.create({
  name: 'file',
  group: 'block',
  atom: true,
  draggable: true,
  addAttributes: () => ({
    src: dataAttr('src', null),
    name: dataAttr('name', ''),
    size: { ...dataAttr('size', 0), parseHTML: (element: HTMLElement) => Math.max(0, Math.trunc(Number(element.getAttribute('data-size')) || 0)) },
    mime: dataAttr('mime', '')
  }),
  parseHTML: () => [{ tag: 'div[data-type="file"]' }],
  renderHTML: ({ HTMLAttributes }) => ['div', mergeAttributes(HTMLAttributes, { 'data-type': 'file' })],
  addNodeView: () => svelteNodeView(FileView),
  addCommands() {
    return {
      insertFile:
        (file) =>
        ({ commands }) =>
          commands.insertContent({ type: this.name, attrs: { ...file } })
    };
  }
});
