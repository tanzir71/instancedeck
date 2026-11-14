import { useEffect, useRef, useState, type ReactNode } from 'react';
import { useStore } from 'zustand';
import { createDesktopStore, type Screen } from './store';
import { Button } from '../components/Button';
import { Dialog, Input, Kbd, Toast } from '../components/Controls';
import {
  AddProfileCard,
  AppGroupHeader,
  EmptyState,
  MainInstanceCard,
  ProfileCard,
} from '../components/ProfileCard';
import { Logo } from '../components/Logo';
import {
  AppsScreen,
  DiagnosticsScreen,
  ProfileDetail,
  SettingsScreen,
} from './ScreenViews';
import { Wizard } from './Wizard';
import { FirstRunDialog } from './FirstRunDialog';
import { AccountGuidance, SignInFlow } from './SignInFlow';
import { profileDetail } from './profileDetail';
import type {
  DesktopClient,
  DesktopSnapshot,
  Profile,
  ProfileAction,
} from './model';
import copy from '../copy/en.json';
import styles from './Desktop.module.css';
import deck from '../../../../design/assets/icons/deck.svg?url';
import apps from '../../../../design/assets/icons/apps.svg?url';
import diagnostics from '../../../../design/assets/icons/diagnostics.svg?url';
import settings from '../../../../design/assets/icons/settings.svg?url';

export type { Screen } from './store';
export function DesktopShell({
  screen,
  onNavigate,
  onWindow,
  children,
  preview = false,
  switcherHotkey = 'Ctrl+Alt+Space',
}: {
  screen: Screen;
  onNavigate: (screen: Screen) => void;
  onWindow: DesktopClient['window'];
  children: ReactNode;
  preview?: boolean;
  switcherHotkey?: string;
}) {
  return (
    <div className={styles.shell}>
      <header className={styles.titlebar}>
        <div data-tauri-drag-region className={styles.drag}>
          <Logo />
        </div>
        <div className={styles.windowControls}>
          {(['minimize', 'maximize', 'close'] as const).map((action, index) => (
            <button
              type="button"
              key={action}
              aria-label={
                copy.desktop[action === 'close' ? 'closeWindow' : action]
              }
              onClick={() => {
                void onWindow(action);
              }}
            >
              {['–', '□', '×'][index]}
            </button>
          ))}
        </div>
      </header>
      <div className={styles.body}>
        <aside className={styles.sidebar}>
          <nav aria-label={copy.brand}>
            {(
              [
                ['deck', deck],
                ['apps', apps],
                ['diagnostics', diagnostics],
                ['settings', settings],
              ] as const
            ).map(([key, icon]) => (
              <button
                type="button"
                key={key}
                className={screen === key ? styles.selected : ''}
                aria-current={screen === key ? 'page' : undefined}
                onClick={() => onNavigate(key)}
              >
                <span
                  className={styles.navIcon}
                  style={{ maskImage: `url(${icon})` }}
                  aria-hidden="true"
                />
                {copy[key]}
              </button>
            ))}
          </nav>
          <div className={styles.switcherHint}>
            <span>{copy.switcher}</span>
            <div>
              {switcherHotkey.split('+').map((key) => (
                <Kbd key={key}>{key}</Kbd>
              ))}
            </div>
          </div>
        </aside>
        <main className={styles.main}>
          {preview && (
            <p className={styles.previewBanner}>{copy.desktop.previewBanner}</p>
          )}
          {children}
        </main>
      </div>
    </div>
  );
}

