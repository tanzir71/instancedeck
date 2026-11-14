import { useEffect, useRef, useState } from 'react';
import { Button } from '../components/Button';
import { Dialog } from '../components/Controls';
import type { DesktopClient, Settings, UpdateInfo } from './model';
import copy from '../copy/en.json';
import styles from './Desktop.module.css';
const text = copy.updates;
export function UpdateCenter({
  client,
  settings,
  expanded = true,
  onShow = () => {},
}: {
  client: DesktopClient;
  settings: Settings;
  expanded?: boolean;
  onShow?: () => void;
}) {
  const [info, setInfo] = useState<UpdateInfo>();
  const [checking, setChecking] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [confirm, setConfirm] = useState(false);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');
  const [downloaded, setDownloaded] = useState<number>();
  const autoAttempt = useRef<
    | { channel: string; client: DesktopClient; promise: Promise<UpdateInfo> }
    | undefined
  >(undefined);
  const allowed = settings.onboardingCompleted && settings.allowUpdateChecks;
  useEffect(() => {
    if (!allowed) {
      autoAttempt.current = undefined;
      setInfo(undefined);
      setChecking(false);
      setError('');
      return;
    }
    if (
      autoAttempt.current?.channel !== settings.updateChannel ||
      autoAttempt.current?.client !== client
    ) {
      autoAttempt.current = {
        channel: settings.updateChannel,
        client,
        promise: client.checkUpdates(),
      };
    }
    let active = true;
    setChecking(true);
    setInfo(undefined);
    setError('');
    void autoAttempt.current.promise
      .then((value) => {
        if (active) setInfo(value);
      })
      .catch((e) => {
        if (active) setError(String(e));
      })
      .finally(() => {
        if (active) setChecking(false);
      });
    return () => {
      active = false;
    };
  }, [client, allowed, settings.updateChannel]);
  useEffect(() => {
    let active = true;
    let stop: (() => void) | undefined;
    void client
      .updateProgress((value) => {
        if (active) setDownloaded(value);
      })
      .then((value) => {
        if (active) stop = value;
        else value();
      })
      .catch((e) => {
        if (active) setError(String(e));
      });
    return () => {
      active = false;
      stop?.();
    };
  }, [client]);
  if (!expanded)
    return info?.status === 'available' ? (
      <div className={styles.errorBanner} role="status">
        <p>{text.available.replace('{version}', info.version ?? '')}</p>
        <Button size="M" kind="secondary" onClick={onShow}>
          {text.review}
        </Button>
      </div>
    ) : null;
  return (
    <section className={styles.about}>
      <h2>{text.title}</h2>
      <p>{text.consent}</p>
      {checking ? (
        <p role="status">{text.checking}</p>
      ) : (
        info && (
          <p role="status">
            {info.status === 'available'
              ? text.available.replace('{version}', info.version ?? '')
              : text[info.status]}
          </p>
        )
      )}
      {info && (
        <p className={styles.muted}>
          {text.installedVersion.replace('{version}', info.currentVersion)}
        </p>
      )}
      {info?.portable && <p>{text.portable}</p>}
      <div className={styles.inline}>
        <Button
          size="M"
          kind="secondary"
          disabled={!allowed || checking || installing}
          onClick={async () => {
            setChecking(true);
            setError('');
            try {
              setInfo(await client.checkUpdates());
            } catch (e) {
              setError(String(e));
            } finally {
              setChecking(false);
            }
          }}
        >
          {text.check}
        </Button>
        {info?.status === 'available' && !info.portable && (
          <Button
            size="M"
            disabled={!allowed || installing}
            onClick={() => setConfirm(true)}
          >
            {text.review}
          </Button>
        )}
        {info?.portable && (
          <Button
            size="M"
            kind="quiet"
            onClick={() =>
              void client
                .openExternal('download')
                .catch((e) => setError(String(e)))
            }
          >
            {text.downloadPage}
          </Button>
        )}
      </div>
      {message && <p role="status">{message}</p>}
      {error && <p role="alert">{error}</p>}
      {confirm && info?.version && (
        <Dialog
          title={text.confirm.replace('{version}', info.version)}
          onClose={() => !installing && setConfirm(false)}
          className={styles.powerDialog}
        >
          <h2>{text.confirm.replace('{version}', info.version)}</h2>
          <p>{text.confirmHelp}</p>
          {info.notes && (
            <>
              <h3>{text.notes}</h3>
              <pre className={styles.report}>{info.notes}</pre>
            </>
          )}
          {installing && (
            <p role="status">
              {text.installing}{' '}
              {downloaded !== undefined &&
                text.downloaded.replace(
                  '{size}',
                  (downloaded / 1024 / 1024).toFixed(1),
                )}
            </p>
          )}
          {error && <p role="alert">{error}</p>}
          <div className={styles.inline}>
            <Button
              size="M"
              disabled={installing || !allowed}
              onClick={async () => {
                setInstalling(true);
                setError('');
                setDownloaded(undefined);
                try {
                  await client.installUpdate(info.version!, true);
                  setMessage(text.started);
                  setConfirm(false);
                } catch (e) {
                  setError(String(e));
                } finally {
                  setInstalling(false);
                }
              }}
            >
              {text.install}
            </Button>
            <Button
              size="M"
              kind="quiet"
              disabled={installing}
              onClick={() => setConfirm(false)}
            >
              {text.cancel}
            </Button>
          </div>
        </Dialog>
      )}
    </section>
  );
}
