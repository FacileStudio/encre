import { Node, mergeAttributes } from '@tiptap/core';
import { toneFill } from '@facile/muse';
import type { BadgeTone } from './custom.js';
import { dataAttr } from './custom.js';

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    encreBadge: {
      insertBadge: (text: string, tone?: BadgeTone) => ReturnType;
      setBadge: (attrs: { tone?: BadgeTone; dot?: boolean }) => ReturnType;
    };
  }
}

const dot = {
  default: false,
  parseHTML: (element: HTMLElement) => element.getAttribute('data-dot') === 'true',
  renderHTML: (attrs: Record<string, unknown>) => (attrs.dot ? { 'data-dot': 'true' } : {})
};

/** A badge: a short toned label inside text; `dot` makes it a status pill. */
export const Badge = Node.create({
  name: 'badge',
  group: 'inline',
  inline: true,
  content: 'text*',
  marks: '',
  addAttributes: () => ({ tone: dataAttr('tone', 'neutral'), dot }),
  parseHTML: () => [{ tag: 'span[data-type="badge"]' }],
  renderHTML: ({ node, HTMLAttributes }) => {
    const tone = (node.attrs.tone in toneFill ? node.attrs.tone : 'neutral') as keyof typeof toneFill;
    const style = `encre-badge rounded-fc-pill px-2 py-0.5 text-fc-xs font-medium ${toneFill[tone]}`;
    return ['span', mergeAttributes(HTMLAttributes, { 'data-type': 'badge', class: style }), 0];
  },
  addCommands() {
    return {
      insertBadge:
        (text, tone = 'neutral') =>
        ({ commands }) =>
          commands.insertContent({ type: this.name, attrs: { tone }, content: text ? [{ type: 'text', text }] : [] }),
      setBadge:
        (attrs) =>
        ({ commands }) =>
          commands.updateAttributes(this.name, attrs)
    };
  }
});
