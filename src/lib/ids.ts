import { schema } from './schema.js';

/** Returns a fresh block id: ten characters of lowercase Crockford base32. */
export function newBlockId(): string {
  const { alphabet, length } = schema.id;
  const bytes = crypto.getRandomValues(new Uint8Array(length));
  return Array.from(bytes, (byte) => alphabet[byte % alphabet.length]).join('');
}

/** Whether `value` has the shape of a block id. */
export function isBlockId(value: unknown): value is string {
  const { alphabet, length } = schema.id;
  return typeof value === 'string' && value.length === length && [...value].every((c) => alphabet.includes(c));
}
