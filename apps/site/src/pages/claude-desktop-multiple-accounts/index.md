---
layout: ../../layouts/ArticleLayout.astro
title: Multiple Claude Desktop Accounts on Windows – InstanceDeck
description:
  Separate Claude Desktop profiles with their own data paths. Learn the Windows
  sign-in callback constraint, guided flow and Cowork limitations.
heading: Run multiple Claude Desktop accounts on Windows
name: Claude Desktop guide
intro:
  Separate app data, sign-ins and settings, with the official app left
  untouched.
---

## Why another launch usually opens the same account

Claude Desktop normally uses one user-data location and a single-instance lock.
A second launch can focus that existing instance instead of creating a separate
account window.

## What InstanceDeck changes

Each profile gets a separate user-data directory and local application-data
root:

```text
--user-data-dir=<profile>\data
LOCALAPPDATA=<profile>\localappdata
CLAUDE_CONFIG_DIR=<profile>\claude-code
```

The last setting is an optional, experimental Claude Code configuration
override. It is not an official vendor guarantee. The
[Claude Desktop recipe](../apps/claude-desktop/) records community provenance
and version limits.

Local process QA passed with Claude Desktop 5.11492.5.0 on Windows 11
build 26200. This proves concurrent process isolation, not account identity or
concurrent Cowork support.

## Windows has one browser callback destination

Browser sign-in returns through the `claude://` protocol. Windows can deliver
that callback to the default package instance rather than the profile that
started it. Launching another data directory does not, by itself, solve that
routing constraint.

## Guided snapshot sign-in

The development build includes a journaled sign-in assistant. Its crash and
restoration harness uses synthetic data; real account verification is still
pending. Use the assistant rather than manually renaming your main data.

The workflow is:

1. Close the main Claude gracefully, then set its proven live data slot aside
   with a same-volume rename.
2. Open a fresh default Claude and let you sign in normally.
3. After you confirm sign-in, close that fresh window, capture its folder and
   adopt it into the selected profile. The capture and previous profile data
   remain as retained backups, including when the profile is on another drive.
4. Restore the original main slot before relaunching it. An interrupted
   operation must offer Resume or Restore on the next start.

Unknown or ambiguous layouts fail closed. A stash is never discarded; it is
renamed back to its original location. Crash tests at every journal transition
and before/after file operations compare synthetic fixture hashes. On restart,
choose Resume or Restore my main Claude. Neither option force-closes the app.

The optional “Looks signed in” hint watches file events and metadata, then waits
for three seconds of quiet. It does not verify an account. “I’ve signed in” is
always available; check the visible account afterwards and record your manual
confirmation in Profile details → Account health.

## MCP configuration per profile

The app's MCP configuration follows its user-data folder. In Profile details,
choose a source Claude profile to copy only `claude_desktop_config.json` on
request. The launcher does not open or parse it, and preserves an existing
destination file as a backup. Stop the destination profile first. MCP
configuration can itself contain secrets; do not publish it in a report.

## Limitations

- Concurrent Cowork virtual machines are unverified because Cowork uses a
  machine-wide service.
- Browser callbacks share one Windows protocol-handler destination.
- Claude Code configuration isolation is experimental.
- Recipes remain Community until the account and recovery QA matrix has
  evidence.

[Codex guide](../codex-multiple-accounts/) ·
[Claude recipe](../apps/claude-desktop/) · [Security model](../security/)
