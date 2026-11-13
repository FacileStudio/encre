import { Icon } from '@facile/muse';
import { flushSync, mount, unmount, type Component } from 'svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { BLOCK_ICONS, EMBED_LOGOS, embedLogo } from '../src/lib/block-icons.js';
import { blockCommands, type BlockCommand } from '../src/lib/blocks.js';
import { EMBED_KINDS } from '../src/lib/embed.js';
import { embedCommands } from '../src/lib/embed-commands.js';
import { fileCommand } from '../src/lib/files.js';
import { labelsFor } from '../src/lib/labels.js';
import SlashMenu from '../src/lib/SlashMenu.svelte';
import { SlashState } from '../src/lib/slash.svelte.js';
import { imageCommand } from '../src/lib/upload.js';

const mounted: ReturnType<typeof mount>[] = [];
const scroll = vi.fn();
Element.prototype.scrollIntoView = function (this: Element, arg?: boolean | ScrollIntoViewOptions) {
  scroll(this, arg);
};
const labels = labelsFor('fr');
const every: BlockCommand[] = [
  ...blockCommands(labels),
  imageCommand(() => undefined, labels),
  ...embedCommands(undefined, labels),
  fileCommand(() => undefined, labels)
];

function place<Props extends Record<string, unknown>>(component: Component<Props>, props: Props): HTMLElement {
  const target = document.createElement('div');
  document.body.append(target);
  mounted.push(mount(component, { target, props }));
  flushSync();
  return target;
}

afterEach(() => {
  mounted.splice(0).forEach((app) => void unmount(app));
  scroll.mockClear();
});

describe('block command icons', () => {
  it('gives every command its own icon from BLOCK_ICONS', () => {
    for (const command of every) expect(command.icon, command.id).toBe(BLOCK_ICONS[command.id]);
  });

  it('offers one embed entry per Facile app, found by the app name, with its brand logo', () => {
    for (const kind of EMBED_KINDS.filter((k) => k !== 'link')) {
      const command = every.find((c) => c.id === `embed-${kind}`);
      expect(command?.icon, kind).toBe(embedLogo(kind));
      expect(command?.keywords, kind).toContain(kind);
    }
  });

  it('maps every embed kind to a logo', () => {
    for (const kind of EMBED_KINDS) expect(EMBED_LOGOS[kind], kind).toMatch(/^solar:/);
    expect(embedLogo('toile')).toBe('solar:tablet-bold-duotone');
    expect(embedLogo('unknown')).toBe(EMBED_LOGOS.link);
  });

  it('draws every block icon and embed logo through muse Icon', () => {
    for (const name of [...Object.values(BLOCK_ICONS), ...Object.values(EMBED_LOGOS)]) {
      const target = place(Icon, { icon: name });
      expect(target.querySelector('svg')?.innerHTML, name).toMatch(/<(path|g|rect|circle)\b/);
    }
  });
});

describe('slash menu', () => {
  function openSlash(): SlashState {
    const slash = new SlashState();
    slash.open = true;
    slash.items = every;
    slash.rect = new DOMRect(0, 0, 10, 10);
    return slash;
  }

  it('shows an icon before every option', () => {
    const target = place(SlashMenu, { slash: openSlash() });
    const options = [...target.querySelectorAll('[role="option"]')];
    expect(options).toHaveLength(every.length);
    for (const option of options) expect(option.querySelector('svg')).not.toBeNull();
  });

  it('scrolls the highlighted option into view as the arrows move it', () => {
    const slash = openSlash();
    const target = place(SlashMenu, { slash });
    const press = (key: string) =>
      slash.key({ event: new KeyboardEvent('keydown', { key }) } as Parameters<SlashState['key']>[0]);
    press('ArrowUp');
    flushSync();
    const options = target.querySelectorAll('[role="option"]');
    expect(slash.index).toBe(every.length - 1);
    expect(scroll).toHaveBeenLastCalledWith(options[every.length - 1], { block: 'nearest' });
    press('ArrowDown');
    flushSync();
    expect(scroll).toHaveBeenLastCalledWith(options[0], { block: 'nearest' });
  });
});
