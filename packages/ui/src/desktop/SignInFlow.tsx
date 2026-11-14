import { useState } from 'react';
import { Dialog } from '../components/Controls';
import { Button } from '../components/Button';
import { RecoveryDialog, SignInAssistant } from './SignInAssistant';
import type { DesktopClient, DesktopSnapshot, Profile } from './model';
import copy from '../copy/en.json';
import styles from './Desktop.module.css';

export function SignInFlow({
  client,
  snapshot,
  profile,
  onDismiss,
  onRefresh,
  onComplete,
}: {
  client: DesktopClient;
  snapshot: DesktopSnapshot;
  profile?: Profile;
  onDismiss: () => void;
  onRefresh: () => Promise<void>;
  onComplete?: (id: string) => void;
}) {
  const [activeId, setActiveId] = useState<string>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const journal = snapshot.signIn?.[0];
  const run = async (
    id: string,
    action: 'start' | 'resume' | 'finish' | 'restore',
  ) => {
    setBusy(true);
    setError('');
    try {
      const result = await client.signIn(id, action, action === 'start');
      setActiveId(result.id);
      onDismiss();
      await onRefresh();
      if (result.state === 'done') onComplete?.(result.profileId);
    } catch (error) {
      setError(String(error));
      await onRefresh().catch(() => {});
    } finally {
      setBusy(false);
    }
  };
  if (journal) {
    if (journal.id !== activeId)
      return (
        <RecoveryDialog
          busy={busy}
          error={error}
          onResume={() => {
            void run(journal.id, 'resume');
          }}
          onRestore={() => {
            void run(journal.id, 'restore');
          }}
        />
      );
    return (
      <SignInAssistant
        name={
          snapshot.profiles.find((p) => p.id === journal.profileId)?.name ??
          copy.desktop.profileName
        }
        step={
          ['preflight', 'stashed'].includes(journal.state)
            ? 0
            : journal.state === 'awaiting-sign-in'
              ? 1
              : journal.state === 'restoring'
                ? 3
                : 2
        }
        journalSaved
        busy={busy}
        error={error || journal.metadataError || undefined}
        looksSignedIn={journal.looksSignedIn}
        primaryLabel={
          journal.state === 'awaiting-sign-in' ? undefined : copy.desktop.resume
        }
        onFinish={() => {
          void run(
            journal.id,
            journal.state === 'awaiting-sign-in' ? 'finish' : 'resume',
          );
        }}
        onRestore={() => {
          if (!busy) void run(journal.id, 'restore');
        }}
      />
    );
  }
  if (!profile) return null;
  if (profile.appId !== 'claude-desktop')
    return (
      <AccountGuidance
        profile={profile}
        client={client}
        onClose={onDismiss}
        onRefresh={onRefresh}
      />
    );
  return (
    <Dialog
      title={copy.desktop.signInNow}
      onClose={() => {
        if (!busy) {
          setError('');
          onDismiss();
        }
      }}
      className={styles.confirm}
    >
      <h2>{copy.desktop.signInTitle.replace('{name}', profile.name)}</h2>
      <p>{copy.desktop.signInBody.replace('{name}', profile.name)}</p>
      <p>{copy.desktop.signInConsent}</p>
      <p>{copy.desktop.accountHealthHelp}</p>
      {error && <p role="alert">{error}</p>}
      <div className={styles.dialogActions}>
        <Button
          size="M"
          kind="secondary"
          disabled={busy}
          onClick={() => {
            setError('');
            onDismiss();
          }}
        >
          {copy.desktop.cancel}
        </Button>
        <Button
          size="M"
          disabled={busy}
          onClick={() => {
            void run(profile.id, 'start');
          }}
        >
          {busy ? copy.desktop.busy : copy.desktop.startSignIn}
        </Button>
      </div>
    </Dialog>
  );
}

export function AccountGuidance({
  profile,
  client,
  onClose,
  onRefresh,
}: {
  profile: Profile;
  client: DesktopClient;
  onClose: () => void;
  onRefresh: () => Promise<void>;
}) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const run = async (action: 'launch' | 'matches' | 'mismatch') => {
    setBusy(true);
    setError('');
    try {
      if (action === 'launch') await client.action(profile.id, 'launch');
      else {
        await client.confirmAccount(profile.id, action === 'matches');
        onClose();
      }
      await onRefresh();
    } catch (error) {
      setError(String(error));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog
      title={copy.desktop.accountHealth}
      onClose={() => {
        if (!busy) onClose();
      }}
      className={styles.confirm}
    >
      <h2>
        {copy.desktop.accountHealth} · {profile.name}
      </h2>
      <p>
        {profile.appId === 'codex'
          ? copy.desktop.codexSignInHelp
          : copy.desktop.accountHealthHelp}
      </p>
      <p>{copy.desktop.manualAccountOnly}</p>
      {error && <p role="alert">{error}</p>}
      <div className={styles.stack}>
        <Button
          size="M"
          disabled={busy}
          onClick={() => {
            void run('launch');
          }}
        >
          {copy.desktop.launch}
        </Button>
        <Button
          size="M"
          kind="secondary"
          disabled={busy}
          onClick={() => {
            void run('matches');
          }}
        >
          {copy.desktop.accountMatches}
        </Button>
        <Button
          size="M"
          kind="quiet"
          disabled={busy}
          onClick={() => {
            void run('mismatch');
          }}
        >
          {copy.desktop.accountMismatch}
        </Button>
        <Button size="M" kind="quiet" disabled={busy} onClick={onClose}>
          {copy.desktop.cancel}
        </Button>
      </div>
    </Dialog>
  );
}
