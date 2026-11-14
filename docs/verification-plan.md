# Handover acceptance ledger

**User update, 2026-11-14:** manual verification is deferred to the user.
Continue implementation and automated checks through the remaining phases, and
push the work to GitHub. Manual account, tray, pinning and hardware-matrix
checks stay unverified. Releases will be unsigned; signing-provider approval is
no longer a dependency. See the newest decision in DECISIONS.md.

The latest instruction selects local verification and Git pushes and authorizes
the public unsigned v0.1.0 preview before manual QA. Project workflows are
manual-only; Pages serves locally built gh-pages files. Publication does not
change any unverified account, installer or hardware result.

The handover and Figma remain authoritative. This ledger preserves the full
scope and identifies the evidence required to close each phase. A passing
scaffold check does not prove isolation, account identity, recovery or shell
integration.

## Phase gates

| Phase | Required evidence                                                                                                                                                                                                                | Current status                                                                              |
| ----- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| P0    | All three dev commands on Windows 11; Windows CI runs Rust tests, TS/Astro checks, lint, Storybook build/test-runner and native x64 build                                                                                        | Passed; see PROGRESS.md and CI run 43576518592                                              |
| P1    | Core coverage ≥85%; template/path/layout/process fixtures; complete CLI; main + two Claude and main + two Codex processes simultaneously, exact discovered app versions recorded                                                 | Passed; 96.92% lines, live proof in PROGRESS.md                                             |
| P2    | Every screen/state in Storybook with zero AA violations; Figma Deck/wizard/sign-in comparisons; keyboard create → launch → close; tray and live status                                                                           | Implemented and automated checks pass; native QA deferred                                   |
| P3    | Journal fsynced before transitions; crash harness at every transition restores fixture main slot byte-identical; guided sign-in, MCP-only copy, Claude Code toggle and Codex manual account health check; both spikes documented | Implemented; crash/recovery fixtures pass; real accounts deferred; optional spikes disabled |
| P4    | Distinct taskbar groups/icons for two Claude profiles; pin A, close and relaunch A correctly; shortcut property persistence; switcher, autostart and DWM spike                                                                   | Implemented; native fixture checks pass; pinning/startup deferred; DWM disabled             |
| P5    | Six generic recipes smoke-tested or honestly community-labelled with reasons; custom editor command preview; metadata-only import/export; optional shared sessions junctions; dark theme                                         | Implemented and automated checks pass; all recipes Community                                |
| P6    | All §9.1 routes live at the project base; Lighthouse ≥95 in all four categories on home/app/FAQ; 1440/390 Figma comparisons for all sections; link/CSS/footer audits                                                             | Passed; live Pages, IndexNow, Lighthouse 97–100; see PROGRESS.md                            |
| P7    | x64/ARM64 NSIS and portable artifacts, checksums and updater feed; site consumes the authorized public unsigned preview; trust files, winget template and Store checklist                                                        | Passed; local verification, public v0.1.0 and prebuilt Pages; manual QA deferred            |

Finish each phase with its evidence in PROGRESS.md and a `phase N: …` commit. P6
is the only phase explicitly allowed to run alongside P1–P5 after P0.

## Invariants and safety boundaries

| Requirement                         | Evidence to collect                                                                                                           |
| ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Official app binaries untouched     | Review filesystem operations and launch plans; no target binary copy/write paths                                              |
| No credential access                | Review reads and diagnostics; fixtures prove layout/sign-in detection uses metadata only; no cookies/auth/token parsing       |
| No hard deletion of user data       | Remove keeps data; Recycle Bin requires typed profile name; rename/move recovery tests; no automatic backup/stash pruning     |
| Graceful close before force         | WM_CLOSE to matching process tree, eight-second wait, explicit force confirmation and bounded process scope                   |
| No injection or in-process hooks    | Out-of-context window events and documented external shell APIs only                                                          |
| No telemetry or application network | Runtime capability/CSP/dependency review; updater is the sole user-enabled network path                                       |
| Unknown layout fails closed         | Metadata fixtures for all live paths, relocation markers, absent/ambiguous layouts and interrupted recovery                   |
| Paths with spaces and Unicode       | Core and Windows tests, wrapper encoding test and manual Windows QA                                                           |
| No hard-coded WindowsApps paths     | PackageManager runtime resolution, version ordering, launch-failure cache invalidation tests                                  |
| Recipe status is honest             | No automatic promotion; dates, versions and provenance shown on app/site; experimental pages noindex                          |
| Visual tokens have one source       | Regenerate CSS/TS from design/tokens.json; contrast, no handwritten hex, no shadows/gradients/surface strokes                 |
| Accessibility                       | AA axe checks, keyboard flow, visible focus, icon labels, reduced motion and RTL stories                                      |
| No fabricated public claims         | No invented metrics/reviews/logos; unsigned policy explicit; no trusted Windows publisher claim                               |
| Required attribution                | Site footer on every HTML route, app About and README carry non-affiliation; site credits link name and domain to tanziro.com |

