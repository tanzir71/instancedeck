import mark from '../../../../design/assets/logo-mark.svg?url';
import darkMark from '../../../../design/assets/logo-mark-on-dark.svg?url';
import copy from '../copy/en.json';
import styles from './Logo.module.css';

export function Logo({ onDark = false }: { onDark?: boolean }) {
  return (
    <span className={`${styles.logo} ${onDark ? styles.dark : ''}`}>
      <img src={onDark ? darkMark : mark} alt="" width="22" height="22" />
      {copy.brand}
    </span>
  );
}