export function DeckScreen({
  snapshot,
  search,
  onSearch,
  onNew,
  onProfileAction,
  onDetails,
  onMainAction,
  onHealth,
  busy = {},
  referenceTime,
}: {
  snapshot: DesktopSnapshot;
  search: string;
  onSearch: (value: string) => void;
  onNew: (appId?: string) => void;
  onProfileAction: (action: ProfileAction, profile: Profile) => void;
  onDetails: (profile: Profile) => void;
  onMainAction: (appId: string, action: 'launch' | 'close') => void;
  onHealth: () => void;
  busy?: Record<string, boolean>;
  referenceTime?: number;
}) {
  const searchRef = useRef<HTMLInputElement>(null);
  const [clock, setClock] = useState(Date.now);
  useEffect(() => {
    if (referenceTime !== undefined) return;
    const timer = window.setInterval(() => setClock(Date.now()), 60000);
    return () => window.clearInterval(timer);
  }, [referenceTime]);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.ctrlKey && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        searchRef.current?.focus();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);
  const count =
    snapshot.profiles.length +
    snapshot.apps.filter((app) => app.installed).length;
  const runningCount =
    Object.values(snapshot.running).filter((pids) => pids.length).length +
    Object.values(snapshot.mainRunning).filter((pids) => pids.length).length;
  const filtered = snapshot.profiles.filter((profile) =>
    `${profile.name} ${snapshot.apps.find((app) => app.recipe.id === profile.appId)?.recipe.name ?? profile.appId}`
      .toLocaleLowerCase()
      .includes(search.toLocaleLowerCase()),
  );
  return (
    <>
      <div className={styles.deckHeader}>
        <div>
          <h1>{copy.yourDeck}</h1>
          <p>
            {copy.desktop.profilesMeta
              .replace('{count}', String(count))
              .replace('{running}', String(runningCount))
              .replace('{root}', snapshot.settings.profilesRoot)}
          </p>
        </div>
        <div className={styles.headerActions}>
          <div className={styles.search}>
            <input
              ref={searchRef}
              type="search"
              aria-label={copy.desktop.search}
              placeholder={copy.desktop.search}
              value={search}
              onChange={(event) => onSearch(event.target.value)}
            />
            <Kbd>Ctrl K</Kbd>
          </div>
          <Button size="M" onClick={() => onNew()}>
            {copy.desktop.newProfile}
          </Button>
        </div>
      </div>
      {snapshot.profiles.length === 0 && !search ? (
        <EmptyState
          title={copy.emptyTitle}
          body={copy.emptyBody}
          action={{ label: copy.desktop.newProfile, onClick: () => onNew() }}
        />
      ) : search && filtered.length === 0 ? (
        <EmptyState
          title={copy.desktop.noResults}
          body={copy.desktop.noResultsBody}
        />
      ) : (
        snapshot.apps
          .filter(
            (app) =>
              app.installed ||
              filtered.some((profile) => profile.appId === app.recipe.id),
          )
          .map((app) => (
            <section
              key={app.recipe.id}
              className={styles.group}
              aria-label={app.recipe.name}
            >
              <AppGroupHeader app={app} onHealthCheck={onHealth} />
              <div className={styles.cards}>
                {!search && app.installed && (
                  <MainInstanceCard
                    appId={app.recipe.id}
                    running={Boolean(
                      snapshot.mainRunning[app.recipe.id]?.length,
                    )}
                    busy={busy[`main:${app.recipe.id}`]}
                    onLaunch={() => onMainAction(app.recipe.id, 'launch')}
                  />
                )}
                {filtered
                  .filter((profile) => profile.appId === app.recipe.id)
                  .map((profile) => (
                    <div key={profile.id}>
                      <ProfileCard
                        profile={profile}
                        running={Boolean(snapshot.running[profile.id]?.length)}
                        detail={profileDetail(
                          profile,
                          Boolean(snapshot.running[profile.id]?.length),
                          snapshot.runningSince?.[profile.id],
                          referenceTime ?? clock,
                        )}
                        busy={busy[profile.id]}
                        onLaunch={() => onProfileAction('launch', profile)}
                        onClose={() => onProfileAction('close', profile)}
                        onDetails={() => onDetails(profile)}
                      />
                      {snapshot.statusErrors[profile.id] && (
                        <p className={styles.statusError}>
                          {copy.desktop.statusUnavailable}:{' '}
                          {snapshot.statusErrors[profile.id]}
                        </p>
                      )}
                    </div>
                  ))}
                {!search &&
                  app.recipe.id === snapshot.apps.at(-1)?.recipe.id && (
                    <AddProfileCard onClick={() => onNew(app.recipe.id)} />
                  )}
              </div>
            </section>
          ))
      )}
    </>
  );
}

export function ConfirmProfileAction({
  profile,
  action,
  busy,
  error,
  onClose,
  onConfirm,
}: {
  profile: Profile;
  action: ProfileAction;
  busy: boolean;
  error?: string;
  onClose: () => void;
  onConfirm: (confirmation: string) => void;
}) {
  const [confirmation, setConfirmation] = useState('');
  const force = action === 'force-close';
  const recycle = action === 'recycle';
  return (
    <Dialog
      title={
        force
          ? copy.desktop.forceTitle
          : recycle
            ? copy.desktop.recycle
            : copy.desktop.remove
      }
      onClose={() => !busy && onClose()}
      className={styles.confirm}
    >
      <h2>
        {force
          ? copy.desktop.forceTitle
          : recycle
            ? copy.desktop.recycle
            : copy.desktop.remove}
      </h2>
      <p>
        {force
          ? copy.desktop.forceBody
          : recycle
            ? copy.desktop.recycleHelp
            : copy.desktop.removeHelp}
      </p>
      <p>
        <strong>{profile.name}</strong>
      </p>
      <Input
        label={copy.desktop.typeName}
        value={confirmation}
        onChange={(event) => setConfirmation(event.target.value)}
        autoComplete="off"
      />
      {error && <p role="alert">{error}</p>}
      <div className={styles.dialogActions}>
        <Button size="M" kind="secondary" onClick={onClose} disabled={busy}>
          {copy.desktop.cancel}
        </Button>
        <Button
          size="M"
          kind={recycle || force ? 'danger' : 'primary'}
          disabled={busy || confirmation !== profile.name}
          onClick={() => onConfirm(confirmation)}
        >
          {busy
            ? copy.desktop.busy
            : force
              ? copy.desktop.forceClose
              : recycle
                ? copy.desktop.recycle
                : copy.desktop.remove}
        </Button>
      </div>
    </Dialog>
  );
}

