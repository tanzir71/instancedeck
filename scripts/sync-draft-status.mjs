import { execFileSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
const tag = process.argv[2];
if (!/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(tag ?? ''))
  throw new Error('Invalid draft tag');
const proof = JSON.parse(
  await readFile('artifacts/p7/release-verification.json', 'utf8'),
);
if (
  proof.tag !== tag ||
  !proof.installerSignaturesVerified ||
  !proof.feedSignatureVerified ||
  !proof.signedVersionMismatchRejected ||
  !proof.tamperedOwnInstallerRejected ||
  proof.privateKeyTransmitted ||
  proof.installerExecuted
)
  throw new Error('Successful local verification evidence is required');
const gh = (args) =>
  execFileSync('gh', args, { encoding: 'utf8', windowsHide: true });
const release = JSON.parse(
  gh([
    'release',
    'view',
    tag,
    '--repo',
    'tanzir71/instancedeck',
    '--json',
    'isDraft,assets,createdAt,body',
  ]),
);
if (!release.isDraft) throw new Error('Only draft readiness is recorded here');
const expected = [
  'build-x64.json',
  'build-arm64.json',
  'SHA256SUMS.txt',
  'latest.json',
  'latest.json.sig',
];
for (const arch of ['x64', 'arm64'])
  for (const suffix of ['setup.exe', 'setup.exe.sig', 'portable.zip'])
    expected.push(`InstanceDeck_${tag.slice(1)}_${arch}-${suffix}`);
const names = release.assets.map((a) => a.name);
if (expected.some((name) => !names.includes(name)))
  throw new Error('Finalized draft artifacts are incomplete');
const runs = JSON.parse(
  gh([
    'run',
    'list',
    '--repo',
    'tanzir71/instancedeck',
    '--workflow',
    'release.yml',
    '--commit',
    proof.commit,
    '--event',
    'push',
    '--json',
    'databaseId,conclusion',
  ]),
);
const run = runs.find((r) => r.conclusion === 'success');
if (!run) throw new Error('The exact tagged release workflow must pass');
await writeFile(
  'apps/site/src/data/draft-release.json',
  JSON.stringify(
    {
      tag,
      version: tag.slice(1),
      createdAt: release.createdAt,
      commit: proof.commit,
      runUrl: `https://github.com/tanzir71/instancedeck/actions/runs/${run.databaseId}`,
      notes: release.body,
      assetNames: names.sort(),
      verified: true,
    },
    null,
    2,
  ) + '\n',
);
console.log(
  `Recorded real ${tag} draft readiness. Private asset download URLs are not published.`,
);
