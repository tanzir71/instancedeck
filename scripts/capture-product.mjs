/* global document */
import { chromium } from '@playwright/test';
import { mkdir, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

const directory = new URL('../apps/site/src/assets/', import.meta.url);
await mkdir(directory, { recursive: true });
const browser = await chromium.launch({ headless: true });
try {
  const page = await browser.newPage({
    viewport: { width: 1280, height: 800 },
    deviceScaleFactor: 2,
  });
  await page.goto(
    'http://127.0.0.1:6006/iframe.html?id=desktop-screens--deck&viewMode=story',
  );
  await page.getByRole('heading', { name: 'Your deck' }).waitFor();
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({
    path: fileURLToPath(new URL('desktop-deck.png', directory)),
  });
  await mkdir(new URL('../artifacts/p4/', import.meta.url), {
    recursive: true,
  });
  await writeFile(
    new URL('../artifacts/p4/product-capture.json', import.meta.url),
    JSON.stringify(
      {
        source: 'Built packages/ui Deck view',
        fixture: 'Figma reference examples, not real accounts',
        dimensions: [2560, 1600],
        deviceScaleFactor: 2,
        nativeVerification: 'Deferred to the user',
      },
      null,
      2,
    ) + '\n',
  );
} finally {
  await browser.close();
}
