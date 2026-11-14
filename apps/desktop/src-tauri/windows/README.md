# Installer data preservation

Based on the
[Tauri CLI 2.12.4 template](https://raw.githubusercontent.com/tauri-apps/tauri/tauri-cli-v2.12.4/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi).
Upstream SHA-256:
`d00089d3b1a2c4a30d7b6e098079100cb8f25e980f84d4515aca4597b95e967d`.

The data-deletion checkbox, callbacks and recursive AppData deletion block are
removed. Uninstall removes only InstanceDeck program files, its shortcuts and
startup registration. Profiles, journals and retained backups stay. The bundled
MIT license covers the original template. No target app binaries are included or
modified. Installer execution remains manual QA.

The initial per-user program location is `%LOCALAPPDATA%\Programs\InstanceDeck`,
separate from the manager state folder. Previously selected install locations
remain controlled by the installer registry metadata.
