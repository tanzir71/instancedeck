# Decisions

## 2026-11-14 — Generic recipes, custom ownership and metadata transfer

Context: the user deferred manual QA while asking for all implementation to
continue. Choice: ship six generic recipes as Community with a local-smoke-test
deferral reason and no invented version reports. Source links and limitations
are in packages/recipes/README.md. Telegram uses the documented paired legacy
`-workdir` switch. Cursor reports partial isolation.

Custom apps are direct executable launches with bounded args/environment
templates. Isolation starts Unknown/Experimental. Secret-like environment names
are rejected. For environment-only apps, track only a PID and creation time
acquired from the actual CreateProcess child handle, then verify its exact exe
against the installed path. Manager-owned receipts survive restarts. An app
handing off to an unrelated singleton fails tracking; no environment or tokens
are read. Alternatives: scanning another process's environment or guessing by
exe name, both rejected.

Import/export contains only profiles metadata. Existing ids are skipped,
overlapping/invalid paths fail, custom recipe ids must already exist, and
imported account claims, last-launch times and sharing are reset. Imports are
reviewed before applying; existing exports and manager metadata are retained.

## 2026-11-14 — Sessions sharing and dark theme

Choice: sharing is off by default and requires explicit UI consent. Journal
intent is synced before same-volume renames or junction creation. Only Codex's
sessions directory is linked to the manager's shared sessions folder. Private
sessions are retained without merging; disabling restores them. Interrupted
changes block launch and metadata removal until resumed or restored. Retired
junctions move outside the profile folder and remain retained. Recycle rejects
all nested reparse points. No transcript or authentication contents are read.

Figma has light frames only. The dark variant conservatively maps the same
generated tokens to dark surfaces and light text; profile colors, white card
labels and primary blue action fills remain fixed. New power dialogs reuse the
existing flat controls and spacing. Alternatives: inventing a second palette,
silently sharing transcripts, or merging/pruning existing history.

## 2026-11-14 — Pages uses the Astro workflow

Context: GitHub Pages appeared with a legacy main-branch/Jekyll source, which
attempted to parse Astro source as YAML and failed. Choice: set its build type
to workflow through the GitHub Pages API. Keep SITE_PUBLISH_ENABLED unset until
the remaining desktop claims ship. The API setting was observed reverting once
and was reapplied; verify it again at final deployment. Alternative: publishing
repository sources or modifying valid Astro files for Jekyll. Reference:
https://docs.github.com/en/rest/pages/pages#update-information-about-a-github-pages-site

## 2026-11-14 — Windows identity uses original project artwork

Context: §4.6 suggests extracting a vendor icon, while AGENTS.md forbids drawing
third-party logos and modifying/copying target binaries. Choice: generate
profile ICO files from InstanceDeck's own neutral deck/I mark and profile color
tokens, at 16, 24, 32, 48, 64, 128 and 256px. Never extract vendor resources.
The shortcut and window share a stable InstanceDeck.appId.ULID identity and
launch through the manager. Existing shortcut files are renamed to retained
backups before replacement, and saved properties are reloaded and checked.
Alternative: vendor icon extraction or file replacement that discards a prior
shortcut. Native taskbar grouping, pinned relaunch and DWM border verification
remain deferred manual checks.

## 2026-11-14 — Snapshot recovery and cross-drive adoption

Context: P3 must survive interruptions without reading credential files,
including profiles stored on another drive. Choice: keep a same-volume capture
beside the default slot, transfer it with opaque Windows CopyFileW operations
into a staged profile directory, then activate with a no-replace rename. The
capture, previous profile data, interrupted transfers and journal history are
retained indefinitely. Rust never opens or parses target data. This
conservatively extends the handover's simple move into a recoverable cross-drive
transfer. Alternative: cross-volume moves that remove the only captured source
before restoration.

Every rename uses MoveFileExW without REPLACE_EXISTING, with at most ten
attempts over fifteen seconds. Reparse points, unknown/ambiguous layouts and low
disk space fail closed. Journal intent is synced before each operation;
restoration validates the resulting main layout before relaunch. A
changed/ambiguous layout keeps recovery pending even after the original folder
has been restored.

Account state is changed only through explicit user confirmation. The optional
metadata hint is scoped to one journal, uses a bounded notify event queue plus
size/mtime sampling, and waits three seconds after a burst. It never establishes
identity. MCP copy transfers exactly the regular claude_desktop_config.json file
and preserves an existing destination as a backup.

The callbackless-sign-in and protocol-handler spikes require the native/manual
verification the user deferred. They remain unverified and disabled; no claude:
handler is registered. Snapshot recovery is the implemented path. Alternative:
shipping callback routing without evidence that the current app supports it.

## 2026-11-14 — Desktop state, dialogs and generated IPC

