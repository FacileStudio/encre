import type { Editor } from '@tiptap/core';
import type { Node as PmNode } from '@tiptap/pm/model';
import { NodeSelection, TextSelection } from '@tiptap/pm/state';

/** The document position of the node at a JSON path such as `doc.content[1].content[0]`, or null when it is not there. */
export function positionOf(doc: PmNode, path: string): number | null {
  if (!/^doc(?:\.content\[\d+\])*$/.test(path)) return null;
  let node = doc;
  let pos = -1;
  for (const match of path.matchAll(/\[(\d+)\]/g)) {
    const index = Number(match[1]);
    if (index >= node.childCount) return null;
    pos += 1;
    for (let i = 0; i < index; i += 1) pos += node.child(i).nodeSize;
    node = node.child(index);
  }
  return pos;
}

/** The position of the block whose id is `blockId`, or null. */
export function blockPosition(doc: PmNode, blockId: string): number | null {
  let found: number | null = null;
  doc.descendants((node, pos) => {
    if (found !== null) return false;
    if (node.attrs.id === blockId) found = pos;
    return found === null;
  });
  return found;
}

/** ProseMirror's scroll threshold and margin that keep a scrolled-to node `inset` pixels below the top, clear of a sticky toolbar. */
export function clearance(inset: number) {
  return {
    scrollThreshold: { top: inset, right: 0, bottom: 0, left: 0 },
    scrollMargin: { top: inset + 5, right: 5, bottom: 5, left: 5 }
  };
}

/** Selects the node at `pos` (a textblock's text, the start of the document when `pos` is -1) and scrolls it into view `inset` pixels below the top. */
export function reveal(editor: Editor, pos: number | null, inset = 0): boolean {
  if (pos === null) return false;
  const { doc, tr } = editor.state;
  const node = pos < 0 ? doc : doc.nodeAt(pos);
  if (!node) return false;
  const from = Math.max(pos, 0);
  const selection = node.isText
    ? TextSelection.create(doc, from, from + node.nodeSize)
    : node.isTextblock
      ? TextSelection.create(doc, from + 1, from + node.nodeSize - 1)
      : pos >= 0 && NodeSelection.isSelectable(node)
        ? NodeSelection.create(doc, pos)
        : TextSelection.near(doc.resolve(from + (pos < 0 ? 0 : 1)));
  editor.view.setProps(clearance(inset));
  editor.view.dispatch(tr.setSelection(selection).scrollIntoView());
  editor.view.focus();
  return true;
}
