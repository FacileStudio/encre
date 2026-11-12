import { Extension, type Editor } from '@tiptap/core';
import { labelsFor, type Labels } from '../labels.js';

/** Draws a diagram from its language and source: SVG markup the app trusts, or an element. */
export type RenderDiagram = (language: string, source: string) => Promise<string | SVGElement>;

/** What node views and menus read from the editor: the interface copy and the app's hooks. */
export interface EncreSettings {
  labels: Labels;
  renderDiagram?: RenderDiagram;
}

declare module '@tiptap/core' {
  interface Storage {
    encreSettings: EncreSettings;
  }
}

/** Carries the labels and the diagram renderer to every node view through the editor storage. */
export const Settings = Extension.create<EncreSettings, EncreSettings>({
  name: 'encreSettings',
  addOptions: () => ({ labels: labelsFor('fr') }),
  addStorage() {
    return { labels: this.options.labels, renderDiagram: this.options.renderDiagram };
  }
});

/** The settings of `editor`, French labels when the extension is missing. */
export function settingsOf(editor: Editor): EncreSettings {
  return (editor.storage as Partial<{ encreSettings: EncreSettings }>).encreSettings ?? { labels: labelsFor('fr') };
}
