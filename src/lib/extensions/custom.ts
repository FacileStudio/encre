import { Node, mergeAttributes } from '@tiptap/core';
import CalloutView from '../nodes/Callout.svelte';
import ChartView from '../nodes/Chart.svelte';
import StatView from '../nodes/Stat.svelte';
import { svelteNodeView } from './node-view.svelte.js';

/** Tone of a callout, the muse status vocabulary. */
export type CalloutTone = 'neutral' | 'info' | 'success' | 'warning' | 'danger';

/** Tone of a column: the callout tones. */
export type ColumnTone = CalloutTone;

/** Tone of a badge: the callout tones plus muse's accent. */
export type BadgeTone = CalloutTone | 'accent';

/** Language of a diagram; the app draws it. */
export type DiagramLanguage = 'mermaid' | 'd2' | 'dot';

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    encreBlocks: {
      setCallout: (tone?: CalloutTone) => ReturnType;
      insertStat: () => ReturnType;
      insertChart: (kind?: 'bar' | 'line') => ReturnType;
    };
  }
}

function parseJson(raw: string, fallback: unknown): unknown {
  try {
    return JSON.parse(raw);
  } catch {
    return fallback;
  }
}

/** An attribute stored as `data-<name>` on the rendered element; `json` for arrays and objects. */
export function dataAttr(name: string, fallback: unknown, json = false) {
  return {
    default: fallback,
    parseHTML: (element: HTMLElement) => {
      const raw = element.getAttribute(`data-${name}`);
      if (raw === null) return fallback;
      return json ? parseJson(raw, fallback) : raw;
    },
    renderHTML: (attrs: Record<string, unknown>) => ({ [`data-${name}`]: json ? JSON.stringify(attrs[name]) : attrs[name] })
  };
}

/** A callout: a toned box with an optional title around block content. */
export const Callout = Node.create({
  name: 'callout',
  group: 'block',
  content: 'block+',
  defining: true,
  addAttributes: () => ({ tone: dataAttr('tone', 'neutral'), title: dataAttr('title', '') }),
  parseHTML: () => [{ tag: 'div[data-type="callout"]' }],
  renderHTML: ({ HTMLAttributes }) => ['div', mergeAttributes(HTMLAttributes, { 'data-type': 'callout' }), 0],
  addNodeView: () => svelteNodeView(CalloutView, true),
  addCommands() {
    return { setCallout: (tone = 'info') => ({ commands }) => commands.wrapIn(this.name, { tone }) };
  }
});

/** A stat: one figure with its label and change. */
export const Stat = Node.create({
  name: 'stat',
  group: 'block',
  atom: true,
  addAttributes: () => ({ label: dataAttr('label', ''), value: dataAttr('value', ''), delta: dataAttr('delta', ''), sub: dataAttr('sub', '') }),
  parseHTML: () => [{ tag: 'div[data-type="stat"]' }],
  renderHTML: ({ HTMLAttributes }) => ['div', mergeAttributes(HTMLAttributes, { 'data-type': 'stat' })],
  addNodeView: () => svelteNodeView(StatView),
  addCommands() {
    return { insertStat: () => ({ commands }) => commands.insertContent({ type: this.name }) };
  }
});

/** A chart: a bar or line chart over labelled series. */
export const Chart = Node.create({
  name: 'chart',
  group: 'block',
  atom: true,
  addAttributes: () => ({
    kind: dataAttr('kind', 'bar'),
    title: dataAttr('title', ''),
    labels: dataAttr('labels', [], true),
    series: dataAttr('series', [], true)
  }),
  parseHTML: () => [{ tag: 'div[data-type="chart"]' }],
  renderHTML: ({ HTMLAttributes }) => ['div', mergeAttributes(HTMLAttributes, { 'data-type': 'chart' })],
  addNodeView: () => svelteNodeView(ChartView),
  addCommands() {
    return { insertChart: (kind = 'bar') => ({ commands }) => commands.insertContent({ type: this.name, attrs: { kind } }) };
  }
});
