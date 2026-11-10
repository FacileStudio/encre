import type { Editor } from '@tiptap/core';
import type { EncreNode } from './schema.js';
import { encreReady, parseMarkdown } from './wasm.js';

const MARKDOWN = /^(#{1,6} |[-*+] |\d+[.)] |> |```|~~~|\|)|\*\*[^*]+\*\*|\[[^\]]+\]\([^)]+\)|`[^`]+`/m;

/** Whether pasted plain text reads as markdown rather than prose. */
export function looksLikeMarkdown(text: string): boolean {
  return MARKDOWN.test(text);
}

/** The content to insert for `doc`: inline content alone when it is a single paragraph. */
export function insertable(doc: EncreNode): EncreNode[] {
  const blocks = doc.content ?? [];
  const only = blocks.length === 1 ? blocks[0] : undefined;
  return only?.type === 'paragraph' ? (only.content ?? []) : blocks;
}

/** Handles a paste of plain-text markdown through encre-wasm; returns false to let Tiptap paste as usual. */
export function pasteMarkdown(editor: Editor, event: ClipboardEvent): boolean {
  const data = event.clipboardData;
  const text = data?.getData('text/plain') ?? '';
  if (!encreReady() || !data || data.getData('text/html') !== '' || !looksLikeMarkdown(text)) return false;
  editor.chain().focus().insertContent(insertable(parseMarkdown(text))).run();
  return true;
}
