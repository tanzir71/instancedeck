---
layout: ../../layouts/ArticleLayout.astro
title: Multiple Codex App Accounts on Windows – InstanceDeck
description:
  Run Codex profiles with separate CODEX_HOME and Electron data paths. Learn
  sign-in steps, known limits and the manual account check.
heading: Run multiple Codex app accounts on Windows
name: Codex guide
intro: A separate Codex home and desktop data path for each profile.
---

## How Codex stores local state

Codex uses `CODEX_HOME` for local state, with `~/.codex` as the default. On
Windows, the usual default is `%USERPROFILE%\.codex`. OpenAI documents
configuration, history, logs and credential-storage options in its
[official configuration guide](https://learn.chatgpt.com/docs/config-file/config-advanced#config-and-state-locations).

InstanceDeck never reads that directory's authentication files. It supplies a
different home when launching a profile.

## How a profile separates it

The Community recipe launches the installed Store app with these settings:

```text
--user-data-dir=<profile>
CODEX_HOME=<profile>\codex-home
CODEX_ELECTRON_USER_DATA_PATH=<profile>\electron
```

It also supplies the installed bundled CLI path when that executable exists.
Package discovery resolves the current installation; no WindowsApps version path
is saved in the recipe.

The Electron override and package launch details are community findings, not an
official OpenAI guarantee. The [recipe page](../apps/codex/) shows provenance
and limitations. Local process QA passed with Codex 26.1106.6628.0 on Windows 11
build 26200; different-account identity has not yet been certified.

## Sign in inside each profile

1. Install the official Codex app for your Windows user.
2. Create a Codex profile in InstanceDeck and launch it.
3. Use the official sign-in screen in that profile.
4. Check the account shown inside the app before starting work. Repeat for each
   profile.

Do not copy or swap authentication files. Each account should complete its own
normal sign-in.

## Check every profile after an update

Some community reports describe app-server versions that ignore `CODEX_HOME`. A
separate process tree is not sufficient proof of a separate account. Open each
profile, inspect its visible account, then record the manual health check in
Profile details → Account health. Diagnostics shows the recorded state. If two
profiles show the same unexpected account, stop and report the app version and
launch strategy without sharing credentials.

## Optional shared history

Separate history is the default. Stop the profile and open Profile details to
enable sessions sharing explicitly. This links only `codex-home\sessions` to a
local shared folder. Every opted-in profile can see those conversations.
Existing private sessions are retained without merging; **Restore private
history** brings them back. Authentication and configuration files are never
linked or copied.

Interrupted changes block launch until sharing is resumed or private history is
restored. Retired links remain outside the profile folder. Recycling rejects
nested reparse points, so restore private history before deleting profile data.
Junction behavior passes synthetic NTFS fixtures; live Codex history
verification remains a manual check.

## Questions

**Does this bypass service limits?** No. Use accounts you own under their
applicable terms. The launcher does not rotate accounts or pool usage.

**Does InstanceDeck inspect my tokens?** No. Its account check is your
confirmation of the account visible in the app.

[Claude Desktop guide](../claude-desktop-multiple-accounts/) ·
[Codex recipe](../apps/codex/) · [Download status](../download/)
