import { Node, mergeAttributes, type CommandProps } from '@tiptap/core';
import type { Node as PMNode } from '@tiptap/pm/model';
import ColumnView from '../nodes/Column.svelte';
import { dataAttr } from './custom.js';
import { svelteNodeView } from './node-view.svelte.js';

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    encreColumns: {
      insertColumns: (count?: number) => ReturnType;
      addSide: () => ReturnType;
      deleteSide: () => ReturnType;
    };
  }
}

const emptyColumn = () => ({ type: 'column', content: [{ type: 'paragraph' }] });

function around({ state }: Pick<CommandProps, 'state'>): { node: PMNode; pos: number; index: number } | null {
  const $from = state.selection.$from;
  for (let depth = $from.depth; depth >= 1; depth -= 1) {
    if ($from.node(depth).type.name === 'columns') return { node: $from.node(depth), pos: $from.before(depth), index: $from.index(depth) };
  }
  return null;
}

function resize(props: CommandProps, change: (count: number, index: number) => number | null): boolean {
  const found = around(props);
  const at = found ? change(found.node.childCount, found.index) : null;
  if (!found || at === null) return false;
  if (!props.dispatch) return true;
  const offset = (i: number) => found.pos + 1 + Array.from({ length: i }, (_, k) => found.node.child(k).nodeSize).reduce((a, b) => a + b, 0);
  if (at >= 0) props.tr.insert(offset(at), props.state.schema.nodeFromJSON(emptyColumn()));
  else props.tr.delete(offset(found.index), offset(found.index + 1));
  return true;
}

/** Two to four columns side by side; they stack on narrow screens. */
export const Columns = Node.create({
  name: 'columns',
  group: 'block',
  content: 'column{2,4}',
  isolating: true,
  parseHTML: () => [{ tag: 'div[data-type="columns"]' }],
  renderHTML: ({ HTMLAttributes }) => ['div', mergeAttributes(HTMLAttributes, { 'data-type': 'columns', class: 'encre-columns' }), 0],
  addCommands() {
    return {
      insertColumns:
        (count = 2) =>
        ({ commands }) =>
          commands.insertContent({ type: this.name, content: Array.from({ length: Math.min(4, Math.max(2, count)) }, emptyColumn) }),
      addSide: () => (props) => resize(props, (count, index) => (count < 4 ? index + 1 : null)),
      deleteSide: () => (props) => resize(props, (count) => (count > 2 ? -1 : null))
    };
  }
});

/** One column: an optional title and tone over block content. */
export const Column = Node.create({
  name: 'column',
  content: 'block+',
  isolating: true,
  addAttributes: () => ({ title: dataAttr('title', ''), tone: dataAttr('tone', 'neutral') }),
  parseHTML: () => [{ tag: 'div[data-type="column"]' }],
  renderHTML: ({ HTMLAttributes }) => ['div', mergeAttributes(HTMLAttributes, { 'data-type': 'column' }), 0],
  addNodeView: () => svelteNodeView(ColumnView, true)
});
