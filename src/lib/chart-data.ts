/** One chart series: a name and one number per label. */
export interface Series {
  name: string;
  data: number[];
}

/** Reads `value` as chart labels, dropping anything that is not a string. */
export function toLabels(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((v): v is string => typeof v === 'string') : [];
}

/** Reads `value` as chart series, dropping malformed entries and non-finite numbers. */
export function toSeries(value: unknown): Series[] {
  if (!Array.isArray(value)) return [];
  return value
    .filter((s): s is { name: unknown; data: unknown } => typeof s === 'object' && s !== null)
    .map((s) => ({
      name: typeof s.name === 'string' ? s.name : '',
      data: Array.isArray(s.data) ? s.data.filter((n): n is number => typeof n === 'number' && Number.isFinite(n)) : []
    }));
}

/** Writes series as editable text, one `name: 1, 2, 3` line each. */
export function seriesToText(series: Series[]): string {
  return series.map((s) => `${s.name}: ${s.data.join(', ')}`).join('\n');
}

/** Reads the text `seriesToText` writes; blank lines are skipped. */
export function textToSeries(text: string): Series[] {
  return text
    .split('\n')
    .filter((line) => line.trim() !== '')
    .map((line) => {
      const at = line.indexOf(':');
      const name = (at < 0 ? '' : line.slice(0, at)).trim();
      const values = (at < 0 ? line : line.slice(at + 1)).split(',').map((v) => Number(v.trim()));
      return { name, data: values.filter((n) => Number.isFinite(n)) };
    });
}

/** Reads a comma-separated label list. */
export function textToLabels(text: string): string[] {
  return text
    .split(',')
    .map((label) => label.trim())
    .filter((label) => label !== '');
}
