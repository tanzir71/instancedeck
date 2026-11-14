import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { resolve } from 'node:path';

test('empty Deck renders shared tokens, local assets and fonts without errors', async ({
  page,
}) => {
  const errors: string[] = [];
  page.on('pageerror', (e) => errors.push(e.message));
  await page.goto('http://127.0.0.1:1420');
  await page.evaluate(() => document.fonts.ready);
  await expect(page.getByRole('heading', { name: 'Your deck' })).toBeVisible();
  await expect(
    page.getByRole('heading', { name: 'Add your first profile' }),
  ).toBeVisible();
  const images = await page.locator('img').evaluateAll((nodes) =>
    nodes.map((n) => ({
      source: n.src,
      valid: n.complete && n.naturalWidth > 0,
      width: n.width,
      height: n.height,
    })),
  );
  expect(images.every((i) => i.valid)).toBe(true);
  expect(images).toHaveLength(5);
  expect(errors).toEqual([]);
  expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([]);
  await page.screenshot({
    path: 'artifacts/p0/desktop-empty.png',
    fullPage: true,
  });
});

test('Astro serves static React output under the project base with required credit', async ({
  page,
}) => {
  await page.goto('http://127.0.0.1:4321/instancedeck/');
  await page.evaluate(() => document.fonts.ready);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(
    'InstanceDeck is in development.',
  );
  await expect(
    page
      .getByRole('contentinfo')
      .getByRole('link', { name: 'Tanzir Rahman', exact: true }),
  ).toHaveAttribute('href', 'https://tanziro.com');
  await expect(page.getByRole('contentinfo')).toContainText(
    'It is not affiliated with, endorsed by or sponsored by',
  );
  expect(await page.locator('a[href^="/"]').count()).toBe(0);
  expect(await page.locator('astro-island').count()).toBe(0);
  expect(
    await page
      .locator('img')
      .evaluateAll((nodes) =>
        nodes.every((image) => image.complete && image.naturalWidth > 0),
      ),
  ).toBe(true);
  expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([]);
  await page.screenshot({
    path: 'artifacts/p0/site-scaffold.png',
    fullPage: true,
  });
});

test('all Storybook stories pass their interactions and WCAG checks', async () => {
  test.setTimeout(180000);
  // Invoke Node directly to avoid shell argument handling on Windows.
  const { stdout } = await promisify(execFile)(
    process.execPath,
    [
      resolve(
        'packages/ui/node_modules/@storybook/test-runner/dist/test-storybook.js',
      ),
      '--url',
      'http://127.0.0.1:6006',
      '--maxWorkers=2',
    ],
    {
      cwd: resolve('packages/ui'),
      timeout: 150000,
      maxBuffer: 4 * 1024 * 1024,
    },
  );
  console.log(stdout);
});

test('capture foundations and Button visual evidence', async ({ page }) => {
  for (const [id, file] of [
    ['foundations-tokens--foundations', 'tokens'],
    ['components-button--primary', 'button'],
  ]) {
    await page.goto(
      `http://127.0.0.1:6006/iframe.html?id=${id}&viewMode=story`,
    );
    await page.locator('#storybook-root').waitFor();
    await expect(page.locator('#storybook-root')).not.toBeEmpty();
    await page.evaluate(() => document.fonts.ready);
    await page.screenshot({ path: `artifacts/p0/${file}.png`, fullPage: true });
  }
});
