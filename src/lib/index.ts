export { default as Editor } from './Editor.svelte';
export type { EditorProps } from './Editor.svelte';
export { default as Document } from './Document.svelte';
export { default as Presence, peers, type Peer, type PresenceProps } from './Presence.svelte';
export { default as Toolbar } from './Toolbar.svelte';
export { default as BubbleMenu } from './BubbleMenu.svelte';
export { default as SlashMenu } from './SlashMenu.svelte';
export { SlashState, slashExtension } from './slash.svelte.js';
export { BLOCK_COMMANDS, blockCommands, findBlocks, type BlockCommand } from './blocks.js';
export { encreExtensions, type EncreExtensionOptions } from './extensions/index.js';
export { caretColor, FRAGMENT, type AwarenessProvider, type CollabOptions, type CollabUser } from './extensions/index.js';
export { compactDocument } from './compact.js';
export { Badge, Callout, Chart, Column, Columns, Diagram, Stat, Table, TableCell, TableHeader, TableRow } from './extensions/index.js';
export type { Align, BadgeTone, CalloutTone, ColumnTone, DiagramLanguage } from './extensions/index.js';
export { Settings, settingsOf, type EncreSettings, type RenderDiagram } from './extensions/index.js';
export { CATALOGUES, labelsFor, type LabelOverrides, type Labels, type Locale } from './labels.js';
export type { Highlight } from './render/context.js';
export { schema, SCHEMA_VERSION, BLOCK_ID_TYPES } from './schema.js';
export type { AttrSpec, EncreNode, EncreSchema, MarkSpec, NodeSpec } from './schema.js';
export { newBlockId, isBlockId } from './ids.js';
export { linkHref, imageSrc } from './render/href.js';
export {
  loadEncre,
  encreReady,
  parseMarkdown,
  validateMarkdown,
  EncreMarkdownError,
  serializeMarkdown,
  serializeCommonMark,
  validateDocument,
  openuiToDocument,
  type MarkdownProblem,
  type MarkdownValidation,
  type WasmSource
} from './wasm.js';
export { looksLikeMarkdown } from './paste.js';
export { IMAGE_TYPES, imageRefusal, type ImageUpload, type UploadedImage } from './upload.js';
