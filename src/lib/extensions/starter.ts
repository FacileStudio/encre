import type { Extensions } from '@tiptap/core';
import StarterKit from '@tiptap/starter-kit';
import { UniqueID } from '@tiptap/extension-unique-id';
import { BLOCK_ID_TYPES } from '../schema.js';
import { newBlockId } from '../ids.js';
import { EncreCode, EncreLink } from './marks.js';
import { TaskItem, TaskList } from '@tiptap/extension-list';
import { EncreCodeBlock, EncreHardBreak, EncreImage, EncreOrderedList } from './nodes.js';

/** StarterKit with every node and mark the encre schema does not have switched off. */
export function encreStarterKit(): Extensions {
  return [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
      underline: false,
      link: false,
      code: false,
      codeBlock: false,
      hardBreak: false,
      orderedList: false
    }),
    EncreLink.configure({ openOnClick: false, autolink: true }),
    EncreCode,
    EncreCodeBlock,
    EncreHardBreak,
    EncreOrderedList,
    TaskList,
    TaskItem.configure({ nested: true }),
    EncreImage,
    UniqueID.configure({ types: BLOCK_ID_TYPES, generateID: newBlockId })
  ];
}
