# Threat model

InstanceDeck separates application state for accounts the user owns. It is not a
sandbox, a credential vault, an account broker or a way to bypass provider
limits. The threat boundary is the Windows user account.

| Threat                                       | Controls and limits                                                                                                                                                                                                                                                      |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Cross-profile state or wrong account         | Explicit data paths, environment templates, exact process markers, account guidance and manual confirmation. Provider updates can break isolation; recipes remain Community until account QA passes.                                                                     |
| Default Claude slot left displaced           | Synced journal intent, same-volume no-replace renames, retained captures/backups, unique-layout checks and startup Resume/Restore. Synthetic interruption tests cover each transition; real-account QA is deferred.                                                      |
| Wrong process closed or given shell identity | Installed executable proof, profile markers or direct-spawn receipts, PID/creation-time validation, stable HWND checks and revalidation before close. No environment, token or window-title inspection. Unprovable identities fail closed.                               |
| Accidental deletion                          | Metadata removal preserves files. Data deletion requires the exact profile name, stopped state and Recycle Bin-only Windows flags. Reparse points and incomplete recovery block deletion. Uninstall preserves all app/profile data.                                      |
| Junction reaches another profile's sessions  | Sharing is explicit and sessions-only; paths/targets are checked, private sessions are retained and changes are journaled. Retired links stay outside recycled trees. No session merging or pruning.                                                                     |
| Malicious update or downgrade                | HTTPS project release URLs, embedded public key, artifact signature and signed-version binding, newer-version policy, explicit installation confirmation and recovery guards. GitHub/CI and the maintainer key remain trust dependencies. Windows binaries are unsigned. |
| Custom recipe executes unwanted code         | Custom mode deliberately launches the chosen executable with the supplied non-secret arguments/environment. It is local, Experimental and Unknown isolation. Review the executable and preview. No downloaded recipe or remote command is executed automatically.        |
| Metadata import redirects operations         | Strict schema, stable ids, manager-shaped paths, no overlaps, existing-id preservation, required known recipes, hotkey checks and a review step. Import transfers no app data and resets local account/sharing claims.                                                   |
| Information leaks in reports                 | Local logs contain launcher metadata, not app-file contents. Diagnostics offers user-path redaction; sharing is user-initiated. Metadata exports can expose names and paths.                                                                                             |

Any process running as the same Windows user can generally read profile data,
including data protected with that user's DPAPI context. Administrator processes
have broader access. A malicious target app can ignore its isolation flags, read
another profile or contact its provider. Reparse-point checks cannot turn a
writable same-user filesystem into a security boundary. A compromised manager or
maintainer/CI account remains a serious risk.

No independent penetration test or security certification is claimed. The manual
Windows 10/11, ARM64, visible-account, sign-in and pinning matrix remains the
release tester's responsibility. See [SECURITY.md](../SECURITY.md) for reports.
