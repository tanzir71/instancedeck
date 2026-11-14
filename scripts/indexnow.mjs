import { readFile } from 'node:fs/promises';
const origin = process.env.SITE_URL ?? 'https://tanzir71.github.io';
const base = (process.env.BASE_PATH ?? '/instancedeck').replace(
  /^\/+|\/+$/g,
  '',
);
const prefix = new URL(
  `${base ? `${base}/` : ''}`,
  `${origin.replace(/\/+$/, '')}/`,
).href;
const key = (
  await readFile(
    new URL('../apps/site/public/indexnow-key.txt', import.meta.url),
    'utf8',
  )
).trim();
if (!/^[a-zA-Z0-9-]{8,128}$/.test(key))
  throw new Error('Invalid public IndexNow verification key');
const keyLocation = `${prefix}indexnow-key.txt`;
const publishedKey = await fetch(keyLocation, {
  signal: AbortSignal.timeout(15000),
});
if (!publishedKey.ok || (await publishedKey.text()).trim() !== key)
  throw new Error('The deployed IndexNow key does not match');
const sitemapResponse = await fetch(`${prefix}sitemap-0.xml`, {
  signal: AbortSignal.timeout(15000),
});
if (!sitemapResponse.ok)
  throw new Error(`Published sitemap returned ${sitemapResponse.status}`);
const sitemap = await sitemapResponse.text();
const urlList = Array.from(
  sitemap.matchAll(/<loc>([^<]+)<\/loc>/g),
  (match) => match[1],
);
if (!urlList.length || urlList.some((url) => !url.startsWith(prefix)))
  throw new Error('Sitemap URL escaped the project base');
const response = await fetch('https://api.indexnow.org/indexnow', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json; charset=utf-8' },
  body: JSON.stringify({
    host: new URL(origin).hostname,
    key,
    keyLocation,
    urlList,
  }),
  signal: AbortSignal.timeout(15000),
});
if (![200, 202].includes(response.status))
  throw new Error(`IndexNow returned ${response.status}`);
console.log(
  `IndexNow accepted ${urlList.length} URLs (${response.status}). This is receipt, not an indexing guarantee.`,
);
