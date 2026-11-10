const SCHEMES = ['http:', 'https:', 'mailto:', 'tel:'];

/** `raw` when it is a link a reader may follow: http, https, mailto, tel, or relative. Otherwise undefined. */
export function linkHref(raw: unknown): string | undefined {
  if (typeof raw !== 'string' || /[\p{Cc}]/u.test(raw)) return undefined;
  const scheme = /^([a-z][a-z0-9+.-]*):/i.exec(raw.trim());
  if (!scheme) return raw;
  return SCHEMES.includes(scheme[0].toLowerCase()) ? raw : undefined;
}

/** `raw` when it is an image source a reader may load: http, https or relative. Otherwise undefined. */
export function imageSrc(raw: unknown): string | undefined {
  const href = linkHref(raw);
  return href && !/^(mailto|tel):/i.test(href) ? href : undefined;
}
