import { chromium } from '@playwright/test';
import { mkdir } from 'node:fs/promises';
const output = 'artifacts/p6';
await mkdir(output, { recursive: true });
const browser = await chromium.launch();
try {
  const page = await browser.newPage({
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 2,
    reducedMotion: 'reduce',
  });
  await page.goto('http://127.0.0.1:4324/instancedeck/');
  await page.evaluate(async () => {
    globalThis.document.querySelectorAll('img').forEach((img) => {
      img.loading = 'eager';
    });
    await globalThis.document.fonts.ready;
    await Promise.all(
      Array.from(globalThis.document.images).map((img) =>
        img.decode().catch(() => {}),
      ),
    );
  });
  await page.screenshot({
    path: `${output}/home-desktop-browser.png`,
    fullPage: true,
  });
  for (const [name, selector] of [
    ['header', 'body > header'],
    ['hero', '#product'],
    ['works', 'section[aria-label="Works with"]'],
    ['pillars', 'section[aria-labelledby="pillars-title"]'],
    ['how', 'section[aria-labelledby="how-title"]'],
    ['use-cases', 'section[aria-labelledby="use-cases-title"]'],
    ['spotlight', 'section[aria-label="Apps"]'],
    ['security', 'section[aria-labelledby="security-title"]'],
    ['faq', 'section[aria-labelledby="faq-title"]'],
    ['cta', 'section[aria-labelledby="cta-title"]'],
    ['footer', 'body > footer'],
  ]) {
    const bounds = await page.locator(selector).boundingBox();
    if (!bounds) throw new Error(`Missing section ${name}`);
    await page.screenshot({
      path: `${output}/site-${name}.png`,
      fullPage: true,
      clip: bounds,
    });
  }
  console.log('Captured all eleven built sections at 1440px.');
} finally {
  await browser.close();
}
