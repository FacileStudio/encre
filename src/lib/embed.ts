/** What an embed block shows: a Facile app in a frame, or a link card. */
export type EmbedKind = 'link' | 'toile' | 'feuilles' | 'vision' | 'docs' | 'nuage';

/** Embed kinds in schema order. */
export const EMBED_KINDS: EmbedKind[] = ['link', 'toile', 'feuilles', 'vision', 'docs', 'nuage'];

/** The kinds drawn as a live frame of the app; the others are link cards. */
export const FRAMED_KINDS: EmbedKind[] = ['toile', 'feuilles', 'vision', 'docs'];

/** The Facile hosts encre recognises, by host name. */
export const FACILE_HOSTS: Record<string, EmbedKind> = {
  'toile.facile.studio': 'toile',
  'feuilles.facile.studio': 'feuilles',
  'vision.facile.studio': 'vision',
  'docs.facile.studio': 'docs',
  'nuage.facile.studio': 'nuage'
};

/** What a host app knows about a link, to enrich its card. */
export interface EmbedPreview {
  title?: string;
  description?: string;
  image?: string;
  site?: string;
}

/** Looks a link up for its card; the editor shows the bare link until it answers. */
export type ResolveEmbed = (url: string) => Promise<EmbedPreview>;

/** `raw` as a URL when it is an https address with a host, the only kind an embed block holds. */
export function httpsUrl(raw: unknown): URL | null {
  if (typeof raw !== 'string' || /[\s\p{Cc}]/u.test(raw) || !raw.startsWith('https://')) return null;
  try {
    const url = new URL(raw);
    return url.host === '' ? null : url;
  } catch {
    return null;
  }
}

/** The kind of embed `url` makes: the Facile app its host names (`hosts` adds or replaces hosts), else `link`. */
export function embedKind(url: string, hosts: Record<string, EmbedKind> = {}): EmbedKind {
  const parsed = httpsUrl(url);
  if (!parsed) return 'link';
  return { ...FACILE_HOSTS, ...hosts }[parsed.host] ?? 'link';
}

/** The frame address of a framed embed: `?embed=1` added, but Vision pages as they are. */
export function frameSrc(url: string, kind: EmbedKind): string {
  const parsed = httpsUrl(url);
  if (!parsed) return '';
  if (kind !== 'vision') parsed.searchParams.set('embed', '1');
  return parsed.toString();
}

/** A byte count for people: `0 B`, `12 KB`, `3.4 MB`. */
export function humanSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let value = Math.max(0, Number(bytes) || 0);
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  const digits = unit === 0 || value >= 10 ? 0 : 1;
  return `${value.toFixed(digits)} ${units[unit]}`;
}

/** The short type tag a file row shows: its extension, else the subtype of its MIME type. */
export function fileTag(name: string, mime: string): string {
  const extension = /\.([a-z0-9]{1,5})$/i.exec(name)?.[1];
  const subtype = mime.split('/')[1]?.split(/[.+-]/).pop();
  return (extension ?? subtype ?? 'file').slice(0, 4).toUpperCase();
}
