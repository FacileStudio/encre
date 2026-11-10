import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [svelte()],
  resolve: process.env.VITEST ? { conditions: ['browser'] } : undefined,
  test: {
    include: process.env.ENCRE_GEN ? ['scripts/gen-*.ts'] : ['test/**/*.test.ts'],
    environment: 'jsdom'
  }
});
