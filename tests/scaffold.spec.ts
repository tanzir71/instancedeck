import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

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
  expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([]);
  await page.screenshot({
    path: 'artifacts/p0/site-scaffold.png',
    fullPage: true,
  });
});

test('all Storybook stories pass their interactions and WCAG checks', async () => {
  test.setTimeout(180000);
  // Run via pnpm so the workspace resolves the test runner and its config together.
  const { stdout } = await promisify(execFile)(
    process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm',
    ['test:storybook'],
    {
      cwd: process.cwd(),
      shell: process.platform === 'win32',
      timeout: 150000,
      maxBuffer: 4 * 1024 * 1024,
    },
  );
  console.log(stdout);
});
