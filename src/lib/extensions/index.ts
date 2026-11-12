import type { Extensions } from '@tiptap/core';
import { Placeholder } from '@tiptap/extensions';
import { collaborationExtensions, type CollabOptions } from './collaboration.js';
import { Badge } from './badge.js';
import { Column, Columns } from './columns.js';
import { Callout, Chart, Stat } from './custom.js';
import { Diagram } from './diagram.js';
import { Settings, type RenderDiagram } from './settings.svelte.js';
import { labelsFor, type Labels } from '../labels.js';
import { encreStarterKit } from './starter.js';
import { Table, TableCell, TableHeader, TableRow } from './table.js';

/** Options of the encre extension set. */
export interface EncreExtensionOptions {
  placeholder?: string;
  collaboration?: CollabOptions;
  /** The interface copy node views show; French when unset. */
  labels?: Labels;
  /** Draws diagrams in the editor; without it a diagram shows its source only. */
  renderDiagram?: RenderDiagram;
}

/** Every Tiptap extension of the encre schema, and nothing outside it. */
export function encreExtensions(options: EncreExtensionOptions = {}): Extensions {
  const collab = options.collaboration;
  const settings = Settings.configure({ labels: options.labels ?? labelsFor('fr'), renderDiagram: options.renderDiagram });
  const blocks = [Table, TableRow, TableHeader, TableCell, Callout, Stat, Chart, Diagram, Columns, Column, Badge];
  const extensions: Extensions = [...encreStarterKit({ undoRedo: !collab }), ...blocks, settings];
  if (collab) extensions.push(...collaborationExtensions(collab));
  if (options.placeholder) extensions.push(Placeholder.configure({ placeholder: options.placeholder }));
  return extensions;
}

export { caretColor, FRAGMENT, type AwarenessProvider, type CollabOptions, type CollabUser } from './collaboration.js';
export { Callout, Chart, Stat, type BadgeTone, type CalloutTone, type ColumnTone, type DiagramLanguage } from './custom.js';
export { Badge } from './badge.js';
export { Column, Columns } from './columns.js';
export { Diagram } from './diagram.js';
export { Settings, settingsOf, type EncreSettings, type RenderDiagram } from './settings.svelte.js';
export { Table, TableCell, TableHeader, TableRow } from './table.js';
export type { Align } from './table-model.js';
