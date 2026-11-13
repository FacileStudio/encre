import type { Snippet } from 'svelte';
import type * as Y from 'yjs';
import type { ResolveEmbed } from './embed.js';
import type { AwarenessProvider, CollabUser } from './extensions/collaboration.js';
import type { FileAttrs } from './extensions/embed.js';
import type { RenderDiagram } from './extensions/settings.svelte.js';
import type { LabelOverrides, Locale } from './labels.js';
import type { EncreNode } from './schema.js';
import type { UploadedImage } from './upload.js';

/** What the editor hands `oncontextmenu`: the event, whether the selection is empty, and the selection as markdown. */
export interface ContextMenuInfo {
  event: MouseEvent;
  empty: boolean;
  markdown: string;
}

/** Props of the encre editor. */
export interface EditorProps {
  content?: EncreNode;
  editable?: boolean;
  /** Text shown in an empty document, read once at mount. */
  placeholder?: string;
  /** Shows the sticky formatting toolbar; true by default. */
  toolbar?: boolean;
  onchange?: (doc: EncreNode) => void;
  class?: string;
  /** The shared document; when set, it is the source of truth and `content` is ignored. Read once at mount, with `provider` and `user`: wrap the editor in `{#key ydoc}` to switch documents. */
  ydoc?: Y.Doc;
  /** A y-websocket `WebsocketProvider` (or a stand-in with `awareness`, `synced` and the `sync` event) for carets and presence. */
  provider?: AwarenessProvider;
  /** This user, as other people see them. */
  user?: CollabUser;
  /** Where the editor shows who is here: `inline` above the toolbar, or `none` for an app that renders `Presence` itself. */
  presence?: 'inline' | 'none';
  /** Uploads a pasted, dropped or picked image and answers its address; without it images are not uploaded. */
  onimage?: (file: File) => Promise<UploadedImage>;
  /** Uploads a pasted, dropped or picked file that is not an image and answers the file block; without it files are not uploaded. */
  onfile?: (file: File) => Promise<FileAttrs>;
  /** Called when a lone https URL is pasted on an empty paragraph; answer false to paste it as text instead of an embed. */
  onembed?: (url: string) => boolean;
  /** Enriches link cards with a title, description and picture. */
  resolveEmbed?: ResolveEmbed;
  /** When set, the editor keeps the browser's menu closed on right click and calls this instead. */
  oncontextmenu?: (info: ContextMenuInfo) => void;
  /** Rendered at the end of the selection bubble menu, such as an "Ask the assistant" button. */
  extras?: Snippet;
  /** Reports a refused image or address, a failed upload, a paste encre could not read as blocks, or a refused export. */
  onerror?: (error: Error) => void;
  /** The largest image, in bytes, handed to `onimage`. */
  maxImageBytes?: number;
  /** The language of the editor's copy: menus, node views and placeholders. */
  locale?: Locale;
  /** Replaces some of that copy. */
  labels?: LabelOverrides;
  /** Draws a diagram from its language and source; without it a diagram shows its source. */
  renderDiagram?: RenderDiagram;
}
