import { describe, expect, it } from 'vitest';
import type { WebsocketProvider } from 'y-websocket';
import type { AwarenessProvider } from '../src/lib/extensions/collaboration.js';

const accept = (provider: WebsocketProvider): AwarenessProvider => provider;

describe('collaboration provider type', () => {
  it('accepts a y-websocket WebsocketProvider, checked by svelte-check', () => {
    expect(typeof accept).toBe('function');
  });
});
