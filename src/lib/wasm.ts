import { placedError } from './refusal.js';
import type { EncreNode } from './schema.js';
import { unusable, wasm } from './wasm-instance.js';

export { encreReady, loadEncre, type WasmSource } from './wasm-instance.js';

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
  return JSON.parse(wasm.validateMarkdown(markdown)) as MarkdownValidation;
}

/**
 * Reads Facile Markdown into a document without ids. Throws an `EncreMarkdownError` when it nests past the limits
 * of FORMAT.md 2.6, with the line and column of the syntax that crosses the limit.
 */
export function parseMarkdown(markdown: string): EncreNode {
  try {
    return JSON.parse(wasm.parse(markdown)) as EncreNode;
  } catch (error) {
    if (unusable()) throw error;
    const message = error instanceof Error ? error.message : String(error);
    throw new EncreMarkdownError(message, validateMarkdown(markdown).errors[0]);
  }
}

function checked(doc: EncreNode, write: (json: string) => string): string {
  try {
    return write(JSON.stringify(doc));
  } catch (error) {
    throw unusable() ? error : placedError(doc, error);
  }
}

/** Writes a valid document as canonical Facile Markdown. Throws an `EncreDocumentError` when the document is invalid. */
export function serializeMarkdown(doc: EncreNode): string {
  return checked(doc, wasm.serialize);
}

/** Writes a valid document as portable CommonMark. Throws an `EncreDocumentError` when the document is invalid. */
export function serializeCommonMark(doc: EncreNode): string {
  return checked(doc, wasm.serializeCommonMark);
}

/** The reason `doc` is invalid, or null when it validates. */
export function validateDocument(doc: EncreNode): string | null {
  try {
    wasm.validate(JSON.stringify(doc));
    return null;
  } catch (error) {
    if (unusable()) throw error;
    return error instanceof Error ? error.message : String(error);
  }
}

/**
 * Converts an OpenUI Lang program (Bulle's chat UI) to a document without ids, for "save as document".
 * Throws with the reason when the program does not parse, has a reference cycle or an unknown component.
 */
export function openuiToDocument(source: string): EncreNode {
  return JSON.parse(wasm.openuiToDocument(source)) as EncreNode;
}
