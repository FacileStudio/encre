import type { Editor } from '@tiptap/core';
import type { Node as PMNode } from '@tiptap/pm/model';
import { compactDocument } from './compact.js';
import type { EncreNode } from './schema.js';
import { encreReady, serializeMarkdown } from './wasm.js';

/** A top-level block and where it starts. */
export interface TopBlock {
  node: PMNode;
  pos: number;
}

/** The top-level blocks the selection touches, in document order. */
export function touchedBlocks(editor: Editor): TopBlock[] {
  const { doc, selection } = editor.state;
  const blocks: TopBlock[] = [];
  const to = Math.max(selection.to, selection.from + 1);
  doc.nodesBetween(selection.from, Math.min(to, doc.content.size), (node, pos) => {
    blocks.push({ node, pos });
    return false;
  });
  if (blocks.length === 0 && doc.childCount > 0) {
    const index = Math.min(selection.$from.index(0), doc.childCount - 1);
    blocks.push({ node: doc.child(index), pos: selection.$from.posAtIndex(index, 0) });
  }
  return blocks;
}

function inlineRange(editor: Editor): EncreNode | null {
  const { selection } = editor.state;
  if (selection.empty || !selection.$from.sameParent(selection.$to) || !selection.$from.parent.isTextblock) return null;
  const { $from, $to } = selection;
  const inline = ($from.parent.content.cut($from.parentOffset, $to.parentOffset).toJSON() ?? []) as EncreNode[];
  return { type: 'doc', content: [{ type: 'paragraph', content: inline }] };
}

/** The selection as Facile Markdown: the text range inside one block, or every block it touches; plain text before encre loads. */
export function selectionMarkdown(editor: Editor): string {
  const { doc, selection } = editor.state;
  const range = inlineRange(editor);
  const blocks = touchedBlocks(editor).map(({ node }) => node.toJSON() as EncreNode);
  const tree = range ?? { type: 'doc', content: blocks };
  if (!encreReady() || !tree.content?.length) return doc.textBetween(selection.from, selection.to, '\n\n');
  try {
    return serializeMarkdown(compactDocument(tree));
  } catch {
    return doc.textBetween(selection.from, selection.to, '\n\n');
  }
}

/** Copies the top-level block under the selection right after it; false when there is none. */
export function duplicateBlock(editor: Editor): boolean {
  const block = touchedBlocks(editor).at(-1);
  if (!block) return false;
  const copy = block.node.type.create({ ...block.node.attrs, id: null }, block.node.content, block.node.marks);
  return editor.chain().focus().insertContentAt(block.pos + block.node.nodeSize, copy.toJSON()).run();
}

/** Deletes the selected range, or the top-level block under an empty selection. */
export function deleteSelection(editor: Editor): boolean {
  if (!editor.state.selection.empty) return editor.chain().focus().deleteSelection().run();
  const block = touchedBlocks(editor)[0];
  if (!block) return false;
  return editor.chain().focus().deleteRange({ from: block.pos, to: block.pos + block.node.nodeSize }).run();
}

/** Copies `<base>#<id>` of the top-level block under the selection to the clipboard and answers the id. */
export function copyBlockLink(editor: Editor, base: string): string | null {
  const id = touchedBlocks(editor)[0]?.node.attrs.id;
  if (typeof id !== 'string') return null;
  void globalThis.navigator?.clipboard?.writeText(`${base}#${id}`).catch(() => undefined);
  return id;
}

/** What a right click reports: the caret moves to the click when it lands outside the selection. */
export function contextInfo(editor: Editor, event: MouseEvent): { event: MouseEvent; empty: boolean; markdown: string } {
  const { from, to } = editor.state.selection;
  let at: number | undefined;
  try {
    at = editor.view.posAtCoords({ left: event.clientX, top: event.clientY })?.pos;
  } catch {
    at = undefined;
  }
  if (at !== undefined && (at < from || at > to)) editor.commands.setTextSelection(at);
  return { event, empty: editor.state.selection.empty, markdown: selectionMarkdown(editor) };
}
