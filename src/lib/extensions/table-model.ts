import type { Node as PMNode, Schema } from '@tiptap/pm/model';

/** Column alignment of a table cell, as GFM writes it in the delimiter row. */
export type Align = 'none' | 'left' | 'center' | 'right';

function cellsOf(table: PMNode): PMNode[][] {
  const rows: PMNode[][] = [];
  table.forEach((row) => rows.push(Array.from({ length: row.childCount }, (_, i) => row.child(i))));
  return rows;
}

function build(schema: Schema, rows: (PMNode | null)[][], aligns: Align[], attrs: PMNode['attrs'] | null = null): PMNode {
  const { table, tableRow, tableHeader, tableCell, paragraph } = schema.nodes;
  const builtRows = rows.map((cells, r) =>
    tableRow!.create(
      null,
      cells.map((cell, c) => {
        const type = r === 0 ? tableHeader! : tableCell!;
        const content = cell?.firstChild ?? paragraph!.create();
        return type.create({ align: aligns[c] ?? 'none' }, content);
      })
    )
  );
  return table!.create(attrs, builtRows);
}

/** A new table of `rows` by `cols` empty cells, the first row being the header. */
export function emptyTable(schema: Schema, rows: number, cols: number): PMNode {
  const grid = Array.from({ length: rows }, () => Array.from({ length: cols }, () => null));
  return build(schema, grid, Array.from({ length: cols }, () => 'none'));
}

/** `table` with an empty row inserted at `index`. */
export function insertRow(schema: Schema, table: PMNode, index: number): PMNode {
  const rows: (PMNode | null)[][] = cellsOf(table);
  const width = rows[0]?.length ?? 1;
  rows.splice(index, 0, Array.from({ length: width }, () => null));
  return build(schema, rows, (cellsOf(table)[0] ?? []).map((cell) => cell.attrs.align as Align), table.attrs);
}

/** `table` with an empty column inserted at `index`. */
export function insertColumn(schema: Schema, table: PMNode, index: number): PMNode {
  const rows: (PMNode | null)[][] = cellsOf(table).map((cells) => [...cells]);
  rows.forEach((cells) => cells.splice(index, 0, null));
  const aligns = (cellsOf(table)[0] ?? []).map((cell) => cell.attrs.align as Align);
  aligns.splice(index, 0, 'none');
  return build(schema, rows, aligns, table.attrs);
}

/** `table` without row `index`, or null when it was the last row. */
export function removeRow(schema: Schema, table: PMNode, index: number): PMNode | null {
  const rows = cellsOf(table);
  if (rows.length <= 1) return null;
  rows.splice(index, 1);
  return build(schema, rows, (cellsOf(table)[0] ?? []).map((cell) => cell.attrs.align as Align), table.attrs);
}

/** `table` without column `index`, or null when it was the last column. */
export function removeColumn(schema: Schema, table: PMNode, index: number): PMNode | null {
  const rows = cellsOf(table);
  if ((rows[0]?.length ?? 0) <= 1) return null;
  rows.forEach((cells) => cells.splice(index, 1));
  const aligns = (cellsOf(table)[0] ?? []).map((cell) => cell.attrs.align as Align);
  aligns.splice(index, 1);
  return build(schema, rows, aligns, table.attrs);
}

/** `table` with every cell of column `index` aligned to `align`. */
export function alignColumn(schema: Schema, table: PMNode, index: number, align: Align): PMNode {
  const aligns = (cellsOf(table)[0] ?? []).map((cell) => cell.attrs.align as Align);
  aligns[index] = align;
  return build(schema, cellsOf(table), aligns, table.attrs);
}
