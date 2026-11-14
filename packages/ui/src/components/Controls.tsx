import {
  useId,
  useState,
  type ButtonHTMLAttributes,
  type InputHTMLAttributes,
  type ReactNode,
  type SelectHTMLAttributes,
} from 'react';
import { Button } from './Button';
import * as DialogPrimitive from '@radix-ui/react-dialog';
import { profileColors, type ProfileColor } from '../desktop/model';
import copy from '../copy/en.json';
import styles from './Controls.module.css';
import switchOn from '../assets/switch-on.svg?url';

export function IconButton({
  label,
  children,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { label: string }) {
  return (
    <button
      type="button"
      {...props}
      aria-label={label}
      title={label}
      className={styles.iconButton}
    >
      {children}
    </button>
  );
}
export function Input({
  label,
  hint,
  className = '',
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { label: string; hint?: string }) {
  const generatedId = useId();
  const id = props.id ?? generatedId;
  return (
    <div className={`${styles.field} ${className}`}>
      <label htmlFor={id}>{label}</label>
      <input
        {...props}
        id={id}
        aria-describedby={hint ? `${id}-hint` : props['aria-describedby']}
      />
      {hint && <small id={`${id}-hint`}>{hint}</small>}
    </div>
  );
}
export function Select({
  label,
  children,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement> & { label: string }) {
  const id = useId();
  return (
    <div className={styles.field}>
      <label htmlFor={id}>{label}</label>
      <select {...props} id={id}>
        {children}
      </select>
    </div>
  );
}
export function Switch({
  label,
  checked,
  onChange,
  disabled = false,
}: {
  label: string;
  checked: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <label className={styles.toggle}>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        aria-label={label}
        disabled={disabled}
        onClick={() => onChange(!checked)}
        className={styles.switch}
      >
        {checked ? (
          <img src={switchOn} width="40" height="22" alt="" />
        ) : (
          <span className={styles.switchOff}>
            <span />
          </span>
        )}
      </button>
      <span>{label}</span>
    </label>
  );
}
export function Checkbox({
  label,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { label: string }) {
  return (
    <label className={styles.toggle}>
      <input {...props} type="checkbox" />
      <span>{label}</span>
    </label>
  );
}
export function ColorSwatchPicker({
  value,
  onChange,
}: {
  value: ProfileColor;
  onChange: (color: ProfileColor) => void;
}) {
  const id = useId();
  return (
    <fieldset className={styles.colors}>
      <legend>{copy.desktop.colour}</legend>
      <div>
        {profileColors.map((color) => (
          <label key={color} style={{ background: `var(--profile-${color})` }}>
            <input
              type="radio"
              name={id}
              value={color}
              checked={value === color}
              onChange={() => onChange(color)}
              aria-label={copy.desktop.colors[color]}
            />
            <span aria-hidden="true" />
          </label>
        ))}
      </div>
    </fieldset>
  );
}
export function Tooltip({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  const id = useId();
  return (
    <span className={styles.tooltip} tabIndex={0} aria-describedby={id}>
      {children}
      <span role="tooltip" id={id}>
        {label}
      </span>
    </span>
  );
}
export function Dialog({
  title,
  children,
  onClose,
  className = '',
  labelId,
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
  className?: string;
  labelId?: string;
}) {
  const [returnFocus] = useState(() =>
    typeof document === 'undefined'
      ? null
      : (document.activeElement as HTMLElement | null),
  );
  return (
    <DialogPrimitive.Root
      open
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay className={styles.dialogOverlay} />
        <DialogPrimitive.Content
          className={`${styles.dialog} ${className}`}
          aria-label={labelId ? undefined : title}
          aria-labelledby={labelId}
          aria-describedby={undefined}
          onPointerDownOutside={(event) => event.preventDefault()}
          onCloseAutoFocus={(event) => {
            if (returnFocus?.isConnected) {
              event.preventDefault();
              returnFocus.focus();
            }
          }}
        >
          <DialogPrimitive.Title asChild>
            <span hidden>{title}</span>
          </DialogPrimitive.Title>
          {children}
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  );
}
export function Sheet({
  title,
  children,
  onClose,
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
}) {
  return (
    <Dialog title={title} onClose={onClose} className={styles.sheet}>
      <div className={styles.sheetHeader}>
        <h2>{title}</h2>
        <IconButton label={copy.desktop.dismiss} onClick={onClose}>
          ×
        </IconButton>
      </div>
      {children}
    </Dialog>
  );
}
export function Toast({
  message,
  onClose,
  error = false,
}: {
  message: string;
  onClose: () => void;
  error?: boolean;
}) {
  return (
    <div className={styles.toast} role={error ? 'alert' : 'status'}>
      <span>{message}</span>
      <IconButton label={copy.desktop.dismiss} onClick={onClose}>
        ×
      </IconButton>
    </div>
  );
}
export function Tabs({
  items,
  value,
  onChange,
}: {
  items: { id: string; label: string }[];
  value: string;
  onChange: (id: string) => void;
}) {
  return (
    <div role="tablist" className={styles.tabs}>
      {items.map((item, index) => (
        <button
          type="button"
          role="tab"
          aria-selected={value === item.id}
          tabIndex={value === item.id ? 0 : -1}
          key={item.id}
          onClick={() => onChange(item.id)}
          onKeyDown={(event) => {
            if (event.key === 'ArrowRight' || event.key === 'ArrowLeft') {
              event.preventDefault();
              const direction = event.key === 'ArrowRight' ? 1 : -1;
              const next = (index + direction + items.length) % items.length;
              const nextItem = items[next];
              if (!nextItem) return;
              onChange(nextItem.id);
              (
                event.currentTarget.parentElement?.children[
                  next
                ] as HTMLButtonElement
              )?.focus();
            }
          }}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}
export function Menu({
  label,
  items,
}: {
  label: string;
  items: { label: string; onClick: () => void; disabled?: boolean }[];
}) {
  return (
    <details className={styles.menu}>
      <summary>{label}</summary>
      <div>
        {items.map((item) => (
          <Button
            key={item.label}
            size="M"
            kind="quiet"
            disabled={item.disabled}
            onClick={(event) => {
              item.onClick();
              event.currentTarget.closest('details')?.removeAttribute('open');
            }}
          >
            {item.label}
          </Button>
        ))}
      </div>
    </details>
  );
}
export function Kbd({ children }: { children: ReactNode }) {
  return <kbd className={styles.kbd}>{children}</kbd>;
}
export function Badge({
  status,
}: {
  status: 'verified' | 'community' | 'experimental';
}) {
  return (
    <span className={`${styles.badge} ${styles[status] ?? ''}`}>
      {copy.desktop[status]}
    </span>
  );
}
export function Stepper({ step }: { step: number }) {
  return (
    <ol className={styles.stepper}>
      {copy.desktop.wizardSteps.map((label, index) => (
        <li
          key={label}
          data-active={index <= step}
          aria-current={index === step ? 'step' : undefined}
        >
          <span />
          {index + 1} · {label}
        </li>
      ))}
    </ol>
  );
}
export function KeyValueList({
  values,
}: {
  values: Record<string, ReactNode>;
}) {
  return (
    <dl className={styles.keyValues}>
      {Object.entries(values).map(([key, value]) => (
        <div key={key}>
          <dt>{key}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}
export function LogTable({ rows }: { rows: Record<string, unknown>[] }) {
  return (
    <div className={styles.logs}>
      <table>
        <caption>{copy.desktop.recentLaunches}</caption>
        <thead>
          <tr>
            <th>{copy.desktop.time}</th>
            <th>{copy.desktop.event}</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row, index) => (
            <tr key={index}>
              <td>{String(row.timestamp ?? '—')}</td>
              <td>
                <pre>{JSON.stringify(row.fields ?? row, null, 2)}</pre>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {rows.length === 0 && <p>{copy.desktop.noLogs}</p>}
    </div>
  );
}
