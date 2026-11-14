---
layout: ../../layouts/ArticleLayout.astro
title: How Multi-instance Profiles Work – InstanceDeck
description:
  See how InstanceDeck separates Windows app profiles using launch arguments,
  environment variables and data paths, without changing official apps.
heading: How to run multiple app instances on Windows
name: How it works
intro:
  Each profile is a separate data folder and a precise set of launch settings.
---

## One app install, separate profiles

InstanceDeck starts the official executable you installed. A recipe supplies the
arguments and environment variables that tell that app where to store its state.
The executable is never patched, copied or repackaged.

For Electron apps, a separate user-data directory can give each process its own
single-instance lock. Some apps need additional settings before they check that
lock. A recipe records those differences instead of assuming one flag works
everywhere.

## Three steps

1. **Pick an app.** InstanceDeck discovers installed Microsoft Store packages
   and standard installs at runtime.
2. **Create a profile.** Choose a name, color and parent folder. A stable
   identifier keeps renaming separate from moving data.
3. **Launch side by side.** Open each profile from your Deck. Launch becomes
   Focus while that profile is running.

## Transparent launch settings

Claude Desktop uses a profile-specific `--user-data-dir` and `LOCALAPPDATA`.
Codex also needs `CODEX_ELECTRON_USER_DATA_PATH` and `CODEX_HOME`. The
[compatibility directory](../apps/) shows the actual recipe settings and known
limitations.

Microsoft Store paths change when apps update. InstanceDeck resolves the current
package location rather than saving a versioned WindowsApps path. It tries a
direct launch, resolves once more after failure, then uses the recipe's
package-context fallback.

## Isolation has limits

Profiles are not a security sandbox. Apps still run as your Windows user and may
access other files permitted to that user. Machine-wide services, protocol
handlers and some app-server behaviors can remain shared. Read each recipe's
limitations and check the visible account after updates.

InstanceDeck closes windows gracefully first. Force close requires confirmation
after the grace period. Removing a profile from the Deck keeps its data;
recycling requires its exact name and stopped processes.

[Read the security model](../security/) ·
[Claude Desktop guide](../claude-desktop-multiple-accounts/) ·
[Codex guide](../codex-multiple-accounts/)
