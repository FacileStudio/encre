import { blockIcon, type BlockCommand } from './blocks.js';
import { embedKind, httpsUrl, type EmbedKind } from './embed.js';
import { labels as french, type Labels } from './labels.js';

type AppKind = Exclude<EmbedKind, 'link'>;

const APPS: { kind: AppKind; label: (labels: Labels) => string; keywords: string }[] = [
  { kind: 'toile', label: (l) => l.embedToile, keywords: 'toile tableau board canvas whiteboard' },
  { kind: 'feuilles', label: (l) => l.embedFeuilles, keywords: 'feuilles classeur tableur spreadsheet sheet' },
  { kind: 'vision', label: (l) => l.embedVision, keywords: 'vision tableau de bord dashboard' },
  { kind: 'docs', label: (l) => l.embedDocs, keywords: 'docs page document' },
  { kind: 'nuage', label: (l) => l.embedNuage, keywords: 'nuage fichier file cloud' }
];

/**
 * The "Link / embed" block command, named in `labels`: asks for an https address and embeds it, as `kind` when given,
 * else as the kind its host names.
 */
export function embedCommand(onerror?: (error: Error) => void, labels: Labels = french, kind?: AppKind): BlockCommand {
  const app = APPS.find((entry) => entry.kind === kind);
  const id = app ? `embed-${app.kind}` : 'embed';
  return {
    id,
    label: app ? app.label(labels) : labels.embed,
    icon: blockIcon(id),
    keywords: app ? `${app.keywords} intégration embed` : 'lien intégration embed link url toile feuilles vision docs',
    run: (editor) => {
      const url = window.prompt(labels.embedPrompt, 'https://')?.trim() ?? '';
      if (url === '' || url === 'https://') return;
      if (!httpsUrl(url)) return onerror?.(new Error(labels.embedInvalid));
      editor.chain().focus().insertEmbed(url, kind ?? embedKind(url)).run();
    }
  };
}

/** The generic "Link / embed" command followed by one command per Facile app (Toile, Feuilles, Vision, Docs, Nuage). */
export function embedCommands(onerror?: (error: Error) => void, labels: Labels = french): BlockCommand[] {
  return [embedCommand(onerror, labels), ...APPS.map((app) => embedCommand(onerror, labels, app.kind))];
}
