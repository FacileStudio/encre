import spec from '../../spec/schema.json';

/** One attribute of a node or mark, as `spec/schema.json` declares it. */
export interface AttrSpec {
  name: string;
  type: string;
  default: unknown;
  values?: unknown[];
  required?: boolean;
  min?: number;
  max?: number;
}

/** One node type of the encre schema. */
export interface NodeSpec {
  name: string;
  group?: string;
  content?: string;
  marks?: string;
  takesMarks?: boolean;
  attrs: AttrSpec[];
}

/** One mark type of the encre schema. */
export interface MarkSpec {
  name: string;
  excludes?: string;
  attrs: AttrSpec[];
}

/** The parsed `spec/schema.json`. */
export interface EncreSchema {
  version: number;
  id: { alphabet: string; length: number };
  nodes: NodeSpec[];
  marks: MarkSpec[];
}

/** The encre schema, the single source every projection is tested against. */
export const schema: EncreSchema = spec;

/** The schema version stored next to each document. */
export const SCHEMA_VERSION = schema.version;

/** Node types that carry a block `id`. */
export const BLOCK_ID_TYPES = schema.nodes.filter((node) => node.attrs.some((attr) => attr.type === 'id')).map((node) => node.name);

/** A ProseMirror JSON node of an encre document. */
export interface EncreNode {
  type: string;
  attrs?: Record<string, unknown>;
  content?: EncreNode[];
  text?: string;
  marks?: { type: string; attrs?: Record<string, unknown> }[];
}
