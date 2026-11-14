/* global document */
import { chromium } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { writeFile } from 'node:fs/promises';
const prefix = 'https://tanzir71.github.io/instancedeck/';
const sitemap = await fetch(`${prefix}sitemap-0.xml`, {
  signal: AbortSignal.timeout(20000),
});
if (!sitemap.ok) throw new Error(`Live sitemap returned ${sitemap.status}`);
const urls = Array.from(
  (await sitemap.text()).matchAll(/<loc>([^<]+)<\/loc>/g),
  (match) => match[1],
);
if (urls.length !== 24 || urls.some((url) => !url.startsWith(prefix)))
  throw new Error('Unexpected published sitemap');
const browser = await chromium.launch({ headless: true });
const results = [];
try {
  const page = await browser.newPage({
    viewport: { width: 1440, height: 1000 },
  });
  const errors = [];
  page.on('pageerror', (error) => errors.push(String(error)));
  for (const url of urls) {
    const response = await page.goto(url, { waitUntil: 'networkidle' });
    if (response?.status() !== 200)
      throw new Error(`Live route failed: ${url}`);
    const check = await page.evaluate(() => ({
      title: document.title,
      canonical: document
        .querySelector('link[rel="canonical"]')
        ?.getAttribute('href'),
      credit: document.querySelectorAll('footer a[href="https://tanziro.com"]')
        .length,
      nonAffiliation: document
        .querySelector('footer')
        ?.textContent?.includes('It is not affiliated with'),
      absoluteRootLinks: document.querySelectorAll('a[href^="/"]').length,
      h1: document.querySelectorAll('h1').length,
    }));
    if (
      check.canonical !== url ||
      check.credit < 1 ||
      !check.nonAffiliation ||
      check.absoluteRootLinks !== 0 ||
      check.h1 !== 1
    )
      throw new Error(
        `Live metadata/footer audit failed: ${url} ${JSON.stringify(check)}`,
      );
    results.push({ url, status: response.status(), ...check });
  }
  await page.goto(prefix, { waitUntil: 'networkidle' });
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({
    path: 'artifacts/p6/live-home-desktop.png',
    fullPage: true,
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload({ waitUntil: 'networkidle' });
  if ((await page.evaluate(() => document.documentElement.scrollWidth)) > 390)
    throw new Error('Live mobile page overflows');
  await page.screenshot({
    path: 'artifacts/p6/live-home-mobile.png',
    fullPage: true,
  });
  const missing = await page.goto(`${prefix}missing/nested/page/`);
  if (
    missing?.status() !== 404 ||
    !(await page
      .getByRole('heading', { name: 'This page is not in the deck.' })
      .count())
  )
    throw new Error('Live custom 404 failed');
  await page.goto(
    `${prefix}storybook/iframe.html?id=desktop-power-features--dark-deck&viewMode=story`,
    { waitUntil: 'networkidle' },
  );
  await page.waitForFunction(
    () => document.documentElement.dataset.theme === 'dark',
  );
  const footerViolations = (
    await new AxeBuilder({ page })
      .include('footer')
      .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'wcag22aa'])
      .analyze()
  ).violations;
  if (footerViolations.length)
    throw new Error(
      `Dark Storybook footer contrast failed: ${JSON.stringify(footerViolations.map((v) => v.id))}`,
    );
  if (errors.length)
    throw new Error(`Live browser errors: ${JSON.stringify(errors)}`);
  await writeFile(
    'artifacts/p6/live-site-audit.json',
    JSON.stringify(
      {
        checkedAt: new Date().toISOString(),
        prefix,
        pages: results,
        mobileWidth: 390,
        custom404: 404,
        errors,
      },
      null,
      2,
    ) + '\n',
  );
  console.log(
    `Verified ${results.length} live routes, mobile layout and nested 404.`,
  );
} finally {
  await browser.close();
}
