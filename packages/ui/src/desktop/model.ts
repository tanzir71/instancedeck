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
  signIn?: SignInProgress[];
}
export interface SignInProgress {
  id: string;
  profileId: string;
  state:
    | 'preflight'
    | 'stashed'
    | 'awaiting-sign-in'
    | 'captured'
    | 'adopting'
    | 'restoring'
    | 'done'
    | 'aborted';
  looksSignedIn?: boolean;
  metadataError?: string | null;
}
export interface CreateProfile {
  id?: string;
  appId: string;
  name: string;
  color: ProfileColor;
  root: string;
  isolateClaudeCode: boolean;
  hotkey?: string | null;
}
export interface LaunchPlan {
  commandLine?: string;
  app: { executable: string; version: string };
  args: string[];
  env: Record<string, string>;
  workingDir: string;
}
export type ProfileAction =
  'launch' | 'close' | 'force-close' | 'remove' | 'recycle' | 'open-folder';
export interface DesktopClient {
  checkUpdates(): Promise<UpdateInfo>;
  installUpdate(version: string, confirmed: boolean): Promise<void>;
  updateProgress(
    listener: (downloaded: number, total?: number) => void,
  ): Promise<() => void>;
  openExternal(
    target: 'credit' | 'source' | 'download' | 'claude-usage' | 'codex-usage',
  ): Promise<void>;
  sharedHistory(id: string, enabled: boolean): Promise<void>;
  addCustom(input: CustomApp): Promise<void>;
  previewCustom(input: CustomApp, root: string): Promise<LaunchPlan>;
  pickImport(): Promise<{
    document: string;
    added: number;
    skipped: number;
  } | null>;
  importProfiles(document: string): Promise<{ added: number; skipped: number }>;
  exportProfiles(): Promise<string | null>;
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
  signIn(
    id: string,
    action: 'start' | 'resume' | 'finish' | 'restore',
    closeMainConfirmed?: boolean,
  ): Promise<SignInProgress>;
  copyMcp(from: string, to: string): Promise<void>;
  confirmAccount(id: string, matches: boolean): Promise<void>;
  shortcuts(
    id: string,
    locations: ('desktop' | 'start-menu')[],
  ): Promise<string[]>;
}
export function monogram(appId: string) {
  return appId === 'claude-desktop'
    ? 'Cl'
    : appId === 'codex'
      ? 'Cx'
      : appId.slice(0, 2);
}
export interface CustomApp {
  id: string;
  name: string;
  executable: string;
  args: string[];
  env: Record<string, string>;
  matchArgument: string;
}
export interface UpdateInfo {
  status: 'available' | 'current' | 'unavailable';
  version: string | null;
  notes: string | null;
  portable: boolean;
  currentVersion: string;
}
