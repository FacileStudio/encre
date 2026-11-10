import { Node, mergeAttributes } from '@tiptap/core';
import DiagramView from '../nodes/Diagram.svelte';
import type { DiagramLanguage } from './custom.js';
import { dataAttr } from './custom.js';
import { svelteNodeView } from './node-view.svelte.js';

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    encreDiagram: {
      insertDiagram: (language?: DiagramLanguage) => ReturnType;
    };
  }
}

/** A diagram: mermaid, d2 or dot source that the app draws; encre never runs it. */
export const Diagram = Node.create({
  name: 'diagram',
  group: 'block',
  content: 'text*',
  marks: '',
  code: true,
  defining: true,
  addAttributes: () => ({ language: dataAttr('language', 'mermaid') }),
  parseHTML: () => [{ tag: 'pre[data-type="diagram"]', preserveWhitespace: 'full' }],
  renderHTML: ({ HTMLAttributes }) => ['pre', mergeAttributes(HTMLAttributes, { 'data-type': 'diagram' }), ['code', 0]],
  addNodeView: () => svelteNodeView(DiagramView, true),
  addCommands() {
    return {
      insertDiagram:
        (language = 'mermaid') =>
        ({ commands }) =>
          commands.insertContent({ type: this.name, attrs: { language } })
    };
  }
});
