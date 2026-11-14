import type { ReactNode } from 'react';
import { Badge } from '../components/Controls';
import { MonogramTile } from '../components/ProfileCard';
import copy from '../copy/en.json';
import { Eyebrow, siteLink } from './LandingSections';
import styles from './Documentation.module.css';

export interface Breadcrumb {
  label: string;
  href?: string;
}
export function Breadcrumbs({
  items,
  linkPrefix = './',
}: {
  items: Breadcrumb[];
  linkPrefix?: string;
}) {
  return (
    <nav className={styles.breadcrumbs} aria-label={copy.marketing.breadcrumbs}>
      <ol>
        <li>
          <a href={siteLink(linkPrefix)}>{copy.marketing.home}</a>
        </li>
        {items.map((item, index) => (
          <li key={`${item.label}-${index}`}>
            <span aria-hidden="true">/</span>
            {item.href ? (
              <a href={item.href}>{item.label}</a>
            ) : (
              <span aria-current="page">{item.label}</span>
            )}
          </li>
        ))}
      </ol>
    </nav>
  );
}

export function DocLayout({
  title,
  intro,
  eyebrow,
  breadcrumbs,
  linkPrefix = './',
  children,
}: {
  title: string;
  intro?: string;
  eyebrow?: string;
  breadcrumbs: Breadcrumb[];
  linkPrefix?: string;
  children: ReactNode;
}) {
  return (
    <main id="main" className={styles.document}>
      <Breadcrumbs items={breadcrumbs} linkPrefix={linkPrefix} />
      <header className={styles.documentHeader}>
        {eyebrow && <Eyebrow>{eyebrow}</Eyebrow>}
        <h1>{title}</h1>
        {intro && <p>{intro}</p>}
      </header>
      <div className={styles.prose}>{children}</div>
    </main>
  );
}

export function ComparisonTable({
  caption,
  columns,
  rows,
}: {
  caption: string;
  columns: string[];
  rows: { label: string; values: ReactNode[] }[];
}) {
  return (
    <div
      className={styles.tableRegion}
      role="region"
      aria-label={caption}
      tabIndex={0}
    >
      <table>
        <caption>{caption}</caption>
        <thead>
          <tr>
            <th scope="col">{copy.marketing.compatibility}</th>
            {columns.map((column) => (
              <th scope="col" key={column}>
                {column}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.label}>
              <th scope="row">{row.label}</th>
              {row.values.map((value, index) => (
                <td key={index}>{value}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function AppDirectoryCard({
  name,
  letters,
  intro,
  status,
  href,
  note,
}: {
  name: string;
  letters: string;
  intro: string;
  status: 'verified' | 'community' | 'experimental';
  href: string;
  note?: string;
}) {
  return (
    <article className={styles.appCard}>
      <div>
        <MonogramTile letters={letters} tone="neutral" size="L" />
        <Badge status={status} />
      </div>
      <h2>
        <a href={href}>{name}</a>
      </h2>
      <p>{intro}</p>
      {note && <small>{note}</small>}
      <a href={href}>{copy.marketing.readGuide}</a>
    </article>
  );
}
