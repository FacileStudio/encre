import init, * as encre from './encre-wasm/encre_wasm.js';

/** Where the WebAssembly comes from: a URL, a response, or the bytes themselves. */
export type WasmSource = BufferSource | WebAssembly.Module | URL | string | Response;

let loading: Promise<void> | null = null;
let ready = false;
let trapped = false;

/**
 * Loads encre-wasm once. Later calls return the same promise; after a failed load, the next call tries again.
 * After a trap the instance is dropped and the next call, or the next encre function, runs on a fresh one.
 */
export function loadEncre(source?: WasmSource): Promise<void> {
  if (trapped) revive();
  loading ??= init(source === undefined ? undefined : { module_or_path: source }).then(
    () => {
      ready = true;
    },
    (error: unknown) => {
      loading = null;
      throw error;
    }
  );
  return loading;
}

/** Whether `loadEncre` has finished, so the synchronous functions below can run. */
export function encreReady(): boolean {
  return ready;
}

/** Whether the last error came from the instance itself, a trap or no loaded module, so it must reach the caller as is. */
export function unusable(): boolean {
  return trapped || !ready;
}

function revive(): void {
  trapped = false;
  const module = encre.__encre_reset();
  try {
    if (module === undefined) throw new Error('encre-wasm has no compiled module');
    encre.initSync({ module });
  } catch {
    ready = false;
    loading = null;
  }
}

/** Wraps an encre-wasm export: it refuses to run before `loadEncre`, and marks the instance trapped when it traps. */
function guarded<A extends unknown[], R>(run: (...args: A) => R): (...args: A) => R {
  return (...args) => {
    if (trapped) revive();
    if (!ready) throw new Error('encre-wasm is not loaded: await loadEncre() first');
    try {
      return run(...args);
    } catch (error) {
      if (error instanceof WebAssembly.RuntimeError || error instanceof RangeError) trapped = true;
      throw error;
    }
  };
}

/** The encre-wasm exports, guarded. */
export const wasm = {
  parse: guarded(encre.parse),
  validateMarkdown: guarded(encre.validateMarkdown),
  serialize: guarded(encre.serialize),
  serializeCommonMark: guarded(encre.serialize_commonmark),
  validate: guarded(encre.validate),
  openuiToDocument: guarded(encre.openuiToDocument)
};

