import type { Editor } from '@tiptap/core';
import { labels as french, type Labels } from './labels.js';

/** A block the slash menu and the toolbar can turn the selection into or insert. */
export interface BlockCommand {
  id: string;
  label: string;
  keywords: string;
  run: (editor: Editor) => void;
}

const chain = (editor: Editor) => editor.chain().focus();

/** Every block an author can create, in menu order, named in `labels`. */
export function blockCommands(labels: Labels): BlockCommand[] {
  return [
    { id: 'paragraph', label: labels.paragraph, keywords: 'texte paragraph text', run: (e) => chain(e).setParagraph().run() },
    ...[1, 2, 3].map((level) => ({
      id: `heading${level}`,
      label: labels.heading(level),
      keywords: `titre heading h${level}`,
      run: (e: Editor) => chain(e).setHeading({ level: level as 1 | 2 | 3 }).run()
    })),
    { id: 'bulletList', label: labels.bulletList, keywords: 'liste puces bullet list', run: (e) => chain(e).toggleBulletList().run() },
    { id: 'orderedList', label: labels.orderedList, keywords: 'liste numérotée ordered list', run: (e) => chain(e).toggleOrderedList().run() },
    { id: 'taskList', label: labels.taskList, keywords: 'tâches task todo', run: (e) => chain(e).toggleTaskList().run() },
    { id: 'blockquote', label: labels.blockquote, keywords: 'citation quote', run: (e) => chain(e).toggleBlockquote().run() },
    { id: 'codeBlock', label: labels.codeBlock, keywords: 'code', run: (e) => chain(e).toggleCodeBlock().run() },
    { id: 'horizontalRule', label: labels.horizontalRule, keywords: 'séparateur divider hr', run: (e) => chain(e).setHorizontalRule().run() },
    { id: 'table', label: labels.table, keywords: 'tableau table', run: (e) => chain(e).insertTable().run() },
    { id: 'callout', label: labels.callout, keywords: 'encadré callout alerte alert', run: (e) => chain(e).setCallout('info').run() },
    { id: 'stat', label: labels.stat, keywords: 'indicateur stat chiffre metric', run: (e) => chain(e).insertStat().run() },
    { id: 'chart', label: labels.chart, keywords: 'graphique chart', run: (e) => chain(e).insertChart('bar').run() },
    { id: 'diagram', label: labels.diagram, keywords: 'diagramme diagram mermaid d2 dot', run: (e) => chain(e).insertDiagram('mermaid').run() },
    { id: 'columns', label: labels.columns, keywords: 'colonnes columns compare pros cons', run: (e) => chain(e).insertColumns(2).run() },
    { id: 'badge', label: labels.badge, keywords: 'pastille badge status statut', run: (e) => chain(e).insertBadge(labels.badgeText).run() }
  ];
}

/** Every block command, with the French copy. */
export const BLOCK_COMMANDS: BlockCommand[] = blockCommands(french);

/** The commands of `commands` (every block by default) matching `query`, by label or keyword. */
export function findBlocks(query: string, commands: BlockCommand[] = BLOCK_COMMANDS): BlockCommand[] {
  const needle = query.trim().toLowerCase();
  if (needle === '') return commands;
  return commands.filter((b) => b.label.toLowerCase().includes(needle) || b.keywords.includes(needle));
}
