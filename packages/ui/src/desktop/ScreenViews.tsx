import { useEffect, useState } from 'react';
import { Button } from '../components/Button';
import {
  Badge,
  ColorSwatchPicker,
  Input,
  KeyValueList,
  LogTable,
  Select,
  Sheet,
  Switch,
  Tabs,
} from '../components/Controls';
import { MonogramTile } from '../components/ProfileCard';
import {
  monogram,
  type DesktopClient,
  type DesktopSnapshot,
  type LaunchPlan,
  type Profile,
  type ProfileAction,
  type Settings,
} from './model';
import copy from '../copy/en.json';
import styles from './Desktop.module.css';
import { SharedHistory } from './SharedHistory';

export function AppsScreen({
  snapshot,
  onRefresh,
  onNewProfile,
  onCloseMain,
  onCustom,
}: {
  snapshot: DesktopSnapshot;
  onRefresh: () => void;
  onNewProfile: (appId: string) => void;
  onCloseMain?: (appId: string) => void;
  onCustom?: () => void;
}) {
  return (
    <>
      <div className={styles.pageHeading}>
        <div>
          <h1>{copy.apps}</h1>
          <p>{copy.desktop.appsIntro}</p>
        </div>
        <Button size="M" kind="secondary" onClick={onRefresh}>
          {copy.desktop.refresh}
        </Button>
      </div>
      <p className={styles.muted}>{copy.desktop.compatibility}</p>
      <div className={styles.appList}>
        {snapshot.apps.map((app) => (
          <article className={styles.appRow} key={app.recipe.id}>
            <MonogramTile
              letters={monogram(app.recipe.id)}
              size="L"
              tone="neutral"
            />
            <div className={styles.stack}>
              <div className={styles.inline}>
                <h2>{app.recipe.name}</h2>
                <Badge status={app.recipe.status} />
              </div>
              <KeyValueList
                values={{
                  [copy.desktop.version]:
                    app.installed?.version ?? copy.desktop.notInstalled,
                  [copy.desktop.installPath]:
                    app.installed?.executable ?? app.error ?? '—',
                }}
              />
              <ul className={styles.limitations}>
                {app.recipe.limitations.map((limit) => (
                  <li key={limit}>{limit}</li>
                ))}
              </ul>
            </div>
            <Button
              size="M"
              disabled={!app.installed}
              onClick={() => onNewProfile(app.recipe.id)}
            >
              {copy.desktop.newProfile}
            </Button>
            {onCloseMain &&
              Boolean(snapshot.mainRunning[app.recipe.id]?.length) && (
                <Button
                  size="M"
                  kind="quiet"
                  onClick={() => onCloseMain(app.recipe.id)}
                >
                  {copy.desktop.closeMain}
                </Button>
              )}
          </article>
        ))}
      </div>
      <Button size="M" kind="quiet" disabled={!onCustom} onClick={onCustom}>
        {copy.desktop.customApp}
      </Button>
    </>
  );
}

