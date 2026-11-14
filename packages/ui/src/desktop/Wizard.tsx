import { useState } from 'react';
import { ulid } from 'ulid';
import { Button } from '../components/Button';
import {
  Checkbox,
  ColorSwatchPicker,
  Dialog,
  IconButton,
  Input,
  Stepper,
  Switch,
} from '../components/Controls';
import { MonogramTile, ProfileCard } from '../components/ProfileCard';
import type {
  AppDetection,
  CreateProfile,
  Profile,
  ProfileColor,
} from './model';
import { monogram } from './model';
import copy from '../copy/en.json';
import styles from './Desktop.module.css';

export function Wizard({
  apps,
  root,
  onClose,
  onCreate,
  onBrowse,
  initialStep = 0,
  initialName = '',
  initialAppId,
  initialColor = 'iris',
  initialId,
}: {
  apps: AppDetection[];
  root: string;
  onClose: () => void;
  onCreate: (
    input: CreateProfile & { shortcuts?: ('desktop' | 'start-menu')[] },
  ) => Promise<void>;
  onBrowse: () => Promise<string | null>;
  initialStep?: number;
  initialName?: string;
  initialAppId?: string;
  initialColor?: ProfileColor;
  initialId?: string;
}) {
  const [step, setStep] = useState(initialStep);
  const [appId, setAppId] = useState(
    initialAppId ?? apps.find((app) => app.installed)?.recipe.id ?? '',
  );
  const [name, setName] = useState(initialName);
  const [color, setColor] = useState<ProfileColor>(initialColor);
  const [location, setLocation] = useState(root);
  const [id] = useState(() => initialId ?? ulid());
  const dataDir = [location.replace(/[\\/]+$/u, ''), appId, id].join('\\');
  const [isolate, setIsolate] = useState(true);
  const [hotkey, setHotkey] = useState('');
  const [desktopShortcut, setDesktopShortcut] = useState(false);
  const [startShortcut, setStartShortcut] = useState(false);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const app = apps.find((item) => item.recipe.id === appId);
  const preview: Profile = {
    id,
    appId,
    name: name.trim() || copy.desktop.profileName,
    color,
    badge: 'monogram',
    dataDir,
    createdAt: '',
    lastLaunchedAt: null,
    signInState: 'needs-sign-in',
    hotkey: null,
    options: { isolateClaudeCode: isolate, sharedCodexHistory: false },
  };
  const next = () => {
    if (step === 1 && (!name.trim() || Array.from(name.trim()).length > 120)) {
      setError(copy.desktop.nameRequired);
      return;
    }
    setError('');
    setStep(step + 1);
  };
  const create = async () => {
    setBusy(true);
    setError('');
    try {
      await onCreate({
        id,
        appId,
        name: name.trim(),
        color,
        root: location,
        isolateClaudeCode: isolate,
        hotkey: hotkey.trim() || null,
        shortcuts: [
          ...(desktopShortcut ? ['desktop' as const] : []),
          ...(startShortcut ? ['start-menu' as const] : []),
        ],
      });
    } catch (error) {
      setError(String(error));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog
      title={copy.desktop.newProfile}
      onClose={() => !busy && onClose()}
      className={styles.wizard}
    >
      <div className={styles.wizardForm}>
        <div className={styles.eyebrowRow}>
          <span>
            {copy.desktop.newProfile}
            {app ? ` · ${app.recipe.name}` : ''}
          </span>
          <IconButton
            label={copy.desktop.dismiss}
            disabled={busy}
            onClick={onClose}
          >
            ×
          </IconButton>
        </div>
        <div className={styles.wizardHeading}>
          <h2>
            {
              [
                copy.desktop.pickApp,
                copy.desktop.nameAndColour,
                copy.desktop.options,
              ][step]
            }
          </h2>
          <Stepper step={step} />
        </div>
        {step === 0 ? (
          <div className={styles.appChoices}>
            {[...apps]
              .sort(
                (a, b) =>
                  Number(Boolean(b.installed)) - Number(Boolean(a.installed)),
              )
              .map((item) => (
                <label
                  key={item.recipe.id}
                  data-selected={item.recipe.id === appId}
                >
                  <input
                    type="radio"
                    name="app"
                    value={item.recipe.id}
                    checked={item.recipe.id === appId}
                    disabled={!item.installed}
                    onChange={() => setAppId(item.recipe.id)}
                  />
                  <MonogramTile
                    letters={monogram(item.recipe.id)}
                    tone="neutral"
                  />
                  <span>
                    <strong>{item.recipe.name}</strong>
                    <small>
                      {item.installed
                        ? `${item.installed.version} · ${item.installed.installType === 'msix' ? copy.desktop.store : copy.desktop.standardInstall}`
                        : copy.desktop.notInstalled}
                    </small>
                  </span>
                </label>
              ))}
            <Button
              size="M"
              kind="quiet"
              disabled
              title={copy.desktop.customPending}
            >
              {copy.desktop.customApp}
            </Button>
            <p className={styles.muted}>{copy.desktop.customPending}</p>
          </div>
        ) : step === 1 ? (
          <>
            <Input
              label={copy.desktop.profileName}
              value={name}
              onChange={(event) => setName(event.target.value)}
              maxLength={120}
              autoComplete="off"
            />
            <ColorSwatchPicker value={color} onChange={setColor} />
            <div className={styles.browseField}>
              <Input
                label={copy.desktop.location}
                value={dataDir}
                title={dataDir}
                readOnly
              />
              <Button
                kind="secondary"
                size="M"
                onClick={async () => {
                  try {
                    const folder = await onBrowse();
                    if (folder) setLocation(folder);
                  } catch (error) {
                    setError(String(error));
                  }
                }}
              >
                {copy.desktop.browse}
              </Button>
            </div>
            {appId === 'claude-desktop' && (
              <Switch
                label={copy.desktop.isolateClaude}
                checked={isolate}
                onChange={setIsolate}
              />
            )}
          </>
        ) : (
          <div className={styles.stack}>
            <p className={styles.muted}>{copy.desktop.shellOptionsIntro}</p>
            <Checkbox
              label={copy.desktop.desktopShortcut}
              checked={desktopShortcut}
              onChange={(event) => setDesktopShortcut(event.target.checked)}
            />
            <Checkbox
              label={copy.desktop.startShortcut}
              checked={startShortcut}
              onChange={(event) => setStartShortcut(event.target.checked)}
            />
            <Input
              label={copy.desktop.hotkey}
              value={hotkey}
              onChange={(event) => setHotkey(event.target.value)}
              hint={copy.desktop.hotkeyHelp}
            />
            {appId === 'claude-desktop' && (
              <p className={styles.muted}>{copy.desktop.signInPending}</p>
            )}
          </div>
        )}
        {error && (
          <p role="alert" className={styles.error}>
            {error}
          </p>
        )}
        <div className={styles.dialogActions}>
          <Button
            kind="secondary"
            size="M"
            disabled={busy}
            onClick={() => (step === 0 ? onClose() : setStep(step - 1))}
          >
            {step === 0 ? copy.desktop.cancel : copy.desktop.back}
          </Button>
          <Button
            size="M"
            disabled={busy || !app?.installed}
            onClick={step === 2 ? create : next}
          >
            {busy
              ? copy.desktop.busy
              : step === 2
                ? copy.desktop.create
                : copy.desktop.continue}
          </Button>
        </div>
      </div>
      <aside className={styles.wizardPreview}>
        <span className={styles.eyebrow}>{copy.desktop.preview}</span>
        <ProfileCard profile={preview} preview />
        <p>{copy.desktop.previewNote}</p>
      </aside>
    </Dialog>
  );
}
