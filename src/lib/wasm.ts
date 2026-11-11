import init, * as encre from './encre-wasm/encre_wasm.js';
import { placedError } from './refusal.js';
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

/** One reason Facile Markdown is invalid: the tree path, and the 1-based line and column of the source. */
export interface MarkdownProblem {
  path: string;
  message: string;
  line: number;
  column: number;
}

/** The verdict of `validateMarkdown`, the same as `encre validate --from markdown --json`. */
export interface MarkdownValidation {
  valid: boolean;
  errors: MarkdownProblem[];
}

/** Why `parseMarkdown` refused its input; `line`, `column` and `path` are set when the source position is known. */
export class EncreMarkdownError extends Error {
  readonly line?: number;
  readonly column?: number;
  readonly path?: string;

  /** Keeps the Rust message as is and adds the position of `at`, when given. */
  constructor(message: string, at?: MarkdownProblem) {
    super(message);
    this.name = 'EncreMarkdownError';
    this.line = at?.line;
    this.column = at?.column;
    this.path = at?.path;
  }
}

/** Reads and validates Facile Markdown; each error names its tree path, line and column. */
export function validateMarkdown(markdown: string): MarkdownValidation {
  loaded();
  return JSON.parse(encre.validateMarkdown(markdown)) as MarkdownValidation;
}

/**
 * Reads Facile Markdown into a document without ids. Throws an `EncreMarkdownError` when it nests past the limits
 * of FORMAT.md 2.6, with the line and column of the syntax that crosses the limit.
 */
export function parseMarkdown(markdown: string): EncreNode {
  loaded();
  try {
    return JSON.parse(encre.parse(markdown)) as EncreNode;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new EncreMarkdownError(message, validateMarkdown(markdown).errors[0]);
  }
}

function checked(doc: EncreNode, write: (json: string) => string): string {
  try {
    return write(JSON.stringify(doc));
  } catch (error) {
    throw placedError(doc, error);
  }
}

/** Writes a valid document as canonical Facile Markdown. Throws an `EncreDocumentError` when the document is invalid. */
export function serializeMarkdown(doc: EncreNode): string {
  loaded();
  return checked(doc, encre.serialize);
}

/** Writes a valid document as portable CommonMark. Throws an `EncreDocumentError` when the document is invalid. */
export function serializeCommonMark(doc: EncreNode): string {
  loaded();
  return checked(doc, encre.serialize_commonmark);
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
