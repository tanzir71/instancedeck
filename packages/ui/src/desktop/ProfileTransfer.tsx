import { useState } from 'react';
import { Button } from '../components/Button';
import { Dialog } from '../components/Controls';
import type { DesktopClient } from './model';
import styles from './Desktop.module.css';
import copy from '../copy/en.json';
const text = copy.power;
export function ProfileTransfer({
  client,
  onImported,
}: {
  client: DesktopClient;
  onImported: () => Promise<void>;
}) {
  const [review, setReview] =
    useState<Awaited<ReturnType<DesktopClient['pickImport']>>>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const run = async (action: () => Promise<void>) => {
    setBusy(true);
    setError('');
    try {
      await action();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <section className={styles.about}>
      <h2>{text.metadata}</h2>
      <p>{text.metadataIntro}</p>
      <div className={styles.inline}>
        <Button
          size="M"
          kind="secondary"
          disabled={busy}
          onClick={() =>
            void run(async () => {
              setReview(await client.pickImport());
            })
          }
        >
          {text.importProfiles}
        </Button>
        <Button
          size="M"
          kind="secondary"
          disabled={busy}
          onClick={() =>
            void run(async () => {
              const path = await client.exportProfiles();
              if (path) setMessage(text.exported.replace('{path}', path));
            })
          }
        >
          {text.exportProfiles}
        </Button>
      </div>
      {message && <p role="status">{message}</p>}
      {error && <p role="alert">{error}</p>}
      {review && (
        <Dialog
          title={text.review}
          onClose={() => !busy && setReview(null)}
          className={styles.powerDialog}
        >
          <h2>{text.review}</h2>
          <p>
            {text.reviewSummary
              .replace('{added}', String(review.added))
              .replace('{skipped}', String(review.skipped))}
          </p>
          <p>{text.pathsHint}</p>
          <div className={styles.inline}>
            <Button
              size="M"
              disabled={busy || review.added === 0}
              onClick={() =>
                void run(async () => {
                  const result = await client.importProfiles(review.document);
                  await onImported();
                  setReview(null);
                  setMessage(
                    text.imported.replace('{count}', String(result.added)),
                  );
                })
              }
            >
              {text.importMetadata}
            </Button>
            <Button
              size="M"
              kind="quiet"
              disabled={busy}
              onClick={() => setReview(null)}
            >
              {text.cancel}
            </Button>
          </div>
          {error && <p role="alert">{error}</p>}
        </Dialog>
      )}
    </section>
  );
}