export function DesktopApp({
  client,
  initialScreen = 'deck',
}: {
  client: DesktopClient;
  initialScreen?: Screen;
}) {
  const [store] = useState(() => createDesktopStore(initialScreen));
  const snapshot = useStore(store, (state) => state.snapshot);
  const error = useStore(store, (state) => state.error);
  const screen = useStore(store, (state) => state.screen);
  const { setSnapshot, setScreen, setError } = store.getState();
  const [search, setSearch] = useState('');
  const [wizard, setWizard] = useState<{ appId?: string }>();
  const [detail, setDetail] = useState<string>();
  const [busy, setBusy] = useState<Record<string, boolean>>({});
  const [toast, setToast] = useState('');
  const [confirmation, setConfirmation] = useState<{
    profile: Profile;
    action: ProfileAction;
  }>();
  const [confirmError, setConfirmError] = useState('');
  const [mainClose, setMainClose] = useState<string>();
  const [signInProfile, setSignInProfile] = useState<Profile>();
  const [accountProfile, setAccountProfile] = useState<Profile>();
  const reload = async (refresh = false) => {
    const value = await client.snapshot(refresh);
    setSnapshot(value);
    setError('');
  };
  useEffect(() => {
    let active = true;
    let unsubscribe: (() => void) | undefined;
    client
      .snapshot()
      .then((value) => {
        if (active) setSnapshot(value);
      })
      .catch((error) => {
        if (active) setError(String(error));
      });
    client
      .subscribe(
        (value) => {
          if (active) {
            setSnapshot(value);
            setError('');
          }
        },
        (error) => {
          if (active) setError(error);
        },
      )
      .then((stop) => {
        if (active) unsubscribe = stop;
        else stop();
      })
      .catch((error) => {
        if (active) setError(String(error));
      });
    return () => {
      active = false;
      unsubscribe?.();
    };
  }, [client, setSnapshot, setError]);
  const act = async (
    action: ProfileAction,
    profile: Profile,
    typedName?: string,
  ) => {
    if (
      ['remove', 'recycle', 'force-close'].includes(action) &&
      typedName === undefined
    ) {
      setConfirmError('');
      setConfirmation({ profile, action });
      return;
    }
    setBusy((previous) => ({ ...previous, [profile.id]: true }));
    try {
      await client.action(profile.id, action, typedName);
      setConfirmation(undefined);
      setConfirmError('');
      if (action === 'remove' || action === 'recycle') setDetail(undefined);
      await reload();
      setToast(copy.desktop.actionDone);
    } catch (error) {
      const message = String(error);
      if (
        action === 'close' &&
        message.toLowerCase().includes('still running')
      ) {
        setConfirmError('');
        setConfirmation({ profile, action: 'force-close' });
      } else if (typedName !== undefined) setConfirmError(message);
      else setError(message);
    } finally {
      setBusy((previous) => ({ ...previous, [profile.id]: false }));
    }
  };
  const mainAction = async (appId: string, action: 'launch' | 'close') => {
    setBusy((previous) => ({ ...previous, [`main:${appId}`]: true }));
    try {
      await client.mainAction(appId, action);
      setMainClose(undefined);
      await reload();
    } catch (error) {
      setError(String(error));
    } finally {
      setBusy((previous) => ({ ...previous, [`main:${appId}`]: false }));
    }
  };
  const selectedProfile = snapshot?.profiles.find(
    (profile) => profile.id === detail,
  );
  return (
    <DesktopShell
      screen={screen}
      onNavigate={setScreen}
      onWindow={async (action) => {
        try {
          await client.window(action);
        } catch (error) {
          setError(String(error));
        }
      }}
      preview={client.preview}
      switcherHotkey={snapshot?.settings.switcherHotkey}
    >
      {error && (
        <div className={styles.errorBanner} role="alert">
          <p>{error}</p>
          <Button
            size="M"
            kind="secondary"
            onClick={() => {
              void reload(true).catch((error) => setError(String(error)));
            }}
          >
            {copy.desktop.retry}
          </Button>
        </div>
      )}
      {!snapshot ? (
        <section className={styles.loading} role="status">
          <h1>{error ? copy.desktop.errorTitle : copy.desktop.loading}</h1>
        </section>
      ) : (
        <>
          {screen === 'deck' ? (
            <DeckScreen
              snapshot={snapshot}
              search={search}
              onSearch={setSearch}
              onNew={(appId) => setWizard({ appId })}
              onProfileAction={(action, profile) => {
                void act(action, profile);
              }}
              onDetails={(profile) => setDetail(profile.id)}
              onMainAction={(appId, action) =>
                action === 'close'
                  ? setMainClose(appId)
                  : void mainAction(appId, action)
              }
              onHealth={() => setScreen('diagnostics')}
              busy={busy}
            />
          ) : screen === 'apps' ? (
            <AppsScreen
              snapshot={snapshot}
              onRefresh={() => {
                void reload(true).catch((error) => setError(String(error)));
              }}
              onNewProfile={(appId) => setWizard({ appId })}
              onCloseMain={setMainClose}
            />
          ) : screen === 'diagnostics' ? (
            <DiagnosticsScreen
              client={client}
              onCopied={() => setToast(copy.desktop.copied)}
            />
          ) : (
            <SettingsScreen
              key={JSON.stringify(snapshot.settings)}
              settings={snapshot.settings}
              onSave={async (value) => {
                await client.saveSettings(value);
                await reload();
                setToast(copy.desktop.saved);
              }}
              onBrowse={client.browse}
            />
          )}
          {wizard && (
            <Wizard
              apps={snapshot.apps}
              root={snapshot.settings.profilesRoot}
              initialAppId={wizard.appId}
              onClose={() => setWizard(undefined)}
              onBrowse={client.browse}
              onCreate={async (input) => {
                const { shortcuts, ...profileInput } = input;
                const created = await client.create(profileInput);
                let shortcutWarning = '';
                if (shortcuts?.length) {
                  try {
                    await client.shortcuts(created.id, shortcuts);
                  } catch (error) {
                    shortcutWarning = `${copy.desktop.shortcutFailed} ${String(error)}`;
                  }
                }
                await reload();
                setWizard(undefined);
                setToast(shortcutWarning || copy.desktop.created);
                if (created.appId === 'claude-desktop')
                  setSignInProfile(created);
              }}
            />
          )}
          {selectedProfile && (
            <ProfileDetail
              key={selectedProfile.id}
              profile={selectedProfile}
              client={client}
              profiles={snapshot.profiles}
              onSignIn={() => {
                setDetail(undefined);
                setSignInProfile(selectedProfile);
              }}
              onAccountHealth={() => {
                setDetail(undefined);
                setAccountProfile(selectedProfile);
              }}
              onClose={() => setDetail(undefined)}
              onSaved={async () => {
                await reload();
                setToast(copy.desktop.saved);
              }}
              onAction={(action, profile) => {
                void act(action, profile);
              }}
            />
          )}
          {!snapshot.settings.onboardingCompleted && (
            <FirstRunDialog
              settings={snapshot.settings}
              onSave={async (settings) => {
                await client.saveSettings(settings);
                await reload();
              }}
            />
          )}
          <SignInFlow
            client={client}
            snapshot={snapshot}
            profile={signInProfile}
            onDismiss={() => setSignInProfile(undefined)}
            onRefresh={reload}
            onComplete={(id) =>
              setAccountProfile(snapshot.profiles.find((p) => p.id === id))
            }
          />
          {accountProfile && (
            <AccountGuidance
              profile={accountProfile}
              client={client}
              onClose={() => setAccountProfile(undefined)}
              onRefresh={reload}
            />
          )}
        </>
      )}
      {confirmation && (
        <ConfirmProfileAction
          key={`${confirmation.profile.id}:${confirmation.action}`}
          {...confirmation}
          busy={Boolean(busy[confirmation.profile.id])}
          error={confirmError}
          onClose={() => setConfirmation(undefined)}
          onConfirm={(value) => {
            void act(confirmation.action, confirmation.profile, value);
          }}
        />
      )}
      {mainClose && (
        <Dialog
          title={copy.desktop.mainCloseTitle}
          onClose={() => setMainClose(undefined)}
          className={styles.confirm}
        >
          <h2>{copy.desktop.mainCloseTitle}</h2>
          <p>{copy.desktop.mainCloseBody}</p>
          <div className={styles.dialogActions}>
            <Button
              kind="secondary"
              size="M"
              onClick={() => setMainClose(undefined)}
            >
              {copy.desktop.cancel}
            </Button>
            <Button
              size="M"
              disabled={busy[`main:${mainClose}`]}
              onClick={() => {
                void mainAction(mainClose, 'close');
              }}
            >
              {copy.desktop.close}
            </Button>
          </div>
        </Dialog>
      )}
      {toast && <Toast message={toast} onClose={() => setToast('')} />}
    </DesktopShell>
  );
}
