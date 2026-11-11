import type { Node as PMNode } from '@tiptap/pm/model';
import { Plugin, type EditorState, type Transaction } from '@tiptap/pm/state';
import { build, cellsOf, type Align } from './table-model.js';

const ALIGNS: readonly string[] = ['none', 'left', 'center', 'right'];

/** The alignment GFM can write for a pasted `text-align` or `align`; anything else is `none`. */
export function cellAlign(element: HTMLElement): Align {
  const align = (element.style.textAlign || element.getAttribute('align') || 'none').toLowerCase();
  return ALIGNS.includes(align) ? (align as Align) : 'none';
}

function isNormal(table: PMNode): boolean {
  const header = cellsOf(table)[0] ?? [];
  return cellsOf(table).every(
    (cells, r) =>
      cells.length === header.length &&
      cells.every((cell, c) => cell.type.name === (r === 0 ? 'tableHeader' : 'tableCell') && cell.attrs.align === header[c]!.attrs.align)
  );
}

/** `table` made rectangular, its first row the header and every column aligned like its header cell. */
export function normalTable(table: PMNode): PMNode {
  const rows: (PMNode | null)[][] = cellsOf(table);
  const width = Math.max(...rows.map((cells) => cells.length));
  const padded = rows.map((cells) => [...cells, ...Array.from({ length: width - cells.length }, () => null)]);
  const aligns = padded[0]!.map((cell) => (cell?.attrs.align as Align | undefined) ?? 'none');
  return build(table.type.schema, padded, aligns, table.attrs);
}

function normalizeTables(transactions: readonly Transaction[], state: EditorState): Transaction | null {
  if (!transactions.some((tr) => tr.docChanged)) return null;
  const found: [number, PMNode][] = [];
  state.doc.descendants((node, pos) => {
    if (node.type.name === 'table' && !isNormal(node)) found.push([pos, node]);
    return node.type.name !== 'table' && !node.isTextblock;
  });
  const tr = state.tr;
  for (const [pos, table] of found.reverse()) tr.replaceWith(pos, pos + table.nodeSize, normalTable(table));
  return found.length > 0 ? tr : null;
}

/** Rewrites every table a paste or `setContent` left out of shape, so the document always validates. */
export const tableNormalizer = () => new Plugin({ appendTransaction: (transactions, _old, state) => normalizeTables(transactions, state) });
