import { useEffect, useId, useState } from 'react';
import { Button } from '../components/Button';
import { Dialog, Input } from '../components/Controls';
import type { CustomApp, DesktopClient, LaunchPlan } from './model';
import styles from './Desktop.module.css';
import copy from '../copy/en.json';
const text = copy.power;

export function CustomAppEditor({
  client,
  root,
  onClose,
  onSaved,
  initial,
}: {
  client: DesktopClient;
  root: string;
  onClose: () => void;
  onSaved: (id: string) => Promise<void>;
  initial?: CustomApp;
}) {
  const [id, setId] = useState(initial?.id ?? 'custom-my-app');
  const [name, setName] = useState(initial?.name ?? '');
  const [executable, setExecutable] = useState(initial?.executable ?? '');
  const [args, setArgs] = useState(
    JSON.stringify(initial?.args ?? ['--user-data-dir={profile}'], null, 2),
  );
  const [env, setEnv] = useState(JSON.stringify(initial?.env ?? {}, null, 2));
  const [matchArgument, setMatchArgument] = useState(
    initial?.matchArgument ?? '--user-data-dir={profile}',
  );
  const [plan, setPlan] = useState<LaunchPlan>();
  const [error, setError] = useState('');
  const [previewError, setPreviewError] = useState('');
  const [busy, setBusy] = useState(false);
  const field = useId();
  useEffect(() => {
    let active = true;
    setPlan(undefined);
    const timer = setTimeout(() => {
      try {
        const input = {
          id,
          name,
          executable,
          args: JSON.parse(args),
          env: JSON.parse(env),
          matchArgument,
        };
        if (
          !Array.isArray(input.args) ||
          input.args.some((v) => typeof v !== 'string') ||
          !input.env ||
          Array.isArray(input.env) ||
          typeof input.env !== 'object' ||
          Object.values(input.env).some((v) => typeof v !== 'string')
        )
          throw new Error(text.jsonShape);
        void client
          .previewCustom(input, root)
          .then((value) => {
            if (active) {
              setPlan(value);
              setPreviewError('');
            }
          })
          .catch((e) => {
            if (active) setPreviewError(String(e));
          });
      } catch (e) {
        if (active) setPreviewError(String(e));
      }
    }, 250);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [client, root, id, name, executable, args, env, matchArgument]);
  return (
    <Dialog
      title={text.customTitle}
      onClose={() => !busy && onClose()}
      className={styles.powerDialog}
    >
      <form
        className={styles.stack}
        onSubmit={async (e) => {
          e.preventDefault();
          setBusy(true);
          setError('');
          try {
            await client.addCustom({
              id,
              name,
              executable,
              args: JSON.parse(args),
              env: JSON.parse(env),
              matchArgument,
            });
            await onSaved(id);
          } catch (e) {
            setError(String(e));
          } finally {
            setBusy(false);
          }
        }}
      >
        <h2>{text.customTitle}</h2>
        <p>{text.customIntro}</p>
        <Input
          label={text.appId}
          value={id}
          onChange={(e) => setId(e.target.value)}
          required
          hint={text.idHint}
        />
        <Input
          label={text.appName}
          value={name}
          onChange={(e) => setName(e.target.value)}
          required
          maxLength={120}
        />
        <Input
          label={text.executable}
          value={executable}
          onChange={(e) => setExecutable(e.target.value)}
          required
          hint={text.exeHint}
        />
        <label className={styles.jsonField} htmlFor={`${field}-args`}>
          {text.args}
          <textarea
            id={`${field}-args`}
            value={args}
            onChange={(e) => setArgs(e.target.value)}
            rows={3}
            spellCheck={false}
          />
        </label>
        <label className={styles.jsonField} htmlFor={`${field}-env`}>
          {text.env}
          <textarea
            id={`${field}-env`}
            value={env}
            onChange={(e) => setEnv(e.target.value)}
            rows={3}
            spellCheck={false}
          />
        </label>
        <Input
          label={text.match}
          value={matchArgument}
          onChange={(e) => setMatchArgument(e.target.value)}
          hint={text.matchHint}
        />
        <h3>{text.preview}</h3>
        <p className={styles.muted}>{text.previewHint}</p>
        {previewError ? (
          <p role="status">{previewError}</p>
        ) : plan ? (
          <pre className={styles.report}>
            {JSON.stringify(
              {
                commandLine: plan.commandLine,
                executable: plan.app.executable,
                args: plan.args,
                env: plan.env,
              },
              null,
              2,
            )}
          </pre>
        ) : (
          <p role="status">{text.resolving}</p>
        )}
        {error && <p role="alert">{error}</p>}
        <div className={styles.inline}>
          <Button
            type="submit"
            size="M"
            disabled={busy || !plan || Boolean(previewError)}
          >
            {busy ? text.saving : text.saveCustom}
          </Button>
          <Button size="M" kind="quiet" onClick={onClose} disabled={busy}>
            {text.cancel}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
