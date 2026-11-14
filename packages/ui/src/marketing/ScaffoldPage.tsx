import { Logo } from '../components/Logo';
import copy from '../copy/en.json';
import styles from './ScaffoldPage.module.css';

export function SiteFooter() {
  return (
    <footer className={styles.footer}>
      <Logo onDark />
      <p>{copy.footerTagline}</p>
      <p className={styles.credit}>
        {copy.creditPrefix} <a href="https://tanziro.com">{copy.creditName}</a>{' '}
        · <a href="https://tanziro.com">{copy.creditDomain}</a>
      </p>
      <p className={styles.legal}>{copy.nonAffiliation}</p>
    </footer>
  );
}
export function ScaffoldPage() {
  return (
    <>
      <header className={styles.header}>
        <Logo />
      </header>
      <main className={styles.main}>
        <p className={styles.eyebrow}>{copy.descriptor}</p>
        <h1>{copy.sitePreview}</h1>
        <p>{copy.siteBody}</p>
        <a href="https://github.com/tanzir71/instancedeck">{copy.source} →</a>
      </main>
      <SiteFooter />
    </>
  );
}
