/* global module */
module.exports = {
  ci: {
    collect: {
      startServerCommand:
        'pnpm exec astro preview --host 127.0.0.1 --port 4323 --ignore-lock',
      startServerReadyPattern: '4323',
      startServerReadyTimeout: 30000,
      url: [
        'http://127.0.0.1:4323/instancedeck/',
        'http://127.0.0.1:4323/instancedeck/apps/claude-desktop/',
        'http://127.0.0.1:4323/instancedeck/faq/',
      ],
      numberOfRuns: 3,
      settings: {
        chromeFlags: '--headless --no-sandbox',
        onlyCategories: [
          'performance',
          'accessibility',
          'best-practices',
          'seo',
        ],
      },
    },
    assert: {
      assertions: {
        'categories:performance': ['error', { minScore: 0.95 }],
        'categories:accessibility': ['error', { minScore: 0.95 }],
        'categories:best-practices': ['error', { minScore: 0.95 }],
        'categories:seo': ['error', { minScore: 0.95 }],
        'largest-contentful-paint': ['error', { maxNumericValue: 1800 }],
        'cumulative-layout-shift': ['error', { maxNumericValue: 0.05 }],
      },
    },
    upload: {
      target: 'filesystem',
      outputDir: '../../artifacts/p6/lighthouse',
    },
  },
};
