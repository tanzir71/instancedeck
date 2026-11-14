# Manual verification before publication

The automated build does not complete these checks. All entries below are
**unverified**. Keep the GitHub Release draft and all recipes Community until
the relevant checks have evidence. Record the tester, date, Windows build,
architecture, InstanceDeck artifact/hash, official app versions and observed
result. Do not attach credentials, profile contents or unredacted account
screenshots to public reports.

Use accounts you own and save work before testing closure or recovery. Review
the unsigned policy and SHA-256 checksums first. WebView2 and official target
apps must be installed separately.

| Scenario                                                                               | Windows 10 22H2 x64 | Windows 11 x64 | Windows 11 ARM64 |
| -------------------------------------------------------------------------------------- | ------------------- | -------------- | ---------------- |
| Main Claude and two profiles display three distinct accounts                           | Unverified          | Unverified     | Unverified       |
| Guided Claude sign-in restores the main account intact                                 | Unverified          | Unverified     | Unverified       |
| Interrupt AwaitingSignIn, restart InstanceDeck, Restore, verify main account           | Unverified          | Unverified     | Unverified       |
| Main Codex and two profiles display their own accounts                                 | Unverified          | Unverified     | Unverified       |
| Update the official apps; existing profiles still launch after path resolution         | Unverified          | Unverified     | Unverified       |
| Profile root on another drive                                                          | Unverified          | Unverified     | Unverified       |
| Windows username and profile path with spaces and non-ASCII characters                 | Unverified          | Unverified     | Unverified       |
| Two profile taskbar groups/icons; pin, close and relaunch the correct profile          | Unverified          | Unverified     | Unverified       |
| Explicit profile-data deletion moves it to the Recycle Bin                             | Unverified          | Unverified     | Unverified       |
| Each generic recipe launches two instances with separate visible state                 | Unverified          | Unverified     | Unverified       |
| Tray launch/focus/close, hotkeys, switcher and opt-in startup                          | Unverified          | Unverified     | Unverified       |
| Shared Codex sessions enable/disable preserves private and shared history              | Unverified          | Unverified     | Unverified       |
| Unsigned NSIS install/upgrade/uninstall preserves manager/profile data                 | Unverified          | Unverified     | Unverified       |
| Portable extraction and launch, with separate WebView2                                 | Unverified          | Unverified     | Unverified       |
| Disabled update checks make no request; a future verified update requires confirmation | Unverified          | Unverified     | Unverified       |

The Win11 x64 launch-engine process-concurrency proof and synthetic filesystem
tests are recorded separately in PROGRESS.md. They do not establish the visible
account or installer outcomes above. Callbackless/protocol-handler and DWM
border experiments remain disabled; concurrent Cowork VMs are not claimed.

Use [the release process](releases.md) to finalize and publish only after
reviewing the results. Store and winget submissions are separate maintainer
steps; no submission has been made.
