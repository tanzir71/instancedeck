# Microsoft Store submission checklist

No Store submission or account registration has been performed. Current builds
are unsigned. The EXE/MSI Store route requires a certificate chaining to a
trusted root; unsigned and self-signed releases do not meet that requirement.
This path is blocked by the selected distribution policy, not a pending signing
approval.

- [ ] Register the developer account and reserve the product name if desired.
- [ ] Re-evaluate the then-current Store certificate requirements before
      spending time on submission.
- [ ] Complete the real Windows/account/installer QA matrix and accessibility
      review.
- [ ] Supply accurate app metadata, privacy/security links, screenshots and
      architecture requirements.
- [ ] Confirm the per-user installer preserves profiles and all retained backups
      on uninstall.
- [ ] Keep the user-enabled updater for EXE/MSI distribution; do not assume
      Store-managed updates.
- [ ] Verify target app terms and trademarks without implying affiliation.

Do not package InstanceDeck as MSIX without a dedicated package-context spike.
Inherited package context could interfere with launching another packaged app.
NSIS and portable GitHub distribution remain the implemented routes.

References checked 2026-11-14:
[Windows signing options](https://learn.microsoft.com/windows/apps/package-and-deploy/code-signing-options),
[Store submission FAQ](https://learn.microsoft.com/windows/apps/publish/faq/submit-your-app).