export function SettingsScreen({
  settings,
  onSave,
  onBrowse,
  onExternal,
}: {
  settings: Settings;
  onSave: (settings: Settings) => Promise<void>;
  onBrowse: () => Promise<string | null>;
  onExternal?: DesktopClient['openExternal'];
}) {
  const [draft, setDraft] = useState(settings);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  return (
    <>
      <div className={styles.pageHeading}>
        <div>
          <h1>{copy.settings}</h1>
          <p>{copy.desktop.settingsIntro}</p>
        </div>
      </div>
      <form
        className={styles.settingsForm}
        onSubmit={async (event) => {
          event.preventDefault();
          setBusy(true);
          setError('');
          try {
            await onSave(draft);
          } catch (error) {
            setError(String(error));
          } finally {
            setBusy(false);
          }
        }}
      >
        <div className={styles.browseField}>
          <Input
            label={copy.desktop.profilesRoot}
            hint={copy.desktop.rootHelp}
            value={draft.profilesRoot}
            onChange={(event) =>
              setDraft({ ...draft, profilesRoot: event.target.value })
            }
          />
          <Button
            size="M"
            kind="secondary"
            onClick={async () => {
              try {
                const path = await onBrowse();
                if (path) setDraft({ ...draft, profilesRoot: path });
              } catch (error) {
                setError(String(error));
              }
            }}
          >
            {copy.desktop.browse}
          </Button>
        </div>
        <Switch
          label={copy.desktop.closeToTray}
          checked={draft.closeToTray}
          onChange={(closeToTray) => setDraft({ ...draft, closeToTray })}
        />
        <Switch
          label={copy.desktop.startWithWindows}
          checked={draft.startWithWindows}
          onChange={(startWithWindows) =>
            setDraft({ ...draft, startWithWindows })
          }
        />
        <Input
          label={copy.desktop.switcherHotkey}
          value={draft.switcherHotkey}
          onChange={(event) =>
            setDraft({ ...draft, switcherHotkey: event.target.value })
          }
        />
        <p className={styles.muted}>{copy.desktop.shellHelp}</p>
        <Switch
          label={copy.desktop.allowUpdates}
          checked={draft.allowUpdateChecks}
          onChange={(allowUpdateChecks) =>
            setDraft({ ...draft, allowUpdateChecks })
          }
        />
        <p className={styles.muted}>{copy.desktop.updatesPending}</p>
        <Select
          label={copy.desktop.updateChannel}
          value={draft.updateChannel}
          onChange={(event) =>
            setDraft({
              ...draft,
              updateChannel: event.target.value as Settings['updateChannel'],
            })
          }
        >
          <option value="stable">{copy.desktop.stable}</option>
          <option value="beta">{copy.desktop.beta}</option>
        </Select>
        <Select
          label={copy.desktop.theme}
          value={draft.theme}
          onChange={(event) =>
            setDraft({
              ...draft,
              theme: event.target.value as Settings['theme'],
            })
          }
        >
          <option value="light">{copy.desktop.light}</option>
          <option value="dark">{copy.desktop.dark}</option>
        </Select>
        {error && <p role="alert">{error}</p>}
        <Button type="submit" size="M" disabled={busy}>
          {busy ? copy.desktop.busy : copy.desktop.save}
        </Button>
      </form>
      <section className={styles.about}>
        <h2>{copy.desktop.about}</h2>
        <p>{copy.descriptor}</p>
        <p>{copy.nonAffiliation}</p>
        <p>
          {copy.creditPrefix}{' '}
          <a
            href="https://tanziro.com"
            onClick={(event) => {
              if (onExternal) {
                event.preventDefault();
                void onExternal('credit').catch((e) => setError(String(e)));
              }
            }}
          >
            {copy.creditName} · {copy.creditDomain}
          </a>
        </p>
      </section>
    </>
  );
}

