const configuredBase = import.meta.env.BASE_URL.replace(/\/+$/, '');
export const basePath = configuredBase === '/' ? '' : configuredBase;
export const siteOrigin = import.meta.env.SITE ?? 'https://tanzir71.github.io';
export const sourceUrl = 'https://github.com/tanzir71/instancedeck';
export function absoluteUrl(path = '') {
  return new URL(`${basePath}/${path.replace(/^\/+/, '')}`, siteOrigin).href;
}
export function relativePrefix(pathname: string) {
  const path = pathname.startsWith(`${basePath}/`)
    ? pathname.slice(basePath.length + 1)
    : pathname.replace(/^\/+/, '');
  const segments = path.split('/').filter(Boolean);
  const depth = path.endsWith('/')
    ? segments.length
    : Math.max(0, segments.length - 1);
  return depth ? '../'.repeat(depth) : './';
}