Context: HANDOVER §5.1 requires Radix, Zustand and tauri-specta. Choice: use
Radix modal primitives with focus restoration and a scoped vanilla Zustand store
for each desktop view. Manager metadata stays in Rust; the browser preview does
not persist it. Generate the tagged IPC request types from Rust and check for
binding drift after Rust tests in CI. The compatible published pair is
tauri-specta 2.0.0-rc.29 / specta 2.0.0-rc.28, pinned exactly; Tauri itself
stays on stable 2.12.5. JSON result numbers retain JSON's JavaScript-number
semantics, while IDs and saved dates are strings. Alternative: manually
maintained request types and custom modal behavior. A 24px switch hit area
preserves the 22px visual while meeting WCAG 2.2 target size.

## 2026-11-14 — Manual verification deferred; unsigned releases

Context: the user explicitly instructed “I will do manual verification later, do
everything else and push to Github.” Choice: continue P3–P7 implementation and
automated verification without waiting for native/manual acceptance gates.
Record those checks as deferred, keep recipes Community, and never claim manual
account, tray, taskbar or architecture verification that has not happened. This
user instruction overrides the earlier phase-order gate only for deferred manual
checks. No further Computer Use is needed for this work.

The user also said releases cannot be signed and allowed self-signed or unsigned
builds. Choice: ship unsigned installers and portable packages, with SHA-256
checksums and clear Windows trust guidance. Do not pursue SignPath or describe
signing approval as pending. Alternative: a self-signed certificate. This
supersedes HANDOVER.md's signing and SignPath requirements and older decisions
or progress notes about future signing approval.

## 2026-11-14 — Site publication follows shipped claims

Context: P6 may run alongside the ordered desktop phases. The Figma home copy
describes guided sign-in, shell identity and generic recipes that are not yet
enabled in the P2 build. Choice: build and audit the complete site now, but keep
Pages publication behind SITE_PUBLISH_ENABLED until those desktop behaviors
ship. Enable that repository variable and deploy once the claims are true; no
additional approval is needed. Alternative: publishing unsupported feature
claims or changing the authority copy into temporary marketing prose.

Guide pages and llms.txt identify the current development limits. No release
version or installer is invented: Download and Changelog read public GitHub
metadata during the build, and fail the build on an API error. Community test
reports remain distinct from manual account verification. Shared-history and
Claude recovery documentation must be updated when their engines ship.

The Pages artifact includes Storybook. Its two HTML entry pages receive the same
shared footer and noindex metadata. Packaged third-party Storybook CSS has
shadow declarations removed so the complete published artifact passes the
flat-style audit. Development Storybook sources stay untouched.

## 2026-11-14 — Marketing responsiveness and complete legal copy

Context: the mobile Figma frame intentionally omits the desktop How it works,
Use cases, app spotlight and FAQ bands. Choice: follow that abridged home layout
at mobile widths; keep the same information available through the guide and FAQ
pages. Tablet and larger layouts retain every band. Alternative: inventing a
longer mobile home that disagrees with the authority frame.

The footer retains the full mandatory non-affiliation statement on every page,
including mobile, even though Figma abbreviates it. This increases its height.
The two underlined tanziro.com credits remain present. Signed-release claims
stay conditional on signing approval. No published release is implied by a
download link while release engineering is unfinished.

## 2026-11-14 — Recycle-only Windows operation

Context: permanent deletion must never be a fallback when a profile does not fit
in the Recycle Bin or the bin is unavailable. Choice: call IFileOperation with
explicit RECYCLEONDELETE, ADDUNDORECORD, EARLYFAILURE, WANTNUKEWARNING,
NOERRORUI and SILENT flags, without any automatic confirmation. Report aborted
operations and require the source folder to be absent before removing its deck
entry. Keep the path-shape and reparse-point checks and the service's typed-name
and stopped-process gates. Alternative: relying on a library's ALLOWUNDO flag
alone. Microsoft's operation flag documentation is the API reference:
https://learn.microsoft.com/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifileoperation-setoperationflags

An actual Windows test recycles only a newly created, disposable fixture under a
Unicode path; that fixture remains recoverable in the Recycle Bin. An
arbitrary-directory rejection test proves the unrelated folder stays in place.

## 2026-11-14 — Desktop UI phase boundaries and accessibility

Context: P2 needs complete screen views before the P3 recovery engine and P4
shell integration exist. Choice: ship real profile launch/close, metadata edits,
diagnostics and settings now; show the later sign-in, shortcuts, hotkeys, custom
recipes and dark-theme controls as unavailable with a concise explanation.
Sign-in and recovery stories depict supplied journal states, never a simulated
operation in the live app. Alternative: presenting unimplemented actions as
successful. Closing a main app is available from Apps with a confirmation; its
Deck card keeps the Figma Launch/Focus layout.

The wizard allocates its stable ULID before creation and shows the final folder.
Rust validates the supplied ID and assembles the path again from the chosen
root. Browsing changes only that parent folder. Renaming never changes the ID.

White secondary card text replaces the Figma paper-72 translucency where it
fails AA on the palette. Experimental badge text uses ink-2 on ochre-tint for
the same reason. The previously recorded OnColor correction remains in force.
Native event permissions are limited to listen/unlisten and titlebar dragging;
all profile actions go through validated Rust requests.

