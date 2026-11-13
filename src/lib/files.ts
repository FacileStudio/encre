import type { Editor } from '@tiptap/core';
import type { BlockCommand } from './blocks.js';
import { embedKind, httpsUrl } from './embed.js';
import type { FileAttrs } from './extensions/embed.js';
import { settingsOf } from './extensions/settings.svelte.js';
import { labels as french, type Labels } from './labels.js';

/** The editor's file upload settings: the app's hook and where refusals go. */
export interface FileUpload {
  onfile: (file: File) => Promise<FileAttrs>;
  onerror?: (error: Error) => void;
}

/** The files of a paste or drop that are not images: images go through `onimage`. */
export function otherFiles(data: DataTransfer | null | undefined): File[] {
  return [...(data?.files ?? [])].filter((file) => !file.type.startsWith('image/'));
}

async function uploadOne(editor: Editor, file: File, upload: FileUpload): Promise<void> {
  try {
    const stored = await upload.onfile(file);
    if (typeof stored?.src !== 'string' || stored.src === '') throw new Error(settingsOf(editor).labels.fileNoSource);
    const size = Math.max(0, Math.trunc(Number(stored.size ?? file.size) || 0));
    const attrs = { src: stored.src, name: stored.name ?? file.name, size, mime: stored.mime ?? file.type };
    if (!editor.isDestroyed) editor.chain().focus().insertFile(attrs).run();
  } catch (reason) {
    upload.onerror?.(reason instanceof Error ? reason : new Error(String(reason)));
  }
}

/** Uploads `files` one after the other through the hook and inserts a file block for each at the caret. */
export async function uploadFiles(editor: Editor, files: File[], upload: FileUpload): Promise<void> {
  for (const file of files) await uploadOne(editor, file, upload);
}

/** Handles a paste or drop of files that are not images; returns false to let the editor handle it. */
export function transferFiles(editor: Editor, data: DataTransfer | null | undefined, upload: FileUpload | undefined): boolean {
  const files = upload ? otherFiles(data) : [];
  if (!upload || files.length === 0) return false;
  void uploadFiles(editor, files, upload);
  return true;
}

/** The "File" block command, named in `labels`: a file picker whose choice goes through the hook. */
export function fileCommand(upload: () => FileUpload | undefined, labels: Labels = french): BlockCommand {
  return {
    id: 'file',
    label: labels.file,
    keywords: 'fichier file pièce jointe attachment pdf',
    run: (editor) => {
      const input = document.createElement('input');
      input.type = 'file';
      input.multiple = true;
      input.addEventListener('change', () => {
        const options = upload();
        if (options) void uploadFiles(editor, [...(input.files ?? [])], options);
      });
      input.click();
    }
  };
}

/** The "Link / embed" block command, named in `labels`: asks for an https address and embeds it. */
export function embedCommand(onerror?: (error: Error) => void, labels: Labels = french): BlockCommand {
  return {
    id: 'embed',
    label: labels.embed,
    keywords: 'lien intégration embed link url toile feuilles vision docs',
    run: (editor) => {
      const url = window.prompt(labels.embedPrompt, 'https://')?.trim() ?? '';
      if (url === '' || url === 'https://') return;
      if (!httpsUrl(url)) return onerror?.(new Error(labels.embedInvalid));
      editor.chain().focus().insertEmbed(url, embedKind(url)).run();
    }
  };
}

/**
 * Turns a lone https URL pasted on an empty paragraph into an embed block, unless `onembed` answers false; returns
 * whether it did.
 */
export function pasteEmbed(editor: Editor, text: string, onembed?: (url: string) => boolean): boolean {
  const url = text.trim();
  const { selection } = editor.state;
  const { $from } = selection;
  const parent = $from.parent;
  const fits = $from.depth > 0 && $from.node(-1).canReplaceWith($from.index(-1), $from.indexAfter(-1), editor.schema.nodes.embed!);
  const emptyParagraph = selection.empty && parent.type.name === 'paragraph' && parent.content.size === 0 && fits;
  if (!emptyParagraph || !httpsUrl(url) || onembed?.(url) === false) return false;
  const from = selection.$from.before();
  const embed = { type: 'embed', attrs: { url, kind: embedKind(url) } };
  return editor.chain().focus().insertContentAt({ from, to: from + parent.nodeSize }, embed).run();
}
