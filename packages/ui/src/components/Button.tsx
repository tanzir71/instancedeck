import type { ButtonHTMLAttributes } from 'react';
import styles from './Button.module.css';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  kind?: 'primary' | 'secondary' | 'inverse' | 'onColor' | 'quiet' | 'danger';
  size?: 'L' | 'M';
}
export function Button({
  kind = 'primary',
  size = 'L',
  className = '',
  type = 'button',
  ...props
}: ButtonProps) {
  return (
    <button
      {...props}
      type={type}
      className={`${styles.button} ${styles[kind]} ${styles[size]} ${className}`}
    />
  );
}
