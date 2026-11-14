import type { DesktopClient, DesktopSnapshot } from './model';
import copy from '../copy/en.json';

export const emptyPreviewSnapshot: DesktopSnapshot = {
  profiles: [],
  apps: [],
  running: {},
  mainRunning: {},
  statusErrors: {},
  settings: {
    schemaVersion: 1,
    profilesRoot: 'C:\\Profiles',
    closeToTray: true,
    startWithWindows: false,
    switcherHotkey: 'Ctrl+Alt+Space',
    allowUpdateChecks: true,
    updateChannel: 'stable',
    theme: 'light',
    onboardingCompleted: true,
  },
};
/** In-memory browser preview; it cannot touch files or launch native processes. */
export function createPreviewClient(
  initial: DesktopSnapshot = emptyPreviewSnapshot,
  simulateProcesses = false,
): DesktopClient {
  let state = structuredClone(initial);
  const listeners = new Set<(state: DesktopSnapshot) => void>();
  const emit = () => {
    for (const listener of listeners) listener(structuredClone(state));
  };
  return {
    checkUpdates: async () => {
      if (!simulateProcesses) throw new Error(copy.desktop.previewBanner);
      return {
        status: 'unavailable',
        version: null,
        notes: null,
        portable: false,
        currentVersion: '0.1.0',
      };
    },
    installUpdate: async () => {
      throw new Error(copy.desktop.previewBanner);
    },
    updateProgress: async () => () => {},
    openExternal: async () => {
      throw new Error(copy.desktop.previewBanner);
    },
    sharedHistory: async (id, enabled) => {
      if (!simulateProcesses) throw new Error(copy.desktop.previewBanner);
      if (state.running[id]?.length)
        throw new Error('The profile is still running');
      state.profiles = state.profiles.map((p) =>
        p.id === id
          ? { ...p, options: { ...p.options, sharedCodexHistory: enabled } }
          : p,
      );
      emit();
    },
    addCustom: async (input) => {
      if (!simulateProcesses) throw new Error(copy.desktop.previewBanner);
      if (state.apps.some((a) => a.recipe.id === input.id))
        throw new Error('Custom app id already exists');
      state.apps.push({
        recipe: {
          id: input.id,
          name: input.name,
          vendor: 'User configured',
          status: 'experimental',
          limitations: ['Unverified custom app'],
          signIn: { strategy: 'manual' },
        },
        installed: {
          version: 'Fixture',
          installType: 'known-path',
          executable: input.executable,
          installDir: input.executable.replace(/[^\\]+$/, ''),
        },
        error: null,
      });
      emit();
    },
    previewCustom: async (input, root) => {
      const expand = (v: string) =>
        v.replaceAll(
          '{profile}',
          `${root}\\${input.id}\\01M7CMM9SCQVK72JN29KCEXWQT`,
        );
      return {
        app: { executable: input.executable, version: 'Preview' },
        commandLine: [input.executable, ...input.args.map(expand)]
          .map(
            (value) =>
              `"${value.replace(/(\\*)"/g, '$1$1\\"').replace(/(\\+)$/g, '$1$1')}"`,
          )
          .join(' '),
        args: input.args.map(expand),
        env: Object.fromEntries(
          Object.entries(input.env).map(([k, v]) => [k, expand(v)]),
        ),
        workingDir: root,
      };
    },
    pickImport: async () => null,
    importProfiles: async () => {
      throw new Error(copy.desktop.previewBanner);
    },
    exportProfiles: async () => {
      throw new Error(copy.desktop.previewBanner);
    },
    preview: true,
    snapshot: async () => structuredClone(state),
    subscribe: async (listener) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    create: async (input) => {
      const id = input.id ?? crypto.randomUUID();
      const profile = {
        id,
        appId: input.appId,
        name: input.name,
        color: input.color,
        badge: 'monogram',
        dataDir: `${input.root}\\${input.appId}\\${id}`,
        options: {
          isolateClaudeCode: input.isolateClaudeCode,
          sharedCodexHistory: false,
        },
        hotkey: input.hotkey ?? null,
        createdAt: new Date().toISOString(),
        lastLaunchedAt: null,
        signInState: 'needs-sign-in' as const,
      };
      state.profiles.push(profile);
      emit();
      return profile;
    },
    update: async (profile) => {
      state.profiles = state.profiles.map((saved) =>
        saved.id === profile.id ? structuredClone(profile) : saved,
      );
      emit();
    },
    action: async (id, action, confirmation) => {
      const profile = state.profiles.find((profile) => profile.id === id);
      if (!profile) throw new Error(copy.desktop.statusUnavailable);
      if (
        ['force-close', 'remove', 'recycle'].includes(action) &&
        confirmation !== profile.name
      )
        throw new Error(copy.desktop.typeName);
      if (action === 'remove' || action === 'recycle') {
        state.profiles = state.profiles.filter((profile) => profile.id !== id);
        delete state.running[id];
      } else if (simulateProcesses)
        state.running[id] = action === 'launch' ? [1] : [];
      else throw new Error(copy.desktop.previewBanner);
      emit();
    },
    mainAction: async (appId, action) => {
      if (!simulateProcesses) throw new Error(copy.desktop.previewBanner);
      state.mainRunning[appId] = action === 'launch' ? [2] : [];
      emit();
    },
    saveSettings: async (settings) => {
      state = { ...state, settings };
      emit();
    },
    diagnostics: async () => ({
      product: copy.brand,
      directories: { profilesRoot: state.settings.profilesRoot },
      profiles: state.profiles.map((profile) => ({
        id: profile.id,
        appId: profile.appId,
        accountState: profile.signInState,
      })),
      claudeLayouts: [],
      launchLogs: [],
      preview: true,
    }),
    plan: async (id) => ({
      app: {
        executable: `${state.settings.profilesRoot}\\official-app.exe`,
        version: 'Preview',
      },
      args: [
        `--user-data-dir=${state.profiles.find((profile) => profile.id === id)?.dataDir}`,
      ],
      env: {},
      workingDir: state.settings.profilesRoot,
    }),
    browse: async () => null,
    window: async () => {},
    signIn: async (id, action) => {
      if (!simulateProcesses) throw new Error(copy.desktop.previewBanner);
      const progress = {
        id: action === 'start' ? `fixture-${id}` : id,
        profileId:
          action === 'start' ? id : (state.signIn?.[0]?.profileId ?? ''),
        state: 'awaiting-sign-in' as const,
      };
      state.signIn =
        action === 'finish' || action === 'restore' ? [] : [progress];
      emit();
      return {
        ...progress,
        state:
          action === 'finish'
            ? 'done'
            : action === 'restore'
              ? 'aborted'
              : progress.state,
      };
    },
    copyMcp: async () => {
      throw new Error(copy.desktop.previewBanner);
    },
    shortcuts: async (id, locations) => {
      if (!simulateProcesses) throw new Error(copy.desktop.previewBanner);
      return locations.map((location) => `${location}\\InstanceDeck-${id}.lnk`);
    },
    confirmAccount: async (id, matches) => {
      if (!simulateProcesses) throw new Error(copy.desktop.previewBanner);
      state.profiles = state.profiles.map((profile) =>
        profile.id === id
          ? { ...profile, signInState: matches ? 'signed-in' : 'needs-sign-in' }
          : profile,
      );
      emit();
    },
  };
}
