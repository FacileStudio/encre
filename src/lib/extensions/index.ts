import type { Extensions } from '@tiptap/core';
import { Placeholder } from '@tiptap/extensions';
import { collaborationExtensions, type CollabOptions } from './collaboration.js';
import { Callout, Chart, Stat } from './custom.js';
import { encreStarterKit } from './starter.js';
import { Table, TableCell, TableHeader, TableRow } from './table.js';

/** Options of the encre extension set. */
export interface EncreExtensionOptions {
  placeholder?: string;
  collaboration?: CollabOptions;
}

/** Every Tiptap extension of the encre schema, and nothing outside it. */
export function encreExtensions(options: EncreExtensionOptions = {}): Extensions {
  const collab = options.collaboration;
  const extensions: Extensions = [...encreStarterKit({ undoRedo: !collab }), Table, TableRow, TableHeader, TableCell, Callout, Stat, Chart];
  if (collab) extensions.push(...collaborationExtensions(collab));
  if (options.placeholder) extensions.push(Placeholder.configure({ placeholder: options.placeholder }));
  return extensions;
}

export { caretColor, FRAGMENT, type AwarenessProvider, type CollabOptions, type CollabUser } from './collaboration.js';
export { Callout, Chart, Stat, type CalloutTone } from './custom.js';
export { Table, TableCell, TableHeader, TableRow } from './table.js';
export type { Align } from './table-model.js';
