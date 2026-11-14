import { useState } from 'react';
import { Button } from '../components/Button';
import { Checkbox } from '../components/Controls';
import type { DesktopClient, Profile } from './model';
import styles from './Desktop.module.css';
import copy from '../copy/en.json';
const text = copy.power;
export function SharedHistory({
  client,
  profile,
  onSaved,
}: {
  client: DesktopClient;
  profile: Profile;
  onSaved: () => Promise<void>;
}) {
  const [consent, setConsent] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');
  const apply = async (enabled: boolean) => {
    setBusy(true);
    setError('');
    try {
      await client.sharedHistory(profile.id, enabled);
      await onSaved();
      setMessage(enabled ? text.historyEnabled : text.historyRestored);
      setConsent(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <section className={styles.stack}>
      <h3>{text.history}</h3>
      <p>
        {text.historyStatus.replace(
          '{status}',
          profile.options.sharedCodexHistory ? text.enabled : text.off,
        )}
      </p>
      <p>{text.historyIntro}</p>
      <Checkbox
        label={text.historyConsent}
        checked={consent}
        disabled={busy}
        onChange={(event) => setConsent(event.target.checked)}
      />
      <div className={styles.inline}>
        <Button
          size="M"
          kind="secondary"
          disabled={busy || !consent}
          onClick={() => void apply(true)}
        >
          {text.enableHistory}
        </Button>
        <Button
          size="M"
          kind="quiet"
          disabled={busy}
          onClick={() => void apply(false)}
        >
          {text.restoreHistory}
        </Button>
      </div>
      {error && <p role="alert">{error}</p>}
      {message && <p role="status">{message}</p>}
    </section>
  );
}
