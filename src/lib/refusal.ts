import { isBlockId } from './ids.js';
import type { EncreNode } from './schema.js';

const PLACED = /^(doc(?:\.content\[\d+\])*): /;

/** The JSON path of a node, `doc.content[1].content[0]`, as indexes into each `content`. */
function indexes(path: string): number[] {
  return [...path.matchAll(/\[(\d+)\]/g)].map((match) => Number(match[1]));
}

/** The valid block id of the node at `path`, or of its nearest ancestor that carries one. */
function enclosingId(doc: EncreNode, path: string): string | undefined {
  let node: EncreNode | undefined = doc;
  let found: string | undefined;
  for (const index of indexes(path)) {
    node = node?.content?.[index];
    const id = node?.attrs?.id;
    if (isBlockId(id)) found = id;
  }
  return found;
}

/** Why a document was refused: an `Error` with the Rust message unchanged, plus the tree `path` and the `blockId` around it. */
export class EncreDocumentError extends Error {
  readonly path: string;
  readonly blockId?: string;

  /** Keeps the message as is; `path` is the JSON path of the failing node, `blockId` the id of the block that holds it. */
  constructor(message: string, path: string, blockId?: string) {
    super(message);
    this.name = 'EncreDocumentError';
    this.path = path;
    this.blockId = blockId;
  }
}

/** The refusal as an `EncreDocumentError` when its message starts with a tree path, else the error unchanged. */
export function placedError(doc: EncreNode, error: unknown): unknown {
  const message = error instanceof Error ? error.message : String(error);
  const path = PLACED.exec(message)?.[1];
  return path === undefined ? error : new EncreDocumentError(message, path, enclosingId(doc, path));
}

/** Runs an export, handing an `EncreDocumentError` to `onerror` before rethrowing it. */
export function reported<T>(write: () => T, onerror?: (error: Error) => void): T {
  try {
    return write();
  } catch (error) {
    if (error instanceof EncreDocumentError) onerror?.(error);
    throw error;
  }
}
