import { Dialog } from '../components/Controls';
import { Button } from '../components/Button';
import copy from '../copy/en.json';
import styles from './Desktop.module.css';

/** A view of journal state supplied by the recovery engine; it never infers sign-in. */
export function SignInAssistant({
  name,
  step = 1,
  journalSaved,
  busy = false,
  error,
  looksSignedIn = false,
  primaryLabel,
  onFinish,
  onRestore,
}: {
  name: string;
  step?: number;
  journalSaved: boolean;
  busy?: boolean;
  error?: string;
  looksSignedIn?: boolean;
  primaryLabel?: string;
  onFinish: () => void;
  onRestore: () => void;
}) {
  return (
    <Dialog
      title={copy.desktop.signInTitle.replace('{name}', name)}
      onClose={onRestore}
      className={styles.signIn}
    >
      <div className={styles.eyebrowRow}>
        <span>{copy.desktop.signInEyebrow}</span>
        {journalSaved && (
          <span className={styles.success}>{copy.desktop.journalSaved}</span>
        )}
      </div>
      <h2 tabIndex={busy ? 0 : undefined}>
        {copy.desktop.signInTitle.replace('{name}', name)}
      </h2>
      <p className={styles.signInBody}>
        {copy.desktop.signInBody.replace('{name}', name)}
      </p>
      <ol className={styles.signInSteps}>
        {copy.desktop.signInSteps.map((label, index) => (
          <li
            key={label}
            data-state={
              index < step ? 'done' : index === step ? 'current' : 'pending'
            }
            aria-current={index === step ? 'step' : undefined}
          >
            <span className={styles.stepNumber}>
              {index < step ? '✓' : index + 1}
            </span>
            <div>
              <h3>{label.replace('{name}', name)}</h3>
              <p>{copy.desktop.signInBodies[index]}</p>
            </div>
            {index <= step && (
              <small>
                {index < step ? copy.desktop.done : copy.desktop.waiting}
              </small>
            )}
          </li>
        ))}
      </ol>
      <p className={styles.recoveryNote}>{copy.desktop.recoveryNote}</p>
      {error && (
        <p role="alert" className={styles.error}>
          {error}
        </p>
      )}
      <div className={styles.dialogActions}>
        <Button size="M" kind="secondary" disabled={busy} onClick={onRestore}>
          {copy.desktop.cancelRestore}
        </Button>
        <Button size="M" disabled={busy || !journalSaved} onClick={onFinish}>
          {busy
            ? copy.desktop.busy
            : looksSignedIn
              ? copy.desktop.looksSignedIn
              : (primaryLabel ?? copy.desktop.finishedSignIn)}
        </Button>
      </div>
    </Dialog>
  );
}
export function RecoveryDialog({
  busy = false,
  error,
  onResume,
  onRestore,
}: {
  busy?: boolean;
  error?: string;
  onResume: () => void;
  onRestore: () => void;
}) {
  return (
    <Dialog
      title={copy.desktop.recoveryTitle}
      onClose={() => {}}
      className={styles.confirm}
    >
      <h2>{copy.desktop.recoveryTitle}</h2>
      <p>{copy.desktop.recoveryBody}</p>
      {error && <p role="alert">{error}</p>}
      <div className={styles.dialogActions}>
        <Button size="M" kind="secondary" disabled={busy} onClick={onRestore}>
          {copy.desktop.restoreMain}
        </Button>
        <Button size="M" disabled={busy} onClick={onResume}>
          {copy.desktop.resume}
        </Button>
      </div>
    </Dialog>
  );
}
