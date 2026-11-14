import { test, expect } from '@playwright/test';
test('update checking is gated by first-run and saved preference', async ({
  page,
}) => {
  for (const story of ['disabled', 'before-first-run']) {
    await page.goto(
      `http://127.0.0.1:6006/iframe.html?id=desktop-updates--${story}&viewMode=story`,
    );
    await expect(
      page.getByRole('button', { name: 'Check for updates' }),
    ).toBeDisabled();
    await expect(page.getByTestId('fixture-checks')).toHaveText('0');
  }
});
test('Strict Mode performs one automatic check and download needs its own confirmation', async ({
  page,
}) => {
  await page.goto(
    'http://127.0.0.1:6006/iframe.html?id=desktop-updates--available&viewMode=story',
  );
  await expect(page.getByRole('status')).toContainText('0.2.0');
  await expect(page.getByTestId('fixture-checks')).toHaveText('1');
  await page.getByRole('button', { name: 'Review update' }).click();
  await expect(page.getByTestId('fixture-installs')).toHaveText('0');
  await expect(page.getByRole('dialog')).toContainText('installer is unsigned');
  await page.screenshot({
    path: 'artifacts/p7/update-confirmation.png',
    fullPage: true,
  });
  await page.getByRole('button', { name: 'Cancel', exact: true }).click();
  await expect(page.getByTestId('fixture-installs')).toHaveText('0');
  await expect(page.getByRole('dialog')).toHaveCount(0);
});
test('integrity failure and portable update states are explicit', async ({
  page,
}) => {
  for (const story of ['integrity-error', 'portable', 'downloading', 'dark']) {
    await page.goto(
      `http://127.0.0.1:6006/iframe.html?id=desktop-updates--${story}&viewMode=story`,
    );
    if (story === 'integrity-error')
      await expect(page.getByRole('dialog').getByRole('alert')).toContainText(
        'Nothing was installed',
      );
    if (story === 'portable') {
      await expect(
        page.getByRole('button', { name: 'Review update' }),
      ).toHaveCount(0);
      await expect(
        page.getByRole('button', { name: 'Open download page' }),
      ).toBeVisible();
    }
    if (story === 'downloading')
      await expect(page.getByRole('dialog').getByRole('status')).toContainText(
        'Downloaded 2.0 MiB',
      );
    if (story === 'dark')
      await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    await page.screenshot({
      path: `artifacts/p7/update-${story}.png`,
      fullPage: true,
    });
  }
});
