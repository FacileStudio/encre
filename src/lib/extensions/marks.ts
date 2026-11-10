import Code from '@tiptap/extension-code';
import Link from '@tiptap/extension-link';

/** Link with the two attributes the schema keeps: `href` and `title`. */
export const EncreLink = Link.extend({
  addAttributes() {
    return {
      href: { default: null, parseHTML: (element: HTMLElement) => element.getAttribute('href') },
      title: { default: '', parseHTML: (element: HTMLElement) => element.getAttribute('title') }
    };
  }
});

/** Inline code that combines with every other mark, as markdown allows. */
export const EncreCode = Code.extend({ excludes: '' });
