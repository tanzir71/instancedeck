// Build-time public metadata only. No browser API calls or target app data.
export interface ReleaseAsset {
  name: string;
  browser_download_url: string;
  size: number;
}
export interface SiteRelease {
  tag_name: string;
  name: string;
  html_url: string;
  published_at: string;
  body: string;
  prerelease: boolean;
  assets: ReleaseAsset[];
}
let pending: Promise<SiteRelease[]> | undefined;
export function loadReleases() {
  return (pending ??= fetchReleases());
}
async function fetchReleases(): Promise<SiteRelease[]> {
  const response = await fetch(
    'https://api.github.com/repos/tanzir71/instancedeck/releases?per_page=100',
    {
      headers: {
        Accept: 'application/vnd.github+json',
        'User-Agent': 'InstanceDeck-static-site',
      },
      signal: AbortSignal.timeout(15000),
    },
  );
  if (!response.ok)
    throw new Error(
      `Cannot build release pages: GitHub returned ${response.status}`,
    );
  const data: unknown = await response.json();
  if (!Array.isArray(data))
    throw new Error('GitHub releases response is not a list');
  return data
    .filter((entry) => !entry.draft)
    .map((entry) => {
      if (
        typeof entry.tag_name !== 'string' ||
        typeof entry.name !== 'string' ||
        typeof entry.published_at !== 'string' ||
        typeof entry.prerelease !== 'boolean' ||
        !Array.isArray(entry.assets) ||
        !trustedReleaseUrl(entry.html_url)
      )
        throw new Error('Unexpected public release metadata');
      const assets: ReleaseAsset[] = entry.assets.map((asset: ReleaseAsset) => {
        if (
          typeof asset.name !== 'string' ||
          !Number.isSafeInteger(asset.size) ||
          asset.size < 0 ||
          !trustedAssetUrl(asset.browser_download_url)
        )
          throw new Error('Unexpected public release asset');
        return {
          name: asset.name,
          size: asset.size,
          browser_download_url: asset.browser_download_url,
        };
      });
      return {
        tag_name: entry.tag_name,
        name: entry.name,
        html_url: entry.html_url,
        published_at: entry.published_at,
        prerelease: entry.prerelease,
        body: typeof entry.body === 'string' ? entry.body : '',
        assets,
      };
    });
}
function trustedReleaseUrl(value: unknown): value is string {
  return (
    typeof value === 'string' &&
    value.startsWith('https://github.com/tanzir71/instancedeck/releases/tag/')
  );
}
function trustedAssetUrl(value: unknown): value is string {
  return (
    typeof value === 'string' &&
    value.startsWith(
      'https://github.com/tanzir71/instancedeck/releases/download/',
    )
  );
}
