import { useEffect, useId, useRef, useState } from 'react';
import { IconButton } from '../components/Controls';
import { MonogramTile } from '../components/ProfileCard';
import {
  monogram,
  type DesktopClient,
  type DesktopSnapshot,
  type Profile,
} from './model';
import copy from '../copy/en.json';
import styles from './GlobalSwitcher.module.css';

export function GlobalSwitcher({ client }: { client: DesktopClient }) {
  const [snapshot, setSnapshot] = useState<DesktopSnapshot>();
  const [query, setQuery] = useState('');
  const [selected, setSelected] = useState(0);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  const searchId = useId();
  const listId = useId();
  const optionPrefix = useId();
  useEffect(() => {
    let active = true;
    let stop: (() => void) | undefined;
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
          if (active) setSnapshot(value);
        },
        (error) => {
          if (active) setError(error);
        },
      )
      .then((value) => {
        if (active) stop = value;
        else value();
      })
      .catch((error) => {
        if (active) setError(String(error));
      });
    input.current?.focus();
    return () => {
      active = false;
      stop?.();
    };
  }, [client]);
  const filtered =
    snapshot?.profiles.filter((profile) =>
      `${profile.name} ${snapshot.apps.find((app) => app.recipe.id === profile.appId)?.recipe.name ?? profile.appId}`
        .toLocaleLowerCase()
        .includes(query.toLocaleLowerCase()),
    ) ?? [];
  const activeIndex = Math.min(selected, Math.max(0, filtered.length - 1));
  useEffect(() => {
    document
      .getElementById(`${optionPrefix}-${activeIndex}`)
      ?.scrollIntoView({ block: 'nearest' });
  }, [activeIndex, query, optionPrefix]);
  const close = async () => {
    try {
      await client.window('close');
    } catch (error) {
      setError(String(error));
    }
  };
  const launch = async (profile: Profile) => {
    setBusy(true);
    setError('');
    try {
      await client.action(profile.id, 'launch');
      await close();
    } catch (error) {
      setError(String(error));
    } finally {
      setBusy(false);
    }
  };
  return (
    <main
      className={styles.palette}
      onKeyDown={(event) => {
        if (event.key === 'Escape' && !event.nativeEvent.isComposing) {
          event.preventDefault();
          void close();
        }
      }}
    >
      <header data-tauri-drag-region>
        <h1>{copy.switcher}</h1>
        <IconButton
          label={copy.desktop.dismiss}
          onClick={() => {
            void close();
          }}
        >
          ×
        </IconButton>
      </header>
      <label htmlFor={searchId}>{copy.desktop.switcherSearch}</label>
      <input
        id={searchId}
        ref={input}
        autoComplete="off"
        role="combobox"
        aria-expanded="true"
        aria-controls={listId}
        aria-autocomplete="list"
        aria-activedescendant={
          filtered[activeIndex] ? `${optionPrefix}-${activeIndex}` : undefined
        }
        readOnly={busy}
        aria-busy={busy}
        value={query}
        onChange={(event) => {
          setQuery(event.target.value);
          setSelected(0);
        }}
        onKeyDown={(event) => {
          if (busy || event.nativeEvent.isComposing) return;
          if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
            event.preventDefault();
            setSelected((value) =>
              filtered.length
                ? (value +
                    (event.key === 'ArrowDown' ? 1 : -1) +
                    filtered.length) %
                  filtered.length
                : 0,
            );
          }
          if (event.key === 'Enter' && filtered[activeIndex]) {
            event.preventDefault();
            void launch(filtered[activeIndex]);
          }
        }}
      />
      {error && <p role="alert">{error}</p>}
      {!snapshot && !error && <p role="status">{copy.desktop.loading}</p>}
      {snapshot && filtered.length === 0 && <p>{copy.desktop.noResults}</p>}
      <ul
        id={listId}
        role="listbox"
        aria-label={copy.yourDeck}
        className={styles.options}
        aria-busy={busy}
      >
        {filtered.map((profile, index) => (
          <li
            id={`${optionPrefix}-${index}`}
            key={profile.id}
            role="option"
            aria-selected={index === activeIndex}
            onMouseMove={() => setSelected(index)}
            onClick={() => {
              if (!busy) void launch(profile);
            }}
          >
            <span
              className={styles.mark}
              style={{ background: `var(--profile-${profile.color})` }}
            >
              <MonogramTile letters={monogram(profile.appId)} size="S" />
            </span>
            <span>
              <strong>{profile.name}</strong>
              <small>
                {snapshot?.apps.find((app) => app.recipe.id === profile.appId)
                  ?.recipe.name ?? profile.appId}
              </small>
            </span>
            <small>
              {snapshot?.running[profile.id]?.length
                ? copy.desktop.focus
                : copy.desktop.launch}
            </small>
          </li>
        ))}
      </ul>
      <p className={styles.hint}>
        {busy ? copy.desktop.busy : copy.desktop.switcherKeys}
      </p>
    </main>
  );
}
