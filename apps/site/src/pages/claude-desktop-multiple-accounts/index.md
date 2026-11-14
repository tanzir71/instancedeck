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

## Guided sign-in design

The journaled assistant is planned and its screen is available for design
review, but its recovery engine has not shipped in the current development
build. Do not manually rename your existing main Claude data to imitate this
flow.

The intended workflow is:

1. Close the main Claude gracefully, then set its proven live data slot aside
   with a same-volume rename.
2. Open a fresh default Claude and let you sign in normally.
3. After you confirm sign-in, close that fresh window and move its data into the
   selected profile.
4. Restore the original main slot before relaunching it. An interrupted
   operation must offer Resume or Restore on the next start.

Unknown or ambiguous layouts must fail closed. A stash must never be removed
before restoration. The implementation will be verified with crash tests at
every journal transition before this feature is enabled.

## MCP configuration per profile

The app's MCP configuration follows its user-data folder. A later integration
will copy only `claude_desktop_config.json` on request, never login state. MCP
configuration can itself contain secrets; do not publish it in a diagnostic
report.

## Limitations

- Concurrent Cowork virtual machines are unverified because Cowork uses a
  machine-wide service.
- Browser callbacks share one Windows protocol-handler destination.
- Claude Code configuration isolation is experimental.
- Recipes remain Community until the account and recovery QA matrix has
  evidence.

[Codex guide](../codex-multiple-accounts/) ·
[Claude recipe](../apps/claude-desktop/) · [Security model](../security/)
