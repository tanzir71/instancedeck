/* global document */
import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { chromium } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
const snapshot = JSON.parse(
  await readFile('apps/site/src/data/draft-release.json', 'utf8'),
);
if (!snapshot?.verified)
  throw new Error('A finalized draft snapshot is required');
const release = JSON.parse(
  execFileSync(
    'gh',
    [
      'release',
      'view',
      snapshot.tag,
      '--repo',
      'tanzir71/instancedeck',
      '--json',
      'isDraft',
    ],
    { encoding: 'utf8', windowsHide: true },
  ),
);
if (!release.isDraft)
  throw new Error('This audit is for a currently unpublished draft');
const live = process.argv.includes('--live');
const base = live
  ? 'https://tanzir71.github.io/instancedeck/'
  : 'http://127.0.0.1:4322/instancedeck/';
const browser = await chromium.launch({ headless: true });
const observations = [];
try {
  const context = await browser.newContext();
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (error) => errors.push(String(error)));
  for (const width of [1440, 390]) {
    await page.setViewportSize({ width, height: 1000 });
    for (const route of ['download', 'changelog']) {
      const url = `${base}${route}/`;
      const response = await page.goto(url, { waitUntil: 'networkidle' });
      if (response?.status() !== 200)
        throw new Error(`Draft route failed: ${url}`);
      await page.evaluate(() => document.fonts.ready);
      const notice = page.getByRole('heading', {
        name: `Draft ${snapshot.tag} is ready for manual verification`,
      });
      if ((await notice.count()) !== 1)
        throw new Error('Real draft readiness is missing');
      if (
        !(await page
          .getByRole('link', { name: 'View automated build', exact: true })
          .getAttribute('href')
          .then((value) => value === snapshot.runUrl))
      )
        throw new Error('Build provenance link mismatch');
      if (
        await page
          .locator(`a[href*="/releases/download/${snapshot.tag}/"]`)
          .count()
      )
        throw new Error('Private draft asset links must not be advertised');
      if (
        (await page.evaluate(() => document.documentElement.scrollWidth)) >
        width
      )
        throw new Error('Draft page overflows');
      const violations = (await new AxeBuilder({ page }).analyze()).violations;
      if (violations.length)
        throw new Error(
          `Draft accessibility: ${violations.map((v) => v.id).join(', ')}`,
        );
      await page.screenshot({
        path: `artifacts/p7/${live ? 'live' : 'local'}-draft-${route}-${width}.png`,
        fullPage: true,
      });
      observations.push({
        url,
        width,
        status: response.status(),
        axeViolations: 0,
        privateAssetLinks: 0,
      });
    }
  }
  if (errors.length)
    throw new Error(`Draft browser errors: ${errors.join(', ')}`);
  await writeFile(
    `artifacts/p7/${live ? 'live' : 'local'}-draft-site.json`,
    JSON.stringify(
      {
        checkedAt: new Date().toISOString(),
        tag: snapshot.tag,
        commit: snapshot.commit,
        runUrl: snapshot.runUrl,
        observations,
        errors,
      },
      null,
      2,
    ) + '\n',
  );
  console.log(
    `Verified actual ${snapshot.tag} draft notice on Download and Changelog at desktop/mobile widths.`,
  );
} finally {
  await browser.close();
}
