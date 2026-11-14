import { useState } from 'react';
import { Button } from '../components/Button';
import { Dialog, Input, Switch } from '../components/Controls';
import type { Settings } from './model';
import copy from '../copy/en.json';
import styles from './Desktop.module.css';

export function FirstRunDialog({
  settings,
  onSave,
}: {
  settings: Settings;
  onSave: (settings: Settings) => Promise<void>;
}) {
  const [draft, setDraft] = useState(settings);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  return (
    <Dialog
      title={copy.desktop.firstRunTitle}
      onClose={() => {}}
      className={styles.confirm}
    >
      <h2>{copy.desktop.firstRunTitle}</h2>
      <p>{copy.desktop.firstRunBody}</p>
      <Input
        label={copy.desktop.profilesRoot}
        value={draft.profilesRoot}
        onChange={(event) =>
          setDraft({ ...draft, profilesRoot: event.target.value })
        }
      />
      <Switch
        label={copy.desktop.allowUpdates}
        checked={draft.allowUpdateChecks}
        onChange={(allowUpdateChecks) =>
          setDraft({ ...draft, allowUpdateChecks })
        }
      />
      <p className={styles.muted}>{copy.desktop.updatesPending}</p>
      {error && <p role="alert">{error}</p>}
      <Button
        size="M"
        disabled={busy}
        onClick={async () => {
          setBusy(true);
          setError('');
          try {
            await onSave({ ...draft, onboardingCompleted: true });
          } catch (error) {
            setError(String(error));
          } finally {
            setBusy(false);
          }
        }}
      >
        {busy ? copy.desktop.busy : copy.desktop.firstRunAction}
      </Button>
    </Dialog>
  );
}
