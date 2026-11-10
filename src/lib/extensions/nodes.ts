import { CodeBlock } from '@tiptap/extension-code-block';
import { HardBreak } from '@tiptap/extension-hard-break';
import { Image } from '@tiptap/extension-image';
import { OrderedList } from '@tiptap/extension-list';
import { Plugin, type EditorState, type Transaction } from '@tiptap/pm/state';

/** Code block whose language is an empty string when unset. */
export const EncreCodeBlock = CodeBlock.extend({
  addAttributes() {
    return { language: { default: '', parseHTML: languageOf, rendered: false } };
  }
});

function languageOf(element: HTMLElement): string | null {
  const name = [...(element.firstElementChild?.classList ?? [])].find((c) => c.startsWith('language-'));
  return name ? name.slice('language-'.length) : null;
}

/** Ordered list with `start` only. */
export const EncreOrderedList = OrderedList.extend({
  addAttributes() {
    return {
      start: {
        default: 1,
        parseHTML: (element: HTMLElement) => Number.parseInt(element.getAttribute('start') ?? '1', 10) || 1
      }
    };
  }
});

/** Hard break that never carries marks, so markdown stays lossless. */
export const EncreHardBreak = HardBreak.extend({
  addProseMirrorPlugins() {
    return [new Plugin({ appendTransaction: (transactions, _old, state) => stripBreakMarks(transactions, state, this.name) })];
  }
});

function stripBreakMarks(transactions: readonly Transaction[], state: EditorState, name: string): Transaction | null {
  if (!transactions.some((tr) => tr.docChanged)) return null;
  const tr = state.tr;
  state.doc.descendants((node, pos) => {
    if (node.type.name === name && node.marks.length > 0) tr.removeMark(pos, pos + 1);
  });
  return tr.steps.length > 0 ? tr : null;
}

/** Block image with `src`, `alt` and `title`. */
export const EncreImage = Image.extend({
  addAttributes() {
    return {
      src: { default: null },
      alt: { default: '' },
      title: { default: '' }
    };
  }
}).configure({ inline: false });
