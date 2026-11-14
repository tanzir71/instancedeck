# Release process

Windows packages remain unsigned. The updater uses a separate free project
integrity key, not a Windows certificate or self-signed root.

1. Update the workspace/package/Tauri versions and `docs/releases/vX.Y.Z.md`.
   `node scripts/check-release-version.mjs` validates the version from
   `RELEASE_TAG`. Keep target-app account/hardware claims unverified until
   tested.
2. Run the Release workflow manually with the intended tag and
   **build_only=true** on the candidate branch. It runs CI, then builds per-user
   NSIS and portable x64/ARM64 packages on native Windows runners. PE
   architecture and unsigned status are checked. No installer or target app is
   launched.
3. Once green, tag the exact commit `vX.Y.Z` and push it. `release.yml` repeats
   verification and creates a **draft** with both installers, portable archives,
   provenance and SHA-256 checksums. It refuses to modify a published release.
4. On the maintainer's Windows PC, finalize the draft:

   ```powershell
   ./scripts/finalize-release.ps1 -Tag v0.1.0
   ```

   The local key defaults to `%LOCALAPPDATA%\instancedeck-updater.key`, outside
   the repository. Only the official signer opens it. The script checks CI
   checksums and tag provenance, signs each installer with its app version,
   verifies signatures offline, builds and signs `latest.json`, and tests
   rejection of a mismatched version and a tampered copy of our own installer.
   It uploads only public artifacts, feed, signatures and refreshed checksums.
   Windows binaries remain NotSigned. All working copies/logs are retained under
   target; no user data is pruned.

5. Complete the [manual matrix](manual-verification.md), review unsigned
   SmartScreen behavior, install and uninstall preservation, then publish the
   draft yourself. Publishing is not implied by automated packaging. Pages
   refreshes public download/changelog metadata when a release is published. A
   checked-in draft-status snapshot can describe real automated draft readiness
   without publishing private asset links.

Keep a secure retained backup of the local key. Losing it breaks the updater
trust chain. The public key in tauri.conf.json must match; do not replace it
silently. There is no private updater key or signing-service credential in the
repository or GitHub Actions. Local finalization keeps the key on the
maintainer's PC and completes release integrity verification before a draft is
published.

Stable checks use the latest published GitHub release. Beta checks use the
`updater-beta` feed alias, updated only from a published, finalized prerelease.
No feed is public while the first release stays draft. The updater reports this
explicitly rather than pretending a draft is an available stable update.

Portable packages require WebView2 to be installed separately and update by
extracting a new package. NSIS is per-user and contains no data-deletion option.
Do not package the manager as MSIX without a dedicated package-context spike.
