import { getContext, setContext } from 'svelte';
import type { RenderDiagram } from '../extensions/settings.svelte.js';
import type { ResolveEmbed } from '../embed.js';
import { labelsFor, type Labels } from '../labels.js';

/** Turns code into HTML for a code block. The app must return safe, sanitized HTML. */
export type Highlight = (code: string, language: string) => string;

/** What every block of a `Document` reads: the copy and the app's hooks. */
export interface RenderOptions {
  labels: Labels;
  renderDiagram?: RenderDiagram;
  highlight?: Highlight;
  resolveEmbed?: ResolveEmbed;
}

const KEY = Symbol('encre-render');

/** Makes `options` available to the blocks below. */
export function provideRender(options: () => RenderOptions): void {
  setContext(KEY, options);
}

/** Reads the options of the nearest `Document` (French copy and no hooks outside one); call it during init. */
export function renderOptions(): () => RenderOptions {
  const read = getContext<(() => RenderOptions) | undefined>(KEY);
  return read ?? (() => ({ labels: labelsFor('fr') }));
}

/** The tone of each line of a unified diff: added, removed, hunk header or context. */
export function diffLines(code: string): { tone: 'add' | 'del' | 'hunk' | 'ctx'; line: string }[] {
  return code.split('\n').map((line) => {
    if (line.startsWith('+') && !line.startsWith('+++')) return { tone: 'add', line };
    if (line.startsWith('-') && !line.startsWith('---')) return { tone: 'del', line };
    if (line.startsWith('@@')) return { tone: 'hunk', line };
    return { tone: 'ctx', line };
  });
}
