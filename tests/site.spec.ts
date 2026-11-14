import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { readdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';

async function files(directory: string): Promise<string[]> {
  const entries = await readdir(directory, { withFileTypes: true });
  return (
    await Promise.all(
      entries.map(async (entry) =>
        entry.isDirectory()
          ? files(join(directory, entry.name))
          : [join(directory, entry.name)],
      ),
    )
  ).flat();
}
test('every built site page has valid metadata, accessible content and intact local links', async ({
  page,
  request,
}) => {
  test.setTimeout(120000);
  const dist = join(process.cwd(), 'apps/site/dist');
  const built = await files(dist);
  const htmlFiles = built.filter(
    (path) => path.endsWith('.html') && !path.includes('storybook'),
  );
  expect(htmlFiles.length).toBeGreaterThanOrEqual(19);
  const titles = new Set<string>();
  const observations = [];
  for (const path of htmlFiles) {
    const relative = path
      .slice(dist.length + 1)
      .replaceAll('\\', '/')
      .replace(/index\.html$/, '');
    const local = `http://127.0.0.1:4322/instancedeck/${relative}`;
    await page.goto(local);
    await page.evaluate(() => document.fonts.ready);
    const title = await page.title();
    expect(title.length, `${relative}: title`).toBeLessThanOrEqual(60);
    expect(titles.has(title), `${relative}: duplicate title`).toBe(false);
    titles.add(title);
    const description = await page
      .locator('meta[name="description"]')
      .getAttribute('content');
    expect(description?.length, `${relative}: description`).toBeLessThanOrEqual(
      155,
    );
    expect(description?.length).toBeGreaterThan(25);
    await expect(page.getByRole('heading', { level: 1 })).toHaveCount(1);
    const footer = page.getByRole('contentinfo');
    await expect(footer).toContainText(
      'It is not affiliated with, endorsed by or sponsored by',
    );
    await expect(footer.locator('a[href="https://tanziro.com"]')).toHaveCount(
      2,
    );
    expect(await page.locator('a[href^="/"]').count()).toBe(0);
    expect(await page.locator('astro-island').count()).toBe(0);
    const canonical = await page
      .locator('link[rel="canonical"]')
      .getAttribute('href');
    expect(canonical).toBe(
      `https://tanzir71.github.io/instancedeck/${relative}`,
    );
    const graph = await page
      .locator('script[type="application/ld+json"]')
      .textContent();
    const schema = JSON.parse(graph!);
    expect(schema['@context']).toBe('https://schema.org');
    expect(Array.isArray(schema['@graph'])).toBe(true);
    const og = await page
      .locator('meta[property="og:image"]')
      .getAttribute('content');
    const response = await request.get(
      `http://127.0.0.1:4322${new URL(og!).pathname}`,
    );
    expect(response.status(), `${relative}: OG image`).toBe(200);
    const png = await response.body();
    expect([png.readUInt32BE(16), png.readUInt32BE(20)]).toEqual([1200, 630]);
    const links = await page
      .locator('a[href]')
      .evaluateAll((nodes) =>
        nodes.map((node) => (node as HTMLAnchorElement).href),
      );
    for (const link of links) {
      const url = new URL(link);
      if (
        url.origin !== 'http://127.0.0.1:4322' ||
        url.pathname.includes('/storybook/')
      )
        continue;
      const target = await request.get(`${url.origin}${url.pathname}`);
      expect(target.status(), `${relative}: ${url.pathname}`).toBe(200);
    }
    expect(
      (await new AxeBuilder({ page }).analyze()).violations,
      relative,
    ).toEqual([]);
    observations.push({
      path: relative || '/',
      title,
      descriptionLength: description!.length,
      links: links.length,
      axeViolations: 0,
    });
  }
  for (const path of built.filter(
    (path) => path.endsWith('.css') && !path.includes('storybook'),
  )) {
    expect(await readFile(path, 'utf8'), path).not.toMatch(/box-shadow\s*:/i);
  }
  await page.goto('http://127.0.0.1:4322/instancedeck/');
  const scripts = await page
    .locator('script[src]')
    .evaluateAll((nodes) =>
      nodes.map((node) => (node as HTMLScriptElement).src),
    );
  let jsGzipBytes = 0;
  const inlineScripts = await page
    .locator('script:not([src]):not([type="application/ld+json"])')
    .allTextContents();
  for (const code of inlineScripts) jsGzipBytes += gzipSync(code).length;
  for (const url of scripts)
    jsGzipBytes += gzipSync(await (await request.get(url)).body()).length;
  expect(jsGzipBytes).toBeLessThan(20 * 1024);
  const llms = await request.get('http://127.0.0.1:4322/instancedeck/llms.txt');
  expect(llms.status()).toBe(200);
  expect(await llms.text()).toContain('independent open-source project');
  await writeFile(
    'artifacts/p6/site-audit.json',
    JSON.stringify({ pages: observations, jsGzipBytes }, null, 2) + '\n',
  );
});

test('built mobile home matches the abridged layout and its menu works without React hydration', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(error.message));
  await page.goto('http://127.0.0.1:4322/instancedeck/');
  const menu = page.getByRole('button', { name: 'Menu', exact: true });
  await menu.focus();
  await page.keyboard.press('Enter');
  const dialog = page.getByRole('dialog');
  await expect(dialog).toBeVisible();
  for (let index = 0; index < 12; index++) {
    await page.keyboard.press('Tab');
    expect(
      await dialog.evaluate((element) =>
        element.contains(document.activeElement),
      ),
    ).toBe(true);
  }
  await page.keyboard.press('Escape');
  await expect(dialog).toBeHidden();
  await expect(menu).toBeFocused();
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(
    390,
  );
  expect(errors).toEqual([]);
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({
    path: 'artifacts/p6/home-mobile-browser.png',
    fullPage: true,
  });
});

test('app, FAQ and comparison content fit a mobile viewport', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  for (const slug of [
    'apps/claude-desktop',
    'faq',
    'compare/parall-for-windows',
  ]) {
    await page.goto(`http://127.0.0.1:4322/instancedeck/${slug}/`);
    await page.evaluate(() => document.fonts.ready);
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth),
      slug,
    ).toBe(390);
    expect((await new AxeBuilder({ page }).analyze()).violations, slug).toEqual(
      [],
    );
    await page.screenshot({
      path: `artifacts/p6/${slug.replaceAll('/', '-')}-mobile.png`,
      fullPage: true,
    });
  }
});

test('custom 404 navigation stays inside the project from a missing nested URL', async ({
  page,
}) => {
  const response = await page.goto(
    'http://127.0.0.1:4322/instancedeck/missing/nested/page/',
  );
  expect(response?.status()).toBe(404);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(
    'This page is not in the deck.',
  );
  const links = await page
    .locator('a[href]')
    .evaluateAll((nodes) => nodes.map((node) => node.getAttribute('href')!));
  expect(
    links.filter(
      (href) => !href.startsWith('#') && !href.startsWith('https://'),
    ).length,
  ).toBe(0);
  expect(links).toContain('https://tanzir71.github.io/instancedeck/apps/');
  expect(links).toContain('https://tanzir71.github.io/instancedeck/download/');
});
