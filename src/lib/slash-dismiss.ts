/**
 * Closes an open slash menu the way a caret popup should: a press anywhere outside the list (the editor included),
 * focus leaving the editor or the window, or a scroll outside the list that would detach it from the caret. Returns
 * the cleanup.
 */
export function watchDismiss(list: () => HTMLElement | null, editor: HTMLElement, dismiss: () => void): () => void {
  const inside = (target: EventTarget | null) => target instanceof Node && Boolean(list()?.contains(target));
  const press = (event: Event) => !inside(event.target) && dismiss();
  const leave = (event: FocusEvent) => !inside(event.relatedTarget) && dismiss();
  window.addEventListener('pointerdown', press, true);
  window.addEventListener('scroll', press, true);
  window.addEventListener('blur', dismiss);
  editor.addEventListener('focusout', leave);
  return () => {
    window.removeEventListener('pointerdown', press, true);
    window.removeEventListener('scroll', press, true);
    window.removeEventListener('blur', dismiss);
    editor.removeEventListener('focusout', leave);
  };
}
