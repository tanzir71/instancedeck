import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  workers: 1,
  reporter: [['list'], ['html', { open: 'never' }]],
  use: { browserName: 'chromium', viewport: { width: 1280, height: 800 } },
  webServer: [
    {
      command: 'pnpm --filter @instancedeck/desktop dev:web',
      url: 'http://127.0.0.1:1420',
      reuseExistingServer: !process.env.CI,
    },
    {
      command:
        'pnpm --filter @instancedeck/site build && pnpm --filter @instancedeck/site exec astro preview --host 127.0.0.1 --port 4322 --ignore-lock',
      url: 'http://127.0.0.1:4322/instancedeck/',
      reuseExistingServer: !process.env.CI,
      timeout: 120000,
    },
    {
      command:
        'pnpm --filter @instancedeck/ui exec storybook dev -p 6006 --host 127.0.0.1 --ci --disable-telemetry',
      url: 'http://127.0.0.1:6006',
      reuseExistingServer: !process.env.CI,
      timeout: 120000,
    },
  ],
});
