import type { Editor, NodeViewRenderer, NodeViewRendererProps } from '@tiptap/core';
import type { Node as PMNode } from '@tiptap/pm/model';
import { flushSync, mount, unmount, type Component } from 'svelte';

/** The live state a node view component reads. */
export class NodeViewState {
  attrs = $state<Record<string, unknown>>({});
  selected = $state(false);
  editable = $state(true);

  constructor(attrs: Record<string, unknown>, editable: boolean) {
    this.attrs = attrs;
    this.editable = editable;
  }
}

/** Props every encre node view component receives. */
export interface NodeViewProps {
  view: NodeViewState;
  update: (attrs: Record<string, unknown>) => void;
  content?: HTMLElement;
}

function updater(editor: Editor, getPos: NodeViewRendererProps['getPos'], current: () => PMNode) {
  return (attrs: Record<string, unknown>) => {
    const pos = typeof getPos === 'function' ? getPos() : undefined;
    if (pos === undefined) return;
    editor.view.dispatch(editor.state.tr.setNodeMarkup(pos, undefined, { ...current().attrs, ...attrs }));
  };
}

function isControl(target: EventTarget | null, content: HTMLElement | undefined): boolean {
  if (!(target instanceof HTMLElement) || content?.contains(target)) return false;
  return target.closest('input, textarea, select, button, [role="menu"], [role="listbox"]') !== null;
}

/** Renders a node with a Svelte component; `withContent` gives the component an editable content host. */
export function svelteNodeView(component: Component<NodeViewProps>, withContent = false): NodeViewRenderer {
  return ({ node, editor, getPos }) => {
    let current = node;
    const dom = document.createElement('div');
    dom.dataset.encreNode = node.type.name;
    const content = withContent ? document.createElement('div') : undefined;
    const view = new NodeViewState({ ...node.attrs }, editor.isEditable);
    const instance = mount(component, { target: dom, props: { view, update: updater(editor, getPos, () => current), content } });
    flushSync();
    return {
      dom,
      contentDOM: content,
      update: (next: PMNode) => {
        if (next.type !== current.type) return false;
        current = next;
        view.attrs = { ...next.attrs };
        return true;
      },
      selectNode: () => (view.selected = true),
      deselectNode: () => (view.selected = false),
      stopEvent: (event: Event) => isControl(event.target, content),
      ignoreMutation: (mutation) => mutation.type !== 'selection' && !content?.contains(mutation.target as globalThis.Node),
      destroy: () => void unmount(instance)
    };
  };
}
