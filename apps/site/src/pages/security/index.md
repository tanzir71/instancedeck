---
layout: ../../layouts/ArticleLayout.astro
title: Security Model – InstanceDeck
description:
  Review InstanceDeck's local isolation model, credential boundaries, data
  recovery safeguards and release signing status.
heading: A security model you can inspect
name: Security
intro:
  Official binaries, explicit launch settings and recoverable data operations.
---

## What the launcher does

InstanceDeck launches installed apps with arguments, environment variables and
separate data paths. It resolves Microsoft Store packages at runtime. It
observes process IDs, creation times, executable paths and profile launch
arguments to keep actions scoped to the selected profile.

The launcher does not patch or repackage apps, inject DLLs, hook APIs inside
their processes or install a background service. All source is
[public under MIT](https://github.com/tanzir71/instancedeck).

## Credential boundary

InstanceDeck never reads cookies, authentication files, tokens or encrypted
credential blobs. It does not decrypt DPAPI data or call private provider usage
APIs. Sign-in belongs to the official app. Account health checks ask you to
confirm the account shown in the app; they do not inspect its credentials.

## Data safety

- **Remove from deck** keeps the profile folder.
- **Recycle profile** requires the exact profile name and stopped processes.
  Windows must move the folder to the Recycle Bin. An abort retains its deck
  entry.
- No user-data path is hard-deleted. Backups and stashes are never automatically
  pruned.
- A sign-in stash must be restored before it can be removed. Unknown or
  ambiguous layouts must fail closed.
- Closing is graceful first, with an eight-second wait. Force close requires a
  separate confirmation and a fresh process-identity check.

## What profiles do not protect against

This is not a sandbox or a separate Windows account. Target processes retain
your Windows user's access. Profiles cannot isolate machine-wide services or
guarantee that every app honors every data-path setting. A compromised app or
administrator may access other local profiles. Do not treat color labels as a
security boundary.

Claude's browser callback uses one Windows protocol handler. Concurrent Cowork
virtual machines remain unverified. Some Codex app-server versions may ignore a
home override, so check every visible account after an update. Recipe status and
reported versions are shown in the [Apps directory](../apps/).

## Release trust

Releases are unsigned. Verify the source and the published SHA-256 checksums. No
trusted-publisher certificate or pending signing approval is claimed. See the
[code signing policy](../code-signing-policy/) and
[download status](../download/). Never disable Windows security protections to
run an untrusted build.

## Reporting a vulnerability

Use
[GitHub's private vulnerability reporting](https://github.com/tanzir71/instancedeck/security/advisories/new)
when available. If it is unavailable, contact the maintainer through
[tanziro.com](https://tanziro.com) to arrange a private channel. Send a minimal
reproduction and affected versions, without cookies, tokens or private profile
contents.
