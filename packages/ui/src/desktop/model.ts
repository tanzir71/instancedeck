export type ProfileColor =
  | 'terracotta'
  | 'teal'
  | 'iris'
  | 'ochre'
  | 'sky'
  | 'plum'
  | 'moss'
  | 'graphite';
export const profileColors: ProfileColor[] = [
  'terracotta',
  'teal',
  'iris',
  'ochre',
  'sky',
  'plum',
  'moss',
  'graphite',
];
export interface Profile {
  id: string;
  appId: string;
  name: string;
  color: ProfileColor;
  badge: string;
  dataDir: string;
  options: { isolateClaudeCode: boolean; sharedCodexHistory: boolean };
  hotkey: string | null;
  createdAt: string;
  lastLaunchedAt: string | null;
  signInState: 'signed-in' | 'unknown' | 'needs-sign-in';
}
export interface AppDetection {
  recipe: {
    id: string;
    name: string;
    vendor: string;
    status: 'community' | 'verified' | 'experimental';
    limitations: string[];
    signIn: { strategy: string };
  };
  installed: {
    version: string;
    installType: string;
    executable: string;
    installDir: string;
  } | null;
  error: string | null;
}
export interface Settings {
  schemaVersion: number;
  profilesRoot: string;
  closeToTray: boolean;
  startWithWindows: boolean;
  switcherHotkey: string;
  allowUpdateChecks: boolean;
  updateChannel: 'stable' | 'beta';
  theme: 'light' | 'dark';
  onboardingCompleted: boolean;
}
export interface DesktopSnapshot {
  profiles: Profile[];
  apps: AppDetection[];
  running: Record<string, number[]>;
  runningSince?: Record<string, number>;
  mainRunning: Record<string, number[]>;
  statusErrors: Record<string, string>;
  settings: Settings;
}
export interface CreateProfile {
  id?: string;
  appId: string;
  name: string;
  color: ProfileColor;
  root: string;
  isolateClaudeCode: boolean;
}
export interface LaunchPlan {
  app: { executable: string; version: string };
  args: string[];
  env: Record<string, string>;
  workingDir: string;
}
export type ProfileAction =
  'launch' | 'close' | 'force-close' | 'remove' | 'recycle' | 'open-folder';
export interface DesktopClient {
  readonly preview?: boolean;
  snapshot(refresh?: boolean): Promise<DesktopSnapshot>;
  subscribe(
    listener: (snapshot: DesktopSnapshot) => void,
    onError: (error: string) => void,
  ): Promise<() => void>;
  create(input: CreateProfile): Promise<Profile>;
  update(profile: Profile): Promise<void>;
  action(
    id: string,
    action: ProfileAction,
    confirmation?: string,
  ): Promise<void>;
  mainAction(appId: string, action: 'launch' | 'close'): Promise<void>;
  saveSettings(settings: Settings): Promise<void>;
  diagnostics(redact: boolean): Promise<Record<string, unknown>>;
  plan(id: string): Promise<LaunchPlan>;
  browse(): Promise<string | null>;
  window(action: 'minimize' | 'maximize' | 'close'): Promise<void>;
}
export function monogram(appId: string) {
  return appId === 'claude-desktop'
    ? 'Cl'
    : appId === 'codex'
      ? 'Cx'
      : appId.slice(0, 2);
}
