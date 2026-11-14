import test from 'node:test';
import assert from 'node:assert/strict';
import { publicReleases } from '../apps/site/src/lib/releases.ts';
const published = {
  draft: false,
  tag_name: 'v0.1.0',
  name: null,
  html_url: 'https://github.com/tanzir71/instancedeck/releases/tag/v0.1.0',
  published_at: '2026-11-14T00:00:00Z',
  prerelease: false,
  body: 'Synthetic public release fixture.',
  assets: [
    {
      name: 'fixture.zip',
      size: 4,
      browser_download_url:
        'https://github.com/tanzir71/instancedeck/releases/download/v0.1.0/fixture.zip',
    },
  ],
};
test('authenticated metadata excludes drafts and internal feed aliases before page data is constructed', () => {
  const result = publicReleases([
    {
      draft: true,
      body: 'Private draft fixture that must never enter page data.',
      assets: [{ browser_download_url: 'invalid-private-draft-link' }],
    },
    { ...published, tag_name: 'updater-beta' },
    published,
  ]);
  assert.equal(result.length, 1);
  assert.equal(result[0].name, 'v0.1.0');
  assert.equal(JSON.stringify(result).includes('Private draft fixture'), false);
  assert.equal(
    JSON.stringify(result).includes('invalid-private-draft-link'),
    false,
  );
});
test('public download URLs must remain in the project release namespace', () => {
  assert.throws(
    () =>
      publicReleases([
        {
          ...published,
          assets: [
            {
              ...published.assets[0],
              browser_download_url: 'https://example.invalid/installer.exe',
            },
          ],
        },
      ]),
    /Unexpected public release asset/,
  );
  assert.throws(
    () =>
      publicReleases([
        { ...published, html_url: 'https://example.invalid/release' },
      ]),
    /Unexpected public release metadata/,
  );
});
test('malformed release responses fail instead of fabricating empty release data', () => {
  assert.throws(
    () => publicReleases({ message: 'API failure fixture' }),
    /not a list/,
  );
  assert.throws(
    () => publicReleases([{ ...published, published_at: null }]),
    /Unexpected public release metadata/,
  );
});
