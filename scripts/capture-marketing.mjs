import { chromium } from '@playwright/test';
import { mkdir, writeFile } from 'node:fs/promises';

const output = 'artifacts/p6';
await mkdir(output, { recursive: true });
const browser = await chromium.launch();
try {
  const page = await browser.newPage({
    viewport: { width: 390, height: 844 },
    deviceScaleFactor: 2,
    reducedMotion: 'reduce',
  });
  const observations = [];
  await page.goto(
    'http://127.0.0.1:6006/iframe.html?id=marketing-details--home-mobile&viewMode=story',
  );
  await page.getByRole('heading', { level: 1 }).waitFor();
  await page.evaluate(() => globalThis.document.fonts.ready);
  await page.screenshot({
    path: `${output}/mobile-foundation.png`,
    fullPage: true,
  });
  observations.push(
    await page.evaluate(() => ({
      story: 'foundation-mobile',
      viewport: globalThis.innerWidth,
      documentWidth: globalThis.document.documentElement.scrollWidth,
      height: globalThis.document.body.scrollHeight,
      credits: Array.from(
        globalThis.document.querySelectorAll('footer a'),
      ).filter((link) => link.href === 'https://tanziro.com/').length,
    })),
  );
  await page.setViewportSize({ width: 1440, height: 900 });
  for (const [section, group] of [
    ['header', 'sections'],
    ['hero', 'sections'],
    ['works', 'sections'],
    ['pillars', 'sections'],
    ['how', 'details'],
    ['use-cases', 'details'],
    ['spotlight', 'details'],
    ['security', 'sections'],
    ['faq', 'details'],
    ['cta', 'sections'],
    ['footer', 'sections'],
  ]) {
    await page.goto(
      `http://127.0.0.1:6006/iframe.html?id=marketing-${group}--${section}-desktop&viewMode=story`,
    );
    const element = page.locator('#storybook-root > *').first();
    await element.waitFor({ state: 'visible' });
    await page.evaluate(() => globalThis.document.fonts.ready);
    await element.screenshot({ path: `${output}/${section}.png` });
    observations.push({ story: section, bounds: await element.boundingBox() });
  }
  await writeFile(
    `${output}/foundation-layout.json`,
    JSON.stringify(observations, null, 2) + '\n',
  );
  console.log(JSON.stringify(observations));
} finally {
  await browser.close();
}
