import { Node, mergeAttributes, type CommandProps } from '@tiptap/core';
import type { Node as PMNode } from '@tiptap/pm/model';
import { TextSelection } from '@tiptap/pm/state';
import { alignColumn, emptyTable, insertColumn, insertRow, removeColumn, removeRow, type Align } from './table-model.js';
import { cellAlign, tableNormalizer } from './table-normal.js';

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    encreTable: {
      insertTable: (options?: { rows?: number; cols?: number }) => ReturnType;
      addRowAfter: () => ReturnType;
      addColumnAfter: () => ReturnType;
      deleteRow: () => ReturnType;
      deleteColumn: () => ReturnType;
      deleteTable: () => ReturnType;
      setColumnAlign: (align: Align) => ReturnType;
      goToCell: (direction: 1 | -1) => ReturnType;
    };
  }
}

const CELL_TYPES = ['tableCell', 'tableHeader'];

interface CellContext {
  table: PMNode;
  tablePos: number;
  row: number;
  col: number;
}

function cellContext({ state }: Pick<CommandProps, 'state'>): CellContext | null {
  const $from = state.selection.$from;
  for (let depth = $from.depth; depth >= 2; depth -= 1) {
    if ($from.node(depth - 2).type.name === 'table' && CELL_TYPES.includes($from.node(depth).type.name)) {
      return { table: $from.node(depth - 2), tablePos: $from.before(depth - 2), row: $from.index(depth - 2), col: $from.index(depth - 1) };
    }
  }
  return null;
}

function cellStart(table: PMNode, tablePos: number, row: number, col: number): number {
  let pos = tablePos + 1;
  for (let r = 0; r < row; r += 1) pos += table.child(r).nodeSize;
  const cells = table.child(row);
  for (let c = 0; c < col; c += 1) pos += cells.child(c).nodeSize;
  return pos + 3;
}

type Rebuild = (ctx: CellContext, props: CommandProps) => { table: PMNode | null; row: number; col: number };

function rebuild(change: Rebuild) {
  return () => (props: CommandProps) => {
    const ctx = cellContext(props);
    if (!ctx) return false;
    const next = change(ctx, props);
    if (!props.dispatch) return true;
    const end = ctx.tablePos + ctx.table.nodeSize;
    if (!next.table) {
      props.tr.delete(ctx.tablePos, end);
      return true;
    }
    if (next.table !== ctx.table) props.tr.replaceWith(ctx.tablePos, end, next.table);
    const pos = cellStart(next.table, ctx.tablePos, next.row, next.col);
    props.tr.setSelection(TextSelection.create(props.tr.doc, pos));
    return true;
  };
}

const alignAttribute = {
  default: 'none',
  parseHTML: cellAlign,
  renderHTML: (attrs: { align?: string }) => (attrs.align && attrs.align !== 'none' ? { style: `text-align: ${attrs.align}` } : {})
};

function cellNode(name: string, tag: 'th' | 'td') {
  return Node.create({
    name,
    content: 'paragraph',
    isolating: true,
    addAttributes: () => ({ align: alignAttribute }),
    parseHTML: () => [{ tag }],
    renderHTML: ({ HTMLAttributes }) => [tag, mergeAttributes(HTMLAttributes), 0]
  });
}

/** A header cell: every cell of the first row. */
export const TableHeader = cellNode('tableHeader', 'th');

/** A body cell. */
export const TableCell = cellNode('tableCell', 'td');

/** A table row. */
export const TableRow = Node.create({
  name: 'tableRow',
  content: '(tableHeader | tableCell)+',
  parseHTML: () => [{ tag: 'tr' }],
  renderHTML: ({ HTMLAttributes }) => ['tr', HTMLAttributes, 0]
});

function moveCell(direction: 1 | -1): Rebuild {
  return ({ table, row, col }, { state }) => {
    const width = table.child(0).childCount;
    const index = row * width + col + direction;
    if (index < 0) return { table, row, col };
    if (index >= table.childCount * width) return { table: insertRow(state.schema, table, table.childCount), row: row + 1, col: 0 };
    return { table, row: Math.floor(index / width), col: index % width };
  };
}

function placeTable({ tr, state, dispatch }: CommandProps, rows: number, cols: number): boolean {
  const table = emptyTable(state.schema, rows, cols);
  if (!dispatch) return true;
  tr.replaceSelectionWith(table);
  tr.doc.descendants((node, pos) => {
    if (node === table) tr.setSelection(TextSelection.create(tr.doc, cellStart(table, pos, 0, 0)));
    return node.isBlock && node !== table;
  });
  return true;
}

/** A GFM table: a header row, then body rows, every column one alignment. */
export const Table = Node.create({
  name: 'table',
  group: 'block',
  content: 'tableRow+',
  isolating: true,
  parseHTML: () => [{ tag: 'table' }],
  renderHTML: ({ HTMLAttributes }) => ['table', HTMLAttributes, ['tbody', 0]],
  addProseMirrorPlugins: () => [tableNormalizer()],
  addCommands() {
    return {
      insertTable: (options = {}) => (props) => placeTable(props, options.rows ?? 3, options.cols ?? 3),
      addRowAfter: rebuild(({ table, row, col }, { state }) => ({ table: insertRow(state.schema, table, row + 1), row: row + 1, col })),
      addColumnAfter: rebuild(({ table, row, col }, { state }) => ({ table: insertColumn(state.schema, table, col + 1), row, col: col + 1 })),
      deleteRow: rebuild(({ table, row, col }, { state }) => ({ table: removeRow(state.schema, table, row), row: Math.max(0, row - 1), col })),
      deleteColumn: rebuild(({ table, row, col }, { state }) => ({ table: removeColumn(state.schema, table, col), row, col: Math.max(0, col - 1) })),
      deleteTable: rebuild(({ row, col }) => ({ table: null, row, col })),
      setColumnAlign: (align: Align) => rebuild(({ table, row, col }, { state }) => ({ table: alignColumn(state.schema, table, col, align), row, col }))(),
      goToCell: (direction: 1 | -1) => rebuild(moveCell(direction))()
    };
  },
  addKeyboardShortcuts() {
    return {
      Tab: () => this.editor.commands.goToCell(1),
      'Shift-Tab': () => this.editor.commands.goToCell(-1)
    };
  }
});
