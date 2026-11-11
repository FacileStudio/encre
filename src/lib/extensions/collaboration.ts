import type { Extensions } from '@tiptap/core';
import Collaboration from '@tiptap/extension-collaboration';
import CollaborationCaret from '@tiptap/extension-collaboration-caret';
import type { Awareness } from 'y-protocols/awareness';
import type * as Y from 'yjs';

/** The y-prosemirror fragment encre documents live in. */
export const FRAGMENT = 'default';

/** Who is editing, as other people see it. */
export interface CollabUser {
  name: string;
  color?: string;
  avatar?: string;
}

/** A y-websocket `WebsocketProvider`, or any stand-in with an awareness and its `sync` event. */
export interface AwarenessProvider {
  awareness: Awareness;
  synced: boolean;
  on(event: 'sync', listener: (synced: boolean) => void): void;
  off(event: 'sync', listener: (synced: boolean) => void): void;
}

type Listener = () => void;

/**
 * The provider as Tiptap's UniqueID expects it: a Hocuspocus-style `synced` event. y-websocket emits
 * `sync` instead, and not at all when it synced before the editor was created.
 */
function tiptapProvider(provider: AwarenessProvider) {
  const wrapped = new Map<Listener, (synced: boolean) => void>();
  return {
    awareness: provider.awareness,
    on(_event: 'synced', listener: Listener) {
      if (provider.synced) return listener();
      const once = (synced: boolean) => synced && listener();
      wrapped.set(listener, once);
      provider.on('sync', once);
    },
    off(_event: 'synced', listener: Listener) {
      const once = wrapped.get(listener);
      if (once) provider.off('sync', once);
      wrapped.delete(listener);
    }
  };
}

/** Collaboration settings of the encre editor. */
export interface CollabOptions {
  document: Y.Doc;
  provider?: AwarenessProvider;
  user?: CollabUser;
}

const PALETTE = ['#7e66ca', '#cc7d1e', '#009696', '#bc3f53', '#629f39', '#bb61a2'];

/** The muse chart color a client's caret takes when the user brings no color, as the `#rrggbb` y-tiptap accepts. */
export function caretColor(clientId: number): string {
  return PALETTE[clientId % PALETTE.length]!;
}

function renderCaret(user: Record<string, unknown>): HTMLElement {
  const caret = document.createElement('span');
  caret.className = 'encre-caret';
  caret.style.setProperty('--encre-caret', String(user.color));
  const label = document.createElement('span');
  label.className = 'encre-caret-label';
  label.textContent = String(user.name ?? '');
  caret.append(label);
  return caret;
}

/** Collaboration on the `default` fragment, plus carets when a provider carries awareness. */
export function collaborationExtensions(options: CollabOptions): Extensions {
  const extensions: Extensions = [Collaboration.configure({ document: options.document, field: FRAGMENT })];
  if (!options.provider) return extensions;
  const color = options.user?.color ?? caretColor(options.document.clientID);
  const user = { name: options.user?.name ?? '', avatar: options.user?.avatar, color };
  extensions.push(
    CollaborationCaret.configure({
      provider: tiptapProvider(options.provider),
      user,
      render: renderCaret,
      selectionRender: (u) => ({ nodeName: 'span', class: 'encre-caret-selection', style: `--encre-caret: ${u.color}` })
    })
  );
  return extensions;
}
