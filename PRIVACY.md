# Privacy

InstanceDeck has no account system, telemetry, analytics, advertising or crash
upload service. Profiles, settings, journals, retained backups and launcher logs
stay on the PC. The default manager folder is `%LOCALAPPDATA%\InstanceDeck`;
profile data can use another chosen folder.

The launcher discovers installed apps and observes process identity,
command-line profile markers, PID/creation times and visible window handles. It
uses file and directory metadata for layout checks and sign-in activity hints.
It does not inspect passwords, cookies, tokens, authentication-file contents or
DPAPI blobs. Account health is the user's explicit confirmation, never an
inferred identity.

Guided Claude sign-in moves folders and uses opaque Windows file operations to
adopt a captured profile, then restores the main slot. It does not parse the
sign-in data. Existing captures, stashes and backups remain retained. The MCP
copy action transfers only the named configuration file without inspecting its
contents. Optional Codex sharing links only the sessions directory after
explicit consent; linked profiles can see those conversations. No transcripts
are merged, read by the launcher or uploaded by InstanceDeck.

App networking is limited to user-enabled update checks and explicitly confirmed
update downloads from this project's GitHub Releases. Checks begin only after
first-run is completed and the visible preference permits them. They send an
ordinary HTTPS request; GitHub/CDN operators can observe the connection's IP
address and request metadata. Disabling checks stops new checks and downloads.
No update installs automatically. Provider/project links open the default
browser only when selected. Target apps and the browser have their own privacy
policies.

Metadata exports contain profile names, ids, paths, options and manual account
state, never app data. Treat paths and names as personal information. Imported
account confirmations, launch times and sharing are reset. Diagnostics can
redact the Windows user folder; review reports before sharing them. Nothing is
sent automatically.

Removing a Deck entry preserves data. Confirmed data deletion uses the Recycle
Bin and refuses permanent-delete fallback. The installer/uninstaller preserves
profile data, manager metadata, journals and retained backups.

The static GitHub Pages site fetches public release metadata at build time. It
uses local fonts and no analytics or third-party scripts. GitHub hosts the site
and receives normal web requests. IndexNow receives the public sitemap URLs; it
never receives profile or account information.
