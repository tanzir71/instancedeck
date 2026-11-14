import { readFile, readdir, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { join, resolve } from 'node:path';
const tag = process.env.RELEASE_TAG;
if (!/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(tag ?? ''))
  throw new Error('Invalid release tag');
const root = resolve('dist/release-artifacts');
const files = (await readdir(root))
  .filter(
    (name) =>
      /^InstanceDeck_.+_(?:x64|arm64)-(?:setup\.exe|portable\.zip)$/.test(
        name,
      ) || /^build-(?:x64|arm64)\.json$/.test(name),
  )
  .sort();
if (files.length !== 6)
  throw new Error(
    'Expected both installers, both portable archives and both build reports',
  );
const checksums = [];
const commit = execFileSync('git', ['rev-parse', `${tag}^{commit}`], {
  encoding: 'utf8',
  windowsHide: true,
}).trim();
for (const arch of ['x64', 'arm64']) {
  const report = JSON.parse(
    await readFile(join(root, `build-${arch}.json`), 'utf8'),
  );
  if (
    report.commit !== commit ||
    report.version !== tag.slice(1) ||
    report.authenticode !== 'unsigned'
  )
    throw new Error('Build reports must match the exact existing release tag');
}
for (const name of files)
  checksums.push(
    `${createHash('sha256')
      .update(await readFile(join(root, name)))
      .digest('hex')}  ${name}`,
  );
await writeFile(join(root, 'SHA256SUMS.txt'), checksums.join('\n') + '\n');
const gh = (args) =>
  execFileSync('gh', args, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
  });
let existing;
try {
  existing = JSON.parse(
    gh([
      'release',
      'view',
      tag,
      '--repo',
      'tanzir71/instancedeck',
      '--json',
      'isDraft',
    ]),
  );
} catch {
  /* Creation below reports any API/authentication failure. */
}
if (existing && !existing.isDraft)
  throw new Error('Refusing to modify a published release');
if (!existing)
  gh([
    'release',
    'create',
    tag,
    '--repo',
    'tanzir71/instancedeck',
    '--verify-tag',
    '--draft',
    '--title',
    `InstanceDeck ${tag} — unsigned preview`,
    '--notes-file',
    `docs/releases/${tag}.md`,
  ]);
gh([
  'release',
  'upload',
  tag,
  '--repo',
  'tanzir71/instancedeck',
  ...files.map((name) => join(root, name)),
  join(root, 'SHA256SUMS.txt'),
  '--clobber',
]);
console.log(
  `Created/updated ${tag} draft with x64/ARM64 packages and checksums. Local integrity finalization is required before publication.`,
);
