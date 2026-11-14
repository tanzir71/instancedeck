import { useId, type CSSProperties, type ReactNode } from 'react';
import { ButtonLink } from '../components/Button';
import { keepDialogFocus } from '../components/dialogFocus';
import { Logo } from '../components/Logo';
import { MonogramTile, StatusDot } from '../components/ProfileCard';
import copy from '../copy/en.json';
import styles from './LandingSections.module.css';

const text = copy.marketing;
const sourceUrl = 'https://github.com/tanzir71/instancedeck';
export function siteLink(prefix: string, path = '') {
  return `${prefix}${path}`;
}
export function Eyebrow({
  children,
  onDark = false,
}: {
  children: ReactNode;
  onDark?: boolean;
}) {
  return (
    <p className={`${styles.eyebrow} ${onDark ? styles.onDark : ''}`}>
      <span aria-hidden="true" />
      {children}
    </p>
  );
}
export function SiteHeader({
  linkPrefix = './',
  scrolled = false,
}: {
  linkPrefix?: string;
  scrolled?: boolean;
}) {
  const menuId = useId();
  const links = [
    [text.product, '#product'],
    [text.how, 'how-it-works/'],
    [text.apps, 'apps/'],
    [text.security, 'security/'],
    [text.docs, 'docs/'],
  ];
  return (
    <header className={styles.header} data-scrolled={scrolled ? '' : undefined}>
      <a
        className={styles.brand}
        href={siteLink(linkPrefix)}
        aria-label={copy.brand}
      >
        <Logo />
      </a>
      <nav className={styles.desktopNav} aria-label={text.navigation}>
        {links.map(([label, path]) => (
          <a key={path} href={siteLink(linkPrefix, path)}>
            {label}
          </a>
        ))}
      </nav>
      <div className={styles.headerActions}>
        <a href={sourceUrl}>{text.github}</a>
        <ButtonLink size="M" href={siteLink(linkPrefix, 'download/')}>
          {text.download}
        </ButtonLink>
      </div>
      <button
        className={styles.menuButton}
        type="button"
        data-site-menu-open
        aria-controls={menuId}
        aria-haspopup="dialog"
        onClick={(event) =>
          (
            event.currentTarget.parentElement?.querySelector(
              'dialog',
            ) as HTMLDialogElement | null
          )?.showModal()
        }
      >
        {text.menu}
      </button>
      <dialog
        id={menuId}
        className={styles.mobileMenu}
        aria-label={text.navigation}
        onKeyDown={(event) => keepDialogFocus(event.currentTarget, event)}
      >
        <div>
          <Logo />
          <button
            type="button"
            data-site-menu-close
            onClick={(event) => event.currentTarget.closest('dialog')?.close()}
          >
            {text.closeMenu}
          </button>
        </div>
        <nav>
          {links.map(([label, path]) => (
            <a
              key={path}
              href={siteLink(linkPrefix, path)}
              onClick={(event) =>
                event.currentTarget.closest('dialog')?.close()
              }
            >
              {label}
            </a>
          ))}
          <a href={sourceUrl}>{text.github}</a>
          <ButtonLink href={siteLink(linkPrefix, 'download/')}>
            {text.downloadWindows}
          </ButtonLink>
        </nav>
      </dialog>
    </header>
  );
}
export function InstanceCard({
  name,
  app,
  letters,
  path,
  mobilePath,
  color,
  running = false,
}: {
  name: string;
  app: string;
  letters: string;
  path: string;
  mobilePath?: string;
  color: string;
  running?: boolean;
}) {
  return (
    <div
      className={styles.instanceCard}
      style={{ background: `var(--${color})` }}
    >
      <div>
        <MonogramTile letters={letters} />
        <StatusDot running={running} onColor />
      </div>
      <div>
        <h3>{name}</h3>
        <p>{app}</p>
        <small className={mobilePath ? styles.desktopCopy : undefined}>
          {path}
        </small>
        {mobilePath && (
          <small className={styles.mobileCopy}>{mobilePath}</small>
        )}
      </div>
    </div>
  );
}
const fanCards = text.fanProfiles;
export function HeroFan() {
  return (
    <div className={styles.fanField} role="img" aria-label={text.fanLabel}>
      <div className={styles.fanStage} aria-hidden="true">
        {fanCards.map((card, index) => (
          <div
            className={styles.fanCard}
            style={
              {
                '--fan-step': index - 2,
                '--fan-delay': `${index * 40}ms`,
              } as CSSProperties
            }
            key={card.name}
          >
            <InstanceCard
              {...card}
              path={card.pathParts.join('\\')}
              mobilePath={[
                ...card.pathParts.slice(0, 2),
                card.pathParts.at(-1),
              ].join('\\')}
            />
          </div>
        ))}
      </div>
    </div>
  );
}
export function Hero({ linkPrefix = './' }: { linkPrefix?: string }) {
  return (
    <section id="product" className={styles.hero} aria-labelledby="hero-title">
      <div className={styles.heroCopy}>
        <Eyebrow>{copy.descriptor}</Eyebrow>
        <h1 id="hero-title">{text.heroTitle}</h1>
        <p className={styles.heroBody}>{text.heroBody}</p>
        <div className={styles.heroActions}>
          <ButtonLink href={siteLink(linkPrefix, 'download/')}>
            {text.downloadWindows}
          </ButtonLink>
          <ButtonLink kind="secondary" href={sourceUrl}>
            {text.source}
          </ButtonLink>
        </div>
        <p className={styles.heroMeta}>{text.heroMeta}</p>
      </div>
      <HeroFan />
    </section>
  );
}
const appNames = [
  ['Cl', 'Claude Desktop', 'claude-desktop'],
  ['Cx', 'Codex', 'codex'],
  ['VS', 'VS Code', 'vscode'],
  ['Cu', 'Cursor', 'cursor'],
  ['Ch', 'Chrome', 'chrome'],
  ['Ed', 'Edge', 'edge'],
  ['Br', 'Brave', 'brave'],
  ['Tg', 'Telegram', 'telegram-desktop'],
] as const;
const initialSlugs = ['claude-desktop', 'codex'];
export function LogoTextStrip({
  linkPrefix = './',
  availableSlugs = initialSlugs,
}: {
  linkPrefix?: string;
  availableSlugs?: readonly string[];
}) {
  return (
    <section className={styles.works} aria-label={text.worksWith}>
      <p>{text.worksWith}</p>
      <div>
        {appNames.map(([letters, name, slug]) => (
          <a
            key={name}
            href={siteLink(
              linkPrefix,
              availableSlugs.includes(slug) ? `apps/${slug}/` : 'apps/#planned',
            )}
          >
            <MonogramTile
              letters={letters}
              size="S"
              className={styles.stripTile}
            />
            <span>{name}</span>
          </a>
        ))}
        <a
          className={styles.anyApp}
          href={siteLink(linkPrefix, 'apps/#planned')}
        >
          <MonogramTile letters="+" size="S" tone="ink" />
          <span>{text.anyApp}</span>
        </a>
      </div>
    </section>
  );
}
export function FeatureTriplet() {
  return (
    <section className={styles.pillars} aria-labelledby="pillars-title">
      <div className={styles.pillarsIntro}>
        <div>
          <Eyebrow>{text.why}</Eyebrow>
          <h2 id="pillars-title">
            {text.pillarsTitle.split('. ')[0]}.<br />
            {text.pillarsTitle.split('. ')[1]}
          </h2>
        </div>
        <p>{text.pillarsIntro}</p>
      </div>
      <div className={styles.features}>
        {text.pillars.map((feature, index) => (
          <article key={feature.title}>
            <div
              className={`${styles.featureVisual} ${styles[`visual${index}`]}`}
              aria-hidden="true"
            >
              {index === 0 ? (
                <>
                  <span>work\</span>
                  <span>acme\</span>
                  <span>personal\</span>
                </>
              ) : index === 1 ? (
                <>
                  <p>claude.exe</p>
                  <pre>
                    {
                      '  --user-data-dir=D:\\Profiles\\work\n  LOCALAPPDATA=…\\work\\local\n  CLAUDE_CONFIG_DIR=…\\work\\cc'
                    }
                  </pre>
                  <p>{text.binaryUnmodified}</p>
                </>
              ) : (
                <>
                  <span>0</span>
                  <p>{text.zeroCaption}</p>
                </>
              )}
            </div>
            <h3>{feature.title}</h3>
            <p className={styles.desktopCopy}>{feature.body}</p>
            <p className={styles.mobileCopy}>{feature.mobileBody}</p>
          </article>
        ))}
      </div>
    </section>
  );
}
export function SecurityBand({ linkPrefix = './' }: { linkPrefix?: string }) {
  return (
    <section className={styles.security} aria-labelledby="security-title">
      <div className={styles.securityIntro}>
        <Eyebrow onDark>{text.securityEyebrow}</Eyebrow>
        <h2 id="security-title">{text.securityTitle}</h2>
        <p>{text.securityBody}</p>
        <a href={siteLink(linkPrefix, 'security/')}>{text.securityLink}</a>
      </div>
      <div className={styles.securityFacts}>
        {text.securityFacts.map((fact) => (
          <div key={fact.title}>
            <span aria-hidden="true" />
            <h3 className={styles.desktopCopy}>{fact.title}</h3>
            <p className={styles.desktopCopy}>{fact.body}</p>
            <h3 className={styles.mobileCopy}>{fact.mobile}</h3>
          </div>
        ))}
      </div>
      <a className={styles.mobileCopy} href={siteLink(linkPrefix, 'security/')}>
        {text.securityLink}
      </a>
    </section>
  );
}
export function CTABand({ linkPrefix = './' }: { linkPrefix?: string }) {
  return (
    <section className={styles.cta} aria-labelledby="cta-title">
      <h2 id="cta-title">{text.ctaTitle}</h2>
      <div>
        <ButtonLink kind="inverse" href={siteLink(linkPrefix, 'download/')}>
          {text.downloadWindows}
        </ButtonLink>
        <p>{text.ctaSmall}</p>
      </div>
    </section>
  );
}
const footerGroups = [
  {
    title: text.product,
    links: [
      [text.download, 'download/'],
      [text.how, 'how-it-works/'],
      [text.apps, 'apps/'],
      [text.changelog, 'changelog/'],
    ],
  },
  {
    title: text.resources,
    links: [
      [text.claudeGuide, 'claude-desktop-multiple-accounts/'],
      [text.codexGuide, 'codex-multiple-accounts/'],
      [text.docs, 'docs/'],
      [text.faq, 'faq/'],
    ],
  },
  {
    title: text.project,
    links: [
      [text.github, sourceUrl],
      [text.security, 'security/'],
      [text.signing, 'code-signing-policy/'],
      [text.contributing, 'contributing/'],
    ],
  },
  {
    title: text.legal,
    links: [
      [text.privacy, 'privacy/'],
      [text.license, 'license/'],
      [text.trademarks, 'trademarks/'],
    ],
  },
];
export function MarketingFooter({
  linkPrefix = './',
  year = 2026,
}: {
  linkPrefix?: string;
  year?: number;
}) {
  return (
    <footer className={styles.footer}>
      <div className={styles.footerTop}>
        <div className={styles.footerBrand}>
          <Logo onDark />
          <p>{copy.footerTagline}</p>
        </div>
        <nav className={styles.footerGroups} aria-label={text.footerNavigation}>
          {footerGroups.map((group) => (
            <div key={group.title}>
              <h2>{group.title}</h2>
              {group.links.map(([label, path = '']) => (
                <a
                  key={label}
                  href={
                    path.startsWith('https:')
                      ? path
                      : siteLink(linkPrefix, path)
                  }
                >
                  {label}
                </a>
              ))}
            </div>
          ))}
        </nav>
        <nav
          className={styles.mobileFooterNav}
          aria-label={text.footerNavigation}
        >
          <div>
            <a href={siteLink(linkPrefix, 'download/')}>{text.download}</a>
            <a href={siteLink(linkPrefix, 'how-it-works/')}>{text.how}</a>
            <a href={siteLink(linkPrefix, 'apps/')}>{text.apps}</a>
            <a href={siteLink(linkPrefix, 'security/')}>{text.security}</a>
          </div>
          <div>
            <a href={siteLink(linkPrefix, 'claude-desktop-multiple-accounts/')}>
              {text.claudeGuideShort}
            </a>
            <a href={siteLink(linkPrefix, 'codex-multiple-accounts/')}>
              {text.codexGuide}
            </a>
            <a href={siteLink(linkPrefix, 'faq/')}>{text.faq}</a>
            <a href={sourceUrl}>{text.github}</a>
          </div>
        </nav>
      </div>
      <div className={styles.footerBottom}>
        <div className={styles.legalCopy}>
          <p>{text.copyright.replace('{year}', String(year))}</p>
          <p>{copy.nonAffiliation}</p>
        </div>
        <p className={styles.credit}>
          {copy.creditPrefix}{' '}
          <a href="https://tanziro.com">{copy.creditName}</a> <span>·</span>{' '}
          <a href="https://tanziro.com">{copy.creditDomain}</a>
        </p>
      </div>
    </footer>
  );
}
