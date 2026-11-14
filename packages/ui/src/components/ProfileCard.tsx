import { Button } from './Button';
import { Badge } from './Controls';
import type { AppDetection, Profile } from '../desktop/model';
import { monogram } from '../desktop/model';
import copy from '../copy/en.json';
import styles from './ProfileCard.module.css';
import runningDot from '../assets/status-running.svg?url';
import idleDot from '../assets/status-idle.svg?url';
import colorRunningDot from '../assets/status-on-color-running.svg?url';
import colorIdleDot from '../assets/status-on-color-idle.svg?url';

export function MonogramTile({
  letters,
  size = 'M',
  tone = 'light',
  className = '',
}: {
  letters: string;
  size?: 'S' | 'M' | 'L';
  tone?: 'light' | 'neutral' | 'ink';
  className?: string;
}) {
  return (
    <span
      className={`${styles.monogram} ${styles[size]} ${styles[tone] ?? ''} ${className}`}
      aria-hidden="true"
    >
      {letters}
    </span>
  );
}
export function StatusDot({
  running,
  onColor = false,
  label,
}: {
  running: boolean;
  onColor?: boolean;
  label?: string;
}) {
  return (
    <span className={styles.status}>
      <img
        src={
          onColor
            ? running
              ? colorRunningDot
              : colorIdleDot
            : running
              ? runningDot
              : idleDot
        }
        width="8"
        height="8"
        alt=""
      />
      {label ?? (running ? copy.desktop.running : copy.desktop.idle)}
    </span>
  );
}
export function ProfileCard({
  profile,
  running = false,
  busy = false,
  detail,
  onLaunch,
  onClose,
  onDetails,
  preview = false,
}: {
  profile: Profile;
  running?: boolean;
  busy?: boolean;
  detail?: string;
  onLaunch?: () => void;
  onClose?: () => void;
  onDetails?: () => void;
  preview?: boolean;
}) {
  const description =
    detail ??
    (profile.signInState === 'signed-in'
      ? copy.desktop.signedIn
      : profile.signInState === 'needs-sign-in'
        ? copy.desktop.notSignedIn
        : copy.desktop.unknownSignIn);
  return (
    <article
      className={styles.card}
      style={{ background: `var(--profile-${profile.color})` }}
      aria-label={profile.name}
    >
      <div className={styles.top}>
        <MonogramTile letters={monogram(profile.appId)} />
        <StatusDot running={running} onColor />
      </div>
      <div className={styles.meta}>
        <h3>
          {onDetails ? (
            <button
              type="button"
              onClick={onDetails}
              aria-label={`${copy.desktop.details}: ${profile.name}`}
            >
              {profile.name}
            </button>
          ) : (
            profile.name
          )}
        </h3>
        <p>{description}</p>
      </div>
      <div className={styles.actions}>
        <Button
          size="M"
          kind="onColor"
          disabled={busy}
          aria-disabled={preview || undefined}
          onClick={preview ? undefined : onLaunch}
          aria-label={`${busy ? copy.desktop.busy : running ? copy.desktop.focus : copy.desktop.launch} ${profile.name}`}
        >
          {busy
            ? copy.desktop.busy
            : running
              ? copy.desktop.focus
              : copy.desktop.launch}
        </Button>
        {running && (
          <Button
            size="M"
            kind="onColor"
            disabled={busy}
            onClick={onClose}
            aria-label={`${copy.desktop.close} ${profile.name}`}
          >
            {copy.desktop.close}
          </Button>
        )}
      </div>
    </article>
  );
}
export function MainInstanceCard({
  appId,
  running,
  busy,
  onLaunch,
}: {
  appId: string;
  running: boolean;
  busy?: boolean;
  onLaunch: () => void;
}) {
  return (
    <article
      className={`${styles.card} ${styles.mainCard}`}
      aria-label={`${copy.desktop.mainInstance} ${appId}`}
    >
      <div className={styles.top}>
        <MonogramTile letters={monogram(appId)} tone="ink" />
        <StatusDot
          running={running}
          label={running ? copy.desktop.running : copy.desktop.notRunning}
        />
      </div>
      <div className={styles.meta}>
        <h3>{copy.desktop.mainInstance}</h3>
        <p>{copy.desktop.unmanaged}</p>
      </div>
      <div className={styles.actions}>
        <Button kind="secondary" size="M" disabled={busy} onClick={onLaunch}>
          {running ? copy.desktop.focus : copy.desktop.launch}
        </Button>
      </div>
    </article>
  );
}
export function AddProfileCard({ onClick }: { onClick: () => void }) {
  return (
    <button type="button" className={styles.add} onClick={onClick}>
      <span aria-hidden="true">+</span>
      {copy.desktop.newProfile}
    </button>
  );
}
export function AppGroupHeader({
  app,
  onHealthCheck,
}: {
  app: AppDetection;
  onHealthCheck: () => void;
}) {
  return (
    <div className={styles.groupHeader}>
      <div>
        <MonogramTile
          letters={monogram(app.recipe.id)}
          size="S"
          tone="neutral"
        />
        <h2>{app.recipe.name}</h2>
        <span className={styles.version}>
          {app.installed?.version} ·{' '}
          {app.installed?.installType === 'msix'
            ? copy.desktop.store
            : copy.desktop.standardInstall}
        </span>
        <Badge status={app.recipe.status} />
      </div>
      <Button size="M" kind="quiet" onClick={onHealthCheck}>
        {copy.desktop.healthCheck}
      </Button>
    </div>
  );
}
export function EmptyState({
  title,
  body,
  action,
}: {
  title: string;
  body: string;
  action?: { label: string; onClick: () => void };
}) {
  return (
    <section className={styles.empty}>
      <div className={styles.art} aria-hidden="true">
        <span />
        <span />
        <span />
      </div>
      <h2>{title}</h2>
      <p>{body}</p>
      {action && (
        <Button size="M" onClick={action.onClick}>
          {action.label}
        </Button>
      )}
    </section>
  );
}
