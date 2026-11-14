import { test, expect } from '@playwright/test';

test('switcher filters and selects a fixture using only the keyboard', async ({
  page,
}) => {
  await page.setViewportSize({ width: 640, height: 420 });
  await page.goto(
    'http://127.0.0.1:6006/iframe.html?id=desktop-global-switcher--ready&viewMode=story',
  );
  const input = page.getByRole('combobox', { name: 'Find a profile' });
  await expect(input).toBeFocused();
  await expect(page.getByRole('option')).toHaveCount(5);
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({ path: 'artifacts/p4/switcher.png' });
  await page.keyboard.type('Research');
  await expect(page.getByRole('option')).toHaveCount(1);
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('Enter');
  await expect(
    page.getByRole('heading', { name: 'Simulated palette closed' }),
  ).toBeVisible();
});
test('Escape can dismiss the palette while a fixture launch remains pending', async ({
  page,
}) => {
  await page.setViewportSize({ width: 640, height: 420 });
  await page.goto(
    'http://127.0.0.1:6006/iframe.html?id=desktop-global-switcher--busy&viewMode=story',
  );
  await expect(page.getByRole('combobox')).toHaveAttribute('readonly');
  await page.keyboard.press('Escape');
  await expect(
    page.getByRole('heading', { name: 'Simulated palette closed' }),
  ).toBeVisible();
});
test('shell options and registration failures have reviewable browser captures', async ({
  page,
}) => {
  for (const name of [
    'wizard-options-configured',
    'settings-hotkey-conflict',
    'profile-shortcut-error',
  ]) {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto(
      `http://127.0.0.1:6006/iframe.html?id=desktop-screens--${name}&viewMode=story`,
    );
    if (name === 'wizard-options-configured') {
      await expect(
        page.getByRole('textbox', { name: 'Profile hotkey' }),
      ).toHaveValue('Ctrl+Alt+1');
      await expect(
        page.getByRole('checkbox', { name: 'Desktop shortcut' }),
      ).toBeChecked();
    } else {
      await expect(page.getByRole('alert')).toBeVisible();
    }
    if (name === 'profile-shortcut-error')
      await page.getByRole('alert').scrollIntoViewIfNeeded();
    await page.evaluate(() => document.fonts.ready);
    await page.screenshot({ path: `artifacts/p4/${name}.png`, fullPage: true });
  }
});
