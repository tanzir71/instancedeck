import { readFile, writeFile, mkdir, readdir } from 'node:fs/promises';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import satori from 'satori';
import { Resvg } from '@resvg/resvg-js';

const root = dirname(
  fileURLToPath(new URL('../package.json', import.meta.url)),
);
const dist = join(root, 'dist');
const tokens = JSON.parse(
  await readFile(join(root, '../../design/tokens.json'), 'utf8'),
);
const colors = Object.fromEntries(
  Object.entries(tokens.color.ui).map(([key, value]) => [key, value.value]),
);
const profiles = Object.values(tokens.color.profile).map(
  (entry) => entry.value,
);
const logo = `data:image/svg+xml;base64,${(await readFile(join(root, '../../design/assets/logo-mark.svg'))).toString('base64')}`;
const fonts = [
  {
    name: 'Source Serif 4',
    data: await readFile(join(root, 'assets/og-fonts/serif.ttf')),
    weight: 400,
    style: 'normal',
  },
  {
    name: 'IBM Plex Sans',
    data: await readFile(join(root, 'assets/og-fonts/sans.ttf')),
    weight: 500,
    style: 'normal',
  },
];
function node(type, style, children) {
  return { type, props: { style, children } };
}
async function* htmlFiles(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (entry.name === 'storybook') continue;
    const path = join(directory, entry.name);
    if (entry.isDirectory()) yield* htmlFiles(path);
    else if (entry.name.endsWith('.html')) yield path;
  }
}
const directory = join(dist, 'og');
await mkdir(directory, { recursive: true });
let count = 0;
for await (const path of htmlFiles(dist)) {
  const html = await readFile(path, 'utf8');
  const title = html
    .match(/<title>([^<]+)<\/title>/)?.[1]
    .replaceAll('&amp;', '&')
    .replaceAll('&#39;', "'");
  const image = html.match(/<meta property="og:image" content="([^"]+)"/);
  if (!title || !image) throw new Error(`Missing OG metadata: ${path}`);
  const filename = new URL(image[1]).pathname.split('/').at(-1);
  if (!filename || !/^[a-z0-9-]+\.png$/.test(filename))
    throw new Error('Unsafe OG filename');
  const svg = await satori(
    node(
      'div',
      {
        display: 'flex',
        flexDirection: 'column',
        width: 1200,
        height: 630,
        padding: '64px',
        background: colors.paper,
        color: colors.ink,
        position: 'relative',
        overflow: 'hidden',
        fontFamily: 'IBM Plex Sans',
      },
      [
        node(
          'div',
          { display: 'flex', alignItems: 'center', gap: 16, fontSize: 28 },
          [
            { type: 'img', props: { src: logo, width: 36, height: 36 } },
            'InstanceDeck',
          ],
        ),
        node(
          'div',
          {
            display: 'flex',
            width: 850,
            marginTop: 60,
            fontFamily: 'Source Serif 4',
            fontSize: title.length > 58 ? 56 : 64,
            lineHeight: 1.06,
            letterSpacing: '-1px',
          },
          title
            .replace(/ [–—] InstanceDeck$/, '')
            .replace(/^InstanceDeck [–—] /, ''),
        ),
        node(
          'div',
          {
            display: 'flex',
            position: 'absolute',
            left: 64,
            top: 534,
            width: 850,
            height: 32,
            fontSize: 22,
            color: colors['ink-2'],
          },
          'Multi-instance app launcher for Windows',
        ),
        ...profiles.slice(0, 3).map((background, index) =>
          node(
            'div',
            {
              display: 'flex',
              position: 'absolute',
              left: 960 + index * 52,
              top: 250 + index * 45,
              width: 190,
              height: 270,
              background,
              transform: `rotate(${index * 7 - 7}deg)`,
            },
            '',
          ),
        ),
      ],
    ),
    { width: 1200, height: 630, fonts },
  );
  await writeFile(join(directory, filename), new Resvg(svg).render().asPng());
  count++;
}
console.log(`Generated ${count} flat 1200×630 Open Graph images.`);
