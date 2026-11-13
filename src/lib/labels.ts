import type { BadgeTone, CalloutTone, ColumnTone, DiagramLanguage } from './extensions/custom.js';
import { de } from './locales/de.js';
import { en } from './locales/en.js';
import { es } from './locales/es.js';
import { fr } from './locales/fr.js';

/** Callout tones in schema order. */
export const TONES: CalloutTone[] = ['neutral', 'info', 'success', 'warning', 'danger'];

/** Badge tones in schema order. */
export const BADGE_TONES: BadgeTone[] = ['neutral', 'accent', 'info', 'success', 'warning', 'danger'];

/** Column tones in schema order. */
export const COLUMN_TONES: ColumnTone[] = ['neutral', 'success', 'danger', 'warning', 'info'];

/** Diagram languages in schema order. */
export const DIAGRAM_LANGUAGES: DiagramLanguage[] = ['mermaid', 'd2', 'dot'];

/** The locales encre ships interface copy for. */
export type Locale = 'fr' | 'en' | 'es' | 'de';

/** Every user-facing string of the editor, its menus and its node views. */
export interface Labels {
  calloutTitle: string;
  tone: string;
  tones: Record<BadgeTone, string>;
  statLabel: string;
  statValue: string;
  statDelta: string;
  statSub: string;
  chartTitle: string;
  chartKind: string;
  chartBar: string;
  chartLine: string;
  chartLabels: string;
  chartSeries: string;
  emptyChart: string;
  toolbar: string;
  bold: string;
  italic: string;
  strike: string;
  highlight: string;
  code: string;
  link: string;
  linkPrompt: string;
  paragraph: string;
  heading: (level: number) => string;
  bulletList: string;
  orderedList: string;
  taskList: string;
  blockquote: string;
  codeBlock: string;
  horizontalRule: string;
  table: string;
  callout: string;
  stat: string;
  chart: string;
  diagram: string;
  diagramLanguage: string;
  diagramSource: string;
  columns: string;
  columnTitle: string;
  addSide: string;
  deleteSide: string;
  badge: string;
  badgeText: string;
  image: string;
  imagePrompt: string;
  imageUploading: string;
  imageType: string;
  imageSize: (max: number) => string;
  imageNoSource: string;
  undo: string;
  redo: string;
  insert: string;
  slashEmpty: string;
  addRow: string;
  addColumn: string;
  deleteRow: string;
  deleteColumn: string;
  deleteTable: string;
  turnInto: string;
  linkRemove: string;
  embed: string;
  embedPrompt: string;
  embedInvalid: string;
  embedOpen: string;
  embedResize: string;
  file: string;
  fileDownload: string;
  fileNoSource: string;
}

/** The shipped catalogues, one per locale. */
export const CATALOGUES: Record<Locale, Labels> = { fr, en, es, de };

/** Strings an app replaces in a catalogue; `tones` may name only some tones. */
export type LabelOverrides = Partial<Omit<Labels, 'tones'>> & { tones?: Partial<Labels['tones']> };

/** The catalogue of `locale` (French when unknown), with `overrides` applied on top. */
export function labelsFor(locale: Locale = 'fr', overrides: LabelOverrides = {}): Labels {
  const base = CATALOGUES[locale] ?? fr;
  return { ...base, ...overrides, tones: { ...base.tones, ...overrides.tones } };
}

/** The French catalogue, the default copy. */
export const labels: Labels = fr;
