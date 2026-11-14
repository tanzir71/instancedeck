import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('mobile site navigation traps focus, closes with Escape and returns focus', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(
    'http://127.0.0.1:6006/iframe.html?id=marketing-sections--header-mobile&viewMode=story',
  );
  const opener = page.getByRole('button', { name: 'Menu', exact: true });
  await opener.focus();
  await page.keyboard.press('Enter');
  const dialog = page.getByRole('dialog', { name: 'Site navigation' });
  await expect(dialog).toBeVisible();
  for (let index = 0; index < 12; index++) {
    await page.keyboard.press('Tab');
    expect(
      await dialog.evaluate((element) =>
        element.contains(document.activeElement),
      ),
    ).toBe(true);
  }
  expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([]);
  await page.keyboard.press('Escape');
  await expect(dialog).toBeHidden();
  await expect(opener).toBeFocused();
});

test('native FAQ details open and close from the keyboard', async ({
  page,
}) => {
  await page.goto(
    'http://127.0.0.1:6006/iframe.html?id=marketing-details--faq-closed&viewMode=story',
  );
  const summary = page.locator('summary');
  await summary.focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('details')).toHaveAttribute('open', '');
  await expect(page.getByText(/No\. InstanceDeck launches/)).toBeVisible();
  await page.keyboard.press('Space');
  await expect(page.locator('details')).not.toHaveAttribute('open');
});
