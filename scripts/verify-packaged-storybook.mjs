/* global document */
import { chromium } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { createServer } from 'node:http';
import { readFile, stat, writeFile } from 'node:fs/promises';
import { resolve, sep, extname } from 'node:path';
const root = resolve('apps/site/dist');
const types = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.webp': 'image/webp',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2',
};
const server = createServer(async (request, response) => {
  try {
    const pathname = decodeURIComponent(
      new URL(request.url, 'http://127.0.0.1').pathname,
    ).replace(/^\/instancedeck\//, '/');
    let path = resolve(root, `.${pathname}`);
    if (path !== root && !path.startsWith(root + sep)) {
      response.writeHead(403).end();
      return;
    }
    if ((await stat(path)).isDirectory()) path = resolve(path, 'index.html');
    const body = await readFile(path);
    response
      .writeHead(200, {
        'Content-Type': types[extname(path)] ?? 'application/octet-stream',
      })
      .end(body);
  } catch {
    response.writeHead(404).end();
  }
});
await new Promise((ready, reject) => {
  server.once('error', reject);
  server.listen(4324, '127.0.0.1', ready);
});
const browser = await chromium.launch({ headless: true });
try {
  const context = await browser.newContext({
    viewport: { width: 1280, height: 900 },
  });
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (error) => errors.push(String(error)));
  const results = [];
  for (const entry of [
    'index.html',
    'iframe.html?id=desktop-power-features--dark-deck&viewMode=story',
  ]) {
    const response = await page.goto(
      `http://127.0.0.1:4324/instancedeck/storybook/${entry}`,
      { waitUntil: 'networkidle' },
    );
    if (response.status() !== 200)
      throw new Error('Packaged Storybook route failed');
    if (entry.startsWith('iframe'))
      await page.waitForFunction(
        () => document.documentElement.dataset.theme === 'dark',
      );
    const footer = page.locator('footer');
    if ((await footer.getByRole('link', { name: /tanziro.com/ }).count()) < 1)
      throw new Error('Missing visible credit');
    const violations = (
      await new AxeBuilder({ page })
        .include('footer')
        .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
        .analyze()
    ).violations;
    if (violations.length)
      throw new Error(
        JSON.stringify(
          violations.map((v) => ({
            id: v.id,
            targets: v.nodes.map((n) => n.target),
          })),
        ),
      );
    results.push({
      entry,
      theme: await page.locator('html').getAttribute('data-theme'),
      footer: await footer.boundingBox(),
      robots: await page.locator('meta[name="robots"]').getAttribute('content'),
      violations: [],
    });
  }
  if (errors.length) throw new Error(JSON.stringify(errors));
  await writeFile(
    'artifacts/p6/packaged-library-browser.json',
    JSON.stringify({ results, errors }, null, 2) + '\n',
  );
  console.log(
    'Packaged Storybook manager and dark preview passed footer/accessibility checks.',
  );
} finally {
  await browser.close();
  server.closeAllConnections();
  await new Promise((done) => server.close(done));
}
