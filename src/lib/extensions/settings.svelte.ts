import { Extension, type Editor } from '@tiptap/core';
import type { ResolveEmbed } from '../embed.js';
import { labelsFor, type Labels } from '../labels.js';

/** Draws a diagram from its language and source: SVG markup the app trusts, or an element. */
export type RenderDiagram = (language: string, source: string) => Promise<string | SVGElement>;

/** What node views and menus read from the editor: the interface copy and the app's hooks. */
export interface EncreSettings {
  labels: Labels;
  renderDiagram?: RenderDiagram;
  resolveEmbed?: ResolveEmbed;
}

class LiveSettings implements EncreSettings {
  labels = $state.raw<Labels>(labelsFor('fr'));
  renderDiagram = $state.raw<RenderDiagram | undefined>();
  resolveEmbed = $state.raw<ResolveEmbed | undefined>();

  constructor(settings: EncreSettings) {
    this.labels = settings.labels;
    this.renderDiagram = settings.renderDiagram;
    this.resolveEmbed = settings.resolveEmbed;
  }
}

/** The editor storage of `Settings`: the settings node views read, reactive, so a change reaches mounted views. */
export interface SettingsStorage {
  live: EncreSettings;
}

declare module '@tiptap/core' {
  interface Storage {
    encreSettings: SettingsStorage;
  }
}

/** Carries the labels, the diagram renderer and the link resolver to every node view through the editor storage. */
export const Settings = Extension.create<EncreSettings, SettingsStorage>({
  name: 'encreSettings',
  addOptions: () => ({ labels: labelsFor('fr') }),
  addStorage() {
    return { live: new LiveSettings(this.options) };
  }
});

/** The settings of `editor`, French labels when the extension is missing. Assigning a field updates mounted node views. */
export function settingsOf(editor: Editor): EncreSettings {
  return (editor.storage as Partial<{ encreSettings: SettingsStorage }>).encreSettings?.live ?? { labels: labelsFor('fr') };
}
