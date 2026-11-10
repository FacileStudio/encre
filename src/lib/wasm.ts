import init, * as encre from './encre-wasm/encre_wasm.js';
import type { EncreNode } from './schema.js';

/** Where the WebAssembly comes from: a URL, a response, or the bytes themselves. */
export type WasmSource = BufferSource | WebAssembly.Module | URL | string | Response;

let loading: Promise<void> | null = null;
let ready = false;

/** Loads encre-wasm once. Later calls return the same promise. */
export function loadEncre(source?: WasmSource): Promise<void> {
  loading ??= init(source === undefined ? undefined : { module_or_path: source }).then(() => {
    ready = true;
  });
  return loading;
}

/** Whether `loadEncre` has finished, so the synchronous functions below can run. */
export function encreReady(): boolean {
  return ready;
}

function loaded(): void {
  if (!ready) throw new Error('encre-wasm is not loaded: await loadEncre() first');
}

/** Reads Facile Markdown into a document without ids. */
export function parseMarkdown(markdown: string): EncreNode {
  loaded();
  return JSON.parse(encre.parse(markdown)) as EncreNode;
}

/** Writes a valid document as canonical Facile Markdown. Throws when the document is invalid. */
export function serializeMarkdown(doc: EncreNode): string {
  loaded();
  return encre.serialize(JSON.stringify(doc));
}

/** Writes a valid document as portable CommonMark. Throws when the document is invalid. */
export function serializeCommonMark(doc: EncreNode): string {
  loaded();
  return encre.serialize_commonmark(JSON.stringify(doc));
}

/** The reason `doc` is invalid, or null when it validates. */
export function validateDocument(doc: EncreNode): string | null {
  loaded();
  try {
    encre.validate(JSON.stringify(doc));
    return null;
  } catch (error) {
    return error instanceof Error ? error.message : String(error);
  }
}

/**
 * Converts an OpenUI Lang program (Bulle's chat UI) to a document without ids, for "save as document".
 * Throws with the reason when the program does not parse, has a reference cycle or an unknown component.
 */
export function openuiToDocument(source: string): EncreNode {
  loaded();
  return JSON.parse(encre.openuiToDocument(source)) as EncreNode;
}