## 2026-11-14 — Claude package application ID

Context: live direct launches passed, but the forced MSIX fallback failed for
Claude Desktop 5.11492.5.0 with the handover's generic App application ID.
Windows' installed app metadata reports `Claude_pzs8sxrjxfjjc!Claude`. Choice:
set Claude's recipe appId to Claude; keep Codex's independently passing App
value. The engine remains generic. Alternative: assuming every vendor uses the
same application ID or hard-coding this exception in the launcher.

## 2026-11-14 — P1 data and process safeguards

Context: initial profile metadata needs migrations and durable updates, while
Windows reuses process IDs. Choice: schema 0 accepts the same required identity,
path and creation fields as schema 1 and supplies only documented defaults for
optional visual/options fields; unknown fields and future versions fail closed.
Every update retains a backup of InstanceDeck's own metadata before atomic
replacement. No profile data is copied or removed by these operations. Process
trees require parent creation time to precede the child; close rechecks
identity, excludes the manager and fails if creation time cannot be proved.
Alternatives: silently dropping fields, replacing without backups, or trusting
PID/parent PID alone.

## 2026-11-14 — Unicode package-context wrapper

Context: §4.4 requires a UTF-16LE BOM .cmd wrapper. cmd.exe cannot directly
interpret Unicode batch files reliably. Choice: store an ASCII PowerShell
EncodedCommand launch line in the required UTF-16 file, decode it explicitly
inside the package host, then pass that line to cmd.exe. All profile paths,
arguments and environment values stay inside the encoded, quoted payload. The
real Windows fixture passed with spaces, Japanese, Zoë, apostrophes, percent
signs, ampersands and exclamation marks. Alternative: relying on an OEM-codepage
batch file or passing unescaped user values through cmd.

## 2026-11-14 — Legacy versioned installs

Context: legacy Squirrel Claude installs may place the executable in app-version
subfolders. Choice: an optional recipe versionPrefix on known-path discovery
selects the highest numeric version containing the requested executable, using
directory names and metadata only. MSIX remains first. Alternative: hard-coded
version paths or launching the updater trampoline.

## 2026-11-14 — OnColor contrast correction

Context: axe reported color-contrast failure for white button text on a teal
profile with the Figma `paper-16` button fill. Calculated contrast is 3.57:1;
all eight profile fills fall below 4.5:1 at that translucency. Neither white nor
ink text reaches 4.5:1 on the resulting iris/plum fill. Choice: mix `paper-16`
at 12.5% with transparent for an effective 2% white fill. This is the smallest
visual change that preserves the white label, flat profile palette, control size
and WCAG AA across every swatch. Keep this explicit design deviation for review.
Alternatives: dark labels (still fail iris/plum), changing the palette, or
enlarging labels enough to qualify as large text.

## 2026-11-14 — Compatible TypeScript and font packaging

Context: the stable TypeScript 7 tag exceeds typescript-eslint's supported range
(`>=4.8.4 <6.1.0`). Fontsource variable packages expose font files, not
`latin.css`. Choice: pin TypeScript 6.0.3 (the newest supported 6.x), explicitly
declare the WASM runtime's required emnapi peers, and self-host Latin WOFF2
files under the font-family names in tokens.json. Alternatives: suppressing peer
warnings or loading unsupported subpaths. React, Astro, Storybook and Tauri 2
remain at their latest stable versions resolved on this date.

## 2026-11-14 — Product defaults

Context: HANDOVER.md §13.2 delegates these choices with defaults. Choice:
InstanceDeck; MIT and fully free; GitHub Pages at
`https://tanzir71.github.io/instancedeck/`; no usage display; Source Serif 4,
IBM Plex Sans and IBM Plex Mono. Alternatives: the named alternate brands, paid
licensing, custom domain, usage reporting and Public Sans.

## 2026-11-14 — Ordered acceptance gates

Context: this workspace starts with only the handover and design assets. Choice:
implement and verify P0 first, including actual Windows CI, before P1. P6 may
start after P0 as explicitly allowed by §11. A scaffold is not a completed
product. Never mark manual account isolation or signing verified without
evidence. Alternative: building later phases before their gates pass.

## 2026-11-14 — Stable dependency resolution

Context: the latest Tauri release on crates.io is a v3 alpha; the stack requires
Tauri 2 stable. Choice: select the newest non-prerelease v2, and resolve the
other tools from their stable registry tags. Pin package manifests and both
lockfiles. Alternatives: adopting prereleases or using stale handover versions.

## 2026-11-14 — Phase 0 preview

Context: profile operations belong to P1–P2, after the scaffold gate. Choice:
the initial desktop opens the empty Deck with an explicitly labelled preview
state; no fake profiles, status counts or operational launch buttons. The site
initially contains a scaffold page with the required footer. Full landing
sections belong to P6. Alternative: presenting demo data as real state.
