import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
import sitemap from '@astrojs/sitemap';
import { execFileSync } from 'node:child_process';

const base = `/${(process.env.BASE_PATH ?? 'instancedeck').replace(/^\/+|\/+$/g, '')}`;
let lastmod;
try {
  lastmod =
    execFileSync(
      'git',
      [
        'log',
        '-1',
        '--format=%cI',
        '--',
        'src',
        '../../packages/ui/src',
        '../../packages/recipes',
      ],
      { encoding: 'utf8' },
    ).trim() || undefined;
} catch {
  /* A source archive has no git history; omit lastmod. */
}
export default defineConfig({
  site: process.env.SITE_URL ?? 'https://tanzir71.github.io',
  base: base === '/' ? '/' : base,
  trailingSlash: 'always',
  output: 'static',
  markdown: { syntaxHighlight: false },
  security: {
    csp: {
      directives: [
        "default-src 'self'",
        "img-src 'self' data:",
        "font-src 'self'",
        "connect-src 'self'",
        "object-src 'none'",
        "base-uri 'self'",
        "form-action 'none'",
      ],
      styleDirective: {
        resources: [
          { resource: "'self'", kind: 'element' },
          { resource: "'unsafe-inline'", kind: 'attribute' },
        ],
      },
    },
  },
  integrations: [
    react(),
    sitemap({
      filter: (page) =>
        !page.endsWith('/404.html') &&
        !page.endsWith('/404/') &&
        !page.includes('/storybook/'),
      serialize: (item) => ({ ...item, ...(lastmod ? { lastmod } : {}) }),
    }),
  ],
  devToolbar: { enabled: false },
});
