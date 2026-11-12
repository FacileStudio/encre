import { Extension, type Editor } from '@tiptap/core';
import { Plugin, PluginKey, type EditorState } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';
import { settingsOf } from './settings.svelte.js';

type Meta = { add: { id: string; pos: number } } | { remove: string };

const key = new PluginKey<DecorationSet>('encreUploadPlaceholder');

function widget(text: string): HTMLElement {
  const element = document.createElement('span');
  element.className = 'encre-uploading';
  element.setAttribute('role', 'status');
  element.textContent = text;
  return element;
}

function next(set: DecorationSet, meta: Meta | undefined, state: EditorState, text: string): DecorationSet {
  if (!meta) return set;
  if ('remove' in meta) return set.remove(set.find(undefined, undefined, (spec) => spec.id === meta.remove));
  const decoration = Decoration.widget(meta.add.pos, () => widget(text), { id: meta.add.id });
  return set.add(state.doc, [decoration]);
}

/** Local, never-shared markers where images are uploading; they follow edits and never enter the document. */
export const UploadPlaceholder = Extension.create({
  name: 'encreUploadPlaceholder',
  addProseMirrorPlugins() {
    const editor = this.editor;
    return [
      new Plugin<DecorationSet>({
        key,
        state: {
          init: () => DecorationSet.empty,
          apply: (tr, set, _old, state) => next(set.map(tr.mapping, tr.doc), tr.getMeta(key) as Meta | undefined, state, settingsOf(editor).labels.imageUploading)
        },
        props: { decorations: (state) => key.getState(state) }
      })
    ];
  }
});

/** Shows the uploading placeholder `id` at `pos`. */
export function addPlaceholder(editor: Editor, id: string, pos: number): void {
  editor.view.dispatch(editor.state.tr.setMeta(key, { add: { id, pos } } satisfies Meta));
}

/** Where the placeholder `id` stands now, or null once it is gone. */
export function placeholderPos(editor: Editor, id: string): number | null {
  const found = key.getState(editor.state)?.find(undefined, undefined, (spec) => spec.id === id) ?? [];
  return found[0]?.from ?? null;
}

/** Removes the placeholder `id`. */
export function removePlaceholder(editor: Editor, id: string): void {
  editor.view.dispatch(editor.state.tr.setMeta(key, { remove: id } satisfies Meta));
}
