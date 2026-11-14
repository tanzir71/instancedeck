import { execFileSync } from 'node:child_process';
import { mkdtemp, readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
const tag = process.env.RELEASE_TAG;
if (!/^v\d+\.\d+\.\d+-[0-9A-Za-z.-]+$/.test(tag ?? ''))
  throw new Error('Beta alias requires an actual versioned prerelease');
const gh = (args) =>
  execFileSync('gh', args, {
    encoding: 'utf8',
    windowsHide: true,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
const release = JSON.parse(
  gh([
    'release',
    'view',
    tag,
    '--repo',
    'tanzir71/instancedeck',
    '--json',
    'isDraft,isPrerelease,assets',
  ]),
);
if (
  release.isDraft ||
  !release.isPrerelease ||
  !release.assets.some((a) => a.name === 'latest.json.sig')
)
  throw new Error(
    'Only published, finalized prereleases can update the beta alias',
  );
const directory = await mkdtemp(join(tmpdir(), 'instancedeck-beta-feed-'));
gh([
  'release',
  'download',
  tag,
  '--repo',
  'tanzir71/instancedeck',
  '--pattern',
  'latest.json',
  '--pattern',
  'latest.json.sig',
  '--dir',
  directory,
]);
const feed = JSON.parse(await readFile(join(directory, 'latest.json'), 'utf8'));
if (
  feed.version !== tag.slice(1) ||
  Object.keys(feed.platforms ?? {})
    .sort()
    .join(',') !== 'windows-aarch64,windows-x86_64' ||
  Object.values(feed.platforms).some(
    (p) =>
      !p.url.startsWith(
        `https://github.com/tanzir71/instancedeck/releases/download/${tag}/InstanceDeck_`,
      ) ||
      !p.url.endsWith('-setup.exe') ||
      typeof p.signature !== 'string' ||
      !p.signature,
  )
)
  throw new Error('Invalid published beta feed metadata');
let exists = false;
try {
  gh(['release', 'view', 'updater-beta', '--repo', 'tanzir71/instancedeck']);
  exists = true;
} catch {
  /* Creation reports API errors. */
}
if (!exists)
  gh([
    'release',
    'create',
    'updater-beta',
    '--repo',
    'tanzir71/instancedeck',
    '--target',
    tag,
    '--prerelease',
    '--latest=false',
    '--title',
    'Beta updater feed index',
    '--notes',
    'Feed index only. Versioned packages and provenance remain on their original prereleases.',
  ]);
gh([
  'release',
  'upload',
  'updater-beta',
  '--repo',
  'tanzir71/instancedeck',
  join(directory, 'latest.json'),
  join(directory, 'latest.json.sig'),
  '--clobber',
]);
console.log(
  `Beta feed now points to ${tag}. Artifact/version integrity remains enforced by the client.`,
);
