export function keepDialogFocus(
  dialog: HTMLDialogElement,
  event: { key: string; shiftKey: boolean; preventDefault: () => void },
) {
  if (event.key !== 'Tab') return;
  const controls = Array.from(
    dialog.querySelectorAll<HTMLElement>(
      'button, a[href], input, select, textarea, [tabindex="0"]',
    ),
  ).filter(
    (element) =>
      !element.matches(':disabled') &&
      element.tabIndex >= 0 &&
      element.getClientRects().length > 0,
  );
  const first = controls[0];
  const last = controls.at(-1);
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last?.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first?.focus();
  }
}
