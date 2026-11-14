import { test, expect } from '@playwright/test';
test('custom editor previews arguments and environment-only isolation without launching', async ({
  page,
}) => {
  await page.goto(
    'http://127.0.0.1:6006/iframe.html?id=desktop-power-features--custom-environment&viewMode=story',
  );
  await expect(page.getByRole('dialog').locator('pre')).toContainText(
    'APP_HOME',
  );
  await expect(page.getByRole('dialog').locator('pre')).not.toContainText(
    '{profile}',
  );
  await expect(page.getByLabel('Profile match argument')).toHaveValue('');
  await page.screenshot({
    path: 'artifacts/p5/custom-environment.png',
    fullPage: true,
  });
  await page
    .getByRole('button', { name: 'Save custom app', exact: true })
    .click();
  await expect(
    page.getByRole('heading', { name: 'Custom recipe saved in memory' }),
  ).toBeVisible();
});
test('metadata import has a review step and preserves errors for correction', async ({
  page,
}) => {
  await page.goto(
    'http://127.0.0.1:6006/iframe.html?id=desktop-power-features--import-error&viewMode=story',
  );
  await page
    .getByRole('button', { name: 'Import profiles', exact: true })
    .click();
  await expect(page.getByRole('dialog')).toContainText(
    '2 profiles will be added',
  );
  await page
    .getByRole('button', { name: 'Import metadata', exact: true })
    .click();
  await expect(page.getByRole('dialog').getByRole('alert')).toContainText(
    'No metadata was changed',
  );
  await page.screenshot({
    path: 'artifacts/p5/import-review-error.png',
    fullPage: true,
  });
});
test('shared history is opt-in and can return to private sessions', async ({
  page,
}) => {
  await page.goto(
    'http://127.0.0.1:6006/iframe.html?id=desktop-power-features--private-history&viewMode=story',
  );
  const enable = page.getByRole('button', { name: 'Enable or resume sharing' });
  await expect(enable).toBeDisabled();
  await page.getByRole('checkbox').check();
  await enable.click();
  await expect(page.getByRole('status')).toHaveText(
    'Sessions sharing is enabled.',
  );
  await page.screenshot({
    path: 'artifacts/p5/shared-sessions.png',
    fullPage: true,
  });
  await page.getByRole('button', { name: 'Restore private history' }).click();
  await expect(page.getByRole('status')).toHaveText(
    'Private sessions restored.',
  );
});
test('dark theme covers deck, modal portals and settings', async ({ page }) => {
  for (const name of [
    'dark-deck',
    'dark-wizard',
    'dark-settings',
    'custom-arguments',
    'custom-invalid',
  ]) {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto(
      `http://127.0.0.1:6006/iframe.html?id=desktop-power-features--${name}&viewMode=story`,
    );
    if (name.startsWith('dark-'))
      await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    if (name === 'custom-arguments')
      await expect(page.getByRole('dialog').locator('pre')).toBeVisible();
    if (name === 'custom-invalid')
      await expect(page.getByRole('status')).toContainText('Match argument');
    await page.evaluate(() => document.fonts.ready);
    await page.screenshot({ path: `artifacts/p5/${name}.png`, fullPage: true });
  }
});
