import type { ColumnTone } from '../extensions/custom.js';

/** The tinted surface and title colour of each column tone, from muse's tokens. */
export const columnTone: Record<ColumnTone, { box: string; title: string }> = {
  neutral: { box: 'bg-fc-component', title: 'text-fc-fg' },
  info: { box: 'bg-fc-info/10', title: 'text-fc-info' },
  success: { box: 'bg-fc-success/10', title: 'text-fc-success' },
  warning: { box: 'bg-fc-warning/10', title: 'text-fc-warning' },
  danger: { box: 'bg-fc-danger/10', title: 'text-fc-danger' }
};
