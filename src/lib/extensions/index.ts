import type { Extensions } from '@tiptap/core';
import { Placeholder } from '@tiptap/extensions';
import { Callout, Chart, Stat } from './custom.js';
import { encreStarterKit } from './starter.js';
import { Table, TableCell, TableHeader, TableRow } from './table.js';

/** Options of the encre extension set. */
export interface EncreExtensionOptions {
  placeholder?: string;
}

/** Every Tiptap extension of the encre schema, and nothing outside it. */
export function encreExtensions(options: EncreExtensionOptions = {}): Extensions {
  const extensions: Extensions = [...encreStarterKit(), Table, TableRow, TableHeader, TableCell, Callout, Stat, Chart];
  if (options.placeholder) extensions.push(Placeholder.configure({ placeholder: options.placeholder }));
  return extensions;
}

export { Callout, Chart, Stat, type CalloutTone } from './custom.js';
export { Table, TableCell, TableHeader, TableRow } from './table.js';
export type { Align } from './table-model.js';
