import { getSchema } from '@tiptap/core';
import type { AttributeSpec, MarkType, NodeType } from '@tiptap/pm/model';
import { describe, expect, it } from 'vitest';
import { encreExtensions } from '../src/lib/extensions/index.js';
import { schema, type AttrSpec } from '../src/lib/schema.js';

const pm = getSchema(encreExtensions());

function attrsOf(type: NodeType | MarkType): Record<string, unknown> {
  const attrs = (type.spec.attrs ?? {}) as Record<string, AttributeSpec>;
  return Object.fromEntries(Object.entries(attrs).map(([name, spec]) => [name, spec.default]));
}

function declared(attrs: AttrSpec[]): Record<string, unknown> {
  return Object.fromEntries(attrs.map((attr) => [attr.name, attr.default]));
}

describe('schema drift between spec/schema.json and the Tiptap extensions', () => {
  it('has exactly the node types of the spec', () => {
    expect(Object.keys(pm.nodes).sort()).toEqual(schema.nodes.map((n) => n.name).sort());
  });

  it('has exactly the mark types of the spec, in rank order', () => {
    expect(Object.keys(pm.marks)).toEqual(schema.marks.map((m) => m.name));
  });

  it.each(schema.nodes.map((n) => [n.name, n] as const))('node %s matches', (name, spec) => {
    const type = pm.nodes[name]!;
    expect(type.spec.group ?? null).toBe(spec.group ?? null);
    expect(type.spec.content ?? null).toBe(spec.content ?? null);
    expect(attrsOf(type)).toEqual(declared(spec.attrs));
    if (spec.marks !== undefined) expect(type.spec.marks).toBe(spec.marks === '_' ? undefined : spec.marks);
  });

  it.each(schema.marks.map((m) => [m.name, m] as const))('mark %s matches', (name, spec) => {
    const type = pm.marks[name]!;
    expect(attrsOf(type)).toEqual(declared(spec.attrs));
    expect(type.spec.excludes ?? null).toBe(spec.excludes ?? null);
  });
});