## Engine and platform inventory

- Versioned recipes, Rust-generated JSON schema, loader/validation and all five
  template tokens; optional env-path suffix; eight built-in recipe identities.
- Versioned profiles store and migrations; ULID paths independent of names;
  configurable root; options, hotkeys and timestamps; durable writes and
  backups.
- Platform traits with fake discovery, process, filesystem and launch adapters.
- MSIX PackageManager, uninstall registry and known-path discovery; highest
  version selection and re-resolution after failure.
- Direct Unicode launch with process flags; one package-context fallback using a
  hidden PowerShell process and a UTF-16LE BOM wrapper; no retry loops.
- Case-insensitive normalized process argument matching with directory-boundary
  checks; default instance detection, PID trees and structured launch logs.
- CLI: apps; profiles list/create/rename/remove; launch; close with explicit
  force; doctor and redacted metadata-only diagnostics.
- Claude live-slot probes; same-volume stash rename, free-space preflight,
  journal/recovery and bounded rename retries; manual finish always available.
- Codex isolated Electron state/home and optional bundled CLI discovery; account
  mismatch is a manual health warning, never inferred from credentials.
- ShellLink/IPropertyStore AUMID and relaunch properties, persisted/reloaded;
  generated badge ICO sizes 16–256; stable HWND detection and launch mutex.

## Desktop and Storybook inventory

Screens: empty/populated Deck; three-step wizard; profile side panel; guided
Claude sign-in and recovery; Apps; Diagnostics; Settings; global switcher;
About.

Required behavior: main unmanaged instances; profile Launch/Focus/Close; search;
rename/color/open folder; resolved args; shortcuts/hotkeys; MCP-only copy;
remove from deck; confirmed Recycle Bin; profiles-root selection; autostart;
close to tray; visible update consent/channel; light/dark preference; grouped
tray menu.

Every shipped component and screen needs meaningful stories for its applicable
empty/loading/error/running/idle/long-name/RTL states. Required primitives and
composites are listed in HANDOVER.md §7; no component is exempt because it
appears only once. Components shared with marketing stay in packages/ui.

## Site inventory

Routes from §9.1: home; Claude guide; Codex guide; app directory; one page per
recipe; how it works; Parall comparison; security; privacy; signing policy;
download; changelog; FAQ; custom 404. Footer links must resolve to real content,
including docs, contributing, MIT license and trademarks.

Home section order: header, hero fan, works-with strip, pillars, how it works,
use cases, Claude/Codex spotlight, security, FAQ, CTA, footer. Use exact Figma
copy/assets/tokens, prescribed fan geometry/motion and real product screenshots
once P2 exists. Compare each desktop section and the 390-wide full page.

Each page requires: unique title ≤60 characters and description ≤155; one H1;
absolute canonical with project base; OG/Twitter with generated 1200×630 image;
typed/validated JSON-LD; visible/structured breadcrumbs on inner pages; internal
related links; image dimensions/alt/lazy loading; sitemap with git lastmod;
llms.txt; no third-party scripts. Home JS <20 KB gzip, LCP <1.8s and CLS <0.05.

Deployment: static Astro, current pinned Actions, Pages permissions/concurrency,
Storybook under /instancedeck/storybook/ with noindex, IndexNow key/changed
URLs, build-time GitHub release data and no invented
availability/checksums/signing.

## Distribution and trust inventory

MIT, README, SECURITY.md, PRIVACY.md, CODE_SIGNING_POLICY.md, CONTRIBUTING.md
and public threat model. Explain that profiles are not a sandbox and same-user
processes can access DPAPI-protected data. Include truthful unsigned-download
guidance for unsigned releases. Updater integrity signatures are distinct from
Windows code signing; no trusted-publisher certificate is claimed.

P7 also requires per-user NSIS, portable ZIP, x64 and ARM64, SHA-256 manifests,
draft releases, updater metadata, winget PR template and docs/store.md. Human
account registration and manual account/pinning QA remain explicitly unverified
until their outcomes are observed; they cannot be replaced with unit-test
claims. The Store EXE/MSI certificate requirement must be documented accurately
under the unsigned distribution decision.
