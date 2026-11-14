import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
export default defineConfig({
  site: 'https://tanzir71.github.io',
  base: '/instancedeck',
  output: 'static',
  integrations: [react()],
});
