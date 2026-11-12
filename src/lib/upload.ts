import type { Editor } from '@tiptap/core';
import type { BlockCommand } from './blocks.js';
import { addPlaceholder, placeholderPos, removePlaceholder } from './extensions/upload-placeholder.js';
import { newBlockId } from './ids.js';
import { settingsOf } from './extensions/settings.svelte.js';
import { labels as french, type Labels } from './labels.js';

/** Where an uploaded image lives, as the app's `onimage` hook answers. */
export interface UploadedImage {
  src: string;
  alt?: string;
}

/** The editor's image upload settings. */
export interface ImageUpload {
  onimage: (file: File) => Promise<UploadedImage>;
  onerror?: (error: Error) => void;
  maxImageBytes?: number;
}

/** The image types the editor uploads; SVG is never one of them. */
export const IMAGE_TYPES = ['image/png', 'image/jpeg', 'image/gif', 'image/webp'];

/** Why `file` cannot be uploaded, in `labels` (French by default), or null when it can. */
export function imageRefusal(file: File, maxBytes?: number, labels: Labels = french): string | null {
  if (!IMAGE_TYPES.includes(file.type)) return labels.imageType;
  if (maxBytes !== undefined && file.size > maxBytes) return labels.imageSize(maxBytes);
  return null;
}

/** The image files of a paste or drop, SVG included so that it is refused out loud. */
export function imageFiles(data: DataTransfer | null | undefined): File[] {
  return [...(data?.files ?? [])].filter((file) => file.type.startsWith('image/'));
}

async function uploadOne(editor: Editor, file: File, pos: number, upload: ImageUpload): Promise<void> {
  const labels = settingsOf(editor).labels;
  const refusal = imageRefusal(file, upload.maxImageBytes, labels);
  if (refusal) return upload.onerror?.(new Error(refusal));
  const id = newBlockId();
  addPlaceholder(editor, id, pos);
  try {
    const image = await upload.onimage(file);
    if (typeof image?.src !== 'string' || image.src === '') throw new Error(labels.imageNoSource);
    const at = editor.isDestroyed ? null : placeholderPos(editor, id);
    if (at === null) return;
    removePlaceholder(editor, id);
    editor.chain().insertContentAt(at, { type: 'image', attrs: { src: image.src, alt: image.alt ?? '' } }).run();
  } catch (reason) {
    if (!editor.isDestroyed) removePlaceholder(editor, id);
    upload.onerror?.(reason instanceof Error ? reason : new Error(String(reason)));
  }
}

/** Uploads `files` through the hook and inserts each image where its placeholder ends up. */
export function uploadImages(editor: Editor, files: File[], pos: number, upload: ImageUpload): Promise<void> {
  return Promise.all(files.map((file) => uploadOne(editor, file, pos, upload))).then(() => undefined);
}

/** Handles a paste of image files; returns false to let the editor paste as usual. */
export function pasteImages(editor: Editor, event: ClipboardEvent, upload: ImageUpload | undefined): boolean {
  const files = upload ? imageFiles(event.clipboardData) : [];
  if (!upload || files.length === 0) return false;
  void uploadImages(editor, files, editor.state.selection.from, upload);
  return true;
}

function dropPos(editor: Editor, event: DragEvent): number {
  try {
    return editor.view.posAtCoords({ left: event.clientX, top: event.clientY })?.pos ?? editor.state.selection.from;
  } catch {
    return editor.state.selection.from;
  }
}

/** Handles a drop of image files at the drop point; returns false to let the editor drop as usual. */
export function dropImages(editor: Editor, event: DragEvent, upload: ImageUpload | undefined): boolean {
  const files = upload ? imageFiles(event.dataTransfer) : [];
  if (!upload || files.length === 0) return false;
  event.preventDefault();
  void uploadImages(editor, files, dropPos(editor, event), upload);
  return true;
}

/** The "Image" block command, named in `labels`: a file picker whose choice goes through the hook at the caret. */
export function imageCommand(upload: () => ImageUpload | undefined, labels: Labels = french): BlockCommand {
  return {
    id: 'image',
    label: labels.image,
    keywords: 'image photo illustration',
    run: (editor) => {
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = IMAGE_TYPES.join(',');
      input.multiple = true;
      input.addEventListener('change', () => {
        const options = upload();
        if (options) void uploadImages(editor, [...(input.files ?? [])], editor.state.selection.from, options);
      });
      input.click();
    }
  };
}
