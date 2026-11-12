/* tslint:disable */
/* eslint-disable */

/**
 * Converts an OpenUI Lang program to a document without ids; the error says why it cannot.
 */
export function openuiToDocument(source: string): string;

/**
 * Reads Facile Markdown and returns the document as ProseMirror JSON, without ids.
 */
export function parse(markdown: string): string;

/**
 * Writes a valid document as canonical Facile Markdown.
 */
export function serialize(doc: string): string;

/**
 * Writes a valid document as portable CommonMark.
 */
export function serialize_commonmark(doc: string): string;

/**
 * Validates a document; the error message says what is wrong and where.
 */
export function validate(doc: string): void;

/**
 * Reads and validates Facile Markdown, as `encre validate --from markdown --json`:
 * `{"valid", "errors": [{"path", "message", "line", "column"}]}` as JSON.
 */
export function validateMarkdown(markdown: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly openuiToDocument: (a: number, b: number) => [number, number, number, number];
    readonly parse: (a: number, b: number) => [number, number, number, number];
    readonly serialize: (a: number, b: number) => [number, number, number, number];
    readonly serialize_commonmark: (a: number, b: number) => [number, number, number, number];
    readonly validate: (a: number, b: number) => [number, number];
    readonly validateMarkdown: (a: number, b: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

/** Drops the current instance, so the next `initSync` or `init` builds a fresh one after a trap; returns its compiled module. */
export function __encre_reset(): WebAssembly.Module | undefined;
