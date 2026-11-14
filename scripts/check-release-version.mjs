import { readFile } from 'node:fs/promises';
const tag = process.env.RELEASE_TAG;
if (!/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(tag ?? ''))
  throw new Error(
    'Release tag must be vMAJOR.MINOR.PATCH, optionally a prerelease',
  );
const version = tag.slice(1);
for (const path of [
  'package.json',
  'apps/desktop/package.json',
  'apps/site/package.json',
  'packages/ui/package.json',
  'packages/recipes/package.json',
  'apps/desktop/src-tauri/tauri.conf.json',
]) {
  const metadata = JSON.parse(await readFile(path, 'utf8'));
  if (metadata.version !== version)
    throw new Error(`Version mismatch: ${path}`);
}
const cargo = await readFile('Cargo.toml', 'utf8');
if (
  cargo.match(/\[workspace.package\][\s\S]*?version\s*=\s*"([^"]+)"/)?.[1] !==
  version
)
  throw new Error('Cargo workspace version mismatch');
const installer = await readFile(
  'apps/desktop/src-tauri/windows/installer.nsi',
  'utf8',
);
if (
  /DeleteAppDataCheckbox|RmDir\s+\/r\s+"\$(?:APPDATA|LOCALAPPDATA)/i.test(
    installer,
  )
)
  throw new Error('Installer must preserve all app data');
const config = JSON.parse(
  await readFile('apps/desktop/src-tauri/tauri.conf.json', 'utf8'),
);
if (
  config.bundle.windows.nsis.installMode !== 'currentUser' ||
  config.bundle.windows.webviewInstallMode.type !== 'skip' ||
  !config.plugins.updater.requireSignedVersion
)
  throw new Error(
    'Release must be per-user, offline at installation, and require signed updater versions',
  );
console.log(`Release ${tag}: package versions and preservation policy agree.`);
