import { schema, type AttrSpec, type EncreNode } from './schema.js';

type Marks = NonNullable<EncreNode['marks']>;

function compactAttrs(specs: AttrSpec[] | undefined, attrs: Record<string, unknown> | undefined): Record<string, unknown> | undefined {
  const out: Record<string, unknown> = {};
  for (const spec of specs ?? []) {
    const value = attrs?.[spec.name];
    if (value !== undefined && value !== null && JSON.stringify(value) !== JSON.stringify(spec.default)) out[spec.name] = value;
  }
  return Object.keys(out).length ? out : undefined;
}

function compactMarks(marks: Marks): Marks {
  const rank = (name: string) => schema.marks.findIndex((m) => m.name === name);
  return [...marks]
    .sort((a, b) => rank(a.type) - rank(b.type))
    .map((mark) => {
      const attrs = compactAttrs(schema.marks.find((m) => m.name === mark.type)?.attrs, mark.attrs);
      return attrs ? { type: mark.type, attrs } : { type: mark.type };
    });
}

/**
 * The canonical stored form of a document: attributes outside the schema or equal to their default are
 * dropped and marks follow rank order. Rust `encre::yjs::read` returns the same form.
 */
export function compactDocument(node: EncreNode): EncreNode {
  const out: EncreNode = { type: node.type };
  const attrs = compactAttrs(schema.nodes.find((n) => n.name === node.type)?.attrs, node.attrs);
  if (attrs) out.attrs = attrs;
  if (node.content?.length) out.content = node.content.map(compactDocument);
  if (node.text !== undefined) out.text = node.text;
  if (node.marks?.length) out.marks = compactMarks(node.marks);
  return out;
}
