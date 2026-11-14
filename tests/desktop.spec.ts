import { test, expect } from '@playwright/test';

test('desktop views render local assets and capture Figma comparison evidence', async ({
  page,
}) => {
  test.setTimeout(120000);
  for (const name of [
    'deck',
    'wizard-name',
    'sign-in',
    'profile',
    'apps',
    'diagnostics',
    'settings',
    'empty',
    'loading',
    'error',
    'recovery',
    'first-run',
  ]) {
    await page.goto(
      `http://127.0.0.1:6006/iframe.html?id=desktop-screens--${name}&viewMode=story`,
    );
    await expect(page.locator('#storybook-root')).not.toBeEmpty();
    await page.evaluate(() => document.fonts.ready);
    if (
      ['wizard-name', 'sign-in', 'profile', 'recovery', 'first-run'].includes(
        name,
      )
    )
      await expect(page.getByRole('dialog')).toBeVisible();
    else await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
    const invalid = await page
      .locator('img')
      .evaluateAll((images) =>
        images
          .filter((image) => !image.complete || image.naturalWidth === 0)
          .map((image) => image.src),
      );
    expect(invalid).toEqual([]);
    await page.screenshot({ path: `artifacts/p2/${name}.png`, fullPage: true });
  }
});

test('keyboard creates a profile, launches it, closes it and keeps confirmation guarded', async ({
  page,
}) => {
  await page.goto(
    'http://127.0.0.1:6006/iframe.html?id=desktop-screens--interactive&viewMode=story',
  );
  const newButton = page
    .getByRole('button', { name: 'New profile', exact: true })
    .first();
  await expect(newButton).toBeVisible();
  // Reach each target through Tab/Shift+Tab; all activation and entry use the keyboard.
  const tabTo = async (target: import('@playwright/test').Locator) => {
    for (let attempt = 0; attempt < 100; attempt++) {
      if (
        await target.evaluate((element) => element === document.activeElement)
      )
        return;
      await page.keyboard.press('Tab');
    }
    throw new Error('Keyboard target could not be reached');
  };
  await tabTo(newButton);
  await page.keyboard.press('Enter');
  const dialog = page.getByRole('dialog');
  await tabTo(dialog.getByRole('button', { name: 'Continue', exact: true }));
  await page.keyboard.press('Enter');
  const nameInput = dialog.getByRole('textbox', { name: 'Profile name' });
  await tabTo(nameInput);
  await page.keyboard.type('Keyboard QA Zoë');
  const previewPath = await dialog
    .getByRole('textbox', { name: 'Data location' })
    .inputValue();
  await tabTo(dialog.getByRole('button', { name: 'Continue', exact: true }));
  await page.keyboard.press('Enter');
  await tabTo(
    dialog.getByRole('button', { name: 'Create profile', exact: true }),
  );
  await page.keyboard.press('Enter');
  await expect(dialog).toBeHidden();
  const launch = page.getByRole('button', {
    name: 'Launch Keyboard QA Zoë',
    exact: true,
  });
  await tabTo(launch);
  await page.keyboard.press('Enter');
  const close = page.getByRole('button', {
    name: 'Close Keyboard QA Zoë',
    exact: true,
  });
  await expect(close).toBeVisible();
  await tabTo(close);
  await page.keyboard.press('Enter');
  await expect(launch).toBeVisible();
  await tabTo(
    page.getByRole('button', {
      name: 'Profile details: Keyboard QA Zoë',
      exact: true,
    }),
  );
  await page.keyboard.press('Enter');
  await expect(
    page.getByRole('dialog', { name: 'Profile details' }),
  ).toContainText(previewPath);
  await tabTo(
    page.getByRole('button', { name: 'Delete profile data', exact: true }),
  );
  await page.keyboard.press('Enter');
  const confirm = page.getByRole('dialog', { name: 'Delete profile data' });
  await expect(
    confirm.getByRole('button', { name: 'Delete profile data', exact: true }),
  ).toBeDisabled();
  await page.keyboard.press('Escape');
  await expect(confirm).toBeHidden();
  await page.screenshot({
    path: 'artifacts/p2/keyboard-flow.png',
    fullPage: true,
  });
});
