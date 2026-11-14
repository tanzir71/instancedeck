import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { writeFile } from 'node:fs/promises';
import { loadReleases } from '../apps/site/src/lib/releases';

test('published packages and release notes are visible and accessible on desktop and mobile', async ({
  page,
}) => {
  test.setTimeout(90000);
  const release = (await loadReleases()).find((item) => !item.prerelease);
  test.skip(!release, 'No public stable release exists yet');
  if (!release) return;
  const packages = release.assets.filter((asset) =>
    /(?:\.exe|\.zip)$/i.test(asset.name),
  );
  expect(packages.length).toBeGreaterThanOrEqual(4);
  const observations = [];
  for (const width of [1440, 390]) {
    await page.setViewportSize({ width, height: 1000 });
    for (const route of ['download', 'changelog']) {
      await page.goto(`http://127.0.0.1:4322/instancedeck/${route}/`);
      await page.evaluate(() => document.fonts.ready);
      if (route === 'download') {
        await expect(
          page.getByRole('heading', { name: release.tag_name, exact: true }),
        ).toBeVisible();
        for (const asset of packages)
          await expect(
            page.getByRole('link', { name: asset.name, exact: true }),
          ).toHaveAttribute('href', asset.browser_download_url);
      } else {
        await expect(
          page.locator(`h2 a[href="${release.html_url}"]`),
        ).toBeVisible();
        if (release.body)
          await expect(
            page
              .locator(`h2 a[href="${release.html_url}"]`)
              .locator('..')
              .locator('..')
              .locator('pre'),
          ).toContainText(release.body);
      }
      await expect(
        page.getByRole('heading', {
          name: `Draft ${release.tag_name} is ready for manual verification`,
          exact: true,
        }),
      ).toHaveCount(0);
      expect(
        await page.evaluate(() => document.documentElement.scrollWidth),
      ).toBeLessThanOrEqual(width);
      expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([]);
      await page.screenshot({
        path: `artifacts/p7/local-public-${route}-${width}.png`,
        fullPage: true,
      });
      observations.push({
        route,
        width,
        version: release.tag_name,
        axeViolations: 0,
      });
    }
  }
  await writeFile(
    'artifacts/p7/local-public-site.json',
    JSON.stringify(
      {
        release: release.tag_name,
        publicPackageLinks: packages.length,
        observations,
      },
      null,
      2,
    ) + '\n',
  );
});
