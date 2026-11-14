import { cp, readdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, relative, dirname, join } from 'node:path';
const dist = resolve('apps/site/dist');
const library = join(dist, 'storybook');
await cp(resolve('packages/ui/storybook-static'), library, { recursive: true });
const home = await readFile(join(dist, 'index.html'), 'utf8');
const footer = home.match(/<footer\b[\s\S]*?<\/footer>/)?.[0];
const styles = Array.from(
  home.matchAll(/<link rel="stylesheet" href="([^"]+)"[^>]*>/g),
  (match) => match[1],
);
if (!footer || !styles.length)
  throw new Error(
    'Cannot package Storybook without the shared site footer and styles',
  );
async function* files(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) yield* files(path);
    else yield path;
  }
}
let count = 0;
for await (const path of files(library)) {
  if (path.endsWith('.css')) {
    // The third-party Storybook manager's CSS follows the product's flat style.
    const css = await readFile(path, 'utf8');
    await writeFile(
      path,
      css.replace(/(?<![-\w])box-shadow\s*:[^;}]+;?/gi, ''),
    );
  }
  if (!path.endsWith('.html')) continue;
  const prefix = relative(dirname(path), dist).replaceAll('\\', '/') + '/';
  let html = await readFile(path, 'utf8');
  html = html.replace(/<meta\s+name="robots"[^>]*>/gi, '');
  const stylesheetLinks = styles
    .map(
      (href) =>
        `<link rel="stylesheet" href="${prefix}${href.split('/_astro/')[1] ? `_astro/${href.split('/_astro/')[1]}` : href}" />`,
    )
    .join('');
  html = html.replace(
    '</head>',
    `<meta name="robots" content="noindex, nofollow" />${stylesheetLinks}${path.endsWith('index.html') ? '<style>html,body{height:auto!important;max-height:none!important;min-height:100%;overflow:visible!important}body{position:static!important}body>#root{height:100vh;min-height:100vh;contain:layout;overflow:hidden}</style>' : ''}</head>`,
  );
  html = html.replace(
    '</body>',
    `${footer.replaceAll('href="./', `href="${prefix}`)}</body>`,
  );
  if (
    !html.includes('https://tanziro.com') ||
    !html.includes('It is not affiliated with')
  )
    throw new Error('Missing packaged footer');
  await writeFile(path, html);
  count++;
}
console.log(
  `Packaged Storybook with noindex and the shared footer on ${count} HTML pages.`,
);
let htmlPages = 0;
let cssFiles = 0;
for await (const path of files(dist)) {
  if (path.endsWith('.html')) {
    const html = await readFile(path, 'utf8');
    if (
      !html.includes('It is not affiliated with') ||
      !html.includes('https://tanziro.com')
    )
      throw new Error(`Missing footer: ${path}`);
    if (/<a\b[^>]*href="\//i.test(html))
      throw new Error(`Absolute-root link: ${path}`);
    htmlPages++;
  }
  if (path.endsWith('.css')) {
    if (/box-shadow\s*:/i.test(await readFile(path, 'utf8')))
      throw new Error(`Forbidden shadow declaration: ${path}`);
    cssFiles++;
  }
}
await writeFile(
  resolve('artifacts/p6/packaged-site-audit.json'),
  JSON.stringify(
    {
      htmlPages,
      cssFiles,
      footerPresent: true,
      absoluteRootLinks: 0,
      shadowDeclarations: 0,
    },
    null,
    2,
  ) + '\n',
);