export function DiagnosticsScreen({
  client,
  onCopied,
}: {
  client: DesktopClient;
  onCopied: () => void;
}) {
  const [redact, setRedact] = useState(true);
  const [report, setReport] = useState<Record<string, unknown>>();
  const [error, setError] = useState('');
  const [tab, setTab] = useState('environment');
  const [request, setRequest] = useState(0);
  useEffect(() => {
    let active = true;
    setReport(undefined);
    client
      .diagnostics(redact)
      .then((value) => {
        if (active) {
          setReport(value);
          setError('');
        }
      })
      .catch((error) => {
        if (active) setError(String(error));
      });
    return () => {
      active = false;
    };
  }, [client, redact, request]);
  return (
    <>
      <div className={styles.pageHeading}>
        <div>
          <h1>{copy.diagnostics}</h1>
          <p>{copy.desktop.diagnosticsIntro}</p>
        </div>
        <Button
          kind="secondary"
          size="M"
          onClick={() => setRequest(request + 1)}
        >
          {copy.desktop.refresh}
        </Button>
      </div>
      <div className={styles.inline}>
        <Switch
          label={copy.desktop.redact}
          checked={redact}
          onChange={setRedact}
        />
        <Button
          size="M"
          disabled={!report}
          onClick={async () => {
            try {
              await navigator.clipboard.writeText(
                JSON.stringify(report, null, 2),
              );
              onCopied();
            } catch (error) {
              setError(String(error));
            }
          }}
        >
          {copy.desktop.copyReport}
        </Button>
      </div>
      <Tabs
        value={tab}
        onChange={setTab}
        items={[
          { id: 'environment', label: copy.desktop.environment },
          { id: 'logs', label: copy.desktop.recentLaunches },
          { id: 'account', label: copy.desktop.accountHealth },
        ]}
      />
      {error && <p role="alert">{error}</p>}
      {!report && !error && <p role="status">{copy.desktop.loading}</p>}
      {report &&
        (tab === 'logs' ? (
          <LogTable
            rows={(report.launchLogs ?? []) as Record<string, unknown>[]}
          />
        ) : tab === 'account' ? (
          <div className={styles.stack}>
            <h2>{copy.desktop.accountHealth}</h2>
            <p>{copy.desktop.accountHealthHelp}</p>
            <pre className={styles.report}>
              {JSON.stringify(report.profiles ?? [], null, 2)}
            </pre>
          </div>
        ) : (
          <div className={styles.stack}>
            <KeyValueList
              values={{
                [copy.desktop.environment]: JSON.stringify(
                  report.directories ?? {},
                ),
                [copy.desktop.layouts]: JSON.stringify(
                  report.claudeLayouts ?? [],
                ),
              }}
            />
            <details>
              <summary>{copy.desktop.report}</summary>
              <pre className={styles.report}>
                {JSON.stringify(report, null, 2)}
              </pre>
            </details>
          </div>
        ))}
    </>
  );
}

