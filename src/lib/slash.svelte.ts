import { Extension, type Editor, type Range } from '@tiptap/core';
import { PluginKey } from '@tiptap/pm/state';
import Suggestion, { exitSuggestion, type SuggestionKeyDownProps, type SuggestionProps } from '@tiptap/suggestion';
import { BLOCK_COMMANDS, findBlocks, type BlockCommand } from './blocks.js';
import { watchDismiss } from './slash-dismiss.js';

/** What the slash menu shows: open or not, the matches, the highlighted one and where. */
export class SlashState {
  open = $state(false);
  items = $state<BlockCommand[]>([]);
  index = $state(0);
  rect = $state<DOMRect | null>(null);
  /** The rendered list, set by `SlashMenu`: presses and scrolls inside it keep the menu open. */
  list: HTMLElement | null = null;
  readonly pluginKey = new PluginKey('encreSlash');
  private pick: ((item: BlockCommand) => void) | null = null;
  private unwatch: (() => void) | null = null;

  show(props: SuggestionProps<BlockCommand, BlockCommand>): void {
    const view = props.editor.view;
    this.unwatch ??= watchDismiss(() => this.list, view.dom, () => exitSuggestion(view, this.pluginKey));
    this.open = true;
    this.items = props.items;
    this.index = Math.min(this.index, Math.max(0, props.items.length - 1));
    this.rect = props.clientRect?.() ?? null;
    this.pick = props.command;
  }

  hide(): void {
    this.unwatch?.();
    this.unwatch = null;
    this.open = false;
    this.index = 0;
    this.pick = null;
  }

  choose(item: BlockCommand | undefined): void {
    if (item) this.pick?.(item);
  }

  key({ event }: SuggestionKeyDownProps): boolean {
    if (!this.open || this.items.length === 0) return false;
    const moves: Record<string, number> = { ArrowDown: 1, ArrowUp: -1 };
    if (event.key in moves) {
      this.index = (this.index + moves[event.key]! + this.items.length) % this.items.length;
      return true;
    }
    if (event.key === 'Enter') this.choose(this.items[this.index]);
    if (event.key === 'Escape') this.hide();
    return event.key === 'Enter' || event.key === 'Escape';
  }
}

function runBlock(editor: Editor, range: Range, item: BlockCommand): void {
  editor.chain().focus().deleteRange(range).run();
  item.run(editor);
}

function renderer(slash: SlashState) {
  return {
    onStart: (props: SuggestionProps<BlockCommand, BlockCommand>) => slash.show(props),
    onUpdate: (props: SuggestionProps<BlockCommand, BlockCommand>) => slash.show(props),
    onExit: () => slash.hide(),
    onKeyDown: (props: SuggestionKeyDownProps) => slash.key(props)
  };
}

/** The `/` trigger that drives a `SlashState`, offering `commands` (every block by default). */
export function slashExtension(slash: SlashState, commands: () => BlockCommand[] = () => BLOCK_COMMANDS): Extension {
  return Extension.create({
    name: 'encreSlash',
    addProseMirrorPlugins() {
      return [
        Suggestion<BlockCommand, BlockCommand>({
          editor: this.editor,
          pluginKey: slash.pluginKey,
          char: '/',
          startOfLine: false,
          items: ({ query }) => findBlocks(query, commands()),
          command: ({ editor, range, props }) => runBlock(editor, range, props),
          render: () => renderer(slash)
        })
      ];
    }
  });
}
