<script module lang="ts">
  import type { AwarenessProvider, CollabUser } from './extensions/collaboration.js';

  /** Props of the presence row. */
  export interface PresenceProps {
    provider: AwarenessProvider;
    class?: string;
  }

  /** A connected client and the user it announces. */
  export interface Peer extends CollabUser {
    clientId: number;
  }

  /** The users announced on `provider`'s awareness, this client first. */
  export function peers(provider: AwarenessProvider): Peer[] {
    const self = provider.awareness.clientID;
    return [...provider.awareness.getStates().entries()]
      .filter(([, state]) => state.user)
      .map(([clientId, state]) => ({ clientId, ...(state.user as CollabUser) }))
      .sort((a, b) => Number(b.clientId === self) - Number(a.clientId === self) || a.clientId - b.clientId);
  }
</script>

<script lang="ts">
  import { Avatar } from '@facile/muse';

  let { provider, class: className = '' }: PresenceProps = $props();

  let list = $state<Peer[]>([]);

  $effect(() => {
    const awareness = provider.awareness;
    const refresh = () => (list = peers(provider));
    refresh();
    awareness.on('change', refresh);
    return () => awareness.off('change', refresh);
  });
</script>

<ul class="flex items-center -space-x-2 {className}" aria-label="Editing now">
  {#each list as peer (peer.clientId)}
    <li class="rounded-fc-pill ring-2 ring-fc-bg" title={peer.name} style:--encre-caret={peer.color}>
      <Avatar name={peer.name} src={peer.avatar} size="sm" class="encre-presence" />
    </li>
  {/each}
</ul>
