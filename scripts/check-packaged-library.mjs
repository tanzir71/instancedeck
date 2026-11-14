import { chromium } from '@playwright/test';
import { writeFile } from 'node:fs/promises';
const browser = await chromium.launch();
const observations = [];
try {
  const page = await browser.newPage({
    viewport: { width: 1280, height: 900 },
  });
  for (const entry of [
    'index.html',
    'iframe.html?id=marketing-sections--hero-desktop&viewMode=story',
  ]) {
    const errors = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await page.goto(`http://127.0.0.1:4324/instancedeck/storybook/${entry}`);
    await page.locator('footer').waitFor();
    if (entry.startsWith('iframe'))
      await page.getByRole('heading', { level: 1 }).waitFor();
    await page.evaluate(() => globalThis.document.fonts.ready);
    const observed = await page.evaluate(() => ({
      footer: globalThis.document
        .querySelector('footer')
        .getBoundingClientRect()
        .toJSON(),
      bodyHeight: globalThis.document.body.scrollHeight,
      viewport: globalThis.innerHeight,
      robots: globalThis.document.querySelector('meta[name="robots"]').content,
      credits: globalThis.document.querySelectorAll(
        'footer a[href="https://tanziro.com"]',
      ).length,
      children: Array.from(globalThis.document.body.children)
        .slice(0, 3)
        .map((node) => ({
          id: node.id,
          position: globalThis.getComputedStyle(node).position,
          bounds: node.getBoundingClientRect().toJSON(),
        })),
    }));
    if (
      observed.credits !== 2 ||
      !observed.robots.includes('noindex') ||
      errors.length
    )
      throw new Error(JSON.stringify({ observed, errors }));
    await page.locator('footer').scrollIntoViewIfNeeded();
    const reachable = await page.locator('footer').evaluate((footer) => {
      const bounds = footer.getBoundingClientRect();
      return (
        globalThis.document
          .elementFromPoint(
            bounds.x + bounds.width / 2,
            Math.min(globalThis.innerHeight - 1, bounds.y + bounds.height / 2),
          )
          ?.closest('footer') === footer
      );
    });
    if (!reachable) {
      console.log(
        await page.evaluate(() => ({
          scrollY: globalThis.scrollY,
          bodyOverflow: globalThis.getComputedStyle(globalThis.document.body)
            .overflow,
          htmlOverflow: globalThis.getComputedStyle(
            globalThis.document.documentElement,
          ).overflow,
          footer: globalThis.document
            .querySelector('footer')
            .getBoundingClientRect()
            .toJSON(),
          center: globalThis.document
            .elementFromPoint(640, 450)
            ?.outerHTML.slice(0, 180),
        })),
      );
      await page.screenshot({ path: 'artifacts/p6/library-covered.png' });
      throw new Error(`Packaged footer is covered: ${entry}`);
    }
    await page.screenshot({
      path: `artifacts/p6/library-${entry.startsWith('iframe') ? 'preview' : 'manager'}-footer.png`,
    });
    observations.push({ entry, ...observed, errors });
  }
  await writeFile(
    'artifacts/p6/packaged-library-browser.json',
    JSON.stringify(observations, null, 2) + '\n',
  );
  console.log(JSON.stringify(observations));
} finally {
  await browser.close();
}
