import { test, expect } from '@playwright/test';

test('sign-in and account guidance states have reviewable browser captures', async ({
  page,
}) => {
  for (const name of [
    'sign-in-consent',
    'sign-in-consent-error',
    'sign-in-metadata-hint',
    'codex-account-guidance',
    'account-guidance-error',
    'profile-mcp',
    'recovery-on-startup',
  ]) {
    await page.goto(
      `http://127.0.0.1:6006/iframe.html?id=desktop-screens--${name}&viewMode=story`,
    );
    await expect(page.getByRole('dialog')).toBeVisible();
    if (name === 'profile-mcp')
      await page
        .getByRole('combobox', { name: 'Source Claude profile' })
        .scrollIntoViewIfNeeded();
    if (name.endsWith('error'))
      await expect(page.getByRole('alert')).toBeVisible();
    await page.evaluate(() => document.fonts.ready);
    await page.screenshot({ path: `artifacts/p3/${name}.png`, fullPage: true });
  }
});

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
    if (name === 'wizard-name') {
      const bounds = await page.getByRole('dialog').boundingBox();
      expect(bounds?.width).toBe(812); // Figma: 500px form + 312px preview.
    }
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
  const signInConsent = page.getByRole('dialog', {
    name: 'Sign in now',
    exact: true,
  });
  await expect(signInConsent).toBeVisible();
  await page.keyboard.press('Escape');
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
  // Radix's focus handoff also establishes the top dialog's keyboard layer.
  // A visible portal alone can precede that handoff on a fast CI renderer.
  await expect(
    confirm.getByRole('textbox', { name: 'Type the profile name to confirm' }),
  ).toBeFocused();
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

test('startup recovery resumes a supplied journal and account mismatch stays a manual report', async ({
  page,
}) => {
  await page.goto(
    'http://127.0.0.1:6006/iframe.html?id=desktop-screens--recovery-on-startup&viewMode=story',
  );
  await expect(
    page.getByRole('dialog', {
      name: 'An interrupted sign-in needs attention',
    }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Resume', exact: true }).click();
  await expect(page.getByRole('dialog')).toContainText('Client — Acme');
  await page
    .getByRole('button', { name: 'I’ve signed in', exact: true })
    .click();
  await expect(page.getByRole('dialog')).toContainText(
    'This is your confirmation only',
  );
  await page
    .getByRole('button', { name: 'Wrong account — flag for review' })
    .click();
  await expect(page.getByRole('dialog')).toBeHidden();
  await page
    .getByRole('button', {
      name: 'Profile details: Client — Acme',
      exact: true,
    })
    .click();
  await expect(page.getByRole('dialog')).toContainText('Not signed in yet');
  await page.screenshot({
    path: 'artifacts/p3/manual-account-warning.png',
    fullPage: true,
  });
});