export function ProfileDetail({
  profile,
  client,
  onClose,
  onSaved,
  onAction,
  onSignIn,
  onAccountHealth,
  profiles = [],
}: {
  profile: Profile;
  client: DesktopClient;
  onClose: () => void;
  onSaved: () => Promise<void>;
  onAction: (action: ProfileAction, profile: Profile) => void;
  onSignIn?: () => void;
  onAccountHealth?: () => void;
  profiles?: Profile[];
}) {
  const [draft, setDraft] = useState(profile);
  const [plan, setPlan] = useState<LaunchPlan>();
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [mcpSource, setMcpSource] = useState('');
  useEffect(() => {
    let active = true;
    client
      .plan(profile.id)
      .then((plan) => {
        if (active) setPlan(plan);
      })
      .catch((error) => {
        if (active) setError(String(error));
      });
    return () => {
      active = false;
    };
  }, [client, profile.id]);
  return (
    <Sheet title={copy.desktop.details} onClose={onClose}>
      <form
        className={styles.stack}
        onSubmit={async (event) => {
          event.preventDefault();
          setBusy(true);
          setError('');
          try {
            await client.update(draft);
            await onSaved();
          } catch (error) {
            setError(String(error));
          } finally {
            setBusy(false);
          }
        }}
      >
        <Input
          label={copy.desktop.profileName}
          value={draft.name}
          maxLength={120}
          onChange={(event) => setDraft({ ...draft, name: event.target.value })}
          required
        />
        <ColorSwatchPicker
          value={draft.color}
          onChange={(color) => setDraft({ ...draft, color })}
        />
        <KeyValueList
          values={{
            [copy.desktop.dataFolder]: profile.dataDir,
            [copy.desktop.accountHealth]:
              profile.signInState === 'signed-in'
                ? copy.desktop.signedIn
                : profile.signInState === 'needs-sign-in'
                  ? copy.desktop.notSignedIn
                  : copy.desktop.unknownSignIn,
          }}
        />
        <Button
          kind="secondary"
          size="M"
          onClick={() => onAction('open-folder', profile)}
        >
          {copy.desktop.openFolder}
        </Button>
        {profile.appId === 'claude-desktop' && (
          <Switch
            label={copy.desktop.isolateClaude}
            checked={draft.options.isolateClaudeCode}
            onChange={(isolateClaudeCode) =>
              setDraft({
                ...draft,
                options: { ...draft.options, isolateClaudeCode },
              })
            }
          />
        )}
        <Input
          label={copy.desktop.hotkey}
          value={draft.hotkey ?? ''}
          onChange={(event) =>
            setDraft({ ...draft, hotkey: event.target.value.trim() || null })
          }
          hint={copy.desktop.hotkeyHelp}
        />
        <Button
          kind="secondary"
          size="M"
          disabled={busy}
          onClick={async () => {
            setBusy(true);
            setError('');
            try {
              await client.shortcuts(profile.id, ['desktop', 'start-menu']);
              await onSaved();
            } catch (error) {
              setError(String(error));
            } finally {
              setBusy(false);
            }
          }}
        >
          {copy.desktop.createShortcuts}
        </Button>
        <p className={styles.muted}>{copy.desktop.shortcutHelp}</p>
        <p className={styles.muted}>{copy.desktop.pinTip}</p>
        {(profile.appId === 'claude-desktop' || profile.appId === 'codex') && (
          <>
            <Button
              size="M"
              kind="quiet"
              onClick={() =>
                void client
                  .openExternal(
                    profile.appId === 'codex' ? 'codex-usage' : 'claude-usage',
                  )
                  .catch((e) => setError(String(e)))
              }
            >
              {copy.updates.usage}
            </Button>
            <p className={styles.muted}>{copy.updates.usageHelp}</p>
          </>
        )}
        <details>
          <summary>{copy.desktop.launchSettings}</summary>
          {plan && (
            <pre className={styles.report}>{JSON.stringify(plan, null, 2)}</pre>
          )}
        </details>
        {profile.appId === 'claude-desktop' && (
          <>
            <Button
              kind="secondary"
              size="M"
              disabled={!onSignIn}
              onClick={onSignIn}
            >
              {copy.desktop.signInNow}
            </Button>
            <Select
              label={copy.desktop.mcpSource}
              value={mcpSource}
              onChange={(event) => setMcpSource(event.target.value)}
            >
              <option value="">{copy.desktop.mcpSource}</option>
              {profiles
                .filter(
                  (p) => p.appId === 'claude-desktop' && p.id !== profile.id,
                )
                .map((p) => (
                  <option value={p.id} key={p.id}>
                    {p.name}
                  </option>
                ))}
            </Select>
            <p className={styles.muted}>{copy.desktop.mcpCopyHelp}</p>
            <Button
              kind="quiet"
              size="M"
              disabled={busy || !mcpSource}
              onClick={async () => {
                setBusy(true);
                setError('');
                try {
                  await client.copyMcp(mcpSource, profile.id);
                  await onSaved();
                } catch (error) {
                  setError(String(error));
                } finally {
                  setBusy(false);
                }
              }}
            >
              {copy.desktop.mcpCopy}
            </Button>
            <p className={styles.muted}>{copy.desktop.signInPending}</p>
          </>
        )}
        {profile.appId === 'codex' && (
          <SharedHistory client={client} profile={profile} onSaved={onSaved} />
        )}
        <Button
          kind="secondary"
          size="M"
          disabled={!onAccountHealth}
          onClick={onAccountHealth}
        >
          {copy.desktop.accountHealth}
        </Button>
        {error && <p role="alert">{error}</p>}
        <Button type="submit" size="M" disabled={busy}>
          {busy ? copy.desktop.busy : copy.desktop.save}
        </Button>
        <div className={styles.dangerZone}>
          <h3>{copy.desktop.remove}</h3>
          <p>{copy.desktop.removeHelp}</p>
          <Button
            size="M"
            kind="secondary"
            onClick={() => onAction('remove', profile)}
          >
            {copy.desktop.remove}
          </Button>
          <h3>{copy.desktop.recycle}</h3>
          <p>{copy.desktop.recycleHelp}</p>
          <Button
            size="M"
            kind="danger"
            onClick={() => onAction('recycle', profile)}
          >
            {copy.desktop.recycle}
          </Button>
        </div>
      </form>
    </Sheet>
  );
}
