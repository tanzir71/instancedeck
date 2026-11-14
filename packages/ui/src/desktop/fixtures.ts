import type { AppDetection, DesktopSnapshot, Profile } from './model';
import { emptyPreviewSnapshot } from './previewClient';

// Figma reference data, used exclusively by Storybook and browser verification.
const app = (id: string, name: string, version: string): AppDetection => ({
  recipe: {
    id,
    name,
    vendor: id === 'codex' ? 'OpenAI' : 'Anthropic',
    status: 'community',
    limitations: [
      id === 'codex'
        ? 'Confirm each profile shows the expected account.'
        : 'Cowork VM sharing is not supported.',
    ],
    signIn: { strategy: id === 'codex' ? 'in-profile' : 'snapshot' },
  },
  installed: {
    version,
    installType: 'msix',
    executable: `C:\\Official apps\\${id}\\app.exe`,
    installDir: `C:\\Official apps\\${id}`,
  },
  error: null,
});
export const fixtureApps = [
  app('claude-desktop', 'Claude Desktop', '3.1623.1.0'),
  app('codex', 'Codex', '26.1027.3384.0'),
];
export const fixtureProfiles: Profile[] = [
  ['work', 'claude-desktop', 'Work', 'iris'],
  ['client', 'claude-desktop', 'Client — Acme', 'teal'],
  ['personal', 'claude-desktop', 'Personal', 'terracotta'],
  ['research', 'codex', 'Research', 'plum'],
  ['qa', 'codex', 'QA', 'ochre'],
].map(([id = '', appId = '', name = '', color = 'iris']) => ({
  id,
  appId,
  name,
  color: color as Profile['color'],
  badge: 'monogram',
  dataDir: `D:\\Profiles\\${appId}\\${id}`,
  options: { isolateClaudeCode: true, sharedCodexHistory: false },
  hotkey: null,
  createdAt: '2026-11-14T08:00:00Z',
  lastLaunchedAt:
    id === 'personal'
      ? '2026-11-13T12:00:00Z'
      : id === 'research'
        ? '2026-11-12T12:00:00Z'
        : null,
  signInState: 'signed-in',
}));
export const fixtureSnapshot: DesktopSnapshot = {
  ...structuredClone(emptyPreviewSnapshot),
  apps: fixtureApps,
  profiles: fixtureProfiles,
  running: { work: [10], client: [11], qa: [12] },
  runningSince: {
    work: Date.parse('2026-11-14T09:00:00Z'),
    client: Date.parse('2026-11-14T11:20:00Z'),
    qa: Date.parse('2026-11-14T11:48:00Z'),
  },
  mainRunning: { 'claude-desktop': [20], codex: [] },
  settings: { ...emptyPreviewSnapshot.settings, profilesRoot: 'D:\\Profiles' },
};
