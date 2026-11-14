import { Logo } from './Logo';
import copy from '../copy/en.json';
import styles from './EmptyDeck.module.css';
import deck from '../../../../design/assets/icons/deck.svg?url';
import apps from '../../../../design/assets/icons/apps.svg?url';
import diagnostics from '../../../../design/assets/icons/diagnostics.svg?url';
import settings from '../../../../design/assets/icons/settings.svg?url';

export function EmptyDeck() {
  return (
    <div className={styles.shell}>
      <header className={styles.titlebar}>
        <Logo />
      </header>
      <div className={styles.body}>
        <aside className={styles.sidebar} aria-label="Application">
          <nav aria-label="Main navigation">
            {[
              ['deck', deck],
              ['apps', apps],
              ['diagnostics', diagnostics],
              ['settings', settings],
            ].map(([key, icon]) => (
              <div
                key={key}
                className={`${styles.navItem} ${key === 'deck' ? styles.selected : ''}`}
                aria-current={key === 'deck' ? 'page' : undefined}
              >
                <img src={icon} width="16" height="16" alt="" />
                {copy[key as 'deck']}
              </div>
            ))}
          </nav>
          <div className={styles.hint}>
            <span>{copy.switcher}</span>
            <div>
              <kbd>Ctrl</kbd> <kbd>Alt</kbd> <kbd>Space</kbd>
            </div>
          </div>
        </aside>
        <main className={styles.main}>
          <div className={styles.heading}>
            <h1>{copy.yourDeck}</h1>
            <p>{copy.emptyMeta}</p>
          </div>
          <section className={styles.empty} aria-labelledby="empty-title">
            <div className={styles.deckArt} aria-hidden="true">
              <span />
              <span />
              <span />
            </div>
            <h2 id="empty-title">{copy.emptyTitle}</h2>
            <p>{copy.emptyBody}</p>
            <small>{copy.preview}</small>
          </section>
        </main>
      </div>
    </div>
  );
}
